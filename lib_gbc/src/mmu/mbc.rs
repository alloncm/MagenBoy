
pub const ROM_BANK_SIZE:u16 = 0x4000;
pub const RAM_BANK_SIZE:u16 = 0x2000;

pub trait Mbc{
    fn read_bank0(&self, address:u16)->u8;
    fn read_current_bank(&self, address:u16)->u8;
    fn write_rom(&mut self, address:u16, value:u8);
    fn read_external_ram(&self, address:u16)->u8;
    fn write_external_ram(&mut self, address:u16, value:u8);
}