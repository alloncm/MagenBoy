pub struct VolumeEnvlope{
    pub increase_envelope:bool,
    pub number_of_envelope_sweep:u8,
    pub envelop_duration_counter:u8
}

impl VolumeEnvlope{
    pub fn reset(&mut self){
        self.increase_envelope = false;
        self.number_of_envelope_sweep = 0;
        self.envelop_duration_counter = 0;
    }
}