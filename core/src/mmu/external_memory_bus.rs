use crate::utils::memory_registers::{BOOT_REGISTER_ADDRESS, SVBK_REGISTER_ADRRESS};
use super::{ram::Ram, carts::Mbc};

pub const GB_BOOT_ROM_SIZE:usize = 0x100;
pub const GBC_BOOT_ROM_SIZE:usize = 0x900;

#[derive(PartialEq, Eq)]
pub enum Bootrom {
    Gb([u8;GB_BOOT_ROM_SIZE]),
    Gbc([u8;GBC_BOOT_ROM_SIZE])
}

pub struct ExternalMemoryBus<'a>{
    ram: Ram,
    mbc: &'a mut dyn Mbc,
    bootrom :Option<Bootrom>,
    bootrom_register:u8,
}

impl<'a> ExternalMemoryBus<'a> {
    pub fn new(mbc:&'a mut dyn Mbc, bootrom: Option<Bootrom>)->Self{
        Self{
            mbc,
            ram:Ram::default(),
            bootrom,
            bootrom_register: 0,
        }
    }

    pub fn read(&mut self, address:u16)->u8 {
        return match address{
            0x0000..=0x00FF=>{
                match self.bootrom{
                    Some(Bootrom::Gb(r)) => r[address as usize],
                    Some(Bootrom::Gbc(r)) => r[address as usize],
                    None=>self.mbc.read_bank0(address),
                }
            }
            0x0100..=0x01FF=>self.mbc.read_bank0(address),
            0x0200..=0x08FF=>{
                match self.bootrom {
                    Some(Bootrom::Gbc(r))=>r[address as usize],
                    Some(Bootrom::Gb(_)) | 
                    None=>self.mbc.read_bank0(address)
                }
            }
            0x0900..=0x3FFF=>self.mbc.read_bank0(address),
            0x4000..=0x7FFF=>self.mbc.read_current_bank(address - 0x4000),
            0xA000..=0xBFFF=>self.mbc.read_external_ram(address - 0xA000),
            0xC000..=0xCFFF=>self.ram.read_bank0(address - 0xC000),
            0xD000..=0xDFFF=>self.ram.read_current_bank(address - 0xD000),
            0xE000..=0xFDFF=>self.ram.read_bank0(address - 0xE000),
            _=>core::panic!("Error: attemp to read invalid external memory bus address: {:#X}", address)
        }
    }

    pub fn write(&mut self, address:u16, value:u8) {
        match address{
            0x0000..=0x7FFF=>self.mbc.write_rom(address, value),
            0xA000..=0xBFFF=>self.mbc.write_external_ram(address - 0xA000, value),
            0xC000..=0xCFFF=>self.ram.write_bank0(address - 0xC000, value),
            0xD000..=0xDFFF=>self.ram.write_current_bank(address-0xD000, value),
            0xE000..=0xFDFF=>self.ram.write_bank0(address - 0xE000, value),
            _=>core::panic!("Error: attemp to write invalid external memory bus address: {:#X}", address)
        }
    }

    pub fn read_boot_reg(&self) -> u8 {self.bootrom_register}
    pub fn write_boot_reg(&mut self, value:u8) {
        self.bootrom_register = value;
        if self.bootrom.is_some() && value != 0{
            self.bootrom = None;
        }
    }

    pub fn read_svbk_reg(&self)->u8 {self.ram.get_bank()}
    pub fn write_svbk_reg(&mut self, value:u8) {self.ram.set_bank(value)}

    #[cfg(feature = "dbg")]
    pub fn get_current_rom_bank(&self)->u16{ self.mbc.get_bank_number() }

    #[cfg(feature = "dbg")]
    pub fn get_current_ram_bank(&self)->u8 { self.ram.get_bank() }
}