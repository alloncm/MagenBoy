extern crate lib_gbc;
use lib_gbc::cpu::gbc_cpu::{GbcCpu,Flag};
use lib_gbc::opcodes::jump_instructions::rst;
use lib_gbc::mmu::memory::Memory;

mod memory_stub;
use crate::memory_stub::MemoryStub;

#[test]
fn rst_C7_test(){
    let mut cpu = GbcCpu::default();
    cpu.stack_pointer =0xFFFE;
    let mut memory = MemoryStub{data:[0;0xFFFF]};

    rst(&mut cpu,&mut memory,0xC7);

    assert_eq!(cpu.program_counter, 0x00);
}

#[test]
fn rst_CF_test(){
    let mut cpu = GbcCpu::default();
    cpu.stack_pointer =0xFFFE;
    let mut memory = MemoryStub{data:[0;0xFFFF]};

    rst(&mut cpu,&mut memory,0xCF);

    assert_eq!(cpu.program_counter, 0x08);
}

#[test]
fn rst_D7_test(){
    let mut cpu = GbcCpu::default();
    cpu.stack_pointer =0xFFFE;
    let mut memory = MemoryStub{data:[0;0xFFFF]};

    rst(&mut cpu,&mut memory,0xD7);

    assert_eq!(cpu.program_counter, 0x10);
}

#[test]
fn rst_DF_test(){
    let mut cpu = GbcCpu::default();
    cpu.stack_pointer =0xFFFE;
    let mut memory = MemoryStub{data:[0;0xFFFF]};

    rst(&mut cpu,&mut memory,0xDF);

    assert_eq!(cpu.program_counter, 0x18);
}

#[test]
fn rst_E7_test(){
    let mut cpu = GbcCpu::default();
    cpu.stack_pointer =0xFFFE;
    let mut memory = MemoryStub{data:[0;0xFFFF]};

    rst(&mut cpu,&mut memory,0xE7);

    assert_eq!(cpu.program_counter, 0x20);
}

#[test]
fn rst_EF_test(){
    let mut cpu = GbcCpu::default();
    cpu.stack_pointer =0xFFFE;
    let mut memory = MemoryStub{data:[0;0xFFFF]};

    rst(&mut cpu,&mut memory,0xEF);

    assert_eq!(cpu.program_counter, 0x28);
}

#[test]
fn rst_F7_test(){
    let mut cpu = GbcCpu::default();
    cpu.stack_pointer =0xFFFE;
    let mut memory = MemoryStub{data:[0;0xFFFF]};

    rst(&mut cpu,&mut memory,0xF7);

    assert_eq!(cpu.program_counter, 0x30);
}

#[test]
fn rst_FF_test(){
    let mut cpu = GbcCpu::default();
    cpu.stack_pointer =0xFFFE;
    let mut memory = MemoryStub{data:[0;0xFFFF]};

    rst(&mut cpu,&mut memory,0xFF);

    assert_eq!(cpu.program_counter, 0x38);
}