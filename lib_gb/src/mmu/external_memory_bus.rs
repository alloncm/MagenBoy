use super::{ram::Ram, carts::Mbc};

pub struct ExternalMemoryBus<'a>{
    ram: Ram,
    mbc: &'a mut Box<dyn Mbc>
}

impl<'a> ExternalMemoryBus<'a> {
    pub fn new(mbc:&'a mut Box<dyn Mbc>)->Self{
        Self{
            mbc,
            ram:Ram::default()
        }
    }

    pub fn read(&mut self, address:u16)->u8 {
        return match address{
            0x0000..=0x3FFF=>self.mbc.read_bank0(address),
            0x4000..=0x7FFF=>self.mbc.read_current_bank(address - 0x4000),
            0xA000..=0xBFFF=>self.mbc.read_external_ram(address - 0xA000),
            0xC000..=0xCFFF=>self.ram.read_bank0(address - 0xC000),
            0xD000..=0xDFFF=>self.ram.read_current_bank(address - 0xD000),
            0xE000..=0xFDFF=>self.ram.read_bank0(address - 0xE000),
            _=>std::panic!("Error: attemp to read invalid external memory bus address: {:#X}", address)
        }
    }

    pub fn write(&mut self, address:u16, value:u8) {
        match address{
            0x0000..=0x7FFF=>self.mbc.write_rom(address, value),
            0xA000..=0xBFFF=>self.mbc.write_external_ram(address - 0xA000, value),
            0xC000..=0xCFFF =>self.ram.write_bank0(address - 0xC000, value),
            0xD000..=0xDFFF=>self.ram.write_current_bank(address-0xD000, value),
            0xE000..=0xFDFF=>self.ram.write_bank0(address - 0xE000, value),
            _=>std::panic!("Error: attemp to write invalid external memory bus address: {:#X}", address)
        }
    }
}