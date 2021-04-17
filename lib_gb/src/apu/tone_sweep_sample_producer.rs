use super::{sample_producer::SampleProducer, sound_utils::DUTY_TABLE};
use super::freq_sweep::FreqSweep;
use super::volume_envelop::VolumeEnvlope;

pub struct ToneSweepSampleProducer{
    pub wave_duty:u8,
    pub sweep:FreqSweep,
    pub envelop:VolumeEnvlope,

    duty_sample_pointer:u8,
}

impl Default for ToneSweepSampleProducer{
    fn default()->Self{
        ToneSweepSampleProducer{
            wave_duty:1,
            sweep:FreqSweep{
                enabled:false,
                sweep_shift:0,
                sweep_decrease:false,
                sweep_counter:0,
                shadow_frequency:0,
                sweep_period:0
            },
            envelop:VolumeEnvlope{
                increase_envelope:false,
                number_of_envelope_sweep:0,
                envelop_duration_counter:0
            },
            duty_sample_pointer:0
        }
    }
}

impl SampleProducer for ToneSweepSampleProducer{

    fn produce(&mut self)->i8{
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
        self.sweep.reset();
    }
}

