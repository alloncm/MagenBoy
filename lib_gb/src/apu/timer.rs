pub struct Timer{
    pub cycles_to_tick:u16,
    cycle_counter:u16
}

impl Timer{
    pub fn new(cycles_to_tick:u16)->Self{
        Timer{
            cycle_counter:0,
            cycles_to_tick:cycles_to_tick
        }
    }

    pub fn cycle(&mut self)->bool{
        self.cycle_counter += 1;
        if self.cycle_counter >= self.cycles_to_tick{
            self.cycle_counter = 0;
            
            return true;
        }

        return false;
    }
}