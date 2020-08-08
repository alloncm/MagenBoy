use crate::cpu::gbc_cpu::*;
use crate::opcodes::opcodes_utils::*;
use crate::mmu::memory::Memory;
use crate::utils::bit_masks::*;
use log::info;

fn a_rotate_flags(cpu:&mut GbcCpu, carry:bool){
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);
    cpu.unset_flag(Flag::Zero);
    cpu.set_by_value(Flag::Carry, carry);
}

fn rotate_left(r:&mut u8)->bool{
    let mut temp:u8 = *r;
    *r = *r<<1;
    temp = temp >> 7;
    *r |= temp & BIT_0_MASK;
    return temp != 0;
}

fn rotate_right(r:&mut u8)->bool{
    let mut temp:u8 = *r;
    *r = *r>>1;
    temp = temp << 7;
    *r |= temp & BIT_7_MASK;
    return temp != 0;
}

fn rotate_left_carry(r:&mut u8, carry:bool)->bool{
    let temp:u8 = *r;
    *r = *r<<1;
    if carry{
        *r|=0x1
    }
    return (temp & BIT_7_MASK) != 0;
}

fn rotate_right_carry(r:&mut u8, carry:bool)->bool{
    let temp:u8 = *r;
    *r = *r>>1;
    if carry{
        *r|=0x80;
    }
    return (temp & BIT_0_MASK) != 0;
}

pub fn rlca(cpu:&mut GbcCpu){
    let carry:bool = rotate_left(cpu.af.high());

    a_rotate_flags(cpu, carry);
}

pub fn rla(cpu:&mut GbcCpu){
    let carry_flag = cpu.get_flag(Flag::Carry);
    let carry:bool = rotate_left_carry(cpu.af.high(), carry_flag);

    a_rotate_flags(cpu, carry);
}   

pub fn rrca(cpu:&mut GbcCpu){
    let carry:bool = rotate_right(cpu.af.high());

    a_rotate_flags(cpu, carry);
}


pub fn rra(cpu:&mut GbcCpu){
    let carry_flag = cpu.get_flag(Flag::Carry);
    let carry:bool = rotate_right_carry(cpu.af.high(), carry_flag);

    a_rotate_flags(cpu, carry);
}   

fn rotate_shift_flags(cpu:&mut GbcCpu, carry:bool, zero:bool){
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);
    cpu.set_by_value(Flag::Zero, zero);
    cpu.set_by_value(Flag::Carry, carry);
}

pub fn rlc_r(cpu:&mut GbcCpu, opcode:u16)
{
    let opcode:u8 = get_cb_opcode(opcode);
    let register_value:u8;
    let carry:bool;

    {
        let register:&mut u8 = get_src_register(cpu, opcode);
        carry = rotate_left(register);
        register_value = *register;
    }

    rotate_shift_flags(cpu, carry, register_value == 0);
}

pub fn rlc_hl(cpu:&mut GbcCpu, memory:&mut dyn Memory)
{
    let mut byte: u8 = memory.read(*cpu.hl.value());
    let carry:bool = rotate_left(&mut byte);
    memory.write(*cpu.hl.value(), byte);
    rotate_shift_flags(cpu, carry, byte == 0);
}

pub fn rl_r(cpu:&mut GbcCpu, opcode:u16)
{
    let opcode:u8 = get_cb_opcode(opcode);
    let register_value:u8;
    let carry:bool;

    {
        let carry_flag = cpu.get_flag(Flag::Carry);
        let register:&mut u8 = get_src_register(cpu, opcode);
        carry = rotate_left_carry(register, carry_flag);
        register_value = *register;
    }

    rotate_shift_flags(cpu, carry, register_value == 0);
}

pub fn rl_hl(cpu:&mut GbcCpu, memory:&mut dyn Memory)
{
    let mut byte: u8 = memory.read(*cpu.hl.value());
    //info!("rl hl: {} val: {:#X}", *cpu.hl.value(), byte);
    let carry:bool = rotate_left_carry(&mut byte, cpu.get_flag(Flag::Carry));
    memory.write(*cpu.hl.value(), byte);
    rotate_shift_flags(cpu, carry, byte == 0);
    //info!("val {:#X} flags {:#b}",byte, cpu.af.low());
}

pub fn rrc_r(cpu:&mut GbcCpu, opcode:u16)
{
    let opcode:u8 = get_cb_opcode(opcode);
    let register_value:u8;
    let carry:bool;

    {
        let register:&mut u8 = get_src_register(cpu, opcode);
        carry = rotate_right(register);
        register_value = *register;
    }

    rotate_shift_flags(cpu, carry, register_value == 0);
}

pub fn rrc_hl(cpu:&mut GbcCpu, memory:&mut dyn Memory)
{
    let mut byte: u8 = memory.read(*cpu.hl.value());
    let carry:bool = rotate_right(&mut byte);
    memory.write(*cpu.hl.value(), byte);
    rotate_shift_flags(cpu, carry, byte == 0);
}

