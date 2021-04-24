pub struct FreqSweep{
    pub enabled:bool,
    pub sweep_counter:u8,
    pub sweep_period:u8, //the original value from the register
    pub sweep_decrease:bool,
    pub sweep_shift:u8,
    pub shadow_frequency:u16
}

impl FreqSweep{
    pub fn reset(&mut self){
        self.sweep_counter = 0;
        self.shadow_frequency = 0;
        self.sweep_shift = 0;
        self.sweep_decrease = false;
        self.enabled = false;
        self.sweep_period = 0;
    }

    pub fn reload_sweep_time(&mut self){
        if self.sweep_period == 0{
            self.sweep_counter = 8;
        }
        else{
            self.sweep_counter = self.sweep_period;
        }
    }

    //Returns true if the overflow check succeded
    pub fn channel_trigger(&mut self, freq:u16){
        self.shadow_frequency = freq;
        self.reload_sweep_time();
        self.enabled = self.sweep_period != 0 || self.sweep_shift != 0;
    }

    pub fn calculate_new_frequency(&self)->u16{
        let new_freq:u16 = self.shadow_frequency >> self.sweep_shift;
        
        return if self.sweep_decrease{
            self.shadow_frequency - new_freq
        }
        else{
            self.shadow_frequency + new_freq
        };
    }

    pub fn check_overflow(freq:u16)->bool{
        freq > 2047
    }
}