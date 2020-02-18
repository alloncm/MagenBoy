use crate::cpu::gbc_cpu::{GbcCpu, Flag};
use crate::machine::memory::Memory;
use crate::opcodes::opcodes_utils::{get_src_register,check_for_half_carry_first_nible};

fn add(cpu:&mut GbcCpu, dest:u8, src:u8 )->u8{
    let (value, overflow) = dest.overflowing_add(src);

    cpu.set_by_value(Flag::Carry, overflow);
    cpu.set_by_value(Flag::Zero, value == 0);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_first_nible(src,dest));
    cpu.unset_flag(Flag::Subtraction);

    return value;
}

fn adc(cpu:&mut GbcCpu, dest:u8, src:u8 ) -> u8{
    let flag = (*cpu.af.high()) & (Flag::Carry as u8) >> 5;
    let addition = add(cpu,dest,src);
    let (value, overflow) = addition.overflowing_add(flag);

    cpu.set_by_value(Flag::Carry, overflow);
    cpu.set_by_value(Flag::Zero, value == 0);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_first_nible(src,dest));

    return value;
}

fn sub(cpu:&mut GbcCpu, dest:u8, src:u8 )->u8{
    let (value, overflow) = dest.overflowing_sub(src);

    cpu.set_by_value(Flag::Carry, overflow);
    cpu.set_by_value(Flag::Zero, value == 0);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_first_nible(src,dest));
    cpu.set_flag(Flag::Subtraction);

    return value;
}

fn subc(cpu:&mut GbcCpu, dest:u8, src:u8 ) -> u8{
    let flag = (*cpu.af.high()) & (Flag::Carry as u8) >> 5;
    let subtraction = sub(cpu,dest,src);
    let (value, overflow) = subtraction.overflowing_sub(flag);

    cpu.set_by_value(Flag::Carry, overflow);
    cpu.set_by_value(Flag::Zero, value == 0);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_first_nible(src,dest));

    return value;
}

fn and(cpu:&mut GbcCpu, dest:u8, src:u8 )->u8{
    let value = dest & src;

    cpu.unset_flag(Flag::Carry);
    cpu.set_by_value(Flag::Zero, value == 0);
    cpu.set_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);

    return value;
}

fn xor(cpu:&mut GbcCpu, dest:u8, src:u8 )->u8{
    let value = dest ^ src;

    cpu.unset_flag(Flag::Carry);
    cpu.set_by_value(Flag::Zero, value == 0);
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);

    return value;
}

fn or(cpu:&mut GbcCpu, dest:u8, src:u8 )->u8{
    let value = dest | src;

    cpu.unset_flag(Flag::Carry);
    cpu.set_by_value(Flag::Zero, value == 0);
    cpu.unset_flag(Flag::HalfCarry);
    cpu.unset_flag(Flag::Subtraction);

    return value;
}


//add A and r
pub fn add_a_r(cpu:&mut GbcCpu, opcode:u8){
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.low();
    *cpu.af.low() = add(cpu, dest, src_reg);
}

//add A and nn
pub fn add_a_nn(cpu:&mut GbcCpu, opcode:u16){
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.low();
    *cpu.af.low() = add(cpu, dest, src);
}

//add A and (hl)
pub fn add_a_hl(cpu:&mut GbcCpu,memory:&dyn Memory){
    let src = memory.read(cpu.hl.value);
    let dest = *cpu.af.low();
    *cpu.af.low() = add(cpu, dest, src);
}

//add A and r + carry flag
pub fn adc_a_r(cpu:&mut GbcCpu, opcode:u8){
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.low();
    *cpu.af.low() = adc(cpu, dest, src_reg);
}

//add A and nn +  carry
pub fn adc_a_nn(cpu:&mut GbcCpu, opcode:u16){
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.low();
    *cpu.af.low() = adc(cpu, dest, src);
}

//add A and (hl) + Scarry
pub fn adc_a_hl(cpu:&mut GbcCpu,memory:&dyn Memory){
    let src = memory.read(cpu.hl.value);
    let dest = *cpu.af.low();
    *cpu.af.low() = adc(cpu, dest, src);
}

//sub r from A
pub fn sub_a_r(cpu:&mut GbcCpu, opcode:u8){
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.low();
    *cpu.af.low() = sub(cpu, dest, src_reg);
}

//sub A and nn
pub fn sub_a_nn(cpu:&mut GbcCpu, opcode:u16){
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.low();
    *cpu.af.low() = sub(cpu, dest, src);
}

//add A and (hl)
pub fn sub_a_hl(cpu:&mut GbcCpu,memory:&dyn Memory){
    let src = memory.read(cpu.hl.value);
    let dest = *cpu.af.low();
    *cpu.af.low() = sub(cpu, dest, src);
}


//sub r from A
pub fn sbc_a_r(cpu:&mut GbcCpu, opcode:u8){
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.low();
    *cpu.af.low() = subc(cpu, dest, src_reg);
}

