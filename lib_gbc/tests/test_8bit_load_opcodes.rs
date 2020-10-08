extern crate lib_gbc;
use lib_gbc::cpu::gb_cpu::GbCpu;
use lib_gbc::cpu::opcodes::load_8bit_instructions;

#[test]
fn test_ld_r_r() {
    let mut cpu = GbCpu::default();
    *cpu.af.high() = 6;
    load_8bit_instructions::ld_r_r(&mut cpu,0x47);
    assert_eq!(*cpu.af.high(), *cpu.bc.high());
}

#[test]
fn test_ld_r_n() {
    let mut cpu = GbCpu::default();
    load_8bit_instructions::ld_r_n(&mut cpu, 0x0676);
    assert_eq!(*cpu.bc.high(), 0x76);
}
