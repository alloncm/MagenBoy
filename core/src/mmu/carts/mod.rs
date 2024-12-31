pub mod rom;
pub mod mbc1;
pub mod mbc3;
pub mod mbc5;

pub use rom::Rom;
pub use mbc1::Mbc1;
pub use mbc3::Mbc3;
pub use mbc5::Mbc5;

use crate::utils::global_static_alloctor::static_alloc_array;

pub const ROM_BANK_SIZE:usize = 0x4000;
pub const RAM_BANK_SIZE:usize = 0x2000;

pub const CGB_FLAG_ADDRESS:usize = 0x143;
pub const MBC_RAM_SIZE_LOCATION:usize = 0x149;

pub fn get_ram_size(ram_size_register:u8)->usize{
    match ram_size_register{
        0x0=>0,
        0x1=>0x800,     // Unofficial - Undefined according to official docs
        0x2=>0x4000,
        0x3=>0x8000,
        0x4=>0x2_0000,
        0x5=>0x1_0000,
        _=>core::panic!("invalid ram size register {:#X}", ram_size_register)
    }
}

pub fn init_ram(ram_reg:u8, external_ram:Option<&'static mut[u8]>)->&'static mut [u8]{
    let ram_size = get_ram_size(ram_reg);
    
    match external_ram{
        Some(ram)=>{
            if ram.len() != ram_size{
                core::panic!("External ram is not in the correct size for the cartridge, the save seems corrupted, either fix or delete it and try again");
            }

            return ram;
        }
        None=>static_alloc_array(ram_size)
    }
}

pub trait Mbc{
    fn get_ram(&mut self)->&mut [u8];
    fn has_battery(&self)->bool;

    fn read_bank0(&self, address:u16)->u8;
    fn read_current_bank(&self, address:u16)->u8;
    fn write_rom(&mut self, address:u16, value:u8);
    fn read_external_ram(&self, address:u16)->u8;
    fn write_external_ram(&mut self, address:u16, value:u8);

    #[cfg(feature = "dbg")]
    fn get_bank_number(&self)->u16;
}