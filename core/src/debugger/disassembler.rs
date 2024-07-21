use crate::{utils::stack_string::StackString, mmu::Memory, cpu::gb_cpu::GbCpu};

use super::INTERNAL_ARRAY_MAX_SIZE;

macro_rules! define_single_opcode_instr {
    ($name:ident) => {
        fn $name(_:u8, _:&mut impl Memory, _:&mut u16)->OpcodeStr{OpcodeStr::from(stringify!($name))}
    };
}

const MAX_OPCODE_CHARS:usize = 12;

type Opcode<Memory> = fn(u8, &mut Memory, &mut u16)->StackString<MAX_OPCODE_CHARS>;
type OpcodeStr = StackString<MAX_OPCODE_CHARS>;

#[derive(Clone, Copy)]
pub struct OpcodeEntry{
    pub address:u16,
    pub string:OpcodeStr
}

pub fn disassemble<M:Memory>(cpu:&GbCpu, memory:&mut M, opcodes_number:u8)->[OpcodeEntry;INTERNAL_ARRAY_MAX_SIZE]{
    let mut disassembled_opcodes = [OpcodeEntry{ address: 0, string: OpcodeStr::default() };INTERNAL_ARRAY_MAX_SIZE];
    let mut pc = cpu.program_counter;
    for i in 0..opcodes_number{
        let opcode = memory.read(pc, 0);
        disassembled_opcodes[i as usize].address = pc;
        pc += 1;
        let func:Opcode<M> = match opcode{
            0x0=>nop,
            0x1|0x11|0x21|0x31=>ld_rr_nn,
            0x2|0x12=>ld_rr_a,
            0x3|0x13|0x23|0x33=>inc_rr,
            0x4|0xC|0x14|0x1C|0x24|0x2C|0x34|0x3C=>inc_r,
            0x5|0xD|0x15|0x1D|0x25|0x2D|0x35|0x3D=>dec_r,
            0x6|0xE|0x16|0x1E|0x26|0x2E|0x36|0x3E=>ld_r_n,
            0x7=>rlca,
            0x8=>ld_nn_sp,
            0x9|0x19|0x29|0x39=>add_hl_rr,
            0xA|0x1A=>ld_a_rr,
            0xB|0x1B|0x2B|0x3B=>dec_rr,
            0xF=>rrca,
            0x10=>stop,
            0x17=>rla,
            0x18=>jr_d,
            0x1F=>rra,
            0x20|0x28|0x30|0x38=>jr_cc_d,
            0x22=>ldi_hl_a,
            0x27=>daa,
            0x2A=>ldi_a_hl,
            0x2F=>cpl,
            0x32=>ldd_hl_a,
            0x37=>scf,
            0x3A=>ldd_a_hl,
            0x3F=>ccf,
            0x40..=0x75|0x77..=0x7F=>ld_r_r,
            0x76=>halt,
            0x80..=0x87=>add_a_r,
            0x88..=0x8F=>adc_a_r,
            0x90..=0x97=>sub_a_r,
            0x98..=0x9F=>sbc_a_r,
            0xA0..=0xA7=>and_a_r,
            0xA8..=0xAF=>xor_a_r,
            0xB0..=0xB7=>or_a_r,
            0xB8..=0xBF=>cp_a_r,
            0xC0|0xC8|0xD0|0xD8=>ret_cc,
            0xC1|0xD1|0xE1|0xF1=>pop,
            0xC2|0xCA|0xD2|0xDA=>jp_cc_nn,
            0xC3=>jp_nn,
            0xC4|0xCC|0xD4|0xDC=>call_cc_nn,
            0xC5|0xD5|0xE5|0xF5=>push,
            0xC6=>add_a_n,
            0xC7|0xCF|0xD7|0xDF|0xE7|0xEF|0xF7|0xFF=>rst,
            0xC9=>ret,
            0xCB=>cb_prefix,
            0xCD=>call,
            0xCE=>adc_a_n,
            0xD6=>sub_a_n,
            0xD9=>reti,
            0xDE=>sbc_a_n,
            0xE0=>ldio_nn_a,
            0xE2=>ldio_c_a,
            0xE6=>and_a_n,
            0xE8=>add_sp_d,
            0xE9=>jp_hl,
            0xEA=>ld_nn_a,
            0xEE=>xor_a_n,
            0xF0=>ldio_a_nn,
            0xF2=>ldio_a_c,
            0xF3=>di,
            0xF6=>or_a_n,
            0xF8=>ld_hl_sp_d,
            0xF9=>ld_sp_hl,
            0xFA=>ld_a_nn,
            0xFB=>ei,
            0xFE=>cp_a_n,

            _=>unknown
        };
        disassembled_opcodes[i as usize].string = func(opcode, memory, &mut pc);
    }

    return disassembled_opcodes;
}

