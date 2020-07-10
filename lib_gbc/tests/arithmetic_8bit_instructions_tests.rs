extern crate lib_gbc;
use lib_gbc::cpu::gbc_cpu::{GbcCpu,Flag};
use lib_gbc::opcodes::arithmetic_8bit_instructions;

#[test]
fn daa_after_add_op(){
    let mut cpu = GbcCpu::default();
    *cpu.af.high() = 0x7D;
    *cpu.af.low() = 0;
    arithmetic_8bit_instructions::daa(&mut cpu);
    assert_eq!(*cpu.af.high(), 0x83);
    assert_eq!(cpu.get_flag(Flag::Carry), false);
}

#[test]
fn daa_after_sub_op(){
    let mut cpu = GbcCpu::default();
    *cpu.af.high() = 0x4B;
    cpu.set_flag(Flag::Subtraction);
    arithmetic_8bit_instructions::daa(&mut cpu);
    assert_eq!(*cpu.af.high(), 0x45);
}

#[test]
fn test_sub_a_nn_for_half_carry_true(){
    let mut cpu = GbcCpu::default();
    *cpu.af.high() = 0x3E;
    let opcode = 0x0F;
    arithmetic_8bit_instructions::sub_a_nn(&mut cpu, opcode);

    assert_eq!(*cpu.af.high(), 0x2F);
    assert_eq!(cpu.get_flag(Flag::Zero),false);
    assert_eq!(cpu.get_flag(Flag::Subtraction),true);
    assert_eq!(cpu.get_flag(Flag::HalfCarry),true);
    assert_eq!(cpu.get_flag(Flag::Carry),false);
}

#[test]
fn test_sub_a_nn_for_half_carry_false(){
    let mut cpu = GbcCpu::default();
    *cpu.af.high() = 0x3E;
    let opcode = 0x3E;
    arithmetic_8bit_instructions::sub_a_nn(&mut cpu, opcode);

    assert_eq!(*cpu.af.high(), 0);
    assert_eq!(cpu.get_flag(Flag::Zero),true);
    assert_eq!(cpu.get_flag(Flag::Subtraction),true);
    assert_eq!(cpu.get_flag(Flag::HalfCarry),false);
    assert_eq!(cpu.get_flag(Flag::Carry),false);
}

#[test]
fn test_sub_a_nn_for_carry_carry(){
    let mut cpu = GbcCpu::default();
    *cpu.af.high() = 0x3E;
    let opcode = 0x40;
    arithmetic_8bit_instructions::sub_a_nn(&mut cpu, opcode);

    assert_eq!(*cpu.af.high(), 0xFE);
    assert_eq!(cpu.get_flag(Flag::Zero),false);
    assert_eq!(cpu.get_flag(Flag::Subtraction),true);
    assert_eq!(cpu.get_flag(Flag::HalfCarry),false);
    assert_eq!(cpu.get_flag(Flag::Carry),true);
}