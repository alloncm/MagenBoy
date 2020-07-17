extern crate lib_gbc;
use lib_gbc::cpu::gbc_cpu::{GbcCpu,Flag};
use lib_gbc::opcodes::arithmetic_16bit_instructions;

#[test]
fn test_add_sp_dd_positive_dd(){
    let mut cpu = GbcCpu::default();
    let opcode:u16 = 88;
    arithmetic_16bit_instructions::add_sp_dd(&mut cpu, opcode);
    assert_eq!(cpu.stack_pointer, 88);
    assert_eq!(cpu.get_flag(Flag::Carry),false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry),false);
    assert_eq!(cpu.get_flag(Flag::Subtraction),false);
    assert_eq!(cpu.get_flag(Flag::Zero),false);
}

#[test]
fn test_add_sp_dd_negative_dd(){
    let mut cpu = GbcCpu::default();
    cpu.stack_pointer = 100;
    let opcode:u16 = 0xCE; //signed is -50
    arithmetic_16bit_instructions::add_sp_dd(&mut cpu, opcode);
    assert_eq!(cpu.stack_pointer, 50);
    assert_eq!(cpu.get_flag(Flag::Carry),false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry),false);
    assert_eq!(cpu.get_flag(Flag::Subtraction),false);
    assert_eq!(cpu.get_flag(Flag::Zero),false);
}


#[test]
fn test_add_sp_dd(){
    let mut cpu = GbcCpu::default();
    cpu.stack_pointer =0xFFF8;
    let opcode:u16 = 2;
    arithmetic_16bit_instructions::add_sp_dd(&mut cpu, opcode);
    assert_eq!(cpu.stack_pointer, 0xFFFA);
    assert_eq!(cpu.get_flag(Flag::Carry),false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry),false);
    assert_eq!(cpu.get_flag(Flag::Subtraction),false);
    assert_eq!(cpu.get_flag(Flag::Zero),false);
}

#[test]
fn test_add_sp_dd_zero_overflow(){
    let mut cpu = GbcCpu::default();
    cpu.stack_pointer =0xFFFF;
    let opcode:u16 = 0x1;
    arithmetic_16bit_instructions::add_sp_dd(&mut cpu, opcode);
    assert_eq!(cpu.stack_pointer, 0);
    assert_eq!(cpu.get_flag(Flag::Carry),true);
    assert_eq!(cpu.get_flag(Flag::HalfCarry),false);
    assert_eq!(cpu.get_flag(Flag::Subtraction),false);
    assert_eq!(cpu.get_flag(Flag::Zero),false);
}

#[test]
fn test_add_sp_dd_zero_half_carry(){
    let mut cpu = GbcCpu::default();
    cpu.stack_pointer =0xFFF;
    let opcode:u16 = 0x1;
    arithmetic_16bit_instructions::add_sp_dd(&mut cpu, opcode);
    assert_eq!(cpu.stack_pointer, 0x1000);
    assert_eq!(cpu.get_flag(Flag::Carry),false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry),true);
    assert_eq!(cpu.get_flag(Flag::Subtraction),false);
    assert_eq!(cpu.get_flag(Flag::Zero),false);
}