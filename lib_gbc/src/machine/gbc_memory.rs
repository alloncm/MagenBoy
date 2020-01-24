use crate::machine::memory::Memory;
use crate::machine::rom::Rom;
use crate::machine::ram::Ram;

const VRAM_SIZE:usize = 0x4000;

pub struct GbcMmu{
    ram: Ram,
    vram: [u8;VRAM_SIZE],
    rom: Rom
}

impl GbcMmu{
    fn get_bank0_ram(&self, address:u16)->u8{
        return self.ram[address-RAM_POS];
    }

    fn get_bank1_ram(&self, address:u16)->u8{
        return self.ram[address-(RAM_POS*2)];
    }

}

impl Memory for GbcMmu{
    fn read(&self, address:u16)->u8{
        return match address{
            0x0..=0x3FFF=>self.rom.get_bank0(address),
            0x4000..=0x7FFF=>self.rom.get_current_bank(address),
            0x8000..=0x9FFF=>self.vram[address],
            0xA000..=0xBFFF=>self.rom.get_external_ram(adress),
            0xC000..=0xCFFF | 
            0xE000..0xFE00=>self.get_bank0_ram(address),
            0xD000..0xE000=>self.get_bank1_ram(address),
            0xFE00..0xFEA0=>std::panic!("not implmented yet"),
            0xFE00..0xFEA0=>std::panic!("not useable"),
            _=>std::panic!("not implemented yet")
        }
    }

    fn write(&mut self, address:u16, value:u8){
        memory[address] = value;
    }
}