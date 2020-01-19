use crate::cpu::gbc_cpu::GbcCpu;
use crate::machine::memory::Memory;

//load src register value into dest register
pub fn ld_r_r(cpu: &mut GbcCpu, dest: u8, src: u8) {
    let src_register_value: u8 = *cpu.get_register(src);
    let dest_register = cpu.get_register(dest);
    *dest_register = src_register_value;
}

//load src value into dest register
pub fn ld_r_n(cpu: &mut GbcCpu, dest: u8, src: u8) {
    *cpu.get_register(dest) = src;
}

pub fn ld_r_hl(cpu:&mut GbcCpu, memory:&Memory, dest:u8){
    *cpu.get_register(dest) = memory.read(cpu.af());
}   