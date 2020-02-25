use crate::cpu::gbc_cpu::{GbcCpu,Flag};
use crate::machine::memory::Memory;

const IE:u16 = 0xFFFF;
const P1:u16 = 0xFF00;

pub fn ccf(cpu:&mut GbcCpu, opcode:u8){
    let carry:bool = cpu.get_flag(Flag::Carry);
    cpu.set_by_value(Flag::Carry, !carry);
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);
}

pub fn scf(cpu:&mut GbcCpu, opcode:u8){
    cpu.set_flag(Flag::Carry);
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);
}

pub fn halt(cpu:&mut GbcCpu, opcode:u8){
    cpu.halt = true;
}

pub fn stop(cpu:&mut GbcCpu, memory: &mut dyn  Memory,opcode:u16){
    if (memory.read(IE) & 0b111111 == 0) && (memory.read(P1) & 0b1111) == 0{
        cpu.stop = true;
    }
}

pub fn di(cpu:&mut GbcCpu, opcode:u8){
    cpu.mie = false;
}

pub fn ei(cpu:&mut GbcCpu, opcode:u8){
    cpu.mie = true;
}