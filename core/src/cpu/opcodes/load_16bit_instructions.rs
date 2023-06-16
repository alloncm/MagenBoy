use crate::cpu::gb_cpu::GbCpu;
use crate::cpu::flag::Flag;
use crate::mmu::memory::Memory;
use super::opcodes_utils::{
    get_arithmetic_16reg,
    opcode_to_u16_value,
    u16_to_high_and_low,
    signed_check_for_half_carry_first_nible_add,
    signed_check_for_carry_first_nible_add
};
use super::opcodes_utils;

//load into 16bit register RR the value NN
pub fn load_rr_nn(cpu:&mut GbCpu, opcode:u32)->u8{
    let reg = (((opcode>>16) & 0xF0)>>4) as u8;
    let nn = opcode_to_u16_value((opcode & 0xFFFF) as u16);
    let reg = get_arithmetic_16reg(cpu, reg);

    *reg = nn;

    // 3 cycles - 3 reading opcode
    return 0;
}

//loads register HL into the SP
pub fn load_sp_hl(cpu:&mut GbCpu)->u8{
    cpu.stack_pointer = *cpu.hl.value();
    
    // 2 cycles - 1 reading opcode, 1 internal operation
    return 1;
}

//pop from the stack pointer to one register
pub fn pop(cpu:&mut GbCpu, memory:&mut impl Memory, opcode:u8)->u8{
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
    
    // 3 cycles - 1 reading opcode, 2 reading sp address and sp+1 address
    return 0;
}

//push to stack the register 
pub fn push(cpu:&mut GbCpu, memory:&mut impl Memory, opcode:u8)->u8{
    let reg = (opcode&0xF0)>>4;
    let value = match reg{
        0xC=>*cpu.bc.value(),
        0xD=>*cpu.de.value(),
        0xE=>*cpu.hl.value(),
        0xF=>*cpu.af.value(),
        _=>panic!("no register")
    };

    opcodes_utils::push(cpu, memory, value);

    // 4 cycles - 1 reading opcode, 2 writing to sp address and sp+1 address, 1 internal operation
    return 1;
}

//load into hl sp + rr
pub fn ld_hl_spdd(cpu:&mut GbCpu, opcode:u16)->u8{
    let dd = (opcode & 0xFF) as i8;
    let temp:i32 = cpu.stack_pointer as i32;
    let value = temp.wrapping_add(dd as i32);

    *cpu.hl.value() = value as u16;

    //check for carry
    cpu.set_by_value(Flag::Carry, signed_check_for_carry_first_nible_add(temp as i16, dd));

    //check for half carry
    //todo check for bugs
    cpu.set_by_value(Flag::HalfCarry, ((temp as i16) & 0xF) + ((dd as i16) & 0xF) > 0xF);
    cpu.set_by_value(Flag::HalfCarry,  signed_check_for_half_carry_first_nible_add(temp as i16, dd));

    cpu.unset_flag(Flag::Zero);
    cpu.unset_flag(Flag::Subtraction);
    
    // 3 cycles - 2 reading opcode, 1 internal operation
    return 1;
}

//load sp into memory
pub fn ld_nn_sp(cpu:&mut GbCpu, memory:&mut impl Memory, opcode:u32)->u8{
    let address = opcode_to_u16_value((opcode & 0xFFFF) as u16);
    let (high, low):(u8, u8) = u16_to_high_and_low(cpu.stack_pointer);
    memory.write(address, low, 1);
    memory.write(address+1, high, 1);

    // 5 cycles - 3 reading opcode, 2 writing sp to nn address
    return 0;
}