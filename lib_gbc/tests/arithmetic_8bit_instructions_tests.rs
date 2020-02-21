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