use crate::machine::memory::Memory;
use crate::machine::rom::Rom;
use crate::machine::ram::Ram;
use crate::machine::vram::VRam;
use crate::machine::io_ports::IoPorts;

const HRAM_SIZE:usize = 0x7E;

pub struct GbcMmu{
    pub ram: Ram,
    pub vram: VRam,
    pub rom: Rom,
    io_ports: IoPorts,
    hram: [u8;HRAM_SIZE],
    interupt_enable_register:u8
}


impl Memory for GbcMmu{
    fn read(&self, address:u16)->u8{
        return match address{
            0x0..=0x3FFF=>self.rom.read_bank0(address),
            0x4000..=0x7FFF=>self.rom.read_current_bank(address-0x4000),
            0x8000..=0x9FFF=>self.vram.read_current_bank(address-0x8000),
            0xA000..=0xBFFF=>self.rom.read_external_ram(address-0xA000),
            0xC000..=0xCFFF =>self.ram.read_bank0(address - 0xC000), 
            0xE000..=0xFDFF=>self.ram.read_bank0(address - 0xE000),
            0xD000..=0xDFFF=>self.ram.read_current_bank(address-0xD000),
            0xFEA0..=0xFEFF=>std::panic!("not useable"),
            0xFF00..=0xFF7F=>self.io_ports.memory[(address - 0xFF00) as usize],
            0xFF80..=0xFFFE=>self.hram[(address-0xFF80) as usize],
            0xFFFF=>self.interupt_enable_register,
            _=>std::panic!("not implemented yet")
        }
    }

    fn write(&mut self, address:u16, value:u8){
        match address{
            0x8000..=0x9FFF=>self.vram.write_current_bank(address-0x8000, value),
            0xA000..=0xBFFF=>self.rom.write_external_ram(address-0xA000,value),
            0xC000..=0xCFFF =>self.ram.write_bank0(address - 0xC000,value), 
            0xE000..=0xFDFF=>self.ram.write_bank0(address - 0xE000,value),
            0xD000..=0xDFFF=>self.ram.write_current_bank(address-0xD000,value),
            0xFEA0..=0xFEFF=>std::panic!("not useable"),
            0xFF00..=0xFF7F=>self.io_ports.memory[(address - 0xFF00) as usize] = value,
            0xFF80..=0xFFFE=>self.hram[(address-0xFF80) as usize] = value,
            0xFFFF=>self.interupt_enable_register = value,
            _=>std::panic!("not implemented yet")
        }
    }
}

impl GbcMmu{
    pub fn new(rom:Rom)->GbcMmu{
        GbcMmu{
            ram:Ram::default(),
            io_ports:IoPorts::default(),
            rom:rom,
            vram:VRam::default(),
            hram:[0;HRAM_SIZE],
            interupt_enable_register:0
        }
    }
}