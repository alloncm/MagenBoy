pub struct Timer{
    cycles_to_tick:u16,
    cycle_counter:u16
}

// By deviding by 4 (shifting right 2) Im losing precison in favor of performance
impl Timer{
    pub fn new(cycles_to_tick:u16)->Self{
        Timer{
            cycle_counter:0,
            cycles_to_tick:cycles_to_tick >> 2
        }
    }

    // This function is a hot spot for the APU, almost every component uses the timer
    #[inline]
    pub fn cycle(&mut self)->bool{
        if self.cycles_to_tick != 0{
            // The calculation used to be this:
            // self.cycle_counter = (self.cycle_counter + 1) % self.cycles_to_tick;
            // After benching with a profiler I found that those 2 lines are much faster, probably cause there is no division here
            self.cycle_counter += 1;
            self.cycle_counter = (self.cycle_counter != self.cycles_to_tick) as u16 * self.cycle_counter;
            return self.cycle_counter == 0;
        }

        return false;
    }

    pub fn update_cycles_to_tick(&mut self, cycles_to_tick:u16){
        self.cycles_to_tick = cycles_to_tick >> 2;
        self.cycle_counter = 0;
    }
}