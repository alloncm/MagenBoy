use std::vec::Vec;

pub struct Rom{
    program: Vec<u8>
}

impl Rom{
    pub fn read_bank0(&self, address:u16)->u8{
        return self.program[address as usize];
    }

    pub fn read_current_bank(&self, address:u16)->u8{
        return self.program[address as usize];
    }

    pub fn read_external_ram(&self, address:u16)->u8{
        std::panic!("no ram supported");
    }

    pub fn write_external_ram(&self, address:u16, value:u8){
        std::panic!("no ram supported");
    }

}