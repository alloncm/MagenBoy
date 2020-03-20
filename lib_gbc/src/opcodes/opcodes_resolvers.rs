use crate::cpu::gbc_cpu::GbcCpu;
use crate::machine::memory::Memory;
use std::option::Option;
use crate::opcodes::{
    arithmetic_16bit_instructions::*,
    arithmetic_8bit_instructions::*,
    cpu_control_instructions::*,
    jump_instructions::*,
    load_16bit_instructions::*,
    load_8bit_instructions::*,
    rotate_shift_instructions::*,
    single_bit_sintructions::*
};


pub type OpcodeFunc = fn(&mut GbcCpu);
pub type U8OpcodeFunc = fn(&mut GbcCpu,u8);
pub type U16OpcodeFunc = fn(&mut GbcCpu,u16);
pub type U32OpcodeFunc = fn(&mut GbcCpu,u32);
pub type MemoryOpcodeFunc = fn(&mut GbcCpu,&mut dyn Memory);
pub type MemoryOpcodeFunc2Bytes = fn(&mut GbcCpu,&mut dyn Memory);
pub type U8MemoryOpcodeFunc = fn(&mut GbcCpu,&mut dyn Memory,u8);
pub type U16MemoryOpcodeFunc = fn(&mut GbcCpu,&mut dyn Memory,u16);
pub type U32MemoryOpcodeFunc = fn(&mut GbcCpu,&mut dyn Memory,u32);


fn prefix_cb_runner_u16_opcode(cpu:&mut GbcCpu, opcode:u16){
    let cb_postfix=0xFF&opcode;
    let opcode_to_run:U16OpcodeFunc = match cb_postfix{
        0x00..=0x05 | 0x07=> rlc_r,
        0x08..=0x0D | 0x0F=>rrc_r,
        0x10..=0x15 | 0x17=>rl_r,
        0x18..=0x1D | 0x1F=>rr_r,
        0x20..=0x25 | 0x27=>sla_r,
        0x28..=0x2D | 0x2F=>sra_r,
        0x30..=0x35 | 0x37=>swap_r,
        0x38..=0x3D | 0x3F=>srl_r,

        0x40..=0x45 | 0x47..=0x4D | 0x4F..=0x55 | 0x57..=0x5D |
        0x5F..=0x65 | 0x67..=0x6D | 0x6F..=0x75 | 0x77..=0x7D | 0x7F =>bit_r,

        0x80..=0x85 | 0x87..=0x8D | 0x8F..=0x95 | 0x97..=0x9D |
        0x9F..=0xA5 | 0xA7..=0xAD | 0xAF..=0xB5 | 0xB7..=0xBD | 0xBF =>res_r,

        0xC0..=0xC5 | 0xC7..=0xCD | 0xCF..=0xD5 | 0xD7..=0xDD |
        0xDF..=0xE5 | 0xE7..=0xED | 0xEF..=0xF5 | 0xF7..=0xFD | 0xFF =>set_r,
        _=>std::panic!("no opcode matching in the cb prefix for u16 opcodes: {}",opcode)
    };

    opcode_to_run(cpu, opcode);
}



pub fn get_opcode_func_resolver()->fn(u8)->Option<OpcodeFunc>{
    |opcode:u8|->Option<OpcodeFunc>{
        match opcode{
            0x00=>Some(|cpu|{}),
            0x07=>Some(rlca),
            0x0F=>Some(rrca),
            0x17=>Some(rla),
            0x1F=>Some(rra),
            0x2F=>Some(cpl),
            0x27=>Some(daa),
            0x37=>Some(scf),
            0x3F=>Some(ccf),
            0x76=>Some(halt),
            0xE9=>Some(jump_hl),
            0xF3=>Some(di),
            0xF9=>Some(load_sp_hl),
            0xFB=>Some(ei),
            _=>None
        }
    }
}

pub fn get_u8_opcode_func_resolver()->fn(u8)->Option<U8OpcodeFunc>{
    |opcode:u8|->Option<U8OpcodeFunc>{
        match opcode{
            0x03|0x13|0x23|0x33=>Some(inc_rr),
            0x04|0x14|0x24|0x0C|0x1C|0x2C|0x3C=>Some(inc_r),
            0x05|0x15|0x25|0x0D|0x1D|0x2D|0x3D=>Some(dec_r),
            0x09|0x19|0x29|0x39=>Some(add_hl_rr),
            0x0B|0x1B|0x2B|0x3B=>Some(dec_rr),
            0x80..=0x85 | 0x87=>Some(add_a_r),
            0x88..=0x8D | 0x8F=>Some(adc_a_r),
            0x90..=0x95 | 0x97=>Some(sub_a_r),
            0x98..=0x9D | 0x9F=>Some(sbc_a_r),
            0xA0..=0xA5 | 0xA7=>Some(and_a_r),
            0xA8..=0xAD | 0xAF=>Some(xor_a_r),
            0xB0..=0xB5 | 0xB7=>Some(or_a_r),
            0xB8..=0xBD | 0xBF=>Some(cp_a_r),
            _=>None
        }
    }
}

