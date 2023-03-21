mod memory_stub;

use memory_stub::MemoryStub;
use lib_gb::cpu::opcodes::rotate_shift_instructions::*;
use lib_gb::cpu::gb_cpu::*;
use lib_gb::cpu::flag::Flag;

#[test]
fn test_rlc_r(){
    let opcode:u16 = 0xCB00;
    let mut cpu = GbCpu::default();
    *cpu.bc.high() = 0x85;
    rlc_r(&mut cpu, opcode);
    assert_eq!(*cpu.bc.high(), 0xB);
}


#[test]
fn test_rl_carry_not_set_r(){
    let opcode:u16 = 0xCB10;
    let mut cpu = GbCpu::default();
    *cpu.bc.high() = 0x85;
    rl_r(&mut cpu, opcode);
    assert_eq!(*cpu.bc.high(), 0xA);
}

#[test]
fn test_rl_carry_set_r(){
    let opcode:u16 = 0xCB10;
    let mut cpu = GbCpu::default();
    *cpu.bc.high() = 0x85;
    cpu.set_flag(Flag::Carry);
    rl_r(&mut cpu, opcode);
    assert_eq!(*cpu.bc.high(), 0xB);
}

#[test]
fn test_rla(){
    let mut cpu = GbCpu::default();
    *cpu.af.high() = 0x85;
    rla(&mut cpu);
    assert_eq!(*cpu.af.high(), 0xA);
}

#[test]
fn test_sla_hl_carry(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 0x0;
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0x0] = 0x80;
    sla_hl(&mut cpu, &mut memory);
    assert_eq!(memory.data[0], 0);
    assert_eq!(cpu.get_flag(Flag::Carry), true);
    assert_eq!(cpu.get_flag(Flag::Zero), true);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}

#[test]
fn test_sla_hl(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 0x0;
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0x0] = 0xFF;
    sla_hl(&mut cpu, &mut memory);
    assert_eq!(memory.data[0], 0xFE);
    assert_eq!(cpu.get_flag(Flag::Carry), true);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}

#[test]
fn test_sla_hl_expects_no_carry(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 0x0;
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0x0] = 0x0F;
    sla_hl(&mut cpu, &mut memory);
    assert_eq!(memory.data[0], 0x1E);
    assert_eq!(cpu.get_flag(Flag::Carry), false);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}

#[test]
fn test_sla_hl_0(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 0xDEF8;
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0xDEF8] = 0x0;
    sla_hl(&mut cpu, &mut memory);
    assert_eq!(memory.data[0xDEF8], 0);
    assert_eq!(cpu.get_flag(Flag::Carry), false);
    assert_eq!(cpu.get_flag(Flag::Zero), true);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}

#[test]
fn test_sla_hl_1(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 0xDEF8;
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0xDEF8] = 0x1;
    sla_hl(&mut cpu, &mut memory);
    assert_eq!(memory.data[0xDEF8], 0x2);
    assert_eq!(cpu.get_flag(Flag::Carry), false);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}

#[test]
fn test_sla_hl_f(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 0xDEF8;
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0xDEF8] = 0xF;
    sla_hl(&mut cpu, &mut memory);
    assert_eq!(memory.data[0xDEF8], 0x1E);
    assert_eq!(cpu.get_flag(Flag::Carry), false);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}

#[test]
fn test_sla_hl_10(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 0xDEF8;
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0xDEF8] = 0x10;
    sla_hl(&mut cpu, &mut memory);
    assert_eq!(memory.data[0xDEF8], 0x20);
    assert_eq!(cpu.get_flag(Flag::Carry), false);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}

#[test]
fn test_sla_hl_1f(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 0xDEF8;
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0xDEF8] = 0x1F;
    sla_hl(&mut cpu, &mut memory);
    assert_eq!(memory.data[0xDEF8], 0x3E);
    assert_eq!(cpu.get_flag(Flag::Carry), false);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}

#[test]
fn test_sla_hl_7f(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 0xDEF8;
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0xDEF8] = 0x7F;
    sla_hl(&mut cpu, &mut memory);
    assert_eq!(memory.data[0xDEF8], 0xFE);
    assert_eq!(cpu.get_flag(Flag::Carry), false);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}

#[test]
fn test_sla_hl_80(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 0xDEF8;
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0xDEF8] = 0x80;
    sla_hl(&mut cpu, &mut memory);
    assert_eq!(memory.data[0xDEF8], 0x0);
    assert_eq!(cpu.get_flag(Flag::Carry), true);
    assert_eq!(cpu.get_flag(Flag::Zero), true);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}

#[test]
fn test_sla_hl_f0(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 0xDEF8;
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0xDEF8] = 0xF0;
    sla_hl(&mut cpu, &mut memory);
    assert_eq!(memory.data[0xDEF8], 0xE0);
    assert_eq!(cpu.get_flag(Flag::Carry), true);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}

#[test]
fn test_sla_hl_ff(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 0xDEF8;
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0xDEF8] = 0xFF;
    sla_hl(&mut cpu, &mut memory);
    assert_eq!(memory.data[0xDEF8], 0xFE);
    assert_eq!(cpu.get_flag(Flag::Carry), true);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}

#[test]
fn test_sla_hl_2(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 0xDEF8;
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0xDEF8] = 0x2;
    sla_hl(&mut cpu, &mut memory);
    assert_eq!(memory.data[0xDEF8], 0x4);
    assert_eq!(cpu.get_flag(Flag::Carry), false);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}

#[test]
fn test_sla_hl_4(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 0xDEF8;
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0xDEF8] = 0x4;
    sla_hl(&mut cpu, &mut memory);
    assert_eq!(memory.data[0xDEF8], 0x8);
    assert_eq!(cpu.get_flag(Flag::Carry), false);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}

#[test]
fn test_sla_hl_20(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 0xDEF8;
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0xDEF8] = 0x20;
    sla_hl(&mut cpu, &mut memory);
    assert_eq!(memory.data[0xDEF8], 0x40);
    assert_eq!(cpu.get_flag(Flag::Carry), false);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}

#[test]
fn test_sla_hl_40(){
    let mut cpu = GbCpu::default();
    *cpu.hl.value_mut() = 0xDEF8;
    let mut memory = MemoryStub{data:[0;0xFFFF], double_speed:false};
    memory.data[0xDEF8] = 0x40;
    sla_hl(&mut cpu, &mut memory);
    assert_eq!(memory.data[0xDEF8], 0x80);
    assert_eq!(cpu.get_flag(Flag::Carry), false);
    assert_eq!(cpu.get_flag(Flag::Zero), false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry), false);
    assert_eq!(cpu.get_flag(Flag::Subtraction), false);
}