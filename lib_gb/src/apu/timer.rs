pub struct Timer {
    cycles_to_tick: u16,
    cycle_counter: u16,
}

impl Timer {
    pub fn new(cycles_to_tick: u16) -> Self {
        Timer {
            cycle_counter: 0,
            cycles_to_tick: cycles_to_tick,
        }
    }

    pub fn cycle(&mut self) -> bool {
        if self.cycles_to_tick != 0 {
            self.cycle_counter += 1;
            if self.cycle_counter >= self.cycles_to_tick {
                self.cycle_counter = 0;

                return true;
            }
        }

        return false;
    }

    pub fn update_cycles_to_tick(&mut self, cycles_to_tick: u16) {
        self.cycles_to_tick = cycles_to_tick;
        self.cycle_counter = 0;
    }
}
