
pub const ROM_BANK_SIZE:u16 = 0x4000;
pub const RAM_BANK_SIZE:u16 = 0x2000;
pub const MBC_RAM_SIZE_LOCATION:usize = 0x149;

pub fn get_ram_size(ram_size_register:u8)->usize{
    match ram_size_register{
        0x0=>0,
        0x1=>0x800,
        0x2=>0x4000,
        0x3=>0x8000,
        0x4=>0x20000,
        0x5=>0x10000,
        _=>std::panic!("invalid ram size register {:#X}", ram_size_register)
    }
}

pub fn init_ram(ram_reg:u8, external_ram:Option<Vec<u8>>)->Vec<u8>{
    let ram_size = get_ram_size(ram_reg);
    
    match external_ram{
        Some(ram)=>{
            if ram.len() != ram_size{
                std::panic!("external rom is not in the correct size for the cartridge");
            }

            return ram;
        }
        None=>vec![0;ram_size]
    }
}

pub trait Mbc{
    fn get_ram(&self)->&[u8];
    fn has_battery(&self)->bool;

    fn read_bank0(&self, address:u16)->u8;
    fn read_current_bank(&self, address:u16)->u8;
    fn write_rom(&mut self, address:u16, value:u8);
    fn read_external_ram(&self, address:u16)->u8;
    fn write_external_ram(&mut self, address:u16, value:u8);
}