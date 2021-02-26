use crate::utils::memory_registers::*;
use super::memory::{UnprotectedMemory, Memory};

pub const IO_PORTS_SIZE:usize = 0x80;

const IO_PORTS_MEMORY_OFFSET:u16 = 0xFF00;

pub const DIV_REGISTER_INDEX:u16 = DIV_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET;
const TAC_REGISTER_INDEX:u16 = TAC_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET;
const STAT_REGISTER_INDEX:u16 = STAT_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET;
const JOYP_REGISTER_INDEX:u16 = JOYP_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET;
const DMA_REGISTER_INDEX:u16 = DMA_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET;

pub struct IoPorts{
    //pub system_counter:u16,
    //pub dma_trasfer_trigger:Option<AccessBus>,
    ports:[u8;IO_PORTS_SIZE], 
    ports_cycle_trigger:[bool; IO_PORTS_SIZE]
}

impl Memory for IoPorts{
    fn read(&self, address:u16)->u8{
        let mut value = self.ports[address as usize];
        match address{
            TAC_REGISTER_INDEX=> value &= 0b111,
            STAT_REGISTER_INDEX => value = (value >> 2) << 2,
            JOYP_REGISTER_INDEX => {
                let joypad_value = self.ports[JOYP_REGISTER_INDEX as usize];
                value = (joypad_value & 0xF) | (value & 0xF0);
            },
            _=>{}
        }

        if address == 0x05 && value != 0{
            //println!("read TIMA register: {}", value);
        }

        value
    }

    fn write(&mut self, address:u16, mut value:u8){
        match address{
            DIV_REGISTER_INDEX=>{
                value = 0;
            },
            TAC_REGISTER_INDEX=> value &= 0b111,
            STAT_REGISTER_INDEX => value = (value >> 2) << 2,
            JOYP_REGISTER_INDEX => {
                let joypad_value = self.ports[JOYP_REGISTER_INDEX as usize];
                value = (joypad_value & 0xF) | (value & 0xF0);
            },
            _=>{}
        }
        if address == 0x07 {
            println!("write TAC register: {}", value);
        }
        self.ports_cycle_trigger[address as usize] = true;

        self.ports[address as usize] = value;
    }
}

impl UnprotectedMemory for IoPorts{
    fn write_unprotected(&mut self, address:u16, value:u8){
        if address == 0x05 {
            //println!("unptorected write TIMA register: {}", value);
        }
        self.ports[address as usize] = value;
    }

    fn read_unprotected(&self, address:u16) ->u8 {
        if address == 0x05 && self.ports[address as usize]!=0{
            //println!("unprotected read TIMA register: {}", self.ports[address as usize]);
        }
        self.ports[address as usize]
    }
}

impl IoPorts{
    pub fn get_ports_cycle_trigger(&mut self)->&mut [bool; IO_PORTS_SIZE]{
        return &mut self.ports_cycle_trigger;
    }
}

impl Default for IoPorts{
    fn default()->Self{
        let mut io_ports = IoPorts{
            ports:[0;IO_PORTS_SIZE],
            //dma_trasfer_trigger:None,
            //system_counter: 0,
            ports_cycle_trigger:[false;IO_PORTS_SIZE]
        };

        //joypad register initiall value
        io_ports.ports[JOYP_REGISTER_INDEX as usize] = 0xFF;

        io_ports
    }
}