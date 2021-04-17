use super::{sample_producer::SampleProducer, volume_envelop::VolumeEnvlope};

pub struct NoiseSampleProducer{
    pub envelop: VolumeEnvlope,
    pub lfsr:u16,
    pub divisor_to_shift:u8,
    pub counter_width:bool,
    pub base_divisor:u8
}

impl Default for NoiseSampleProducer{
    fn default() -> Self {
        Self{
            envelop:VolumeEnvlope{
                envelop_duration_counter:0,
                increase_envelope:false,
                number_of_envelope_sweep:0
            },
            base_divisor:0,
            counter_width:false,
            divisor_to_shift:0,
            lfsr:0
        }
    }
}

impl SampleProducer for NoiseSampleProducer{
    fn produce(&mut self)->i8 {
        return 0;
    }

    fn reset(&mut self) {
        
    }

    fn get_updated_frequency_ticks(&self,_freq:u16)->u16 {
        0
    }
}