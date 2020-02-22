use crate::cpu::gbc_cpu::{GbcCpu, Flag};
use crate::opcodes::opcodes_utils::{
    get_arithmetic_16reg,
    check_for_half_carry_third_nible
};

pub fn add_hl_rr(cpu:&mut GbcCpu, opcode:u8){
    let reg = opcode & 0xF0;
    let reg = *get_arithmetic_16reg(cpu, reg);

    let (value,overflow) = cpu.hl.value.overflowing_add(reg);
    cpu.set_by_value(Flag::Carry, overflow);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_third_nible(cpu.hl.value, reg));
    cpu.unset_flag(Flag::Subtraction);

    cpu.hl.value = value;
}

pub fn add_sp_dd(cpu:&mut GbcCpu, opcode:u16){
    let dd = (opcode & 0xFF) as i8;
    let mut operation_res:(u16, bool) = (0,false);
    
    let mut temp = cpu.stack_pointer as i32;
    temp += dd as i32;
    operation_res.0 = temp as u16;
    if temp < 0{
        operation_res.1 = true;   
    }

    cpu.unset_flag(Flag::Zero);
    cpu.unset_flag(Flag::Subtraction);
    cpu.set_by_value(Flag::Carry, operation_res.1);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_third_nible(cpu.program_counter, dd as u16));

    cpu.stack_pointer = operation_res.0;
}

pub fn inc_rr(cpu:&mut GbcCpu, opcode:u8){
    let reg = opcode & 0xF0;
    let reg = get_arithmetic_16reg(cpu, reg);
    *reg+=1;
}


pub fn dec_rr(cpu:&mut GbcCpu, opcode:u8){
    let reg = opcode & 0xF0;
    let reg = get_arithmetic_16reg(cpu, reg);
    *reg-=1;
}