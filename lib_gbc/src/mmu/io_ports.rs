
const IO_PORTS_SIZE:usize = 0x80;

const DIVIDER_REGISTER_INDEX:u16 = 0x04;
const TIMER_CONTROL_REGISTER_INDEX:u16 = 0x04;

pub struct IoPorts{
    system_counter:u16,
    ports:[u8;IO_PORTS_SIZE]
}

impl IoPorts{
    pub fn read(&self, address:u16)->u8{
        self.ports[address as usize]
    }

    pub fn write(&mut self, address:u16, mut value:u8){
        if address == DIVIDER_REGISTER_INDEX{
            value = 0;
            self.system_counter = 0;
        }
        else if address == TIMER_CONTROL_REGISTER_INDEX{
            value &= 111;
        }

        self.ports[address as usize] = value;
    } 
    
    pub fn increase_system_counter(&mut self){
        self.system_counter = self.system_counter.wrapping_add(1);
        self.ports[DIVIDER_REGISTER_INDEX as usize] = (self.system_counter >> 8) as u8;
    }
}

impl Default for IoPorts{
    fn default()->Self{
        IoPorts{
            ports:[0;IO_PORTS_SIZE],
            system_counter: 0
        }
    }
}