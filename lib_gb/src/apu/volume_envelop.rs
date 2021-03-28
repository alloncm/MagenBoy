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
        if self.number_of_envelope_sweep > 0 {
            self.envelop_duration_counter += 1;

            if self.envelop_duration_counter == self.number_of_envelope_sweep{
                if self.increase_envelope{
                    let new_vol = *volume + 1;
                    *volume = std::cmp::min(new_vol, 0xF);
                }
                else{
                    let new_vol = *volume as i8 - 1;
                    *volume = std::cmp::max::<i8>(new_vol, 0) as u8;
                }

                self.envelop_duration_counter = 0;
            }
        }
    }
}