fn unknown(opcode:u8, _:&mut impl Memory, _:&mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("unknown-{:#X}", opcode))}

define_single_opcode_instr!(nop);
define_single_opcode_instr!(stop);
define_single_opcode_instr!(halt);
define_single_opcode_instr!(rlca);
define_single_opcode_instr!(rrca);
define_single_opcode_instr!(rla);
define_single_opcode_instr!(rra);
define_single_opcode_instr!(daa);
define_single_opcode_instr!(scf);
define_single_opcode_instr!(cpl);
define_single_opcode_instr!(ccf);
define_single_opcode_instr!(di);
define_single_opcode_instr!(ei);
define_single_opcode_instr!(ret);
define_single_opcode_instr!(reti);

fn add_a_r(opcode:u8, _memory:&mut impl Memory, _pc:&mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("add a,{}", get_src_register(opcode)))}

fn adc_a_r(opcode:u8, _memory:&mut impl Memory, _pc:&mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("adc a,{}", get_src_register(opcode)))}

fn sub_a_r(opcode:u8, _memory:&mut impl Memory, _pc:&mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("sub a,{}", get_src_register(opcode)))}

fn sbc_a_r(opcode:u8, _memory:&mut impl Memory, _pc:&mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("sbc a,{}", get_src_register(opcode)))}

fn and_a_r(opcode:u8, _memory:&mut impl Memory, _pc:&mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("and a,{}", get_src_register(opcode)))}

fn xor_a_r(opcode:u8, _memory:&mut impl Memory, _pc:&mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("xor a,{}", get_src_register(opcode)))}

fn or_a_r(opcode:u8, _memory:&mut impl Memory, _pc:&mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("or a,{}", get_src_register(opcode)))}

fn cp_a_r(opcode:u8, _memory:&mut impl Memory, _pc:&mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("cp a,{}", get_src_register(opcode)))}

fn ld_r_r(opcode:u8, _:&mut impl Memory, _: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("ld {},{}", get_dest_register(opcode), get_src_register(opcode)))}

fn add_a_n(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("add a,{:#x}", read_memory(memory, pc)))}
fn sub_a_n(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("sub a,{:#x}", read_memory(memory, pc)))}
fn and_a_n(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("and a,{:#x}", read_memory(memory, pc)))}
fn or_a_n(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("or a,{:#x}", read_memory(memory, pc)))}
fn adc_a_n(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("adc a,{:#x}", read_memory(memory, pc)))}
fn sbc_a_n(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("sbc a,{:#x}", read_memory(memory, pc)))}
fn xor_a_n(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("xor a,{:#x}", read_memory(memory, pc)))}
fn cp_a_n(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("cp a,{:#x}", read_memory(memory, pc)))}
fn ld_nn_a(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("ld {:#X},a", read_memory_u16(memory, pc)))}
fn ld_a_nn(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("ld a,{:#X}", read_memory_u16(memory, pc)))}

fn rst(opcode:u8, _:&mut impl Memory, _: &mut u16)->OpcodeStr{
    let mut address = ((opcode & 0b11_0000) >> 4) * 0x10;
    if (opcode & 0b1000) != 0{
        address += 0x8;
    }
    return OpcodeStr::from_args(format_args!("rst {:#X}", address));
}

