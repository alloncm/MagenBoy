use crate::cpu::gb_cpu::GbCpu;
use crate::cpu::flag::Flag;
use crate::mmu::memory::Memory;
use super::opcodes_utils::{
    get_src_register, 
    check_for_half_carry_first_nible_add,
    check_for_half_carry_first_nible_sub,
    get_reg_two_rows
};

fn add(cpu:&mut GbCpu, dest:u8, src:u8 )->u8{
    let (value, overflow) = dest.overflowing_add(src);

    cpu.set_by_value(Flag::Carry, overflow);
    cpu.set_by_value(Flag::Zero, value == 0);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_first_nible_add(src,dest));
    cpu.unset_flag(Flag::Subtraction);

    return value;
}

fn adc(cpu:&mut GbCpu, dest:u8, src:u8 ) -> u8{
    let flag = (*cpu.af.low() & (Flag::Carry as u8)) >> 4;
    let (value_to_add, value_of) = src.overflowing_add(flag);
    let half_carry_of = check_for_half_carry_first_nible_add(src, flag);
    let (value, overflow) = dest.overflowing_add(value_to_add);

    cpu.set_by_value(Flag::Carry, overflow || value_of);
    cpu.set_by_value(Flag::Zero, value == 0);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_first_nible_add(dest,value_to_add) || half_carry_of);
    cpu.unset_flag(Flag::Subtraction);

    return value;
}

fn sub(cpu:&mut GbCpu, dest:u8, src:u8 )->u8{
    let (value, overflow) = dest.overflowing_sub(src);

    cpu.set_by_value(Flag::Carry, overflow);
    cpu.set_by_value(Flag::Zero, value == 0);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_first_nible_sub(dest,src));
    cpu.set_flag(Flag::Subtraction);

    return value;
}

fn subc(cpu:&mut GbCpu, dest:u8, src:u8 ) -> u8{
    let flag = (*cpu.af.low() & (Flag::Carry as u8)) >> 4;
    let (value_to_sub, value_of) = src.overflowing_add(flag);
    let half_carry = check_for_half_carry_first_nible_add(src, flag);
    let (value, overflow) = dest.overflowing_sub(value_to_sub);

    cpu.set_by_value(Flag::Carry, overflow || value_of);
    cpu.set_by_value(Flag::Zero, value == 0);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_first_nible_sub(dest, value_to_sub) || half_carry);
    cpu.set_flag(Flag::Subtraction);

    return value;
}

fn and(cpu:&mut GbCpu, dest:u8, src:u8 )->u8{
    let value = dest & src;

    cpu.unset_flag(Flag::Carry);
    cpu.set_by_value(Flag::Zero, value == 0);
    cpu.set_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);

    return value;
}

fn xor(cpu:&mut GbCpu, dest:u8, src:u8 )->u8{
    let value = dest ^ src;

    cpu.unset_flag(Flag::Carry);
    cpu.set_by_value(Flag::Zero, value == 0);
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);

    return value;
}

fn or(cpu:&mut GbCpu, dest:u8, src:u8 )->u8{
    let value = dest | src;

    cpu.unset_flag(Flag::Carry);
    cpu.set_by_value(Flag::Zero, value == 0);
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);

    return value;
}


//add A and r
pub fn add_a_r(cpu:&mut GbCpu, opcode:u8)->u8{
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.high();
    *cpu.af.high() = add(cpu, dest, src_reg);
    
    //cycles
    return 1;
}

//add A and nn
pub fn add_a_nn(cpu:&mut GbCpu, opcode:u16)->u8{
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.high();
    *cpu.af.high() = add(cpu, dest, src);
    
    //cycles
    return 2;
}

//add A and (hl)
pub fn add_a_hl(cpu:&mut GbCpu, memory:&mut impl Memory)->u8{
    let src = memory.read(*cpu.hl.value());
    let dest = *cpu.af.high();
    *cpu.af.high() = add(cpu, dest, src);
    
    //cycles
    return 2;
}

//add A and r + carry flag
pub fn adc_a_r(cpu:&mut GbCpu, opcode:u8)->u8{
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.high();
    *cpu.af.high() = adc(cpu, dest, src_reg);
    
    //cycles
    return 1;
}

