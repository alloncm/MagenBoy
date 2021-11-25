use crate::utils::memory_registers::*;

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
pub_io_port_index!(TAC_REGISTER_INDEX, TAC_REGISTER_ADDRESS);
pub_io_port_index!(TIMA_REGISTER_INDEX, TIMA_REGISTER_ADDRESS);
pub_io_port_index!(TMA_REGISTER_INDEX, TMA_REGISTER_ADDRESS);

pub_io_port_index!(JOYP_REGISTER_INDEX, JOYP_REGISTER_ADDRESS);
pub_io_port_index!(NR10_REGISTER_INDEX, NR10_REGISTER_ADDRESS);
pub_io_port_index!(NR11_REGISTER_INDEX, NR11_REGISTER_ADDRESS);
pub_io_port_index!(NR12_REGISTER_INDEX, NR12_REGISTER_ADDRESS);
pub_io_port_index!(NR13_REGISTER_INDEX, NR13_REGISTER_ADDRESS);
pub_io_port_index!(NR14_REGISTER_INDEX, NR14_REGISTER_ADDRESS);
pub_io_port_index!(NR21_REGISTER_INDEX, NR21_REGISTER_ADDRESS);
pub_io_port_index!(NR22_REGISTER_INDEX, NR22_REGISTER_ADDRESS);
pub_io_port_index!(NR23_REGISTER_INDEX, NR23_REGISTER_ADDRESS);
pub_io_port_index!(NR24_REGISTER_INDEX, NR24_REGISTER_ADDRESS);
pub_io_port_index!(NR30_REGISTER_INDEX, NR30_REGISTER_ADDRESS);
pub_io_port_index!(NR31_REGISTER_INDEX, NR31_REGISTER_ADDRESS);
pub_io_port_index!(NR32_REGISTER_INDEX, NR32_REGISTER_ADDRESS);
pub_io_port_index!(NR33_REGISTER_INDEX, NR33_REGISTER_ADDRESS);
pub_io_port_index!(NR34_REGISTER_INDEX, NR34_REGISTER_ADDRESS);
pub_io_port_index!(NR41_REGISTER_INDEX, NR41_REGISTER_ADDRESS);
pub_io_port_index!(NR42_REGISTER_INDEX, NR42_REGISTER_ADDRESS);
pub_io_port_index!(NR43_REGISTER_INDEX, NR43_REGISTER_ADDRESS);
pub_io_port_index!(NR44_REGISTER_INDEX, NR44_REGISTER_ADDRESS);
pub_io_port_index!(NR50_REGISTER_INDEX, NR50_REGISTER_ADDRESS);
pub_io_port_index!(NR51_REGISTER_INDEX, NR51_REGISTER_ADDRESS);
pub_io_port_index!(NR52_REGISTER_INDEX, NR52_REGISTER_ADDRESS);