extern crate lib_gbc;
use lib_gbc::cpu::gbc_cpu::{GbcCpu,Flag};
use lib_gbc::opcodes::arithmetic_16bit_instructions;

#[test]
fn test_add_sp_dd_positive_dd(){
    let mut cpu = GbcCpu::default();
    let opcode:u16 = 88;
    arithmetic_16bit_instructions::add_sp_dd(&mut cpu, opcode);
    assert_eq!(cpu.stack_pointer, 88);
}

#[test]
fn test_add_sp_dd_negative_dd(){
    let mut cpu = GbcCpu::default();
    cpu.stack_pointer = 100;
    let opcode:u16 = 0xCE; //signed is -50
    arithmetic_16bit_instructions::add_sp_dd(&mut cpu, opcode);
    assert_eq!(cpu.stack_pointer, 50);
}