fn add_sp_d(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{
    let value = read_memory(memory, pc) as i8;
    return OpcodeStr::from_args(format_args!("add sp,{}", value));
}

fn ld_hl_sp_d(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{
    let value = read_memory(memory, pc) as i8;
    return OpcodeStr::from_args(format_args!("ld hl,sp+{}", value));
}

fn jp_hl(_:u8, _:&mut impl Memory, _:&mut u16)->OpcodeStr{OpcodeStr::from("jp hl")}
fn ld_sp_hl(_:u8, _:&mut impl Memory, _:&mut u16)->OpcodeStr{OpcodeStr::from("ld sp,hl")}

fn jr_d(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{
    let value = read_memory(memory, pc) as i8;
    OpcodeStr::from_args(format_args!("jr {}", value))
}

fn jr_cc_d(opcode:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{
    let cc = get_cc(opcode);
    let value = read_memory(memory, pc) as i8;
    return OpcodeStr::from_args(format_args!("jr {},{}", cc, value));
}

fn ld_rr_nn(opcode:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{
    let reg = get_rr_register(opcode, true);
    let value = read_memory_u16(memory, pc);
    return OpcodeStr::from_args(format_args!("ld {},{:#X}", reg, value));
}

fn ld_rr_a(opcode:u8, _:&mut impl Memory, _: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("ld {},a", get_rr_register(opcode, true)))}

fn ldi_hl_a(_:u8, _:&mut impl Memory, _:&mut u16)->OpcodeStr{OpcodeStr::from("ldi hl,a")}
fn ldi_a_hl(_:u8, _:&mut impl Memory, _:&mut u16)->OpcodeStr{OpcodeStr::from("ldi a,hl")}
fn ldd_hl_a(_:u8, _:&mut impl Memory, _:&mut u16)->OpcodeStr{OpcodeStr::from("ldd hl,a")}
fn ldd_a_hl(_:u8, _:&mut impl Memory, _:&mut u16)->OpcodeStr{OpcodeStr::from("ldd a,hl")}

fn inc_rr(opcode:u8, _:&mut impl Memory, _: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("inc {}", get_rr_register(opcode, true)))}

fn inc_r(opcode:u8, _:&mut impl Memory, _: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("inc {}", get_dest_register(opcode)))}
fn dec_r(opcode:u8, _:&mut impl Memory, _: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("dec {}", get_dest_register(opcode)))}

fn ld_r_n(opcode:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{
    let reg = get_dest_register(opcode);
    let value = read_memory(memory, pc);
    return OpcodeStr::from_args(format_args!("ld {},{:#X}", reg, value));
}

fn add_hl_rr(opcode:u8, _:&mut impl Memory, _: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("add hl,{}", get_rr_register(opcode, true)))}
fn ld_nn_sp(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{
    let value = read_memory_u16(memory, pc);
    return OpcodeStr::from_args(format_args!("ld {},sp", value));
}

fn ld_a_rr(opcode:u8, _:&mut impl Memory, _: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("ld a,{}", get_rr_register(opcode, true)))}
fn dec_rr(opcode:u8, _:&mut impl Memory, _: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("dec {}", get_rr_register(opcode, true)))}

fn ret_cc(opcode:u8, _:&mut impl Memory, _: &mut u16)->OpcodeStr{
    let cc = get_cc(opcode);
    return OpcodeStr::from_args(format_args!("ret {}", cc));
}

fn pop(opcode:u8, _:&mut impl Memory, _: &mut u16)->OpcodeStr{
    let reg = get_rr_register(opcode, false);
    return OpcodeStr::from_args(format_args!("pop {}", reg));
}

fn push(opcode:u8, _:&mut impl Memory, _: &mut u16)->OpcodeStr{
    let reg = get_rr_register(opcode, false);
    return OpcodeStr::from_args(format_args!("push {}", reg));
}

fn jp_cc_nn(opcode:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{
    let cc = get_cc(opcode);
    let value = read_memory_u16(memory, pc);
    return OpcodeStr::from_args(format_args!("jp {},{:#X}", cc, value));
}

fn jp_nn(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{
    let value = read_memory_u16(memory, pc);
    return OpcodeStr::from_args(format_args!("jp {:#X}", value));
}

fn ldio_nn_a(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("ldio {:#X},a", read_memory(memory, pc)))}
fn ldio_a_nn(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("ldio a,{:#X}", read_memory(memory, pc)))}
fn ldio_a_c(_:u8, _:&mut impl Memory, _: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("ldio a,c"))}
fn ldio_c_a(_:u8, _:&mut impl Memory, _: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("ldio c,a"))}

