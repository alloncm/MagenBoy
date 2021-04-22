use super::{
    audio_device::*, 
    channel::Channel, 
    frame_sequencer::*,
    freq_sweep::FreqSweep, 
    noise_sample_producer::NoiseSampleProducer, 
    sound_terminal::SoundTerminal, 
    tone_sample_producer::ToneSampleProducer, 
    tone_sweep_sample_producer::ToneSweepSampleProducer, 
    wave_sample_producer::WaveSampleProducer
};
use crate::{
    mmu::memory::UnprotectedMemory, 
    utils::{bit_masks::set_bit_u8, memory_registers::{NR10_REGISTER_ADDRESS, NR52_REGISTER_ADDRESS}}
};

pub const AUDIO_BUFFER_SIZE:usize = 0x400;

pub struct GbApu<Device: AudioDevice>{
    pub wave_channel:Channel<WaveSampleProducer>,
    pub sweep_tone_channel:Channel<ToneSweepSampleProducer>,
    pub tone_channel: Channel<ToneSampleProducer>,
    pub noise_channel: Channel<NoiseSampleProducer>,

    pub frame_sequencer: FrameSequencer,

    audio_buffer:[Sample;AUDIO_BUFFER_SIZE],
    current_t_cycle:u32,
    device:Device,
    pub right_terminal:SoundTerminal,
    pub left_terminal:SoundTerminal,
    pub enabled:bool,

    last_enabled_state:bool
}

impl<Device: AudioDevice> GbApu<Device>{
    pub fn new(device: Device) -> Self {
        GbApu{
            frame_sequencer:FrameSequencer::default(),
            sweep_tone_channel: Channel::<ToneSweepSampleProducer>::new(),
            wave_channel:Channel::<WaveSampleProducer>::new(),
            tone_channel: Channel::<ToneSampleProducer>::new(),
            noise_channel: Channel::<NoiseSampleProducer>::new(),
            audio_buffer:[Sample{left_sample:0.0, right_sample:0.0}; AUDIO_BUFFER_SIZE],
            current_t_cycle:0,
            device:device,
            right_terminal: SoundTerminal::default(),
            left_terminal: SoundTerminal::default(),
            enabled:false, 
            last_enabled_state: false
        }
    }

    pub fn cycle(&mut self, memory:&mut impl UnprotectedMemory, m_cycles_passed:u8){
        //converting m_cycles to t_cycles
        let t_cycles = m_cycles_passed * 4;

        if self.enabled{
            for _ in 0..t_cycles{   

                let tick = self.frame_sequencer.cycle();
                self.update_channels_for_frame_squencer(tick);
            
                let mut samples:[f32;4] = [0.0;4];
                samples[0] = self.sweep_tone_channel.get_audio_sample();
                samples[1] = self.tone_channel.get_audio_sample();
                samples[2] = self.wave_channel.get_audio_sample();
                samples[3] = self.noise_channel.get_audio_sample();

                let left_sample = self.left_terminal.mix_terminal_samples(&samples);
                let right_sample = self.right_terminal.mix_terminal_samples(&samples);
            
                self.audio_buffer[self.current_t_cycle as usize].left_sample = left_sample;
                self.audio_buffer[self.current_t_cycle as usize].right_sample = right_sample;
                
                self.current_t_cycle += 1;

                self.push_buffer_if_full();
            }

            self.update_registers(memory);
        }
        else{
            for _ in 0..t_cycles{
                self.audio_buffer[self.current_t_cycle as usize] = Sample{right_sample:0.0, left_sample:0.0};
                self.current_t_cycle += 1;

                self.push_buffer_if_full();
            }

            //Reseting the apu state
            for i in NR10_REGISTER_ADDRESS..NR52_REGISTER_ADDRESS{
                memory.write_unprotected(i, 0);
            }

            self.tone_channel.reset();
            self.sweep_tone_channel.reset();
            self.wave_channel.reset();
            self.noise_channel.reset();
            self.frame_sequencer.reset();
        }            

        self.last_enabled_state = self.enabled;
    }

    fn push_buffer_if_full(&mut self){
        if self.current_t_cycle as usize >= AUDIO_BUFFER_SIZE{
            self.current_t_cycle = 0;
            self.device.push_buffer(&self.audio_buffer);
        }
    }

    fn update_channels_for_frame_squencer(&mut self, tick:TickType){
        if tick.frequency_sweep{
            if self.sweep_tone_channel.enabled{
                let sweep:&mut FreqSweep = &mut self.sweep_tone_channel.sample_producer.sweep;
                if sweep.sweep_counter > 0{
                    sweep.sweep_counter -= 1;
                }
                if sweep.sweep_counter == 0{
                    sweep.reload_sweep_time();

                    Self::update_sweep_frequency(&mut self.sweep_tone_channel);
                }
            }
        }
        if tick.length_counter{
            self.sweep_tone_channel.update_length_register();
            self.wave_channel.update_length_register();
            self.noise_channel.update_length_register();
            self.tone_channel.update_length_register();
        }
        if tick.volume_envelope{
            if self.sweep_tone_channel.enabled{
                self.sweep_tone_channel.sample_producer.envelop.tick(&mut self.sweep_tone_channel.current_volume);
            }
            if self.tone_channel.enabled{
                self.tone_channel.sample_producer.envelop.tick(&mut self.tone_channel.current_volume);
            }
            if self.noise_channel.enabled{
                self.noise_channel.sample_producer.envelop.tick(&mut self.noise_channel.current_volume);
            }
        }
    }

    fn update_registers(&mut self, memory:&mut impl UnprotectedMemory){

        let mut control_register = memory.read_unprotected(0xFF26);
        set_bit_u8(&mut control_register, 3, self.noise_channel.enabled && self.noise_channel.length_enable && self.noise_channel.sound_length != 0);
        set_bit_u8(&mut control_register, 2, self.wave_channel.enabled && self.wave_channel.length_enable && self.wave_channel.sound_length != 0);
        set_bit_u8(&mut control_register, 1, self.tone_channel.enabled && self.tone_channel.length_enable && self.tone_channel.sound_length != 0);
        set_bit_u8(&mut control_register, 0, self.sweep_tone_channel.enabled && self.sweep_tone_channel.length_enable && self.sweep_tone_channel.sound_length != 0);

        memory.write_unprotected(NR52_REGISTER_ADDRESS, control_register);
    }

    pub fn update_sweep_frequency(channel:&mut Channel<ToneSweepSampleProducer>){
        let sweep:&mut FreqSweep = &mut channel.sample_producer.sweep;
        if sweep.enabled && sweep.sweep_period != 0{
            //calculate a new freq
            let mut new_freq = sweep.calculate_new_frequency();
            if FreqSweep::check_overflow(new_freq){
                channel.enabled = false;
            }
    
            //load shadow and freq register with new value
            if new_freq <= 2047 && sweep.sweep_shift > 0{
                sweep.shadow_frequency = new_freq;
                channel.frequency = new_freq;
    
                //Another overflow check
                new_freq = sweep.calculate_new_frequency();
                if FreqSweep::check_overflow(new_freq){
                    channel.enabled = false;
                }
            }
        }
    }
}
