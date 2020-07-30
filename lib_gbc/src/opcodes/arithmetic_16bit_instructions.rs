use crate::cpu::gbc_cpu::{GbcCpu, Flag};
use crate::opcodes::opcodes_utils::{
    get_arithmetic_16reg,
    check_for_half_carry_third_nible_add,
    signed_check_for_half_carry_first_nible_add,
    signed_check_for_carry_first_nible_add
};
use std::convert::TryFrom;

pub fn add_hl_rr(cpu:&mut GbcCpu, opcode:u8){
    let reg = opcode >> 4;
    let reg = *get_arithmetic_16reg(cpu, reg);
    let hl_value = *cpu.hl.value();

    let (value,overflow) = hl_value.overflowing_add(reg);
    cpu.set_by_value(Flag::Carry, overflow);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_third_nible_add(hl_value, reg));
    cpu.unset_flag(Flag::Subtraction);

    *cpu.hl.value() = value;
}

pub fn add_sp_dd(cpu:&mut GbcCpu, opcode:u16){
    let dd = (opcode & 0xFF) as i8;
    let mut operation_res:(u16, bool) = (0,false);
    
    let mut temp = cpu.stack_pointer as i32;
    temp += dd as i32;
    match u16::try_from(temp){
        Ok(value)=>operation_res.0 = value,
        Err(_)=>{
            operation_res.0 = temp as u16;
            operation_res.1 = true;
        }
    };

    cpu.unset_flag(Flag::Zero);
    cpu.unset_flag(Flag::Subtraction);
    cpu.set_by_value(Flag::Carry, signed_check_for_carry_first_nible_add(temp as i16, dd));
    cpu.set_by_value(Flag::HalfCarry, signed_check_for_half_carry_first_nible_add(temp as i16, dd));

    cpu.stack_pointer = operation_res.0;
}

pub fn inc_rr(cpu:&mut GbcCpu, opcode:u8){
    let reg = (opcode & 0xF0)>>4;
    let reg = get_arithmetic_16reg(cpu, reg);
    *reg = (*reg).wrapping_add(1);
}


pub fn dec_rr(cpu:&mut GbcCpu, opcode:u8){
    let reg = (opcode & 0xF0)>>4;
    let reg = get_arithmetic_16reg(cpu, reg);
    *reg = (*reg).wrapping_sub(1);
}