//add A and nn +  carry
pub fn adc_a_nn(cpu:&mut GbCpu, opcode:u16)->u8{
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.high();
    *cpu.af.high() = adc(cpu, dest, src);
    
    //cycles
    return 2;
}

//add A and (hl) + Scarry
pub fn adc_a_hl(cpu:&mut GbCpu, memory:&mut impl Memory)->u8{
    let src = memory.read(*cpu.hl.value());
    let dest = *cpu.af.high();
    *cpu.af.high() = adc(cpu, dest, src);
    
    //cycles
    return 2;
}

//sub r from A
pub fn sub_a_r(cpu:&mut GbCpu, opcode:u8)->u8{
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.high();
    *cpu.af.high() = sub(cpu, dest, src_reg);
    
    //cycles
    return 1;
}

//sub A and nn
pub fn sub_a_nn(cpu:&mut GbCpu, opcode:u16)->u8{
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.high();
    *cpu.af.high() = sub(cpu, dest, src);
    
    //cycles
    return 2;
}

//sub A and (hl)
pub fn sub_a_hl(cpu:&mut GbCpu, memory:&mut impl Memory)->u8{
    let src = memory.read(*cpu.hl.value());
    let dest = *cpu.af.high();
    *cpu.af.high() = sub(cpu, dest, src);
    
    //cycles
    return 2;
}


//sub r from A
pub fn sbc_a_r(cpu:&mut GbCpu, opcode:u8)->u8{
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.high();
    *cpu.af.high() = subc(cpu, dest, src_reg);
    
    //cycles
    return 1;
}

//sub A and nn
pub fn sbc_a_nn(cpu:&mut GbCpu, opcode:u16)->u8{
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.high();
    *cpu.af.high() = subc(cpu, dest, src);
    
    //cycles
    return 2;
}

//sub A and (hl)
pub fn sbc_a_hl(cpu:&mut GbCpu, memory:&mut impl Memory)->u8{
    let src = memory.read(*cpu.hl.value());
    let dest = *cpu.af.high();
    *cpu.af.high() = subc(cpu, dest, src);
    
    //cycles
    return 2;
}

//and A and r
pub fn and_a_r(cpu:&mut GbCpu, opcode:u8)->u8{
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.high();
    *cpu.af.high() = and(cpu, dest, src_reg);
    
    //cycles
    return 1;
}

//and A and nn
pub fn and_a_nn(cpu:&mut GbCpu, opcode:u16)->u8{
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.high();
    *cpu.af.high() = and(cpu, dest, src);
    
    //cycles
    return 2;
}

//and A and (hl)
pub fn and_a_hl(cpu:&mut GbCpu, memory:&mut impl Memory)->u8{
    let src = memory.read(*cpu.hl.value());
    let dest = *cpu.af.high();
    *cpu.af.high() = and(cpu, dest, src);
    
    //cycles
    return 2;
}

//xor A and r
pub fn xor_a_r(cpu:&mut GbCpu, opcode:u8)->u8{
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.high();
    *cpu.af.high() = xor(cpu, dest, src_reg);
    
    //cycles
    return 1;
}

//xor A and nn
pub fn xor_a_nn(cpu:&mut GbCpu, opcode:u16)->u8{
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.high();
    *cpu.af.high() = xor(cpu, dest, src);
    
    //cycles
    return 2;
}

//xor A and (hl)
pub fn xor_a_hl(cpu:&mut GbCpu, memory:&mut impl Memory)->u8{
    let src = memory.read(*cpu.hl.value());
    let dest = *cpu.af.high();
    *cpu.af.high() = xor(cpu, dest, src);
    
    //cycles
    return 2;
}


//or A and r
pub fn or_a_r(cpu:&mut GbCpu, opcode:u8)->u8{
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.high();
    *cpu.af.high() = or(cpu, dest, src_reg);
    
    //cycles
    return 1;
}

//or A and nn
pub fn or_a_nn(cpu:&mut GbCpu, opcode:u16)->u8{
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.high();
    *cpu.af.high() = or(cpu, dest, src);
    
    //cycles
    return 2;
}

//or A and (hl)
pub fn or_a_hl(cpu:&mut GbCpu, memory:&mut impl Memory)->u8{
    let src = memory.read(*cpu.hl.value());
    let dest = *cpu.af.high();
    *cpu.af.high() = or(cpu, dest, src);
    
    //cycles
    return 2;
}

