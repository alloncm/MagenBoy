use crate::cpu::gbc_cpu::{GbcCpu, Flag};
use crate::machine::memory::Memory;
use crate::opcodes::opcodes_utils::{
    get_arithmetic_16reg,
    check_for_half_carry_third_nible
};

pub fn add_hl_rr(cpu:&mut GbcCpu, opcode:u8){
    let reg = opcode & 0xF0;
    let reg = get_arithmetic_16reg(cpu, reg);

    let (value,overflow) = cpu.hl.value.overflowing_add(reg);
    cpu.set_by_value(Flag::Carry, overflow);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_third_nible(cpu.hl.value, reg);
    cpu.unset_flag(Flag::Subtraction);

    cpu.hl.value = value;
}