use crate::cpu::gbc_cpu::GbcCpu;
use crate::machine::memory::Memory;
use crate::machine::gbc_memory::GbcMmu;
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

type OpcodeFunc = fn(&mut GbcCpu);
type U8OpcodeFunc = fn(&mut GbcCpu,u8);
type U16OpcodeFunc = fn(&mut GbcCpu,u16);
type U32OpcodeFunc = fn(&mut GbcCpu,u32);
type MemoryOpcodeFunc = fn(&mut GbcCpu,&mut dyn Memory);
type MemoryOpcodeFunc2Bytes = fn(&mut GbcCpu,&mut dyn Memory);
type U8MemoryOpcodeFunc = fn(&mut GbcCpu,&mut dyn Memory,u8);
type U16MemoryOpcodeFunc = fn(&mut GbcCpu,&mut dyn Memory,u16);
type U32MemoryOpcodeFunc = fn(&mut GbcCpu,&mut dyn Memory,u32);


pub struct GameBoy {
    pub cpu: GbcCpu,
    pub mmu: GbcMmu,
    opcode_func_resolver:fn(u8)->OpcodeFunc,
    u8_opcode_func_resolver:fn(u8)->U8OpcodeFunc,
    u16_opcode_func_resolver:fn(u8)->U16OpcodeFunc,
    u32_opcode_func_resolver:fn(u8)->U32OpcodeFunc,
    memory_opcode_func_resolver:fn(u8)->MemoryOpcodeFunc,
    memory_opcode_func_2bytes_resolver:fn(u8,u8)->MemoryOpcodeFunc2Bytes,
    u8_memory_opcode_func_resolver:fn(u8)->U8MemoryOpcodeFunc,
    u16_memory_opcode_func_resolver:fn(u8,u8)->U16MemoryOpcodeFunc,
    u32_memory_opcode_func_resolver:fn(u8)->U32MemoryOpcodeFunc,
}

impl GameBoy{

    fn fetch_next_byte(&self)->u8{
        self.mmu.read(self.cpu.program_counter)
    }

    pub fn cycle(&self){
        let opcode = self.fetch_next_byte();
    }
}

impl Default for GameBoy{
    fn default()->GameBoy{
        GameBoy{
            cpu:GbcCpu::default(),
            mmu:GbcMmu::default(),
            opcode_func_resolver:get_opcode_func_resolver(),
            memory_opcode_func_resolver:get_memory_opcode_func_resolver(),
            memory_opcode_func_2bytes_resolver:get_memory_opcode_func_2bytes_resolver(),
            u8_opcode_func_resolver:get_u8_opcode_func_resolver(),
            u8_memory_opcode_func_resolver:get_u8_memory_opcode_func_resolver(),
            u16_memory_opcode_func_resolver:get_u16_memory_opcode_func_resolver(),
            u16_opcode_func_resolver:get_u16_opcode_func_resolver(),
            u32_opcode_func_resolver:get_u32_opcode_func_resolver(),
            u32_memory_opcode_func_resolver:get_u32_memory_opcode_func_resolver()
        }
    }
}

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



fn get_opcode_func_resolver()->fn(u8)->OpcodeFunc{
    |opcode:u8|->OpcodeFunc{
        match opcode{
            0x00=>|cpu|{},
            0x07=>rlca,
            0x0F=>rrca,
            0x17=>rla,
            0x1F=>rra,
            0x2F=>cpl,
            0x27=>daa,
            0x37=>scf,
            0x3F=>ccf,
            0x76=>halt,
            0xE9=>jump_hl,
            0xF3=>di,
            0xF9=>load_sp_hl,
            0xFB=>ei,
            _=>std::panic!("no opcode in the config: {}",opcode)
        }
    }
}

fn get_u8_opcode_func_resolver()->fn(u8)->U8OpcodeFunc{
    |opcode:u8|->U8OpcodeFunc{
        match opcode{
            0x03|0x13|0x23|0x33=>inc_rr,
            0x04|0x14|0x24|0x0C|0x1C|0x2C|0x3C=>inc_r,
            0x05|0x15|0x25|0x0D|0x1D|0x2D|0x3D=>dec_r,
            0x09|0x19|0x29|0x39=>add_hl_rr,
            0x0B|0x1B|0x2B|0x3B=>dec_rr,
            0x80..=0x85 | 0x87=>add_a_r,
            0x88..=0x8D | 0x8F=>adc_a_r,
            0x90..=0x95 | 0x97=>sub_a_r,
            0x98..=0x9D | 0x9F=>sbc_a_r,
            0xA0..=0xA5 | 0xA7=>and_a_r,
            0xA8..=0xAD | 0xAF=>xor_a_r,
            0xB0..=0xB5 | 0xB7=>or_a_r,
            0xB8..=0xBD | 0xBF=>cp_a_r,
            _=>std::panic!("no opcode for opcode {}",opcode)
        }
    }
}

