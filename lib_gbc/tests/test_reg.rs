extern crate lib_gbc;
use lib_gbc::cpu::register::Reg;

#[test]
fn test_low_reg(){
    let mut r = Reg{value:0};
    *r.get_low() = 10;
    assert_eq!(r.value, 10);
    assert_eq!(*r.get_low(), 10);
}

#[test]
fn test_high_reg(){
    let mut r = Reg{value:0};
    *r.get_high() = 0x10;
    assert_eq!(r.value, 0x1000);
    assert_eq!(*r.get_high(), 0x10);
}

#[test]
fn test_low_high(){
    let mut r = Reg{value:0};
    *r.get_high() = 0x10;
    *r.get_low() = 0xFF;
    assert_eq!(r.value, 0x10FF);
    assert_eq!(*r.get_high(), 0x10);
    assert_eq!(*r.get_low(), 0xFF);
}

#[test]
fn test_value(){
    let mut r = Reg{value:0};
    r.value = 0x10FF;
    assert_eq!(r.value, 0x10FF);
    assert_eq!(*r.get_high(), 0x10);
    assert_eq!(*r.get_low(), 0xFF);
}