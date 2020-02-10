use crate::cpu::gbc_cpu::GbcCpu;
use crate::machine::memory::Memory;

//load into 16bit register RR the value NN
pub fn load_rr_nn(cpu:&mut GbcCpu, opcode:u32){
    let reg = (opcode>>16) & 0xF;
    let nn = (opcode&0xFFFF) as u16;
    let reg = match reg{
        0x0=>&mut cpu.bc.value,
        0x1=>&mut cpu.de.value,
        0x2=>&mut cpu.hl.value,
        0x3=>&mut cpu.stack_pointer,
        _=>panic!("no register")
    };

    *reg = nn;
}

//loads register HL into the SP
pub fn load_sp_hl(cpu:&mut GbcCpu){
    cpu.stack_pointer = cpu.hl.value;
}

//pop from the stack pointer to one register
pub fn pop(cpu:&mut GbcCpu, memory:&dyn Memory, opcode:u8){
    let reg = opcode&0xF0;
    let reg = match reg{
        0xC=>&mut cpu.bc,
        0xD=>&mut cpu.de,
        0xE=>&mut cpu.hl,
        0xF=>&mut cpu.af,
        _=>panic!("no register")
    };

    *reg.high() = memory.read(cpu.stack_pointer);
    *reg.low() = memory.read(cpu.stack_pointer+1);
    cpu.stack_pointer+=2;
}