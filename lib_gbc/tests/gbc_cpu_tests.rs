
extern crate lib_gbc;
use lib_gbc::cpu::GbcCpu;

#[test]
fn test_af()
{
    let mut cpu:GbcCpu = GbcCpu::default();
    cpu.a = 0x5;
    cpu.f = 0x6;
    assert_eq!(cpu.af(),0x0506);
}

#[test]
fn test_bc()
{
    let mut cpu:GbcCpu = GbcCpu::default();
    cpu.b = 0x5;
    cpu.c = 0x6;
    assert_eq!(cpu.bc(),0x0506);
}

#[test]
fn test_de()
{
    let mut cpu:GbcCpu = GbcCpu::default();
    cpu.d = 0x5;
    cpu.e = 0x6;
    assert_eq!(cpu.de(),0x0506);
}

#[test]
fn test_hl()
{
    let mut cpu:GbcCpu = GbcCpu::default();
    cpu.h = 0x5;
    cpu.l = 0x6;
    assert_eq!(cpu.hl(),0x0506);
}