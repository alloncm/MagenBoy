use crate::cpu::gb_cpu::GbCpu;
use crate::cpu::flag::Flag;
use crate::mmu::memory::Memory;

const P1:u16 = 0xFF00;

pub fn ccf(cpu:&mut GbCpu)->u8{
    let carry:bool = cpu.get_flag(Flag::Carry);
    cpu.set_by_value(Flag::Carry, !carry);
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);
    
    //cycles
    return 1;
}

pub fn scf(cpu:&mut GbCpu)->u8{
    cpu.set_flag(Flag::Carry);
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);
    
    //cycles
    return 1;
}

pub fn halt(cpu:&mut GbCpu)->u8{
    cpu.halt = true;
    
    //cycles
    return 1;
}

pub fn stop(cpu:&mut GbCpu, memory: &mut impl  Memory)->u8{
    if (cpu.interupt_enable & 0b11111 == 0) && (memory.read(P1) & 0b1111 == 0){
        cpu.stop = true;
    }

    //cycles
    return 1;
}

pub fn di(cpu:&mut GbCpu)->u8{
    cpu.mie = false;
    
    //cycles
    return 1;
}

pub fn ei(cpu:&mut GbCpu)->u8{
    cpu.mie = true;
    
    //cycles
    return 1;
}