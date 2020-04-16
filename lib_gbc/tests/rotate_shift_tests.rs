extern crate lib_gbc;
use lib_gbc::opcodes::rotate_shift_instructions::*;
use lib_gbc::cpu::gbc_cpu::*;

#[test]
fn test_rlc_r(){
    let opcode:u16 = 0xCB00;
    let mut cpu = GbcCpu::default();
    *cpu.bc.high() = 0x85;
    rlc_r(&mut cpu, opcode);
    assert_eq!(*cpu.bc.high(), 0xB);
}


#[test]
fn test_rl_carry_not_set_r(){
    let opcode:u16 = 0xCB10;
    let mut cpu = GbcCpu::default();
    *cpu.bc.high() = 0x85;
    rl_r(&mut cpu, opcode);
    assert_eq!(*cpu.bc.high(), 0xA);
}

#[test]
fn test_rl_carry_set_r(){
    let opcode:u16 = 0xCB10;
    let mut cpu = GbcCpu::default();
    *cpu.bc.high() = 0x85;
    cpu.set_flag(Flag::Carry);
    rl_r(&mut cpu, opcode);
    assert_eq!(*cpu.bc.high(), 0xB);
}

#[test]
fn test_rla(){
    let opcode:u16 = 0xCB10;
    let mut cpu = GbcCpu::default();
    *cpu.af.high() = 0x85;
    rla(&mut cpu);
    assert_eq!(*cpu.af.high(), 0xA);
}