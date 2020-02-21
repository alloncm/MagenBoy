use crate::cpu::gbc_cpu::{GbcCpu, Flag};

pub fn get_src_register(cpu: &mut GbcCpu, opcode:u8)-> &mut u8{
    let reg_num = opcode & 0b111;
    return match reg_num{
        0x0=>cpu.bc.low(),
        0x1=>cpu.bc.high(),
        0x2=>cpu.de.low(),
        0x3=>cpu.de.high(),
        0x4=>cpu.hl.low(),
        0x5=>cpu.hl.high(),
        0x7=>cpu.af.low(),
        _=>panic!("no register: {}",reg_num)
    };
}

pub fn check_for_half_carry_third_nible(a:u16, b:u8)->bool{
    ((a & 0xFFF) + (b as u16)) & 0x1000 == 0x1000
}

pub fn check_for_half_carry_first_nible_add(a:u8, b:u8)->bool{
    ((a & 0xF) + (b & 0xF)) & 0x10 == 0x10
}

pub fn check_for_half_carry_first_nible_sub(a:u8, b:u8)->bool{
    let sa = a as i16;
    let sb = b as i16;
    ((sa & 0xF) - (sb & 0xF)) < 0
}