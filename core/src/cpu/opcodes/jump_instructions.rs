use crate::{cpu::{gb_cpu::GbCpu, flag::Flag}, mmu::Memory};
use super::opcodes_utils::{pop,push};

fn push_pc(cpu:&mut GbCpu, memory: &mut impl Memory){
    push(cpu, memory, cpu.program_counter);
}

pub fn call(cpu:&mut GbCpu, memory:&mut impl Memory, opcode:u32)->u8{
    let address_to_jump = (((opcode & 0xFF) as u16)<<8) | (((opcode & 0xFF00)as u16)>>8);
    push_pc(cpu, memory);
    cpu.program_counter = address_to_jump;
    
    // 6 cycles - 3 reading opcode, 2 writing pc to sp address, 1 internal operation
    return 1;
}

fn call_if_true(cpu:&mut GbCpu, memory:&mut impl Memory, opcode:u32, flag:bool)->u8{
    if flag{
        // 6 cycles - 6 as call opcode
        return call(cpu, memory, opcode);
    }
    
    // 3 cycles - 3 reading opcode (no call executed)
    return 0;
}

pub fn call_cc(cpu:&mut GbCpu, memory:&mut impl Memory, opcode:u32)->u8{
    let flag = (((0xFF0000 & opcode) >> 16) & 0b00011000)>>3;
    let zero:bool = cpu.get_flag(Flag::Zero);
    let carry:bool = cpu.get_flag(Flag::Carry);
    match flag{
        0b00=>call_if_true(cpu, memory, opcode, !zero),
        0b01=>call_if_true(cpu, memory, opcode, zero),
        0b10=>call_if_true(cpu, memory, opcode, !carry),
        0b11=>call_if_true(cpu, memory, opcode, carry),
        _=>core::panic!("error call opcode {}",opcode)
    }
}

pub fn ret(cpu:&mut GbCpu, memory:&mut impl Memory)->u8{
    cpu.program_counter = pop(cpu, memory);
    
    // 4 cycles - 1 reading opcode, 2 writing pc to sp address, 1 internal operation
    return 1;
}

fn ret_if_true(cpu:&mut GbCpu, memory:&mut impl Memory, flag:bool)->u8{
    if flag{
        let cycles = ret(cpu, memory);
        
        // 5 cycles - 4 as ret opcode, 1 internal operation
        return cycles+1;
    }
    
    // 2 cycles - 1 reading opcode, 1 internal operation
    return 1;
}

pub fn ret_cc(cpu:&mut GbCpu, memory:&mut impl Memory, opcode:u8)->u8{
    let flag:u8 = (opcode & 0b00011000)>>3;
    let zero:bool = cpu.get_flag(Flag::Zero);
    let carry:bool = cpu.get_flag(Flag::Carry);
    match flag{
        0b00=>ret_if_true(cpu, memory, !zero),
        0b01=>ret_if_true(cpu, memory, zero),
        0b10=>ret_if_true(cpu, memory, !carry),
        0b11=>ret_if_true(cpu, memory, carry),
        _=>core::panic!("error call opcode {}",opcode)
    }
}

pub fn rst(cpu:&mut GbCpu, memory:&mut impl Memory, opcode:u8)->u8{
    let t:u8 = (opcode & 0b00111000)>>3;
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
    
    // 4 cycles - 1 reading opcode, 2 writing pc to sp address, 1 internal operation
    return 1;
}

pub fn reti(cpu:&mut GbCpu, memory:&mut impl Memory)->u8{
    let cycles = ret(cpu, memory);
    cpu.mie = true;

    // 4 cycles - 4 as ret opcode
    return cycles;
}

fn jump_if_true(cpu:&mut GbCpu, opcode:u32, flag:bool)->u8{
    if flag{
        // 4 cycles - 4 as jump opcode
        return jump(cpu, opcode);
    }
    
    // 3 cycles - 3 reading opcode
    return 0;
}

pub fn jump(cpu:&mut GbCpu, opcode:u32)->u8{
    let address = (((opcode & 0xFF) as u16)<<8) | (((opcode & 0xFF00)as u16)>>8);
    cpu.program_counter = address;
    
    // 4 cycles - 3 reading opcode, 1 internal operation
    return 1;
}

pub fn jump_cc(cpu:&mut GbCpu, opcode:u32)->u8{
    let flag:u8 = ((((opcode & 0xFF0000)>>16) & 0b00011000)>>3) as u8;
    let zero:bool = cpu.get_flag(Flag::Zero);
    let carry:bool = cpu.get_flag(Flag::Carry);
    match flag{
        0b00=>jump_if_true(cpu, opcode, !zero),
        0b01=>jump_if_true(cpu, opcode, zero),
        0b10=>jump_if_true(cpu, opcode, !carry),
        0b11=>jump_if_true(cpu, opcode, carry),
        _=>core::panic!("error call opcode {}",opcode)
    }
}

pub fn jump_hl(cpu:&mut GbCpu)->u8{
    cpu.program_counter = cpu.hl.value();
    
    // 1 cycles - 1 reading opcode
    return 0;
}

fn jump_r_if_true(cpu:&mut GbCpu, opcode:u16, flag:bool)->u8{
    if flag{
        // 3 cycles - 3 as jump_r opcode
        return jump_r(cpu, opcode);
    }

    // 2 cycles - 2 reading opcode (no jump)
    return 0;
}

pub fn jump_r(cpu:&mut GbCpu, opcode:u16)->u8{
    let address = opcode&0xFF;
    let address = address as i8;
    cpu.program_counter = cpu.program_counter.wrapping_add(address as u16);
    
    // 3 cycles - 2 reading opcode, 1 internal operation
    return 1;
}

pub fn jump_r_cc(cpu:&mut GbCpu, opcode:u16)->u8{
    let flag:u8 = (((opcode &0xFF00)>>8 & 0b00011000) as u8)>>3;
    let zero:bool = cpu.get_flag(Flag::Zero);
    let carry:bool = cpu.get_flag(Flag::Carry);

    match flag{
        0b00=>jump_r_if_true(cpu, opcode, !zero),
        0b01=>jump_r_if_true(cpu, opcode, zero),
        0b10=>jump_r_if_true(cpu, opcode, !carry),
        0b11=>jump_r_if_true(cpu, opcode, carry),
        _=>core::panic!("error call opcode {}",opcode)
    }
}