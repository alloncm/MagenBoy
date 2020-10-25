use super::sample_producer::SampleProducer;

pub struct WaveSampleProducer{
    pub wave_sumples:[u8;16],

    sample_counter:u8
}

impl Default for WaveSampleProducer{
    fn default() -> Self {
        WaveSampleProducer{
            wave_sumples:[0;16],
            sample_counter:0
        }
    }
}

impl SampleProducer for WaveSampleProducer{
    fn produce(&mut self) ->u8 {
        let mut sample = self.wave_sumples[self.sample_counter as usize];

        if self.sample_counter % 2 != 0{
            sample &= 0x0F;
        }
        else{
            sample &= 0xF0;
        }

        self.sample_counter+=1;

        if self.sample_counter >= 32{
            self.sample_counter = 0;
        }

        return sample;
    }
}