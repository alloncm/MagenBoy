use crate::utils::memory_registers::*;
use super::access_bus::AccessBus;

const IO_PORTS_SIZE:usize = 0x80;

const IO_PORTS_MEMORY_OFFSET:u16 = 0xFF00;

macro_rules! io_port_index{
    ($name:ident, $reg_address:expr) => {
        const $name:u16 = $reg_address - IO_PORTS_MEMORY_OFFSET;
    };
}

pub const DIV_REGISTER_INDEX:u16 = DIV_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET;

io_port_index!(TAC_REGISTER_INDEX, TAC_REGISTER_ADDRESS);
io_port_index!(STAT_REGISTER_INDEX, STAT_REGISTER_ADDRESS);
io_port_index!(JOYP_REGISTER_INDEX, JOYP_REGISTER_ADDRESS);
io_port_index!(DMA_REGISTER_INDEX, DMA_REGISTER_ADDRESS);
io_port_index!(NR30_REGISTER_INDEX, NR30_REGISTER_ADDRESS);
io_port_index!(NR31_REGISTER_INDEX, NR31_REGISTER_ADDRESS);
io_port_index!(NR32_REGISTER_INDEX, NR32_REGISTER_ADDRESS);
io_port_index!(NR33_REGISTER_INDEX, NR33_REGISTER_ADDRESS);
io_port_index!(NR34_REGISTER_INDEX, NR34_REGISTER_ADDRESS);

pub struct IoPorts{
    pub system_counter:u16,
    pub dma_trasfer_trigger:Option<AccessBus>,
    ports:[u8;IO_PORTS_SIZE]
}

impl IoPorts{
    pub fn read(&self, address:u16)->u8{
        let value = self.ports[address as usize];
        match address{
            NR30_REGISTER_INDEX=> value | 0x7F,
            NR31_REGISTER_INDEX=> value | 0xFF,
            NR32_REGISTER_INDEX=> value | 0x9F,
            NR33_REGISTER_INDEX=> value | 0xFF,
            NR34_REGISTER_INDEX=> value | 0xBF,
            0x27..=0x2F=>0xFF,//Not used
            _=>value
        }
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
            DMA_REGISTER_INDEX=> {
                self.dma_trasfer_trigger = match value{
                    0..=0x7F=>Some(AccessBus::External),
                    0x80..=0x9F=>Some(AccessBus::Video),
                    0xA0..=0xFF=>Some(AccessBus::External)
                }
            },
            NR31_REGISTER_INDEX=>self.ports[NR31_REGISTER_INDEX as usize] = 0xFF,
            _=>{}
        }

        self.ports[address as usize] = value;
    }

    pub fn write_unprotected(&mut self, address:u16, value:u8){
        self.ports[address as usize] = value;
    }
}

impl Default for IoPorts{
    fn default()->Self{
        let mut io_ports = IoPorts{
            ports:[0;IO_PORTS_SIZE],
            dma_trasfer_trigger:None,
            system_counter: 0
        };

        //joypad register initiall value
        io_ports.ports[JOYP_REGISTER_INDEX as usize] = 0xFF;

        io_ports
    }
}