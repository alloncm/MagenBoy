use crate::cpu::gbc_cpu::{GbcCpu, Flag};
use crate::machine::memory::Memory;
use crate::opcodes::opcodes_utils::{get_src_register,check_for_half_carry_first_nible};



fn add(cpu:&mut GbcCpu, dest:u8, src:u8 )->u8{
    let (value, overflow) = dest.overflowing_add(src);

    if overflow{
        cpu.set_flag(Flag::Carry);
    }
    else{
        cpu.unset_flag(Flag::Carry);
    }
    if value == 0{
        cpu.set_flag(Flag::Zero);
    }
    else{
        cpu.unset_flag(Flag::Zero);
    }
    if check_for_half_carry_first_nible(src, dest){
        cpu.set_flag(Flag::HalfCarry);
    }
    else{
        cpu.set_flag(Flag::HalfCarry);
    }

    cpu.unset_flag(Flag::Subtraction);

    return value;
}
//add A and r
pub fn add_a_r(cpu:&mut GbcCpu, opcode:u8){
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.low();
    *cpu.af.low() = add(cpu, dest, src_reg);
}

//add A and nn
pub fn add_a_nn(cpu:&mut GbcCpu, opcode:u16){
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.low();
    *cpu.af.low() = add(cpu, dest, src);
}

//add A and (hl)
pub fn add_a_hl(cpu:&mut GbcCpu,memory:&dyn Memory){
    let src = memory.read(cpu.hl.value);
    let dest = *cpu.af.low();
    *cpu.af.low() = add(cpu, dest, src);
}