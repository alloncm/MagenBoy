pub mod gb_cpu;
pub mod register;
pub mod opcodes;
pub mod flag;
pub mod opcode_runner;

use crate::mmu::memory::Memory;

use self::gb_cpu::GbCpu;

impl GbCpu {
    pub fn prepare_for_interrupt(&mut self, memory: &mut impl Memory, address: u16)->u8{
        //reseting MIE register
        self.mie = false;
        //pushing PC
        self::opcodes::opcodes_utils::push(self, memory, self.program_counter);
        //jumping to the interupt address
        self.program_counter = address;
        //unhalting the CPU
        self.halt = false;

        //cycles passed
        return 5;
    }   
}