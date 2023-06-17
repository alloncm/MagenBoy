use crate::utils::memory_registers::*;

pub const IO_PORTS_SIZE:usize = 0x80;

pub const IO_PORTS_MEMORY_OFFSET:u16 = 0xFF00;

macro_rules! pub_io_port_index{
    ($name:ident, $reg_address:expr) => {
        pub const $name:u16 = $reg_address - IO_PORTS_MEMORY_OFFSET;
    };
}

pub_io_port_index!(DIV_REGISTER_INDEX, DIV_REGISTER_ADDRESS);
pub_io_port_index!(TAC_REGISTER_INDEX, TAC_REGISTER_ADDRESS);
pub_io_port_index!(TIMA_REGISTER_INDEX, TIMA_REGISTER_ADDRESS);
pub_io_port_index!(TMA_REGISTER_INDEX, TMA_REGISTER_ADDRESS);

pub_io_port_index!(LCDC_REGISTER_INDEX, LCDC_REGISTER_ADDRESS);
pub_io_port_index!(STAT_REGISTER_INDEX, STAT_REGISTER_ADDRESS);
pub_io_port_index!(SCY_REGISTER_INDEX, SCY_REGISTER_ADDRESS);
pub_io_port_index!(SCX_REGISTER_INDEX, SCX_REGISTER_ADDRESS);
pub_io_port_index!(LY_REGISTER_INDEX, LY_REGISTER_ADDRESS);
pub_io_port_index!(LYC_REGISTER_INDEX, LYC_REGISTER_ADDRESS);
pub_io_port_index!(DMA_REGISTER_INDEX, DMA_REGISTER_ADDRESS);
pub_io_port_index!(WY_REGISTER_INDEX, WY_REGISTER_ADDRESS);
pub_io_port_index!(WX_REGISTER_INDEX, WX_REGISTER_ADDRESS);
pub_io_port_index!(KEY1_REGISTER_INDEX, KEY1_REGISTER_ADDRESS);
pub_io_port_index!(BOOT_REGISTER_INDEX, BOOT_REGISTER_ADDRESS);
pub_io_port_index!(BGP_REGISTER_INDEX, BGP_REGISTER_ADDRESS);
pub_io_port_index!(OBP0_REGISTER_INDEX, OBP0_REGISTER_ADDRESS);
pub_io_port_index!(OBP1_REGISTER_INDEX, OBP1_REGISTER_ADDRESS);
pub_io_port_index!(VBK_REGISTER_INDEX, VBK_REGISTER_ADDRESS);
pub_io_port_index!(HDMA1_REGISTER_INDEX, HDMA1_REGISTER_ADDRESS);
pub_io_port_index!(HDMA2_REGISTER_INDEX, HDMA2_REGISTER_ADDRESS);
pub_io_port_index!(HDMA3_REGISTER_INDEX, HDMA3_REGISTER_ADDRESS);
pub_io_port_index!(HDMA4_REGISTER_INDEX, HDMA4_REGISTER_ADDRESS);
pub_io_port_index!(HDMA5_REGISTER_INDEX, HDMA5_REGISTER_ADDRESS);
pub_io_port_index!(BGPI_REGISTER_INDEX, BGPI_REGISTER_ADDRESS);
pub_io_port_index!(BGPD_REGISTER_INDEX, BGPD_REGISTER_ADDRESS);
pub_io_port_index!(OBPI_REGISTER_INDEX, OBPI_REGISTER_ADDRESS);
pub_io_port_index!(OBPD_REGISTER_INDEX, OBPD_REGISTER_ADDRESS);
pub_io_port_index!(IF_REGISTER_INDEX, IF_REGISTER_ADDRESS);

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