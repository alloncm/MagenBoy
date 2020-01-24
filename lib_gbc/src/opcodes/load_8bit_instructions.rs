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

//load the value in address of HL into fest register
pub fn ld_r_hl(cpu:&mut GbcCpu, memory:&dyn Memory, dest:u8){
    *cpu.get_register(dest) = memory.read(cpu.af());
}   

//load the value in reg_src into the address of HL in memory
pub fn ld_hl_r(cpu:&mut GbcCpu, memory:&mut dyn Memory, reg_src:u8){
    memory.write(cpu.hl(), *cpu.get_register(reg_src));
}

//load the valie src into the address HL in memory
pub fn ld_hl_n(cpu: &mut GbcCpu, memory:&mut dyn Memory, src: u8){
    memory.write(cpu.hl(), src);
}

//load the value in address of BC into register A
pub fn ld_a_bc(cpu: &mut GbcCpu, memory:&dyn Memory){
    cpu.a = memory.read(cpu.bc());
}

//load the value in address of DE into register A
pub fn ld_a_de(cpu: &mut GbcCpu, memory:&dyn Memory){
    cpu.a = memory.read(cpu.de());
}

//
pub fn ld_a_nn(cpu: &mut GbcCpu, memory:&dyn Memory, address:u16){
    cpu.a = memory.read(address);
}