use crate::cpu::gbc_cpu::{GbcCpu, Flag};
use crate::mmu::memory::Memory;

fn push_pc(cpu:&mut GbcCpu, memory: &mut dyn Memory){
    let high = (cpu.program_counter & 0xFF00) as u8;
    let low = (cpu.program_counter & 0xFF) as u8;
    memory.write(cpu.stack_pointer-1, high);
    memory.write(cpu.stack_pointer-2, low);
    cpu.stack_pointer-=2;
}

pub fn call(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u32){
    let address_to_jump = (opcode & 0xFFFF) as u16;
    push_pc(cpu, memory);
    cpu.program_counter = address_to_jump;
}

fn call_if_true(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u32, flag:bool){
    if flag{
        call(cpu, memory, opcode);
    }
}

pub fn call_cc(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u32){
    let flag = ((0xFF0000 & opcode) >> 16) & 0b00011000;
    let zero:bool = cpu.get_flag(Flag::Zero);
    let carry:bool = cpu.get_flag(Flag::Carry);
    match flag{
        0b00=>call_if_true(cpu, memory, opcode, !zero),
        0b01=>call_if_true(cpu, memory, opcode, zero),
        0b10=>call_if_true(cpu, memory, opcode, !carry),
        0b11=>call_if_true(cpu, memory, opcode, carry),
        _=>std::panic!("error call opcode {}",opcode)
    }
}

pub fn ret(cpu:&mut GbcCpu, memory:&mut dyn Memory){
    let mut pc:u16 = memory.read(cpu.stack_pointer) as u16;
    pc |= (memory.read(cpu.stack_pointer+1) as u16)<<8;
    cpu.program_counter = pc;
    cpu.stack_pointer+=2;
}

fn ret_if_true(cpu:&mut GbcCpu, memory:&mut dyn Memory, flag:bool){
    if flag{
        ret(cpu, memory);
    }
}

pub fn ret_cc(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u8){
    let flag:u8 = opcode & 0b00011000;
    let zero:bool = cpu.get_flag(Flag::Zero);
    let carry:bool = cpu.get_flag(Flag::Carry);
    match flag{
        0b00=>ret_if_true(cpu, memory, !zero),
        0b01=>ret_if_true(cpu, memory, zero),
        0b10=>ret_if_true(cpu, memory, !carry),
        0b11=>ret_if_true(cpu, memory, carry),
        _=>std::panic!("error call opcode {}",opcode)
    }
}

pub fn rst(cpu:&mut GbcCpu, memory:&mut dyn Memory, opcode:u8){
    let t:u8 = (opcode & 0b0011100)>>2;
    let mut value:u8 = 0;
    if t & 0b001 > 0{
        value+=0x8;   
    }
    if t & 0b010 > 0{
        value+=0x10;
    }
    if t & 0b100 > 0{
        value+=0x20;
    }

    push_pc(cpu, memory);
    cpu.program_counter = value as u16;
    
}

pub fn reti(cpu:&mut GbcCpu, memory:&mut dyn Memory){
    ret(cpu, memory);
    cpu.mie = true;
}

fn jump_if_true(cpu:&mut GbcCpu, opcode:u32, flag:bool){
    if flag{
        jump(cpu, opcode);
    }
}

pub fn jump(cpu:&mut GbcCpu, opcode:u32){
    let address = (opcode & 0xFFFF) as u16;
    cpu.program_counter = address;
}

pub fn jump_cc(cpu:&mut GbcCpu, opcode:u32){
    let flag:u8 = ((opcode &0xFF0000) & 0b00011000) as u8;
    let zero:bool = cpu.get_flag(Flag::Zero);
    let carry:bool = cpu.get_flag(Flag::Carry);
    match flag{
        0b00=>jump_if_true(cpu, opcode, !zero),
        0b01=>jump_if_true(cpu, opcode, zero),
        0b10=>jump_if_true(cpu, opcode, !carry),
        0b11=>jump_if_true(cpu, opcode, carry),
        _=>std::panic!("error call opcode {}",opcode)
    }
}

pub fn jump_hl(cpu:&mut GbcCpu){
    cpu.program_counter = cpu.hl.value;
}

fn jump_r_if_true(cpu:&mut GbcCpu, opcode:u16, flag:bool){
    if flag{
        jump_r(cpu, opcode);
    }
}

pub fn jump_r(cpu:&mut GbcCpu, opcode:u16){
    let address = opcode&0xFF;
    let address = address as i8;
    cpu.program_counter = cpu.program_counter.wrapping_add(address as u16);
}

pub fn jump_r_cc(cpu:&mut GbcCpu, opcode:u16){
    let flag:u8 = ((opcode &0xFF00) & 0b00011000) as u8;
    let zero:bool = cpu.get_flag(Flag::Zero);
    let carry:bool = cpu.get_flag(Flag::Carry);
    match flag{
        0b00=>jump_r_if_true(cpu, opcode, !zero),
        0b01=>jump_r_if_true(cpu, opcode, zero),
        0b10=>jump_r_if_true(cpu, opcode, !carry),
        0b11=>jump_r_if_true(cpu, opcode, carry),
        _=>std::panic!("error call opcode {}",opcode)
    }
}