//sub A and nn
pub fn sbc_a_nn(cpu:&mut GbcCpu, opcode:u16){
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.low();
    *cpu.af.low() = subc(cpu, dest, src);
}

//sub A and (hl)
pub fn sbc_a_hl(cpu:&mut GbcCpu,memory:&dyn Memory){
    let src = memory.read(cpu.hl.value);
    let dest = *cpu.af.low();
    *cpu.af.low() = subc(cpu, dest, src);
}

//and A and r
pub fn and_a_r(cpu:&mut GbcCpu, opcode:u8){
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.low();
    *cpu.af.low() = and(cpu, dest, src_reg);
}

//and A and nn
pub fn and_a_nn(cpu:&mut GbcCpu, opcode:u16){
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.low();
    *cpu.af.low() = and(cpu, dest, src);
}

//and A and (hl)
pub fn and_a_hl(cpu:&mut GbcCpu,memory:&dyn Memory){
    let src = memory.read(cpu.hl.value);
    let dest = *cpu.af.low();
    *cpu.af.low() = and(cpu, dest, src);
}

//xor A and r
pub fn xor_a_r(cpu:&mut GbcCpu, opcode:u8){
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.low();
    *cpu.af.low() = xor(cpu, dest, src_reg);
}

//xor A and nn
pub fn xor_a_nn(cpu:&mut GbcCpu, opcode:u16){
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.low();
    *cpu.af.low() = xor(cpu, dest, src);
}

//xor A and (hl)
pub fn xor_a_hl(cpu:&mut GbcCpu,memory:&dyn Memory){
    let src = memory.read(cpu.hl.value);
    let dest = *cpu.af.low();
    *cpu.af.low() = xor(cpu, dest, src);
}


//or A and r
pub fn or_a_r(cpu:&mut GbcCpu, opcode:u8){
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.low();
    *cpu.af.low() = or(cpu, dest, src_reg);
}

//or A and nn
pub fn or_a_nn(cpu:&mut GbcCpu, opcode:u16){
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.low();
    *cpu.af.low() = or(cpu, dest, src);
}

//or A and (hl)
pub fn or_a_hl(cpu:&mut GbcCpu,memory:&dyn Memory){
    let src = memory.read(cpu.hl.value);
    let dest = *cpu.af.low();
    *cpu.af.low() = or(cpu, dest, src);
}

//cp A and r
pub fn cp_a_r(cpu:&mut GbcCpu, opcode:u8){
    let src_reg = *get_src_register(cpu, opcode);
    let dest = *cpu.af.low();
    sub(cpu, dest, src_reg);
}

//cp A and nn
pub fn cp_a_nn(cpu:&mut GbcCpu, opcode:u16){
    let src = (opcode & 0xFF) as u8;
    let dest = *cpu.af.low();
    sub(cpu, dest, src);
}

//or A and (hl)
pub fn cp_a_hl(cpu:&mut GbcCpu,memory:&dyn Memory){
    let src = memory.read(cpu.hl.value);
    let dest = *cpu.af.low();
    sub(cpu, dest, src);
}

fn inc_set_flags(cpu:&mut GbcCpu, reg:u8){
    cpu.unset_flag(Flag::Subtraction);
    cpu.set_by_value(Flag::Zero, reg.wrapping_add(1) == 0);
    cpu.set_by_value(Flag::HalfCarry, check_for_half_carry_first_nible(reg, 1));
}

pub fn inc_r(cpu:&mut GbcCpu, opcode:u8){
    let reg_value:u8;
    {
        let reg = opcode&(0b11111000);
        let reg = match reg{
            0x00=>cpu.bc.low(),
            0x01=>cpu.bc.high(),
            0x10=>cpu.de.low(),
            0x11=>cpu.de.high(),
            0x20=>cpu.hl.low(),
            0x21=>cpu.hl.high(),
            0x31=>cpu.af.low(),
            _=>panic!("no register")
        };
        
        reg_value = *reg;
        *reg = (*reg).wrapping_add(1);
    }

    inc_set_flags(cpu, reg_value);
}

pub fn inc_hl(cpu:&mut GbcCpu,memory:&mut dyn Memory){
    let value = memory.read(cpu.hl.value);
    memory.write(cpu.hl.value, value.wrapping_add(1));
    inc_set_flags(cpu, value);
}

pub fn dec_hl(cpu:&mut GbcCpu,memory:&mut dyn Memory){
    memory.write(cpu.hl.value, memory.read(cpu.hl.value)-1);
}

pub fn dec_r(cpu:&mut GbcCpu, opcode:u8){
    let reg = opcode&(0b11111000);
    let reg = match reg{
        0x00=>cpu.bc.low(),
        0x01=>cpu.bc.high(),
        0x10=>cpu.de.low(),
        0x11=>cpu.de.high(),
        0x20=>cpu.hl.low(),
        0x21=>cpu.hl.high(),
        0x31=>cpu.af.low(),
        _=>panic!("no register")
    };
    *reg-=1;
}