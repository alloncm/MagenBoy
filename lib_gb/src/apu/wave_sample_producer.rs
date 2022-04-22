use super::sample_producer::SampleProducer;

pub struct WaveSampleProducer{
    pub wave_samples:[u8;16],
    pub volume:u8,

    pub nr30_dac_state:bool, // The raw value of the register,
    // saving it casue the in the wave channel the on/off bit controls the dac and not the channel itself, 
    // and in order to turn on the channel we need to make sure that the dac is on

    sample_counter:u8
}

impl Default for WaveSampleProducer{
    fn default() -> Self {
        WaveSampleProducer{
            wave_samples:[0;16],
            volume:0,
            sample_counter:0,
            nr30_dac_state:false,
        }
    }
}

impl SampleProducer for WaveSampleProducer{
    fn produce(&mut self) ->u8 {
        self.sample_counter = (self.sample_counter + 1) % 32;
        
        let mut sample = self.wave_samples[(self.sample_counter/2) as usize];

        if self.sample_counter % 2 == 0{
            sample >>= 4;
        }
        else{
            sample &= 0x0F;
        }

        return self.shift_by_volume(sample);
    }

    fn reset(&mut self) {
        self.volume = 0;
        self.sample_counter = 0;
        self.nr30_dac_state = false;
    }

    fn get_updated_frequency_ticks(&self, freq:u16)->u16 {
        (2048 - freq).wrapping_mul(2)
    }
}

impl WaveSampleProducer{
    pub fn reset_counter(&mut self){
        self.sample_counter = 0;
    }
    
    fn shift_by_volume(&self, sample:u8)->u8{
        match self.volume{
            0=>0,
            1=>sample,
            2=>sample >> 1,
            3=>sample >> 2,
            _=>std::panic!("wave channel volume value is invalid {}", self.volume)
        }
    }
}