//cp A and r
pub fn cp_a_r(cpu:&mut GbCpu, opcode:u8)->u8{
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.high();
    sub(cpu, dest, src_reg);
    
    //cycles
    return 1;
}

//cp A and nn
pub fn cp_a_nn(cpu:&mut GbCpu, opcode:u16)->u8{
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.high();
    sub(cpu, dest, src);
    
    //cycles
    return 2;
}

//or A and (hl)
pub fn cp_a_hl(cpu:&mut GbCpu, memory:&mut impl Memory)->u8{
    let src = memory.read(*cpu.hl.value());
    let dest = *cpu.af.high();
    sub(cpu, dest, src);
    
    //cycles
    return 2;
}

pub fn inc_r(cpu:&mut GbCpu, opcode:u8)->u8{
    let original_reg:u8;
    let finished_reg:u8;
    {
        let reg = get_reg_two_rows(cpu, opcode);
        original_reg = *reg;
        *reg = (*reg).wrapping_add(1);
        finished_reg = *reg;
    }
    cpu.set_by_value(Flag::Zero, finished_reg == 0);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_first_nible_add(original_reg, 1));
    cpu.unset_flag(Flag::Subtraction);

    //cycles
    return 1;
}

pub fn inc_hl(cpu:&mut GbCpu, memory:&mut impl Memory)->u8{
    let value = memory.read(*cpu.hl.value());
    let altered_value = value.wrapping_add(1);
    memory.write(*cpu.hl.value(), altered_value);
    
    cpu.set_by_value(Flag::Zero, altered_value == 0);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_first_nible_add(value, 1));
    cpu.unset_flag(Flag::Subtraction);

    //cycles
    return 3;
}

pub fn dec_r(cpu:&mut GbCpu, opcode:u8)->u8{
    let original_reg:u8;
    let finished_reg:u8;
    {
        let reg = get_reg_two_rows(cpu, opcode);
        original_reg = *reg;
        *reg = (*reg).wrapping_sub(1);
        finished_reg = *reg;
    }
    cpu.set_by_value(Flag::Zero, finished_reg == 0);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_first_nible_sub(original_reg, finished_reg));
    cpu.set_flag(Flag::Subtraction);

    //cycles
    return 1;
}

pub fn dec_hl(cpu:&mut GbCpu, memory:&mut impl Memory)->u8{
    let value = memory.read(*cpu.hl.value());
    let altered_value = value.wrapping_sub(1);
    memory.write(*cpu.hl.value(), altered_value);
    
    cpu.set_by_value(Flag::Zero, altered_value == 0);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_first_nible_sub(value, altered_value));
    cpu.set_flag(Flag::Subtraction);
    
    //cycles
    return 3;
}

pub fn cpl(cpu:&mut GbCpu)->u8{
    *cpu.af.high() ^= 0xFF;
    cpu.set_flag(Flag::HalfCarry);
    cpu.set_flag(Flag::Subtraction);
    
    //cycles
    return 1;
}

pub fn daa(cpu:&mut GbCpu)->u8{
    let low_a = *cpu.af.high() & 0xF;
    let mut daa_value:u8 = 0;
    let mut carry:bool = false;

    if cpu.get_flag(Flag::Subtraction){
        if cpu.get_flag(Flag::Carry){
            daa_value |= 0x60;
            carry = true;
        }
        if cpu.get_flag(Flag::HalfCarry){
            daa_value |= 0x6;
        }
        *cpu.af.high() = (*cpu.af.high()).wrapping_sub(daa_value);    
    }
    else{
        if *cpu.af.high() > 0x99 || cpu.get_flag(Flag::Carry){
            daa_value |= 0x60;
            carry = true;
        }
        if low_a > 0x9 || cpu.get_flag(Flag::HalfCarry){
            daa_value |= 0x6;
        }
        *cpu.af.high() = (*cpu.af.high()).wrapping_add(daa_value);
    }

    let zero = *cpu.af.high() == 0;
    cpu.set_by_value(Flag::Carry, carry);
    cpu.set_by_value(Flag::Zero, zero);
    cpu.unset_flag(Flag::HalfCarry);

    //cycles
    return 1;
}