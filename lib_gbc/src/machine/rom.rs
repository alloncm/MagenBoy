use std::vec::Vec;

pub struct Rom{
    program: Vec<u8>
}

impl Rom{
    pub fn get_bank0(&self, address:u16)->u8{
        return self.program[address as usize];
    }

    pub fn get_current_bank(&self, address:u16)->u8{
        return self.program[address as usize];
    }

    pub fn get_external_ram(&self, address:u16)->u8{
        std::panic!("no ram supported");
    }
}