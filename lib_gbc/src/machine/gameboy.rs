use crate::cpu::gbc_cpu::GbcCpu;
use crate::machine::memory::Memory;


pub struct GameBoy {
    pub cpu: GbcCpu,
    pub mmu:dyn Memory
}

impl GameBoy{
    fn fetch_first_byte(&mut self)->u8{
        self.mmu.read(self.cpu.program_counter)
    }

    pub fn cycle(&mut self){
        let opcode = self.fetch_first_byte();
    }
}
