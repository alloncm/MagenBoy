use crate::cpu::gbc_cpu::{GbcCpu, Flag};
use crate::machine::memory::Memory;
use crate::opcodes::opcodes_utils::{
    check_for_half_carry_third_nible,
    get_arithmetic_16reg
};

//load into 16bit register RR the value NN
pub fn load_rr_nn(cpu:&mut GbcCpu, opcode:u32){
    let reg = (((opcode>>16) & 0xF0)>>4) as u8;
    let mut nn = ((opcode&0xFF)<<8) as u16;
    nn |= ((opcode&0xFF00)>>8) as u16;
    let reg = get_arithmetic_16reg(cpu, reg);

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

    *reg.low() = memory.read(cpu.stack_pointer);
    *reg.high() = memory.read(cpu.stack_pointer+1);
    cpu.stack_pointer+=2;
}

//push to stack the register 
pub fn push(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u8){
    let reg = (opcode&0xF0)>>4;
    let reg = match reg{
        0xC=>&mut cpu.bc,
        0xD=>&mut cpu.de,
        0xE=>&mut cpu.hl,
        0xF=>&mut cpu.af,
        _=>panic!("no register")
    };

    memory.write(cpu.stack_pointer-1, *reg.high());
    memory.write(cpu.stack_pointer-2, *reg.low());
    cpu.stack_pointer-=2;
}

//load into hl sp + rr
pub fn ld_hl_spdd(cpu:&mut GbcCpu, opcode:u16){
    let dd = (opcode & 0xFF) as i8;
    let temp:i32 = cpu.stack_pointer as i32;
    let value = temp.wrapping_add(dd as i32);

    cpu.hl.value = value as u16;

    //check for carry
    cpu.set_by_value(Flag::Carry, value<0);

    //check for half carry
    //todo check for bugs
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_third_nible(cpu.stack_pointer,dd as u16));

    cpu.unset_flag(Flag::Zero);
    cpu.unset_flag(Flag::Subtraction);
}

//load sp into memory
pub fn ld_nn_sp(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u32){
    let address = (opcode & 0xFFFF) as u16;
    let low = (cpu.stack_pointer & 0xFF) as u8;
    let high = ((cpu.stack_pointer & 0xFF)>>8) as u8;
    memory.write(address, low);
    memory.write(address+1, high);
}