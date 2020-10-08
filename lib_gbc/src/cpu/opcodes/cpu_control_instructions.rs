use crate::cpu::gb_cpu::GbCpu;
use crate::cpu::flag::Flag;
use crate::mmu::memory::Memory;

const P1:u16 = 0xFF00;

pub fn ccf(cpu:&mut GbCpu){
    let carry:bool = cpu.get_flag(Flag::Carry);
    cpu.set_by_value(Flag::Carry, !carry);
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);
}

pub fn scf(cpu:&mut GbCpu){
    cpu.set_flag(Flag::Carry);
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);
}

pub fn halt(cpu:&mut GbCpu){
    cpu.halt = true;
}

pub fn stop(cpu:&mut GbCpu, memory: &mut dyn  Memory){
    if (cpu.interupt_enable & 0b11111 == 0) && (memory.read(P1) & 0b1111 == 0){
        cpu.stop = true;
    }
}

pub fn di(cpu:&mut GbCpu){
    cpu.mie = false;
}

pub fn ei(cpu:&mut GbCpu){
    cpu.mie = true;
}