use crate::cpu::gbc_cpu::{GbcCpu,Flag};
use crate::mmu::memory::Memory;

const IE:u16 = 0xFFFF;
const P1:u16 = 0xFF00;

pub fn ccf(cpu:&mut GbcCpu){
    let carry:bool = cpu.get_flag(Flag::Carry);
    cpu.set_by_value(Flag::Carry, !carry);
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);
}

pub fn scf(cpu:&mut GbcCpu){
    cpu.set_flag(Flag::Carry);
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);
}

pub fn halt(cpu:&mut GbcCpu){
    cpu.halt = true;
}

pub fn stop(cpu:&mut GbcCpu, memory: &mut dyn  Memory){
    if (memory.read(IE) & 0b111111 == 0) && (memory.read(P1) & 0b1111) == 0{
        cpu.stop = true;
    }
}

pub fn di(cpu:&mut GbcCpu){
    cpu.mie = false;
}

pub fn ei(cpu:&mut GbcCpu){
    cpu.mie = true;
}