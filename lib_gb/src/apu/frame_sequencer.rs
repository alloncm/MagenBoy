use super::timer::Timer;

pub struct TickType{
    length_counter:bool,
    volume_envelope:bool,
    frequency_sweep:bool
}

pub struct FrameSequencer{
    timer:Timer,
    length_counter_cycles:u32,
    volume_envelope_cycles:u32,
    frequency_sweep_cycles:u32
}

impl FrameSequencer{
    pub fn cycle(&mut self)->TickType{
        let mut tick = TickType{
            frequency_sweep:false,
            volume_envelope: false,
            length_counter: false
        };

        if self.timer.cycle(){
            self.length_counter_cycles += 1;
            self.volume_envelope_cycles += 1;
            self.frequency_sweep_cycles += 1;

            if self.length_counter_cycles >= 2{
                self.length_counter_cycles = 0;
                tick.length_counter = true;
            }
            if self.volume_envelope_cycles >=8{
                self.volume_envelope_cycles = 0;
                tick.volume_envelope = true;
            }
            if self.frequency_sweep_cycles  >= 4{
                self.frequency_sweep_cycles = 0;
                tick.frequency_sweep = true;
            }
        }

        return tick;
    }
}