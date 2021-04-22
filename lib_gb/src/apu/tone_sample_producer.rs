use super::{sample_producer::SampleProducer, sound_utils::DUTY_TABLE, volume_envelop::VolumeEnvlope};

pub struct ToneSampleProducer{
    pub wave_duty:u8,
    pub envelop:VolumeEnvlope,

    duty_sample_pointer:u8,
}


impl Default for ToneSampleProducer{
    fn default()->Self{
        ToneSampleProducer{
            wave_duty:1,
            envelop:VolumeEnvlope{
                increase_envelope:false,
                number_of_envelope_sweep:0,
                envelop_duration_counter:0
            },
            duty_sample_pointer:0
        }
    }
}

impl SampleProducer for ToneSampleProducer{
    fn produce(&mut self) ->u8 {
        self.duty_sample_pointer = (self.duty_sample_pointer + 1) % 8;

        let sample = DUTY_TABLE[self.wave_duty as usize][self.duty_sample_pointer as usize];

        return sample;
    }

    fn reset(&mut self) {
        self.wave_duty = 0;
        self.envelop.reset();
        self.duty_sample_pointer = 0;
    }

    fn get_updated_frequency_ticks(&self, freq:u16)->u16 {
        (2048 - freq).wrapping_mul(4)
    }
}