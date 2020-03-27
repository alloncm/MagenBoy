use crate::cpu::gbc_cpu::*;
use crate::opcodes::opcodes_utils::*;
use crate::mmu::memory::Memory;


fn get_bit_number(opcode:u8)->u8{
    let bit_number:u8 = (opcode & 0b00111000)>>3;
    return 1<<bit_number;
}

fn set_flags_bit(cpu:&mut GbcCpu, zero:bool){
    cpu.set_by_value(Flag::Zero, zero);
    cpu.unset_flag(Flag::Subtraction);
    cpu.set_flag(Flag::HalfCarry);
}

pub fn bit_r(cpu:&mut GbcCpu, opcode:u16){
    let opcode = get_cb_opcode(opcode);
    let register:&mut u8 = get_src_register(cpu, opcode);
    let bit_number = get_bit_number(opcode);
    let bit = *register & bit_number;
    set_flags_bit(cpu, bit == 0);
}

pub fn bit_hl(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u16){
    let opcode = get_cb_opcode(opcode);
    let byte = memory.read(cpu.hl.value);
    let bit_number = get_bit_number(opcode);
    let bit = byte & bit_number;
    set_flags_bit(cpu, bit == 0);
}

pub fn set_r(cpu:&mut GbcCpu, opcode:u16){
    let opcode = get_cb_opcode(opcode);
    let register:&mut u8 = get_src_register(cpu, opcode);
    let bit_number = get_bit_number(opcode);
    *register |= bit_number;
}

pub fn set_hl(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u16){
    let opcode = get_cb_opcode(opcode);
    let mut byte = memory.read(cpu.hl.value);
    let bit_number = get_bit_number(opcode);
    byte |= bit_number;
    memory.write(cpu.hl.value, byte);
}

pub fn res_r(cpu:&mut GbcCpu, opcode:u16){
    let opcode = get_cb_opcode(opcode);
    let register:&mut u8 = get_src_register(cpu, opcode);
    let bit_number = get_bit_number(opcode);
    let bit_mask:u8 = 0xFF ^ bit_number;
    *register &= bit_mask;
}

pub fn res_hl(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u16){
    let opcode = get_cb_opcode(opcode);
    let mut byte = memory.read(cpu.hl.value);
    let bit_number = get_bit_number(opcode);
    let bit_mask:u8 = 0xFF ^ bit_number;
    byte &= bit_mask;
    memory.write(cpu.hl.value, byte);
}