use super::channel::Channel;
use super::wave_sample_producer::WaveSampleProducer;
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

                let tick: TickType = self.frame_sequencer.cycle();
            
                self.prepare_wave_channel(memory);
                self.audio_buffer[self.current_t_cycle as usize] = self.wave_channel.get_audio_sample();
                self.update_registers(memory);
            
                self.current_t_cycle += 1;
            }
        }
        else{
            self.current_t_cycle += t_cycles as u32;
        }
    }

    fn update_channels_for_frame_squencer(&mut self ){
        
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
        self.wave_channel.timer.cycles_to_tick = (2048 - freq)*2;
        self.wave_channel.trigger = nr34 & BIT_7_MASK != 0;
        self.wave_channel.length_enable = nr34 & BIT_6_MASK != 0;

        for i in 0..=0xF{
            self.wave_channel.sample_producer.wave_samples[i] = memory.read(0xFF30 + i as u16);
        }
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

