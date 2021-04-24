use super::{sample_producer::SampleProducer, sound_utils::DUTY_TABLE};
use super::freq_sweep::FreqSweep;
use super::volume_envelop::VolumeEnvlope;

pub struct SquareSampleProducer{
    pub wave_duty:u8,
    pub sweep:Option<FreqSweep>,
    pub envelop:VolumeEnvlope,

    duty_sample_pointer:u8,
}

impl SquareSampleProducer{
    pub fn new_with_sweep()->Self{
        SquareSampleProducer{
            wave_duty:1,
            sweep:Option::Some(FreqSweep{
                enabled:false,
                sweep_shift:0,
                sweep_decrease:false,
                sweep_counter:0,
                shadow_frequency:0,
                sweep_period:0
            }),
            envelop:VolumeEnvlope{
                increase_envelope:false,
                number_of_envelope_sweep:0,
                envelop_duration_counter:0
            },
            duty_sample_pointer:0
        }
    }

    pub fn new()->Self{
        SquareSampleProducer{
            wave_duty:1,
            sweep:Option::None,
            envelop:VolumeEnvlope{
                increase_envelope:false,
                number_of_envelope_sweep:0,
                envelop_duration_counter:0
            },
            duty_sample_pointer:0
        }
    }
}

impl SampleProducer for SquareSampleProducer{

    fn produce(&mut self)->u8{
        self.duty_sample_pointer = (self.duty_sample_pointer + 1) % 8;

        let sample = DUTY_TABLE[self.wave_duty as usize][self.duty_sample_pointer as usize];

        return sample;
    }

    fn get_updated_frequency_ticks(&self, freq:u16) ->u16 {
        (2048 - freq).wrapping_mul(4)
    }

    fn reset(&mut self) {
        self.wave_duty = 0;
        self.duty_sample_pointer = 0;

        self.envelop.reset();
        if let Some(sweep) = self.sweep.as_mut(){
            sweep.reset();   
        }
    }
}

