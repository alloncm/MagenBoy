use crate::cpu::gbc_cpu::GbcCpu;
use crate::mmu::memory::Memory;
use crate::opcodes::opcodes_utils::{
    get_src_register,
    get_reg_two_rows
};

const IO_PORTS_ADDRESS:u16 = 0xFF00;



fn get_dest_register(cpu: &mut GbcCpu, opcode:u8)->&mut u8{
    let reg_num = opcode & 0b11111000;
    return match reg_num{
        0x40=>cpu.bc.high(),
        0x48=>cpu.bc.low(),
        0x50=>cpu.de.high(),
        0x58=>cpu.de.low(),
        0x60=>cpu.hl.high(),
        0x68=>cpu.hl.low(),
        0x78=>cpu.af.high(),
        _=>panic!("no register: {}",reg_num)
    };
}

//load src register value into dest register
pub fn ld_r_r(cpu: &mut GbcCpu, opcode:u8) {
    let src_register_value:u8 = *get_src_register(cpu, opcode);
    let dest_register = get_dest_register(cpu, opcode);
    *dest_register = src_register_value;
}

//load src value into dest register
pub fn ld_r_n(cpu: &mut GbcCpu, opcode:u16) {
    let reg = get_reg_two_rows(cpu,((opcode&0xFF00)>>8) as u8);
    let n = (opcode&0xFF) as u8;
    *reg = n;
}

//load the value in address of HL into dest register
pub fn ld_r_hl(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u8){
    let reg = opcode&0b11111000;
    let hl_value = cpu.hl.value;
    let reg = match reg{
        0x40=>cpu.bc.high(),
        0x41=>cpu.bc.low(),
        0x50=>cpu.de.high(),
        0x51=>cpu.de.low(),
        0x60=>cpu.hl.high(),
        0x61=>cpu.hl.low(),
        0x71=>cpu.af.high(),
        _=>panic!("no register")
    };

    *reg = memory.read(hl_value);
}   

//load the value in reg_src into the address of HL in memory
pub fn ld_hl_r(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u8){
    memory.write(cpu.hl.value, *get_src_register(cpu, opcode));
}

//load the valie src into the address HL in memory
pub fn ld_hl_n(cpu: &mut GbcCpu, memory:&mut dyn Memory, opcode:u16){
    let src = (0xFF & opcode) as u8;
    memory.write(cpu.hl.value, src);
}

//load the value in address of BC into register A
pub fn ld_a_bc(cpu: &mut GbcCpu, memory:&mut dyn Memory){
    *cpu.af.low() = memory.read(cpu.bc.value);
}

//load the value in address of DE into register A
pub fn ld_a_de(cpu: &mut GbcCpu, memory:&mut dyn Memory){
    *cpu.af.low() = memory.read(cpu.de.value);
}

//load the value at address NN into register A
pub fn ld_a_nn(cpu: &mut GbcCpu, memory:&mut dyn Memory, opcode:u32){
    let address = (0xFFFF & opcode) as u16;
    *cpu.af.low() = memory.read(address);
}

//load the value in register A into the address of BC
pub fn ld_bc_a(cpu: &mut GbcCpu, memory:&mut dyn Memory){
    memory.write(cpu.bc.value, *cpu.af.low());
}

//load the value in register A into the address of DE
pub fn ld_de_a(cpu: &mut GbcCpu, memory:&mut dyn Memory){
    memory.write(cpu.de.value, *cpu.af.low());
}

//load the value in register A into the address of NN
pub fn ld_nn_a(cpu: &mut GbcCpu, memory:&mut dyn Memory, opcode:u32){
    let address = (0xFFFF & opcode) as u16;
    memory.write(address, *cpu.af.low());
}

//load value in register A into address HL and then increment register HL value
pub fn ldi_hl_a(cpu: &mut GbcCpu, memory:&mut dyn Memory){
    memory.write(cpu.hl.value, *cpu.af.low());
    cpu.inc_hl();
}

//load into register A the value in address HL and then increment register HL value
pub fn ldi_a_hl(cpu: &mut GbcCpu, memory:&mut dyn Memory){
    *cpu.af.low() = memory.read(cpu.hl.value);
    cpu.inc_hl();
}

//load value in register A into address HL and then decrement register HL value
pub fn ldd_hl_a(cpu: &mut GbcCpu, memory:&mut dyn Memory){
    memory.write(cpu.hl.value, *cpu.af.low());
    cpu.dec_hl();
}

//load into register A the value in address HL and then decrement register HL value
pub fn ldd_a_hl(cpu: &mut GbcCpu, memory:&mut dyn Memory){
    *cpu.af.low() = memory.read(cpu.hl.value);
    cpu.dec_hl();
}

//load into register A the value in io port N
pub fn ld_a_ioport_n(cpu: &mut GbcCpu, memory:&mut dyn Memory, opcode:u16){
    let io_port = 0x00FF & opcode;
    *cpu.af.low() = memory.read(IO_PORTS_ADDRESS + (io_port as u16));
}

//load into io port N the value in register A
pub fn ld_ioport_n_a(cpu: &mut GbcCpu, memory: &mut dyn Memory, opcode:u16){
    let io_port = 0x00FF & opcode;
    memory.write(IO_PORTS_ADDRESS + (io_port as u16), *cpu.af.low());
}

//load into io port C the value in register A
pub fn ld_ioport_c_a(cpu: &mut GbcCpu, memory: &mut dyn Memory){
    memory.write(IO_PORTS_ADDRESS + (*cpu.bc.high() as u16), *cpu.af.low());
}

pub fn ld_a_ioport_c(cpu: &mut GbcCpu, memory: &mut dyn Memory){
    *cpu.af.low() = memory.read(IO_PORTS_ADDRESS + (*cpu.bc.high() as u16));
}