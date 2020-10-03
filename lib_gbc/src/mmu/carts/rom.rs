use std::vec::Vec;
use super::mbc::Mbc;
use super::mbc::ROM_BANK_SIZE;

const RAM_SZIE:usize = 0x2000;

pub struct Rom{
    program: Vec<u8>,
    external_ram:[u8;RAM_SZIE]
}

impl Mbc for Rom{
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
    
    pub fn new(vec:Vec<u8>)->Rom{
        Rom{
            program:vec,
            external_ram:[0;RAM_SZIE]
        }
    }
}