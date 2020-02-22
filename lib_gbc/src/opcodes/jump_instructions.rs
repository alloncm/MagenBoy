use crate::cpu::gbc_cpu::{GbcCpu, Flag};
use crate::machine::memory::Memory;

pub fn call(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u32){
    let address_to_jump = (opcode & 0xFFFF) as u16;
    let high = (cpu.program_counter & 0xFF00) as u8;
    let low = (cpu.program_counter & 0xFF) as u8;
    memory.write(cpu.stack_pointer-1, high);
    memory.write(cpu.stack_pointer-2, low);
    cpu.stack_pointer-=2;
    cpu.program_counter = address_to_jump;
}