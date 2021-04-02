use super::sample_producer::SampleProducer;

pub struct WaveSampleProducer{
    pub wave_samples:[u8;16],
    pub volume:u8,

    sample_counter:u8
}

impl Default for WaveSampleProducer{
    fn default() -> Self {
        WaveSampleProducer{
            wave_samples:[0;16],
            volume:0,
            sample_counter:0
        }
    }
}

impl SampleProducer for WaveSampleProducer{
    fn produce(&mut self) ->i8 {
        let mut sample = self.wave_samples[(self.sample_counter/2) as usize];

        if self.sample_counter % 2 != 0{
            sample &= 0x0F;
        }
        else{
            sample &= 0xF0;
            sample >>=4;
        }

        self.sample_counter+=1;

        if self.sample_counter >= 32{
            self.sample_counter = 0;
        }

        return self.shift_by_volume(sample) as i8;
    }

    fn reset(&mut self) {
        self.volume = 0;
        self.sample_counter = 0;
    }

    fn get_updated_frequency_ticks(freq:u16)->u16 {
        (2048 - freq).wrapping_mul(2)
    }
}

impl WaveSampleProducer{
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