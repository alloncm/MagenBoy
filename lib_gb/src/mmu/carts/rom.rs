use std::vec::Vec;
use super::mbc::Mbc;
use super::mbc::*;

pub struct Rom{
    program: Vec<u8>,
    external_ram:Vec<u8>,
    battery:bool
}

impl Mbc for Rom{
    
    fn get_ram(&self) ->&[u8] {
        self.external_ram.as_slice()
    }

    fn has_battery(&self) ->bool {
        self.battery
    }

    fn read_bank0(&self, address:u16)->u8{
        return self.program[address as usize];
    }

    fn write_rom(&mut self, _address: u16, _value: u8){
        //Just ignoring this as some games accidently wrtite here (ahhm Tetris)
    }

    fn read_current_bank(&self, address:u16)->u8{
        return self.program[(ROM_BANK_SIZE + address) as usize];
    }

    fn read_external_ram(&self, address:u16)->u8{
        self.external_ram[address as usize]
    }

    fn write_external_ram(&mut self, address:u16, value:u8){
        self.external_ram[address as usize] = value
    }

}

impl Rom{
    
    pub fn new(vec:Vec<u8>, battery:bool, ram:Option<Vec<u8>>)->Rom{
        let mut rom = Rom{
            program:vec,
            external_ram:Vec::new(),
            battery:battery
        };

        rom.external_ram = init_ram(rom.program[MBC_RAM_SIZE_LOCATION], ram);

        rom
    }
}