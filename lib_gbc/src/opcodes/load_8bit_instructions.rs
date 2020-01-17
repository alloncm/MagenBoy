use crate::cpu::gbc_cpu::GbcCpu;

pub fn ld_r_r(cpu: &mut GbcCpu, dest: u8, src: u8) {
    let src_register_value: u8 = *cpu.get_register(src);
    let dest_register = cpu.get_register(dest);
    *dest_register = src_register_value;
}
