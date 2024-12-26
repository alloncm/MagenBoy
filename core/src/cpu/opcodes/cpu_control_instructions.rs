use crate::{mmu::Memory, cpu::{gb_cpu::GbCpu, flag::Flag}, utils::memory_registers::{IE_REGISTER_ADDRESS, JOYP_REGISTER_ADDRESS, KEY1_REGISTER_ADDRESS}};

pub fn ccf(cpu:&mut GbCpu)->u8{
    let carry:bool = cpu.get_flag(Flag::Carry);
    cpu.set_by_value(Flag::Carry, !carry);
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);
    
    // 1 cycles - 1 reading opcode
    return 0;
}

pub fn scf(cpu:&mut GbCpu)->u8{
    cpu.set_flag(Flag::Carry);
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);
    
    // 1 cycles - 1 reading opcode
    return 0;
}

pub fn halt(cpu:&mut GbCpu, memory: &mut impl Memory)->u8{
    cpu.halt = true;
    memory.set_halt(true);

    // 1 cycles - 1 reading opcode
    return 0;
}


// For some reason inlining boost perf on gbc
#[inline]
pub fn stop(cpu:&mut GbCpu, memory: &mut impl Memory)->u8{
    if (memory.read(IE_REGISTER_ADDRESS, 0) & 0b11111 == 0) && (memory.read(JOYP_REGISTER_ADDRESS, 0) & 0b1111 == 0){
        cpu.stop = true;
    }
    if memory.read(KEY1_REGISTER_ADDRESS, 0) & 1 != 0{
        cpu.double_speed = !cpu.double_speed;
        memory.set_double_speed_mode(cpu.double_speed);
        memory.write(KEY1_REGISTER_ADDRESS, 0, 0);
    }

    // 1 cycles - 1 reading opcode
    return 0;
}

pub fn di(cpu:&mut GbCpu)->u8{
    cpu.mie = false;
    
    // 1 cycles - 1 reading opcode
    return 0;
}

pub fn ei(cpu:&mut GbCpu)->u8{
    cpu.mie = true;
    
    // 1 cycles - 1 reading opcode
    return 0;
}