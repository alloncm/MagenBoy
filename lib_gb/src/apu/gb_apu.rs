use super::channel::Channel;
use super::wave_sample_producer::WaveSampleProducer;
use super::tone_sweep_sample_producer::ToneSweepSampleProducer;
use super::audio_device::AudioDevice;
use super::sound_terminal::SoundTerminal;
use super::frame_sequencer::{
    FrameSequencer,
    TickType,
};
use crate::mmu::memory::Memory;
use crate::utils::bit_masks::*;

pub const AUDIO_BUFFER_SIZE:usize = 0x400;

pub struct GbApu<Device: AudioDevice>{
    pub wave_channel:Channel<WaveSampleProducer>,
    pub sweep_tone_channel:Channel<ToneSweepSampleProducer>,

    frame_sequencer: FrameSequencer,
    audio_buffer:[f32;AUDIO_BUFFER_SIZE],
    current_t_cycle:u32,
    device:Device,
    terminal1:SoundTerminal,
    terminal2:SoundTerminal,
    enabled:bool
}

impl<Device: AudioDevice> GbApu<Device>{
    pub fn new(device: Device) -> Self {
        GbApu{
            frame_sequencer:FrameSequencer::default(),
            sweep_tone_channel: Channel::<ToneSweepSampleProducer>::new(),
            wave_channel:Channel::<WaveSampleProducer>::new(),
            audio_buffer:[0.0; AUDIO_BUFFER_SIZE],
            current_t_cycle:0,
            device:device,
            terminal1: SoundTerminal::default(),
            terminal2: SoundTerminal::default(),
            enabled:false
        }
    }

    pub fn cycle(&mut self, memory:&mut dyn Memory, m_cycles_passed:u8){
        self.prepare_control_registers(memory);

        //converting m_cycles to t_cycles
        let t_cycles = m_cycles_passed * 4;
        //add timer 

        if self.enabled{
            for _ in 0..t_cycles{   
                if self.current_t_cycle as usize >= AUDIO_BUFFER_SIZE{
                    self.current_t_cycle = 0;
                    self.device.push_buffer(&self.audio_buffer);
                }

                self.prepare_wave_channel(memory);
                self.prepare_tone_sweep_channel(memory);

                let tick = self.frame_sequencer.cycle();
                self.update_channels_for_frame_squencer(tick);
            
                let sample = self.sweep_tone_channel.get_audio_sample();
            
                self.audio_buffer[self.current_t_cycle as usize] = sample;
                
                self.update_registers(memory);
            
                self.current_t_cycle += 1;
            }
        }
        else{
            self.current_t_cycle += t_cycles as u32;
        }
    }

    fn update_channels_for_frame_squencer(&mut self, tick:TickType){
        if tick.frequency_sweep{
            if self.sweep_tone_channel.enabled{
                let sweep = &mut self.sweep_tone_channel.sample_producer.sweep;
                if sweep.time_sweep != 0 && sweep.sweep_shift != 0{
                    let mut shifted_freq:i32 = (sweep.shadow_frequency >> sweep.sweep_shift) as i32;

                    if sweep.sweep_decrease{
                        shifted_freq *= -1;
                    }

                    shifted_freq += sweep.shadow_frequency as i32;

                    if shifted_freq >= 2048 || shifted_freq <= 0{
                        self.sweep_tone_channel.enabled = false;
                    }
                    else{
                        sweep.time_sweep -= 1;
                        self.sweep_tone_channel.frequency = shifted_freq as u16;
                        self.sweep_tone_channel.timer.cycles_to_tick = (2048 - self.sweep_tone_channel.frequency).wrapping_mul(4);
                    }
                }
            }
        }
        if tick.length_counter{
            if self.sweep_tone_channel.length_enable && self.sweep_tone_channel.enabled{
                self.sweep_tone_channel.sound_length -= 1;
                if self.sweep_tone_channel.sound_length == 0{
                    self.sweep_tone_channel.enabled = false;
                }
            }
        }
        if tick.volume_envelope{
            if self.sweep_tone_channel.enabled{
                let envelop = &mut self.sweep_tone_channel.sample_producer.envelop;

                if envelop.number_of_envelope_sweep > 0 {
                    envelop.envelop_duration_counter += 1;
                    if envelop.envelop_duration_counter == envelop.number_of_envelope_sweep{
                        if envelop.increase_envelope{
                            let new_vol = self.sweep_tone_channel.volume + 1;
                            self.sweep_tone_channel.volume = std::cmp::min(new_vol, 0xF);
                        }
                        else{
                            let new_vol = self.sweep_tone_channel.volume as i8 - 1;
                            self.sweep_tone_channel.volume = std::cmp::max::<i8>(new_vol, 0) as u8;
                        }

                        envelop.envelop_duration_counter = 0;
                    }
                }
            }
        }
    }

