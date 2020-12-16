use super::sample_producer::SampleProducer;
use super::freq_sweep::FreqSweep;
use super::volume_envelop::VolumeEnvlope;

pub struct ToneSweepSampleProducer{
    pub wave_duty:u8,
    pub sweep:FreqSweep,
    pub envelop:VolumeEnvlope,

    clocks_counter:u8,
    duty_counter:u8

}

impl Default for ToneSweepSampleProducer{
    fn default()->Self{
        ToneSweepSampleProducer{
            wave_duty:1,
            sweep:FreqSweep{
                sweep_shift:0,
                sweep_decrease:false,
                time_sweep:0,
                shadow_frequency:0
            },
            envelop:VolumeEnvlope{
                increase_envelope:false,
                number_of_envelope_sweep:0
            },
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

