use crate::cpu::gbc_cpu::{GbcCpu, Flag};
use crate::machine::memory::Memory;
use crate::opcodes::opcodes_utils::check_for_half_carry_third_nible;

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
pub fn pop(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u8){
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

//push to stack the register 
pub fn push(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u8){
    let reg = opcode&0xF0;
    let reg = match reg{
        0xC=>&mut cpu.bc,
        0xD=>&mut cpu.de,
        0xE=>&mut cpu.hl,
        0xF=>&mut cpu.af,
        _=>panic!("no register")
    };

    memory.write(cpu.stack_pointer, *reg.high());
    memory.write(cpu.stack_pointer-1, *reg.low());
    cpu.stack_pointer-=2;
}

//load into hl sp + rr
pub fn ld_hl_spnn(cpu:&mut GbcCpu, opcode:u16){
    let nn = opcode & 0xFF;
    let (value,overflow) = cpu.stack_pointer.overflowing_add(nn);
    cpu.hl.value = value;

    //check for carry
    if overflow{
        cpu.set_flag(Flag::Carry);
    }

    //check for half carry
    if check_for_half_carry_third_nible(cpu.stack_pointer,nn as u8){
        cpu.set_flag(Flag::HalfCarry);
    }

    cpu.unset_flag(Flag::Zero);
    cpu.unset_flag(Flag::Subtraction);
}

//load sp into memory
pub fn ld_nn_sp(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u32){
    let address = (opcode & 0xFFFF) as u16;
    let low = (cpu.stack_pointer & 0xFF) as u8;
    let high = ((cpu.stack_pointer & 0xFF)>>8) as u8;
    memory.write(address, low);
    memory.write(address, high);
}