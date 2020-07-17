use crate::cpu::gbc_cpu::{GbcCpu, Flag};
use crate::mmu::memory::Memory;
use crate::opcodes::opcodes_utils::{
    check_for_half_carry_third_nible_add,
    get_arithmetic_16reg,
    opcode_to_u16_value,
    u16_to_high_and_low
};
use crate::opcodes::opcodes_utils;

//load into 16bit register RR the value NN
pub fn load_rr_nn(cpu:&mut GbcCpu, opcode:u32){
    let reg = (((opcode>>16) & 0xF0)>>4) as u8;
    let nn = opcode_to_u16_value((opcode & 0xFFFF) as u16);
    let reg = get_arithmetic_16reg(cpu, reg);

    *reg = nn;
}

//loads register HL into the SP
pub fn load_sp_hl(cpu:&mut GbcCpu){
    cpu.stack_pointer = *cpu.hl.value();
}

//pop from the stack pointer to one register
pub fn pop(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u8){
    let poped_value = opcodes_utils::pop(cpu, memory);
    let reg = (opcode&0xF0)>>4;
    let reg = match reg{
        0xC=>&mut cpu.bc,
        0xD=>&mut cpu.de,
        0xE=>&mut cpu.hl,
        0xF=>&mut cpu.af,
        _=>panic!("no register")
    };

    *reg.value() = poped_value;
}

//push to stack the register 
pub fn push(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u8){
    let reg = (opcode&0xF0)>>4;
    let value = match reg{
        0xC=>*cpu.bc.value(),
        0xD=>*cpu.de.value(),
        0xE=>*cpu.hl.value(),
        0xF=>*cpu.af.value(),
        _=>panic!("no register")
    };

    opcodes_utils::push(cpu, memory, value);
}

//load into hl sp + rr
pub fn ld_hl_spdd(cpu:&mut GbcCpu, opcode:u16){
    let dd = (opcode & 0xFF) as i8;
    let temp:i32 = cpu.stack_pointer as i32;
    let value = temp.wrapping_add(dd as i32);

    *cpu.hl.value() = value as u16;

    //check for carry
    cpu.set_by_value(Flag::Carry, value<=0);

    //check for half carry
    //todo check for bugs
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_third_nible_add(cpu.stack_pointer,dd as u16));

    cpu.unset_flag(Flag::Zero);
    cpu.unset_flag(Flag::Subtraction);
}

//load sp into memory
pub fn ld_nn_sp(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u32){
    let address = opcode_to_u16_value((opcode & 0xFFFF) as u16);
    let (high, low):(u8, u8) = u16_to_high_and_low(cpu.stack_pointer);
    memory.write(address, low);
    memory.write(address+1, high);
}