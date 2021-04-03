use crate::utils::memory_registers::*;
use super::memory::{UnprotectedMemory, Memory};

pub const IO_PORTS_SIZE:usize = 0x80;

pub const IO_PORTS_MEMORY_OFFSET:u16 = 0xFF00;

macro_rules! io_port_index{
    ($name:ident, $reg_address:expr) => {
        const $name:u16 = $reg_address - IO_PORTS_MEMORY_OFFSET;
    };
}
macro_rules! pub_io_port_index{
    ($name:ident, $reg_address:expr) => {
        pub const $name:u16 = $reg_address - IO_PORTS_MEMORY_OFFSET;
    };
}

pub_io_port_index!(DIV_REGISTER_INDEX, DIV_REGISTER_ADDRESS);

io_port_index!(TAC_REGISTER_INDEX, TAC_REGISTER_ADDRESS);
io_port_index!(STAT_REGISTER_INDEX, STAT_REGISTER_ADDRESS);
io_port_index!(JOYP_REGISTER_INDEX, JOYP_REGISTER_ADDRESS);
io_port_index!(NR10_REGISTER_INDEX, NR10_REGISTER_ADDRESS);
io_port_index!(NR11_REGISTER_INDEX, NR11_REGISTER_ADDRESS);
io_port_index!(NR13_REGISTER_INDEX, NR13_REGISTER_ADDRESS);
io_port_index!(NR14_REGISTER_INDEX, NR14_REGISTER_ADDRESS);
io_port_index!(NR21_REGISTER_INDEX, NR21_REGISTER_ADDRESS);
io_port_index!(NR23_REGISTER_INDEX, NR23_REGISTER_ADDRESS);
io_port_index!(NR24_REGISTER_INDEX, NR24_REGISTER_ADDRESS);
io_port_index!(NR30_REGISTER_INDEX, NR30_REGISTER_ADDRESS);
io_port_index!(NR31_REGISTER_INDEX, NR31_REGISTER_ADDRESS);
io_port_index!(NR32_REGISTER_INDEX, NR32_REGISTER_ADDRESS);
io_port_index!(NR33_REGISTER_INDEX, NR33_REGISTER_ADDRESS);
io_port_index!(NR34_REGISTER_INDEX, NR34_REGISTER_ADDRESS);
io_port_index!(NR41_REGISTER_INDEX, NR41_REGISTER_ADDRESS);
io_port_index!(NR44_REGISTER_INDEX, NR44_REGISTER_ADDRESS);
io_port_index!(NR52_REGISTER_INDEX, NR52_REGISTER_ADDRESS);

pub struct IoPorts{
    ports:[u8;IO_PORTS_SIZE], 
    ports_cycle_trigger:[bool; IO_PORTS_SIZE]
}

impl Memory for IoPorts{
    fn read(&self, address:u16)->u8{
        let value = self.ports[address as usize];
        match address{
            NR10_REGISTER_INDEX=> value | 0b1000_0000,
            NR11_REGISTER_INDEX=> value | 0b0011_1111,
            NR13_REGISTER_INDEX=> 0xFF,
            NR14_REGISTER_INDEX=> value | 0b1011_1111,
            0x15 => 0xFF,
            NR21_REGISTER_INDEX=> value | 0b0011_1111,
            NR23_REGISTER_INDEX=> 0xFF,
            NR24_REGISTER_INDEX=> value | 0b1011_1111,
            NR30_REGISTER_INDEX=> value | 0b0111_1111,
            NR31_REGISTER_INDEX=> value | 0xFF,
            NR32_REGISTER_INDEX=> value | 0b1001_1111,
            NR33_REGISTER_INDEX=> value | 0xFF,
            NR34_REGISTER_INDEX=> value | 0b1011_1111,
            0x1F => 0xFF,
            NR41_REGISTER_INDEX=> 0xFF,
            NR44_REGISTER_INDEX=> value | 0b1011_1111,
            NR52_REGISTER_INDEX=> value | 0b0111_0000,
            0x27..=0x2F => 0xFF,//Not used
            TAC_REGISTER_INDEX=> value & 0b111,
            STAT_REGISTER_INDEX => (value >> 2) << 2,
            JOYP_REGISTER_INDEX => {
                let joypad_value = self.ports[JOYP_REGISTER_INDEX as usize];
                (joypad_value & 0xF) | (value & 0xF0)
            },
            _=>value
        }
    }

    fn write(&mut self, address:u16, mut value:u8){
        match address{
            DIV_REGISTER_INDEX=> value = 0,
            TAC_REGISTER_INDEX=> value &= 0b111,
            STAT_REGISTER_INDEX => value = (value >> 2) << 2,
            JOYP_REGISTER_INDEX => {
                let joypad_value = self.ports[JOYP_REGISTER_INDEX as usize];
                value = (joypad_value & 0xF) | (value & 0xF0);
            }
            _=>{}
        }
        
        self.ports_cycle_trigger[address as usize] = true;

        self.ports[address as usize] = value;
    }
}

impl UnprotectedMemory for IoPorts{
    fn write_unprotected(&mut self, address:u16, value:u8){
        self.ports[address as usize] = value;
    }

    fn read_unprotected(&self, address:u16) ->u8 {
        self.ports[address as usize]
    }
}

impl IoPorts{
    pub fn get_ports_cycle_trigger(&mut self)->&mut [bool; IO_PORTS_SIZE]{
        return &mut self.ports_cycle_trigger;
    }

    pub fn clear_io_ports_triggers(&mut self){
        unsafe{std::ptr::write_bytes::<bool>(self.ports_cycle_trigger.as_mut_ptr(), 0, IO_PORTS_SIZE);}
    }
}

impl Default for IoPorts{
    fn default()->Self{
        let mut io_ports = IoPorts{
            ports:[0;IO_PORTS_SIZE],
            ports_cycle_trigger:[false;IO_PORTS_SIZE]
        };

        //joypad register initiall value
        io_ports.ports[JOYP_REGISTER_INDEX as usize] = 0xFF;

        io_ports
    }
}