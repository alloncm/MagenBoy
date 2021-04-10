use super::timer::Timer;

pub struct TickType{
    pub length_counter:bool,
    pub volume_envelope:bool,
    pub frequency_sweep:bool
}

pub struct FrameSequencer{
    timer:Timer,
    counter:u8
}

impl Default for FrameSequencer{
    fn default() -> Self {
        FrameSequencer{
            timer: Timer::new(8192),
            counter:0
        }
    }
}

impl FrameSequencer{
    pub fn cycle(&mut self)->TickType{
        let mut tick = TickType{
            frequency_sweep:false,
            volume_envelope: false,
            length_counter: false
        };

        if self.timer.cycle(){
            self.counter %= 8;

            match self.counter{
                0 | 4 => tick.length_counter = true,
                2 | 6 => {
                    tick.length_counter = true;
                    tick.frequency_sweep = true;
                },
                7 => tick.volume_envelope = true,
                1 | 3 | 5 => {},
                _=>std::panic!("wrong modolu operation in the fs")
            }

            self.counter += 1;
        }

        return tick;
    }

    pub fn should_next_step_clock_length(&self)->bool{
        self.counter % 2 == 0
    }

    pub fn reset(&mut self){
        self.timer.update_cycles_to_tick(8192);
        self.counter = 0;
    }
}