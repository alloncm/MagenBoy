pub struct FreqSweep{
    pub time_sweep:u8,
    pub sweep_decrease:bool,
    pub sweep_shift:u8,
    pub shadow_frequency:u16
}

impl FreqSweep{
    pub fn reset(&mut self){
        self.time_sweep = 0;
        self.shadow_frequency = 0;
        self.sweep_shift = 0;
        self.sweep_decrease = false;
    }
}