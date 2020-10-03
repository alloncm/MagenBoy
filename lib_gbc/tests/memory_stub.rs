extern crate lib_gbc;
use lib_gbc::mmu::memory::Memory;

pub struct MemoryStub{
    pub data:[u8;0xFFFF]
}

impl Memory for MemoryStub{
    fn read(&self, address:u16)->u8{
        self.data[address as usize]
    }

    fn write(&mut self, address:u16, value:u8){
        self.data[address as usize] = value;
    }
}