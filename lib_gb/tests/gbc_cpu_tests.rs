use lib_gb::cpu::gb_cpu::GbCpu;

#[test]
fn test_inc_hl(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 8200;
    cpu.inc_hl();
    assert_eq!(*cpu.hl.value_mut(), 8201);
    *cpu.hl.value_mut() = 0xFFFF;
    cpu.inc_hl();
    assert_eq!(*cpu.hl.value_mut(),0);
}

#[test]
fn test_dec_hl(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 8200;
    cpu.dec_hl();
    assert_eq!(*cpu.hl.value_mut(), 8199);
    *cpu.hl.value_mut() = 0;
    cpu.dec_hl();
    assert_eq!(*cpu.hl.value_mut() ,0xFFFF);
}