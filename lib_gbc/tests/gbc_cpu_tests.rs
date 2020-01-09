
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