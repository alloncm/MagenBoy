use crate::utils::memory_registers::*;

const IO_PORTS_SIZE:usize = 0x80;

const IO_PORTS_MEMORY_OFFSET:u16 = 0xFF00;

const DIV_REGISTER_INDEX:u16 = DIV_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET;
const TAC_REGISTER_INDEX:u16 = TAC_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET;
const STAT_REGISTER_INDEX:u16 = STAT_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET;
const JOYP_REGISTER_INDEX:u16 = JOYP_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET;

pub struct IoPorts{
    system_counter:u16,
    ports:[u8;IO_PORTS_SIZE]
}

impl IoPorts{
    pub fn read(&self, address:u16)->u8{
        self.ports[address as usize]
    }

    pub fn write(&mut self, address:u16, mut value:u8){
        match address{
            DIV_REGISTER_INDEX=>{
                value = 0;
                self.system_counter = 0;
            },
            TAC_REGISTER_INDEX=> value &= 0b111,
            STAT_REGISTER_INDEX => value = (value >> 2) << 2,
            JOYP_REGISTER_INDEX => {
                let joypad_value = self.ports[JOYP_REGISTER_INDEX as usize];
                value = (joypad_value & 0xF) | (value & 0xF0);
            },
            _=>{}
        }

        self.ports[address as usize] = value;
    } 
    
    pub fn increase_system_counter(&mut self){
        self.system_counter = self.system_counter.wrapping_add(4);
        self.ports[DIV_REGISTER_INDEX as usize] = (self.system_counter >> 8) as u8;
    }

    pub fn write_unprotected(&mut self, address:u16, value:u8){
        self.ports[address as usize] = value;
    }
}

impl Default for IoPorts{
    fn default()->Self{
        let mut io_ports = IoPorts{
            ports:[0;IO_PORTS_SIZE],
            system_counter: 0
        };

        //joypad register initiall value
        io_ports.ports[JOYP_REGISTER_INDEX as usize] = 0xFF;

        io_ports
    }
}