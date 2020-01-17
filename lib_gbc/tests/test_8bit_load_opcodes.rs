extern crate lib_gbc;
use lib_gbc::cpu::gbc_cpu::GbcCpu;
use lib_gbc::opcodes::load_8bit_instructions;

#[test]
fn test_ld_r_r() {
    let mut cpu = GbcCpu::default();
    cpu.a = 6;
    load_8bit_instructions::ld_r_r(&mut cpu, 0, 7);
    assert_eq!(cpu.a, cpu.b);
}

#[test]
fn test_ld_r_n() {
    let mut cpu = GbcCpu::default();
    load_8bit_instructions::ld_r_n(&mut cpu, 0, 120);
    assert_eq!(cpu.b, 120);
}