    fn prepare_control_registers(&mut self, memory:&dyn Memory){
        let channel_control = memory.read(0xFF24);
        self.terminal1.enabled = channel_control & BIT_3_MASK != 0;
        self.terminal2.enabled = channel_control & BIT_7_MASK != 0;
        //todo: add volume
        self.terminal1.volume = 7;
        self.terminal2.volume = 7;

        let channels_output_terminals = memory.read(0xFF25);

        for i in 0..4{
            self.terminal1.channels[i as usize] = channels_output_terminals & (1 << i) != 0;
        }
        for i in 0..4{
            self.terminal2.channels[i as usize] = channels_output_terminals & (0b10000 << i) != 0;
        }

        let master_sound = memory.read(0xFF26);
        self.enabled = master_sound & BIT_7_MASK != 0;
    }

    fn prepare_wave_channel(&mut self, memory:&dyn Memory){
        self.wave_channel.sound_length = memory.read(0xFF1B);
        self.wave_channel.enabled = memory.read(0xFF1A) & BIT_7_MASK != 0;
        //I want bits 5-6
        self.wave_channel.sample_producer.volume = (memory.read(0xFF1C)>>5) & 0b011;
        let mut freq = memory.read(0xFF1D) as u16;
        let nr34 = memory.read(0xFF1E);
        freq |= ((nr34 & 0b111) as u16) << 8;
        self.wave_channel.frequency = freq;

        // According to the docs the frequency is 65536/(2048-x) Hz
        // After some calculations if we are running in 0x400000 Hz this should be the 
        // amount of cycles we should trigger a new sample
        // 
        // Rate is for how many cycles I should trigger.
        // So I did the frequency of the cycles divided by the frequency of this channel
        // which is 0x400000 / 65536 (2048 - x) = 64(2048 - x)
        self.wave_channel.timer.cycles_to_tick = (2048 - freq).wrapping_mul(64);
        self.wave_channel.trigger = nr34 & BIT_7_MASK != 0;
        self.wave_channel.length_enable = nr34 & BIT_6_MASK != 0;

        for i in 0..=0xF{
            self.wave_channel.sample_producer.wave_samples[i] = memory.read(0xFF30 + i as u16);
        }
    }

    fn prepare_tone_sweep_channel(&mut self, memory:&mut dyn Memory){
        let nr10 = memory.read(0xFF10);
        let nr11 = memory.read(0xFF11);
        let nr12 = memory.read(0xFF12);
        let nr13 = memory.read(0xFF13);
        let nr14 = memory.read(0xFF14);

        if nr14 & 0b1000_0000 != 0{
            // Sweep register (nr10)
            self.sweep_tone_channel.sample_producer.sweep.sweep_decrease = (nr10 & 0b1000) != 0;
            self.sweep_tone_channel.sample_producer.sweep.sweep_shift = nr10 & 0b111;

            // sound length and wave pattern register (nr11)
            self.sweep_tone_channel.sample_producer.wave_duty = (nr11 & 0b1100_0000) >> 6;
            self.sweep_tone_channel.sound_length = nr11 & 0b11_1111;

            // Volume envelop register (nr12)
            self.sweep_tone_channel.volume = (nr12 & 0b1111_0000) >> 4;
            self.sweep_tone_channel.sample_producer.envelop.increase_envelope = (nr12 & 0b1000) != 0;
            self.sweep_tone_channel.sample_producer.envelop.number_of_envelope_sweep = nr12 & 0b111;

            // Freqeuncy registers (nr13 nr14)
            self.sweep_tone_channel.frequency = nr13 as u16 | ((nr14 as u16 & 0b111) << 8);
            self.sweep_tone_channel.length_enable = nr14 & 0b0100_0000 != 0;

            //Other shit
            self.sweep_tone_channel.enabled = true;

            self.sweep_tone_channel.sample_producer.sweep.shadow_frequency = self.sweep_tone_channel.frequency;
            self.sweep_tone_channel.sample_producer.sweep.time_sweep = (nr10 & 0b0111_0000) >> 4;

            // turn this bit off
            memory.write(0xFF14, nr14 & 0b0111_1111);
        }

        // See the wave for the calculation this channle freq is 131072/(2048-x) Hz
        self.sweep_tone_channel.timer.cycles_to_tick = (2048 - self.sweep_tone_channel.frequency).wrapping_mul(4);
    }

    fn update_registers(&mut self, memory:&mut dyn Memory){
        memory.write(0xFF1B, self.wave_channel.sound_length);

        let mut control_register = memory.read(0xFF26);
        Self::set_bit(&mut control_register, 3, self.wave_channel.enabled);
    }


    //TODO: delete when refactor this func is copied from the register handler
    fn set_bit(value:&mut u8, bit_number:u8, set:bool){
        let mask = 1 << bit_number;
        if set{
            *value |= mask;
        }
        else{
            let inverse_mask = !mask;
            *value &= inverse_mask;
        }
    }
}

