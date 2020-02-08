extern crate lib_gbc;
use lib_gbc::cpu::gbc_cpu::GbcCpu;

#[test]
fn test_af() {
    let mut cpu: GbcCpu = GbcCpu::default();
    cpu.a = 0x5;
    cpu.f = 0x6;
    assert_eq!(cpu.af(), 0x0506);
}

#[test]
fn test_bc() {
    let mut cpu: GbcCpu = GbcCpu::default();
    cpu.b = 0x5;
    cpu.c = 0x6;
    assert_eq!(cpu.bc(), 0x0506);
}

#[test]
fn test_de() {
    let mut cpu: GbcCpu = GbcCpu::default();
    cpu.d = 0x5;
    cpu.e = 0x6;
    assert_eq!(cpu.de(), 0x0506);
}

#[test]
fn test_hl() {
    let mut cpu: GbcCpu = GbcCpu::default();
    cpu.h = 0x5;
    cpu.l = 0x6;
    assert_eq!(cpu.hl(), 0x0506);
}

#[test]
fn test_get_register() {
    let mut cpu = GbcCpu::default();
    cpu.a = 6;
    let value = *cpu.get_register(0b111);
    assert_eq!(value, cpu.a);
}

#[test]
fn test_set_16bit_register(){
    let mut cpu = GbcCpu::default();
    cpu.set_16bit_register(0, 0x1111);
    assert_eq!(cpu.b, 0x11);
    assert_eq!(cpu.c, 0x11);
}
