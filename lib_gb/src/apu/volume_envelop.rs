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

    pub fn tick(&mut self, volume:&mut u8){
        if self.number_of_envelope_sweep != 0 {
            if self.envelop_duration_counter > 0{
                self.envelop_duration_counter -= 1;
            }

            if self.envelop_duration_counter == 0{
                self.envelop_duration_counter = self.number_of_envelope_sweep;
                if (*volume < 0xF && self.increase_envelope) || (*volume > 0 && !self.increase_envelope){
                    if self.increase_envelope{
                        *volume += 1;
                    }
                    else{
                        *volume -= 1;
                    }
                }
            }
        }
    }
}