use lib_gb::cpu::gb_cpu::GbCpu;
use lib_gb::cpu::flag::Flag;
use lib_gb::cpu::opcodes::arithmetic_8bit_instructions;
mod memory_stub;
use crate::memory_stub::MemoryStub;

#[test]
fn daa_after_add_op(){
    let mut cpu = GbCpu::default();
    *cpu.af.high() = 0x7D;
    *cpu.af.low() = 0;
    arithmetic_8bit_instructions::daa(&mut cpu);
    assert_eq!(*cpu.af.high(), 0x83);
    assert_eq!(cpu.get_flag(Flag::Carry), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
}

#[test]
fn test_sub_a_nn_for_half_carry_true(){
    let mut cpu = GbCpu::default();
    *cpu.af.high() = 0x3E;
    let opcode = 0x0F;
    arithmetic_8bit_instructions::sub_a_nn(&mut cpu, opcode);

    assert_eq!(*cpu.af.high(), 0x2F);
    assert_eq!(cpu.get_flag(Flag::Zero),false);
    assert_eq!(cpu.get_flag(Flag::Subtraction),true);
    assert_eq!(cpu.get_flag(Flag::HalfCarry),true);
    assert_eq!(cpu.get_flag(Flag::Carry),false);
}

#[test]
fn test_sub_a_nn_for_half_carry_false(){
    let mut cpu = GbCpu::default();
    *cpu.af.high() = 0x3E;
    let opcode = 0x3E;
    arithmetic_8bit_instructions::sub_a_nn(&mut cpu, opcode);

    assert_eq!(*cpu.af.high(), 0);
    assert_eq!(cpu.get_flag(Flag::Zero),true);
    assert_eq!(cpu.get_flag(Flag::Subtraction),true);
    assert_eq!(cpu.get_flag(Flag::HalfCarry),false);
    assert_eq!(cpu.get_flag(Flag::Carry),false);
}

#[test]
fn test_sub_a_nn_for_carry(){
    let mut cpu = GbCpu::default();
    *cpu.af.high() = 0x3E;
    let opcode = 0x40;
    arithmetic_8bit_instructions::sub_a_nn(&mut cpu, opcode);

    assert_eq!(*cpu.af.high(), 0xFE);
    assert_eq!(cpu.get_flag(Flag::Zero),false);
    assert_eq!(cpu.get_flag(Flag::Subtraction),true);
    assert_eq!(cpu.get_flag(Flag::HalfCarry),false);
    assert_eq!(cpu.get_flag(Flag::Carry),true);
}

#[test]
fn test_sbc_nn_on_carry_set_expeced_no_carry(){
    let mut cpu = GbCpu::default();
    *cpu.af.high() = 0x3B;
    cpu.set_flag(Flag::Carry);
    let opcode = 0x2A;
    arithmetic_8bit_instructions::sbc_a_nn(&mut cpu, opcode);

    assert_eq!(*cpu.af.high(), 0x10);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::Carry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), true);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
}


#[test]
fn test_sbc_nn_on_carry_set_expeced_carry_and_half_carry(){
    let mut cpu = GbCpu::default();
    *cpu.af.high() = 0x3B;
    cpu.set_flag(Flag::Carry);
    let opcode = 0x4F;
    arithmetic_8bit_instructions::sbc_a_nn(&mut cpu, opcode);

    assert_eq!(*cpu.af.high(), 0xEB);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::Carry), true);
    assert_eq!(cpu.get_flag(Flag::Subtraction), true);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), true);
}

#[test]
fn test_adc_nn_on_carry_set_expeced_half_carry(){
    let mut cpu = GbCpu::default();
    *cpu.af.high() = 0xE1;
    cpu.set_flag(Flag::Carry);
    let opcode = 0x0F;
    arithmetic_8bit_instructions::adc_a_nn(&mut cpu, opcode);

    assert_eq!(*cpu.af.high(), 0xF1);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::Carry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), true);
}

#[test]
fn test_adc_nn_on_carry_set_expeced_carry(){
    let mut cpu = GbCpu::default();
    *cpu.af.high() = 0xE1;
    cpu.set_flag(Flag::Carry);
    let opcode = 0x3B;
    arithmetic_8bit_instructions::adc_a_nn(&mut cpu, opcode);

    assert_eq!(*cpu.af.high(), 0x1D);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::Carry), true);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
}

#[test]
fn test_adc_nn_on_carry_set_expeced_carry_half_carry_zero(){
    let mut cpu = GbCpu::default();
    *cpu.af.high() = 0xE1;
    cpu.set_flag(Flag::Carry);
    let opcode = 0x1E;
    arithmetic_8bit_instructions::adc_a_nn(&mut cpu, opcode);

    assert_eq!(*cpu.af.high(), 0);
    assert_eq!(cpu.get_flag(Flag::Zero), true);
    assert_eq!(cpu.get_flag(Flag::Carry), true);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), true);
}


#[test]
fn test_inc_hl(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value() = 0x50;
    cpu.set_flag(Flag::Carry);
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    
    arithmetic_8bit_instructions::inc_hl(&mut cpu, &mut memory);

    assert_eq!(*cpu.hl.value(), 0x50);
    assert_eq!(memory.data[0x50], 1);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}


#[test]
fn test_inc_hl_half_carry(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value() = 0x50;
    cpu.set_flag(Flag::Carry);
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0x50] = 0x0F;
    
    arithmetic_8bit_instructions::inc_hl(&mut cpu, &mut memory);

    assert_eq!(*cpu.hl.value(), 0x50);
    assert_eq!(memory.data[0x50], 0x10);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), true);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}