fn get_u16_opcode_func_resolver()->fn(u8)->U16OpcodeFunc{
    |opcode:u8|->U16OpcodeFunc{
        match opcode{
            0x18=>jump_r,
            0x20|0x28|0x30|0x38=>jump_r_cc,
            0xC6=>add_a_nn,
            0xCB=>prefix_cb_runner_u16_opcode,
            0xCE=>adc_a_nn,
            0xD6=>sub_a_nn,
            0xDE=>sbc_a_nn,
            0xE6=>and_a_nn,
            0xE8=>add_sp_dd,
            0xEE=>xor_a_nn,
            0xF6=>or_a_nn,
            0xF8=>ld_hl_spdd,
            0xFE=>cp_a_nn,
            _=>std::panic!("no opcode to match u16 opcodes: {}",opcode)
        }
    }
}

fn get_u32_opcode_func_resolver()->fn(u8)->U32OpcodeFunc{
    |opcode:u8|->U32OpcodeFunc{
        match opcode{
            0x01 | 0x11 | 0x21 | 0x31=>load_rr_nn,
            0xC2 | 0xD2 | 0xCA | 0xDA=>jump_cc,
            0xC3=>jump,
            _=>std::panic!("no u24 opcode: {}",opcode)
        }
    }
}

fn get_memory_opcode_func_resolver()->fn(u8)->MemoryOpcodeFunc{
    |opcode:u8|->MemoryOpcodeFunc{
        match opcode{
            0x02=>ld_bc_a,
            0x0A=>ld_a_bc,
            0x12=>ld_de_a,
            0x1A=>ld_a_de,
            0x22=>ldi_hl_a,
            0x2A=>ldi_a_hl,
            0x32=>ldd_hl_a,
            0x34=>inc_hl,
            0x35=>dec_hl,
            0x3A=>ldd_a_hl,
            0x86=>add_a_hl,
            0x8E=>adc_a_hl,
            0x96=>sub_a_hl,
            0x9E=>sbc_a_hl,
            0xA6=>and_a_hl,
            0xAE=>xor_a_hl,
            0xB6=>or_a_hl,
            0xBE=>cp_a_hl,
            0xC9=>ret,
            0xD9=>reti,
            0xE2=>ld_ioport_c_a,
            0xF2=>ld_a_ioport_c,
            _=>std::panic!("no opcode{}",opcode)
        }   
    } 
}

fn get_memory_opcode_func_2bytes_resolver()->fn(u8,u8)->MemoryOpcodeFunc2Bytes{
    |opcode:u8,next_byte:u8|->MemoryOpcodeFunc2Bytes{


        if opcode == 0x10 && next_byte == 0{
            return stop;
        }

        match next_byte{
            0x06=>rlc_hl,
            0x0E=>rrc_hl,
            0x16=>rl_hl,
            0x1E=>rr_hl,
            0x26=>sla_hl,
            0x2E=>sra_hl,
            0x36=>swap_hl,
            0x3E=>srl_hl,
            _=>std::panic!("no opcode stop: {}",opcode)
        }
    }
}

fn get_u8_memory_opcode_func_resolver()->fn(u8)->U8MemoryOpcodeFunc{
    |opcode:u8|->U8MemoryOpcodeFunc{
        match opcode{
            0x46|0x4E|0x56|0x5E|0x66|0x6E|0x7F=>ld_r_hl,
            0x70..=0x75 | 0x77=>ld_hl_r,
            0xC0|0xC8|0xD0|0xD8=>ret_cc,
            0xC1|0xD1|0xE1|0xF1=>pop,
            0xC5|0xD5|0xE5|0xF5=>push,
            0xC6|0xCF|0xD6|0xDF|0xE6|0xEF|0xF6|0xFF=>rst,
            _=>std::panic!("no opcode stop: {}",opcode)
        }
    }
}

fn get_u16_memory_opcode_func_resolver()->fn(u8,u8)->U16MemoryOpcodeFunc{
    |opcode:u8, next_byte:u8|->U16MemoryOpcodeFunc{
        match opcode{
            0x36=>ld_hl_n,
            0xE1=>ld_ioport_n_a,
            0xF1=>ld_a_ioport_n,
            0xCB=>match next_byte{
                0x46|0x4E|0x56|0x5E|0x66|0x6E|0x76|0x7E=>bit_hl,
                0x86|0x8E|0x96|0x9E|0xA6|0xAE|0xB6|0xBE=>res_hl,
                0xC6|0xCE|0xD6|0xDE|0xE6|0xEE|0xF6|0xFE=>set_hl,
                _=>std::panic!("no pocode"),
            },
            _=>std::panic!("no pocode"),
        }
    }
}

fn get_u32_memory_opcode_func_resolver()->fn(u8)->U32MemoryOpcodeFunc{
    |opcode:u8|->U32MemoryOpcodeFunc{
        match opcode{
            0x08=>ld_nn_sp,
            0xC4|0xCC|0xD4|0xDC=>call_cc,
            0xCD=>call,
            0xEA=>ld_nn_a, 
            0xFA=>ld_a_nn,
            _=>std::panic!("no pocode"),
        }
    }
}