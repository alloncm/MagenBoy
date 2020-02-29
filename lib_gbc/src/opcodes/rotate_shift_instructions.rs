use crate::cpu::gbc_cpu::*;

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
    *r |= temp & 0b00000001;
    return temp != 0;
}

fn rotate_right(r:&mut u8)->bool{
    let mut temp:u8 = *r;
    *r = *r>>1;
    temp = temp << 7;
    *r |= temp & 0b10000000;
    return temp != 0;
}

fn rotate_left_carry(r:&mut u8)->bool{
    let temp:u8 = *r;
    *r = *r<<1;
    return temp & 0b10000000 != 0;
}

fn rotate_right_carry(r:&mut u8)->bool{
    let temp:u8 = *r;
    *r = *r>>1;
    return temp & 0b00000001 != 0;
}

pub fn rlca(cpu:&mut GbcCpu, opcode:u8){
    let carry:bool = rotate_left_carry(cpu.af.high());

    a_rotate_flags(cpu, carry);
}

pub fn rla(cpu:&mut GbcCpu, opcode:u8){
    let carry:bool = rotate_left(cpu.af.high());

    a_rotate_flags(cpu, carry);
}   

pub fn rrca(cpu:&mut GbcCpu, opcode:u8){
    let carry:bool = rotate_right_carry(cpu.af.high());

    a_rotate_flags(cpu, carry);
}


pub fn rra(cpu:&mut GbcCpu, opcode:u8){
    let carry:bool = rotate_right(cpu.af.high());

    a_rotate_flags(cpu, carry);
}   

fn get_cb_opcode(cb_opcode:u16)->u8{
    (cb_opcode & 0xFF) as u8
}