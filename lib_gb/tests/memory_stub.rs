use lib_gb::mmu::memory::Memory;

pub struct MemoryStub{
    pub data:[u8;0xFFFF]
}

impl Memory for MemoryStub{
    fn read(&mut self, address:u16, _m_cycles:u8)->u8{
        self.data[address as usize]
    }

    fn write(&mut self, address:u16, value:u8, _m_cycles:u8){
        self.data[address as usize] = value;
    }
}