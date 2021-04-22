use lib_gb::cpu::register::Reg;

#[test]
fn test_low_reg(){
    let mut r = Reg::default();
    *r.low() = 10;
    assert_eq!(*r.value(), 10);
    assert_eq!(*r.low(), 10);
}

#[test]
fn test_high_reg(){
    let mut r = Reg::default();
    *r.high() = 0x10;
    assert_eq!(*r.value(), 0x1000);
    assert_eq!(*r.high(), 0x10);
}

#[test]
fn test_low_high(){
    let mut r = Reg::default();
    *r.high() = 0x10;
    *r.low() = 0xFF;
    assert_eq!(*r.value(), 0x10FF);
    assert_eq!(*r.high(), 0x10);
    assert_eq!(*r.low(), 0xFF);
}

#[test]
fn test_value(){
    let mut r = Reg::default();
    *r.value() = 0x10FF;
    assert_eq!(*r.value(), 0x10FF);
    assert_eq!(*r.high(), 0x10);
    assert_eq!(*r.low(), 0xFF);
}