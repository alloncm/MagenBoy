use super::memory::Memory;
use super::ram::Ram;
use super::vram::VRam;
use crate::utils::memory_registers::DMA_REGISTER_ADDRESS;
use super::mbc::Mbc;
use std::boxed::Box;

pub const BOOT_ROM_SIZE:usize = 0xFF;
const HRAM_SIZE:usize = 0x7F;
const SPRITE_ATTRIBUTE_TABLE_SIZE:usize = 0xA0;
const IO_PORTS_SIZE:usize = 0x80;

pub struct GbcMmu{
    pub ram: Ram,
    pub vram: VRam,
    pub dma_trasfer_trigger:bool,
    pub finished_boot:bool,
    boot_rom:[u8;BOOT_ROM_SIZE],
    mbc: Box<dyn Mbc>,
    io_ports: [u8;IO_PORTS_SIZE],
    sprite_attribute_table:[u8;SPRITE_ATTRIBUTE_TABLE_SIZE],
    hram: [u8;HRAM_SIZE],
    interupt_enable_register:u8
}


impl Memory for GbcMmu{
    fn read(&self, address:u16)->u8{
        return match address{
            0x0..=0xFF=>{
                if self.finished_boot{
                    return self.mbc.read_bank0(address);
                }
                
                return self.boot_rom[address as usize];
            },
            0x100..=0x3FFF=>self.mbc.read_bank0(address),
            0x4000..=0x7FFF=>self.mbc.read_current_bank(address-0x4000),
            0x8000..=0x9FFF=>self.vram.read_current_bank(address-0x8000),
            0xA000..=0xBFFF=>self.mbc.read_external_ram(address-0xA000),
            0xC000..=0xCFFF =>self.ram.read_bank0(address - 0xC000), 
            0xD000..=0xDFFF=>self.ram.read_current_bank(address-0xD000),
            0xE000..=0xFDFF=>self.ram.read_bank0(address - 0xE000),
            0xFE00..=0xFE9F=>self.sprite_attribute_table[(address-0xFE00) as usize],
            0xFEA0..=0xFEFF=>0x0,
            0xFF00..=0xFF7F=>self.io_ports[(address - 0xFF00) as usize],
            0xFF80..=0xFFFE=>self.hram[(address-0xFF80) as usize],
            0xFFFF=>self.interupt_enable_register
        }
    }

    fn write(&mut self, address:u16, value:u8){
        if address == DMA_REGISTER_ADDRESS{
            self.dma_trasfer_trigger = true;
        }

        match address{
            0x0..=0x7FFF=>self.mbc.write_rom(address, value),
            0x8000..=0x9FFF=>self.vram.write_current_bank(address-0x8000, value),
            0xA000..=0xBFFF=>self.mbc.write_external_ram(address-0xA000,value),
            0xC000..=0xCFFF =>self.ram.write_bank0(address - 0xC000,value), 
            0xE000..=0xFDFF=>self.ram.write_bank0(address - 0xE000,value),
            0xD000..=0xDFFF=>self.ram.write_current_bank(address-0xD000,value),
            0xFE00..=0xFE9F=>self.sprite_attribute_table[(address-0xFE00) as usize] = value,
            0xFEA0..=0xFEFF=>{},
            0xFF00..=0xFF7F=>self.io_ports[(address - 0xFF00) as usize] = value,
            0xFF80..=0xFFFE=>self.hram[(address-0xFF80) as usize] = value,
            0xFFFF=>self.interupt_enable_register = value
        }
    }
}

impl GbcMmu{
    pub fn new(mbc:Box<dyn Mbc>, boot_rom:[u8;BOOT_ROM_SIZE])->Self{
        GbcMmu{
            ram:Ram::default(),
            io_ports:[0;IO_PORTS_SIZE],
            mbc:mbc,
            vram:VRam::default(),
            sprite_attribute_table:[0;SPRITE_ATTRIBUTE_TABLE_SIZE],
            hram:[0;HRAM_SIZE],
            interupt_enable_register:0,
            dma_trasfer_trigger:false,
            boot_rom:boot_rom,
            finished_boot:false
        }
    }
}