pub fn get_u16_opcode_func_resolver()->fn(u8)->Option<U16OpcodeFunc>{
    |opcode:u8|->Option<U16OpcodeFunc>{
        match opcode{
            0x18=>Some(jump_r),
            0x20|0x28|0x30|0x38=>Some(jump_r_cc),
            0xC6=>Some(add_a_nn),
            0xCB=>Some(prefix_cb_runner_u16_opcode),
            0xCE=>Some(adc_a_nn),
            0xD6=>Some(sub_a_nn),
            0xDE=>Some(sbc_a_nn),
            0xE6=>Some(and_a_nn),
            0xE8=>Some(add_sp_dd),
            0xEE=>Some(xor_a_nn),
            0xF6=>Some(or_a_nn),
            0xF8=>Some(ld_hl_spdd),
            0xFE=>Some(cp_a_nn),
            _=>None
        }
    }
}

pub fn get_u32_opcode_func_resolver()->fn(u8)->Option<U32OpcodeFunc>{
    |opcode:u8|->Option<U32OpcodeFunc>{
        match opcode{
            0x01 | 0x11 | 0x21 | 0x31=>Some(load_rr_nn),
            0xC2 | 0xD2 | 0xCA | 0xDA=>Some(jump_cc),
            0xC3=>Some(jump),
            _=>None
        }
    }
}

pub fn get_memory_opcode_func_resolver()->fn(u8)->Option<MemoryOpcodeFunc>{
    |opcode:u8|->Option<MemoryOpcodeFunc>{
        match opcode{
            0x02=>Some(ld_bc_a),
            0x0A=>Some(ld_a_bc),
            0x12=>Some(ld_de_a),
            0x1A=>Some(ld_a_de),
            0x22=>Some(ldi_hl_a),
            0x2A=>Some(ldi_a_hl),
            0x32=>Some(ldd_hl_a),
            0x34=>Some(inc_hl),
            0x35=>Some(dec_hl),
            0x3A=>Some(ldd_a_hl),
            0x86=>Some(add_a_hl),
            0x8E=>Some(adc_a_hl),
            0x96=>Some(sub_a_hl),
            0x9E=>Some(sbc_a_hl),
            0xA6=>Some(and_a_hl),
            0xAE=>Some(xor_a_hl),
            0xB6=>Some(or_a_hl),
            0xBE=>Some(cp_a_hl),
            0xC9=>Some(ret),
            0xD9=>Some(reti),
            0xE2=>Some(ld_ioport_c_a),
            0xF2=>Some(ld_a_ioport_c),
            _=>None
        }   
    } 
}

pub fn get_memory_opcode_func_2bytes_resolver()->fn(u8,u8)->Option<MemoryOpcodeFunc2Bytes>{
    |opcode:u8,next_byte:u8|->Option<MemoryOpcodeFunc2Bytes>{
        if opcode == 0x10 && next_byte == 0{
            return Some(stop);
        }

        match next_byte{
            0x06=>Some(rlc_hl),
            0x0E=>Some(rrc_hl),
            0x16=>Some(rl_hl),
            0x1E=>Some(rr_hl),
            0x26=>Some(sla_hl),
            0x2E=>Some(sra_hl),
            0x36=>Some(swap_hl),
            0x3E=>Some(srl_hl),
            _=>None
        }
    }
}

pub fn get_u8_memory_opcode_func_resolver()->fn(u8)->Option<U8MemoryOpcodeFunc>{
    |opcode:u8|->Option<U8MemoryOpcodeFunc>{
        match opcode{
            0x46|0x4E|0x56|0x5E|0x66|0x6E|0x7F=>Some(ld_r_hl),
            0x70..=0x75 | 0x77=>Some(ld_hl_r),
            0xC0|0xC8|0xD0|0xD8=>Some(ret_cc),
            0xC1|0xD1|0xE1|0xF1=>Some(pop),
            0xC5|0xD5|0xE5|0xF5=>Some(push),
            0xC6|0xCF|0xD6|0xDF|0xE6|0xEF|0xF6|0xFF=>Some(rst),
            _=>None
        }
    }
}

pub fn get_u16_memory_opcode_func_resolver()->fn(u8,u8)->Option<U16MemoryOpcodeFunc>{
    |opcode:u8, next_byte:u8|->Option<U16MemoryOpcodeFunc>{
        match opcode{
            0x36=>Some(ld_hl_n),
            0xE1=>Some(ld_ioport_n_a),
            0xF1=>Some(ld_a_ioport_n),
            0xCB=>match next_byte{
                0x46|0x4E|0x56|0x5E|0x66|0x6E|0x76|0x7E=>Some(bit_hl),
                0x86|0x8E|0x96|0x9E|0xA6|0xAE|0xB6|0xBE=>Some(res_hl),
                0xC6|0xCE|0xD6|0xDE|0xE6|0xEE|0xF6|0xFE=>Some(set_hl),
                _=>None
            },
            _=>None
        }
    }
}

pub fn get_u32_memory_opcode_func_resolver()->fn(u8)->Option<U32MemoryOpcodeFunc>{
    |opcode:u8|->Option<U32MemoryOpcodeFunc>{
        match opcode{
            0x08=>Some(ld_nn_sp),
            0xC4|0xCC|0xD4|0xDC=>Some(call_cc),
            0xCD=>Some(call),
            0xEA=>Some(ld_nn_a), 
            0xFA=>Some(ld_a_nn),
            _=>None
        }
    }
}