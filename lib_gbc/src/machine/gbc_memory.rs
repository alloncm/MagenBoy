use crate::machine::memory::Memory;
use crate::machine::rom::Rom;
use crate::machine::ram::Ram;

const VRAM_SIZE:usize = 0x4000;

pub struct GbcMmu{
    ram: Ram,
    vram: [u8;VRAM_SIZE],
    rom: Rom
}


impl Memory for GbcMmu{
    fn read(&self, address:u16)->u8{
        return match address{
            0x0..=0x3FFF=>self.rom.get_bank0(address),
            0x4000..=0x7FFF=>self.rom.get_current_bank(address),
            0x8000..=0x9FFF=>self.vram[address],
            0xA000..=0xBFFF=>self.rom.get_external_ram(address),
            0xC000..=0xCFFF =>self.ram.read_bank0(address - 0xC000), 
            0xE000..=0xFDFF=>self.ram.read_bank0(address - 0xE000),
            0xD000..=0xDFFF=>self.ram.read_current_bank(address-0xD000),
            0xFEA0..=0xFEFF=>std::panic!("not useable"),
            _=>std::panic!("not implemented yet")
        }
    }

    fn write(&mut self, address:u16, value:u8){
        memory[address] = value;
    }
}