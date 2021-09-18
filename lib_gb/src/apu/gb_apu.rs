use super::{
    audio_device::*, 
    channel::Channel, 
    frame_sequencer::*,
    freq_sweep::FreqSweep, 
    noise_sample_producer::NoiseSampleProducer, 
    sound_terminal::SoundTerminal,
    square_sample_producer::SquareSampleProducer, 
    wave_sample_producer::WaveSampleProducer,
    sound_utils::NUMBER_OF_CHANNELS
};

pub const AUDIO_BUFFER_SIZE:usize = 0x400;

pub struct GbApu<Device: AudioDevice>{
    pub wave_channel:Channel<WaveSampleProducer>,
    pub sweep_tone_channel:Channel<SquareSampleProducer>,
    pub tone_channel: Channel<SquareSampleProducer>,
    pub noise_channel: Channel<NoiseSampleProducer>,
    pub frame_sequencer: FrameSequencer,
    pub right_terminal:SoundTerminal,
    pub left_terminal:SoundTerminal,
    pub enabled:bool,

    audio_buffer:[StereoSample;AUDIO_BUFFER_SIZE],
    current_t_cycle:u32,
    device:Device,
    last_enabled_state:bool
}

impl<Device: AudioDevice> GbApu<Device>{
    pub fn new(device: Device) -> Self {
        GbApu{
            frame_sequencer:FrameSequencer::default(),
            sweep_tone_channel: Channel::<SquareSampleProducer>::new(SquareSampleProducer::new_with_sweep()),
            wave_channel:Channel::<WaveSampleProducer>::new(WaveSampleProducer::default()),
            tone_channel: Channel::<SquareSampleProducer>::new(SquareSampleProducer::new()),
            noise_channel: Channel::<NoiseSampleProducer>::new(NoiseSampleProducer::default()),
            audio_buffer:[StereoSample::const_defualt(); AUDIO_BUFFER_SIZE],
            current_t_cycle:0,
            device:device,
            right_terminal: SoundTerminal::default(),
            left_terminal: SoundTerminal::default(),
            enabled:false, 
            last_enabled_state: false
        }
    }

    pub fn cycle(&mut self, m_cycles_passed:u8){
        //converting m_cycles to t_cycles
        let t_cycles = m_cycles_passed * 4;

        if self.enabled{
            for _ in 0..t_cycles{   

                let tick = self.frame_sequencer.cycle();
                self.update_channels_for_frame_squencer(tick);
            
                let mut samples:[Sample;NUMBER_OF_CHANNELS] = [SAMPLE_CONSTANT_DEFAULT;NUMBER_OF_CHANNELS];
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
        }
        else{
            for _ in 0..t_cycles{
                self.audio_buffer[self.current_t_cycle as usize] = StereoSample::const_defualt();
                self.current_t_cycle += 1;

                self.push_buffer_if_full();
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
                let sweep:&mut FreqSweep = &mut self.sweep_tone_channel.sample_producer.sweep.as_mut().unwrap();
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
                self.sweep_tone_channel.sample_producer.envelop.tick();
            }
            if self.tone_channel.enabled{
                self.tone_channel.sample_producer.envelop.tick();
            }
            if self.noise_channel.enabled{
                self.noise_channel.sample_producer.envelop.tick();
            }
        }
    }


    pub fn update_sweep_frequency(channel:&mut Channel<SquareSampleProducer>){
        let sweep:&mut FreqSweep = &mut channel.sample_producer.sweep.as_mut().unwrap();
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
