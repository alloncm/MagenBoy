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
    fn produce(&mut self) ->i8 {
        if self.duty_sample_pointer >= 8{
            self.duty_sample_pointer = 0;
        }

        let sample = DUTY_TABLE[self.wave_duty as usize][self.duty_sample_pointer as usize];

        self.duty_sample_pointer += 1;

        return sample;
    }
}