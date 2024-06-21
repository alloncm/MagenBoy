use super::{
    audio_device::*, 
    channel::Channel, 
    frame_sequencer::*,
    freq_sweep::FreqSweep, 
    noise_sample_producer::NoiseSampleProducer, 
    sound_terminal::SoundTerminal,
    square_sample_producer::SquareSampleProducer, 
    wave_sample_producer::WaveSampleProducer,
    NUMBER_OF_CHANNELS
};

pub struct GbApu<Device: AudioDevice>{
    pub wave_channel:Channel<WaveSampleProducer>,
    pub sweep_tone_channel:Channel<SquareSampleProducer>,
    pub tone_channel: Channel<SquareSampleProducer>,
    pub noise_channel: Channel<NoiseSampleProducer>,
    pub frame_sequencer: FrameSequencer,
    pub right_terminal:SoundTerminal,
    pub left_terminal:SoundTerminal,
    pub enabled:bool,

    pub nr50_register:u8, // The register orignal raw value
    pub nr51_register:u8, // The register orignal raw value

    audio_buffer:[StereoSample;BUFFER_SIZE],
    current_m_cycle:u32,
    device:Device,
}

impl<Device: AudioDevice> GbApu<Device>{
    pub fn new(device: Device) -> Self {
        GbApu{
            frame_sequencer:FrameSequencer::default(),
            sweep_tone_channel: Channel::<SquareSampleProducer>::new(SquareSampleProducer::new_with_sweep()),
            wave_channel:Channel::<WaveSampleProducer>::new(WaveSampleProducer::default()),
            tone_channel: Channel::<SquareSampleProducer>::new(SquareSampleProducer::new()),
            noise_channel: Channel::<NoiseSampleProducer>::new(NoiseSampleProducer::default()),
            audio_buffer:crate::utils::create_array(StereoSample::const_defualt),
            current_m_cycle:0,
            device:device,
            right_terminal: SoundTerminal::default(),
            left_terminal: SoundTerminal::default(),
            enabled:false,
            nr50_register:0,
            nr51_register:0,
        }
    }

    pub fn cycle(&mut self, m_cycles_passed:u32)->u32{
        if self.enabled{
            for _ in 0..m_cycles_passed{   

                let tick = self.frame_sequencer.cycle();
                self.update_channels_for_frame_squencer(tick);
            
                let mut samples:[Sample;NUMBER_OF_CHANNELS] = [DEFAULT_SAPMPLE ; NUMBER_OF_CHANNELS];
                samples[0] = self.sweep_tone_channel.get_audio_sample();
                samples[1] = self.tone_channel.get_audio_sample();
                samples[2] = self.wave_channel.get_audio_sample();
                samples[3] = self.noise_channel.get_audio_sample();

                let left_sample = self.left_terminal.mix_terminal_samples(&samples);
                let right_sample = self.right_terminal.mix_terminal_samples(&samples);
            
                self.audio_buffer[self.current_m_cycle as usize].left_sample = left_sample;
                self.audio_buffer[self.current_m_cycle as usize].right_sample = right_sample;
                
                self.current_m_cycle += 1;

                self.push_buffer_if_full();
            }
        }
        else{
            for _ in 0..m_cycles_passed{
                self.audio_buffer[self.current_m_cycle as usize] = StereoSample::const_defualt();
                self.current_m_cycle += 1;

                self.push_buffer_if_full();
            }
        }

        return BUFFER_SIZE as u32 - self.current_m_cycle;
    }

    pub fn reset(&mut self){
        self.tone_channel.reset();
        self.sweep_tone_channel.reset();
        self.wave_channel.reset();
        self.noise_channel.reset();
        self.frame_sequencer.reset();
        self.nr50_register = 0;
        self.nr51_register = 0;
    }

    fn push_buffer_if_full(&mut self){
        if self.current_m_cycle as usize >= BUFFER_SIZE{
            self.current_m_cycle = 0;
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
