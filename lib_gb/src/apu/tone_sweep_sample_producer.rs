use super::sample_producer::SampleProducer;

pub struct ToneSweepSampleProducer{
    pub wave_duty:u8,

    clocks_counter:u8,
    duty_counter:u8

}

impl Default for ToneSweepSampleProducer{
    fn default()->Self{
        ToneSweepSampleProducer{
            wave_duty:1,
            clocks_counter:0,
            duty_counter:0
        }
    }
}

impl SampleProducer for ToneSweepSampleProducer{

    fn produce(&mut self)->u8{
        if self.clocks_counter > 8{
            self.clocks_counter = 0;
        }

        if self.duty_counter <= self.wave_duty{
            self.duty_counter += 1;
            return 15;
        }

        self.clocks_counter += 1;

        return 0;
    }
}

