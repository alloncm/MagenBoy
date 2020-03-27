use std::vec::Vec;

pub trait Mbc{
    fn init(program:Vec<u8>)->dyn Mbc;
    fn switch_bank(&self, bank:u8);
    fn read_bank0(&self, address:u16)->u8;
    fn read_current_bank(&self, address:u16)->u8;
    fn write_bank0(&self, address:u16, value:u8);
    fn write_current_bank(&self, address:u16, value:u8);
    fn read_external_ram(&self, address:u16)->u8;
    fn write_external_ram(&self, address:u16, value:u8);
}