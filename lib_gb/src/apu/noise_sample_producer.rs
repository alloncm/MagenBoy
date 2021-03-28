use super::{sample_producer::SampleProducer, volume_envelop::VolumeEnvlope};

pub struct NoiseSampleProducer{
    pub envelop: VolumeEnvlope,
}

impl Default for NoiseSampleProducer{
    fn default() -> Self {
        Self{
            envelop:VolumeEnvlope{
                envelop_duration_counter:0,
                increase_envelope:false,
                number_of_envelope_sweep:0
            }
        }
    }
}

impl SampleProducer for NoiseSampleProducer{
    fn produce(&mut self)->i8 {
        return 0;
    }

    fn reset(&mut self) {
        
    }
}