pub fn rr_r(cpu:&mut GbcCpu, opcode:u16)
{
    let opcode:u8 = get_cb_opcode(opcode);
    let register_value:u8;
    let carry:bool;

    {
        let carry_flag = cpu.get_flag(Flag::Carry);
        let register:&mut u8 = get_src_register(cpu, opcode);
        carry = rotate_right_carry(register, carry_flag);
        register_value = *register;
    }

    rotate_shift_flags(cpu, carry, register_value == 0);
}

pub fn rr_hl(cpu:&mut GbcCpu, memory:&mut dyn Memory)
{
    //debug
    info!("rr_hl");
    let mut byte: u8 = memory.read(*cpu.hl.value());
    let carry_flag = cpu.get_flag(Flag::Carry);
    let carry:bool = rotate_right_carry(&mut byte, carry_flag);
    memory.write(*cpu.hl.value(), byte);
    rotate_shift_flags(cpu, carry, byte == 0);
}

fn shift_left(r:&mut u8)->bool{
    let temp:u8 = *r;
    *r = *r<<1;
    return temp & BIT_7_MASK != 0;
}

fn arithmetic_shift_right(r:&mut u8)->bool{
    let temp:u8 = *r;
    *r = *r>>1;
    *r |= temp & BIT_7_MASK;
    return temp & BIT_0_MASK != 0;
}

fn logical_shift_right(r:&mut u8)->bool{
    let temp:u8 = *r;
    *r = *r>>1;
    return temp & BIT_0_MASK != 0;
}

pub fn sla_r(cpu:&mut GbcCpu, opcode:u16)
{
    let opcode:u8 = get_cb_opcode(opcode);
    let register_value:u8;
    let carry:bool;

    {
        let register:&mut u8 = get_src_register(cpu, opcode);
        carry = shift_left(register);
        register_value = *register;
    }

    rotate_shift_flags(cpu, carry, register_value == 0);
}

pub fn sla_hl(cpu:&mut GbcCpu, memory:&mut dyn Memory)
{
    let mut byte: u8 = memory.read(*cpu.hl.value());
    //debug
    info!("sla hl: {} val: {:#X}", *cpu.hl.value(), byte);
    let carry:bool = shift_left(&mut byte);
    memory.write(*cpu.hl.value(), byte);
    rotate_shift_flags(cpu, carry, byte == 0);
    //debug
    info!("val {:#X} flags {:#b} hl {:#X}",byte, cpu.af.low(), *cpu.hl.value());
}

fn swap_nibbles(r:&mut u8){
    let value = *r;
    let mut temp:u8 = (value&0xF0)>>4;
    temp |= (value &0xF)<<4;
    *r = temp;
}

fn set_swap_flags(cpu:&mut GbcCpu, zero:bool){
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);
    cpu.set_by_value(Flag::Zero,zero);
    cpu.unset_flag(Flag::Carry);
}

pub fn swap_r(cpu:&mut GbcCpu, opcode:u16){
    let value:u8;
    let opcode:u8 = get_cb_opcode(opcode);
    {
        let register:&mut u8 = get_src_register(cpu, opcode);
        swap_nibbles(register);
        value = *register;
    }

    set_swap_flags(cpu, value == 0);
}

pub fn swap_hl(cpu:&mut GbcCpu, memory:&mut dyn Memory)
{
    let mut byte: u8 = memory.read(*cpu.hl.value());
    swap_nibbles(&mut byte);
    memory.write(*cpu.hl.value(), byte);
    set_swap_flags(cpu, byte == 0);
}

pub fn sra_r(cpu:&mut GbcCpu, opcode:u16)
{
    let opcode:u8 = get_cb_opcode(opcode);
    let register_value:u8;
    let carry:bool;
    {
        let register:&mut u8 = get_src_register(cpu, opcode);
        carry = arithmetic_shift_right(register);
        register_value = *register;
    }

    rotate_shift_flags(cpu, carry, register_value == 0);
}

pub fn sra_hl(cpu:&mut GbcCpu, memory:&mut dyn Memory)
{
    let mut byte: u8 = memory.read(*cpu.hl.value());
    let carry:bool = arithmetic_shift_right(&mut byte);
    memory.write(*cpu.hl.value(), byte);
    rotate_shift_flags(cpu, carry, byte == 0);
}

pub fn srl_r(cpu:&mut GbcCpu, opcode:u16)
{
    let opcode:u8 = get_cb_opcode(opcode);
    let register_value:u8;
    let carry:bool;
    {
        let register:&mut u8 = get_src_register(cpu, opcode);
        carry = logical_shift_right(register);
        register_value = *register;
    }

    rotate_shift_flags(cpu, carry, register_value == 0);
}

pub fn srl_hl(cpu:&mut GbcCpu, memory:&mut dyn Memory)
{
    let mut byte: u8 = memory.read(*cpu.hl.value());
    let carry:bool = logical_shift_right(&mut byte);
    memory.write(*cpu.hl.value(), byte);
    rotate_shift_flags(cpu, carry, byte == 0);
}
