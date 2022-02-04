pub struct VolumeEnvlope{
    pub volume:u8,
    pub current_volume:u8,

    pub increase_envelope:bool,
    pub number_of_envelope_sweep:u8,
    pub envelop_duration_counter:u8,

    pub nrx2_register:u8, // The original register raw value
}

impl VolumeEnvlope{
    pub fn reset(&mut self){
        self.increase_envelope = false;
        self.number_of_envelope_sweep = 0;
        self.envelop_duration_counter = 0;
        self.nrx2_register = 0;
    }

    pub fn tick(&mut self){
        if self.number_of_envelope_sweep != 0 {
            if self.envelop_duration_counter > 0{
                self.envelop_duration_counter -= 1;
            }

            if self.envelop_duration_counter == 0{
                self.envelop_duration_counter = self.number_of_envelope_sweep;
                if (self.current_volume < 0xF && self.increase_envelope) || (self.current_volume > 0 && !self.increase_envelope){
                    if self.increase_envelope{
                        self.current_volume += 1;
                    }
                    else{
                        self.current_volume -= 1;
                    }
                }
            }
        }
    }
}

impl Default for VolumeEnvlope{
    fn default() -> Self {
        VolumeEnvlope{
            current_volume:0,
            volume:0,
            increase_envelope:false,
            number_of_envelope_sweep:0,
            envelop_duration_counter:0,
            nrx2_register:0
        }
    }
}