fn call_cc_nn(opcode:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{
    let cc = get_cc(opcode);
    let value = read_memory_u16(memory, pc);
    return OpcodeStr::from_args(format_args!("call {},{:#X}", cc, value));
}

fn call(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{OpcodeStr::from_args(format_args!("call {:#X}",read_memory_u16(memory, pc)))}

fn cb_prefix(_:u8, memory:&mut impl Memory, pc: &mut u16)->OpcodeStr{
    let opcode = read_memory(memory, pc);
    let func = match opcode{
        0x0..=0x7=>rlc_r,
        0x8..=0xF=>rrc_r,
        0x10..=0x17=>rl_r,
        0x18..=0x1F=>rr_r,
        0x20..=0x27=>sla_r,
        0x28..=0x2F=>sra_r,
        0x30..=0x37=>swap_r,
        0x38..=0x3F=>srl_r,
        0x40..=0x7F=>bit_n_r,
        0x80..=0xBF=>res_n_r,
        0xC0..=0xFF=>set_n_r
    };
    return func(opcode);
}

fn rlc_r(opcode:u8)->OpcodeStr{OpcodeStr::from_args(format_args!("rlc {}", get_src_register(opcode)))}
fn rrc_r(opcode:u8)->OpcodeStr{OpcodeStr::from_args(format_args!("rrc {}", get_src_register(opcode)))}
fn rr_r(opcode:u8)->OpcodeStr{OpcodeStr::from_args(format_args!("rr {}", get_src_register(opcode)))}
fn rl_r(opcode:u8)->OpcodeStr{OpcodeStr::from_args(format_args!("rl {}", get_src_register(opcode)))}
fn sla_r(opcode:u8)->OpcodeStr{OpcodeStr::from_args(format_args!("sla {}", get_src_register(opcode)))}
fn sra_r(opcode:u8)->OpcodeStr{OpcodeStr::from_args(format_args!("sra {}", get_src_register(opcode)))}
fn swap_r(opcode:u8)->OpcodeStr{OpcodeStr::from_args(format_args!("swap {}", get_src_register(opcode)))}
fn srl_r(opcode:u8)->OpcodeStr{OpcodeStr::from_args(format_args!("srl {}", get_src_register(opcode)))}

fn bit_n_r(opcode:u8)->OpcodeStr{OpcodeStr::from_args(format_args!("bit {},{}", cb_bit_index(opcode), get_src_register(opcode)))}
fn res_n_r(opcode:u8)->OpcodeStr{OpcodeStr::from_args(format_args!("res {},{}", cb_bit_index(opcode), get_src_register(opcode)))}
fn set_n_r(opcode:u8)->OpcodeStr{OpcodeStr::from_args(format_args!("set {},{}", cb_bit_index(opcode), get_src_register(opcode)))}

fn cb_bit_index(opcode: u8) -> u8 {
    (opcode & 0b11_1000) >> 3
}

fn get_src_register(opcode:u8)->&'static str{get_r_register(opcode & 0b111)}
fn get_dest_register(opcode:u8)->&'static str{get_r_register((opcode & 0b0011_1000) >> 3)}

fn get_r_register(index:u8)->&'static str{
    match index{
        0=>"b",
        1=>"c",
        2=>"d",
        3=>"e",
        4=>"h",
        5=>"l",
        6=>"[hl]",
        7=>"a",
        _=>unreachable!()
    }
}

fn get_cc(opcode: u8)->&'static str{
    return match (opcode & 0b1_1000) >> 3{
        0=>"nz",
        1=>"nc",
        2=>"z",
        3=>"c",
        _=>unreachable!()
    };
}

fn get_rr_register(opcode:u8, use_sp:bool)->&'static str{
    match (opcode & 0b0011_0000) >> 4{
        0=>"bc",
        1=>"de",
        2=>"hl",
        3=>if use_sp{"sp"}else{"af"},
        _=>unreachable!()
    }
}

fn read_memory(memory:&mut impl Memory, pc:&mut u16)->u8{
    let val = memory.read(*pc, 0);
    *pc += 1;
    return val;
}

fn read_memory_u16(memory:&mut impl Memory, pc:&mut u16)->u16{
    let mut val = read_memory(memory, pc) as u16;
    val |= (read_memory(memory, pc) as u16) << 8; 
    return val;
}