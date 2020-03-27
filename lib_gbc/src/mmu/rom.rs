use std::vec::Vec;
use super::mbc::Mbc;

pub struct Rom{
    pub program: Vec<u8>
}

impl Rom{

    pub fn new(vec:Vec<u8>)->Rom{
        Rom{
            program:vec
        }
    }

    pub fn read_bank0(&self, address:u16)->u8{
        return self.program[address as usize];
    }

    pub fn read_current_bank(&self, address:u16)->u8{
        return self.program[address as usize];
    }

    pub fn read_external_ram(&self, _address:u16)->u8{
        std::panic!("no ram supported");
    }

    pub fn write_external_ram(&self, _address:u16, _value:u8){
        std::panic!("no ram supported");
    }

}