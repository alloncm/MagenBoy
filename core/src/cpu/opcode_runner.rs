use crate::mmu::Memory;
use super::{
    gb_cpu::GbCpu, 
    opcodes::{
        arithmetic_8bit_instructions::*,
        rotate_shift_instructions::*,
        cpu_control_instructions::*,
        jump_instructions::*,
        load_16bit_instructions::*,
        arithmetic_16bit_instructions::*,
        load_8bit_instructions::*,
        single_bit_sintructions::*,
    }
};


type U16MemoryOpcodeFunc<T> = fn(&mut GbCpu,&mut T,u16)->u8;
type U32MemoryOpcodeFunc<T> = fn(&mut GbCpu,&mut T,u32)->u8;

impl GbCpu{
    pub fn run_opcode(&mut self, memory:&mut impl Memory)->u8{
        let opcode = self.fetch_next_byte(memory);
    
        match opcode{
            //Stop
            0x10=>{
                // TODO: verify if stop is 1 byte or 2 bytes
                let next_byte = self.fetch_next_byte(memory);
                if next_byte == 0{
                    stop(self, memory)
                }
                else{
                    core::panic!("Invalid stop opcode, second byte: {:#X}", next_byte);
                }
            }
    
            //just cpu
            0x00=>0,    // 1 cycles - 1 reading opcode
            0x07=>rlca(self),
            0x0F=>rrca(self),
            0x17=>rla(self),
            0x1F=>rra(self),
            0x2F=>cpl(self),
            0x27=>daa(self),
            0x37=>scf(self),
            0x3F=>ccf(self),
            0xE9=>jump_hl(self),
            0xF3=>di(self),
            0xF9=>load_sp_hl(self),
            0xFB=>ei(self),
    
            //cpu and opcode
            0x03|0x13|0x23|0x33=>inc_rr(self, opcode),
            0x04|0x14|0x24|0x0C|0x1C|0x2C|0x3C=>inc_r(self, opcode),
            0x05|0x15|0x25|0x0D|0x1D|0x2D|0x3D=>dec_r(self, opcode),
            0x09|0x19|0x29|0x39=>add_hl_rr(self, opcode),
            0x0B|0x1B|0x2B|0x3B=>dec_rr(self, opcode),
            0x40..=0x45 | 0x47..=0x4D | 0x4F..=0x55 | 0x57..=0x5D |
            0x5F..=0x65 | 0x67..=0x6D | 0x6F | 0x78..=0x7D | 0x7F=>ld_r_r(self, opcode),
            0x80..=0x85 | 0x87=>add_a_r(self, opcode),
            0x88..=0x8D | 0x8F=>adc_a_r(self, opcode),
            0x90..=0x95 | 0x97=>sub_a_r(self, opcode),
            0x98..=0x9D | 0x9F=>sbc_a_r(self, opcode),
            0xA0..=0xA5 | 0xA7=>and_a_r(self, opcode),
            0xA8..=0xAD | 0xAF=>xor_a_r(self, opcode),
            0xB0..=0xB5 | 0xB7=>or_a_r(self, opcode),
            0xB8..=0xBD | 0xBF=>cp_a_r(self, opcode),
    
            //u16 opcode
            0x06|0x0E|0x16|0x1E|0x26|0x2E|0x3E=>run_u16_opcode(self, memory, opcode, ld_r_n),
            0x18=>run_u16_opcode(self, memory, opcode, jump_r),
            0x20|0x28|0x30|0x38=>run_u16_opcode(self, memory, opcode, jump_r_cc),
            0xC6=>run_u16_opcode(self, memory, opcode, add_a_nn),
            0xCE=>run_u16_opcode(self, memory, opcode, adc_a_nn),
            0xD6=>run_u16_opcode(self, memory, opcode, sub_a_nn),
            0xDE=>run_u16_opcode(self, memory, opcode, sbc_a_nn),
            0xE6=>run_u16_opcode(self, memory, opcode, and_a_nn),
            0xE8=>run_u16_opcode(self, memory, opcode, add_sp_dd),
            0xEE=>run_u16_opcode(self, memory, opcode, xor_a_nn),
            0xF6=>run_u16_opcode(self, memory, opcode, or_a_nn),
            0xF8=>run_u16_opcode(self, memory, opcode, ld_hl_spdd),
            0xFE=>run_u16_opcode(self, memory, opcode, cp_a_nn),   
    
            //u32 opcodes
            0x01 | 0x11 | 0x21 | 0x31=>run_u32_opcode(self, memory, opcode, load_rr_nn),
            0xC2 | 0xD2 | 0xCA | 0xDA=>run_u32_opcode(self, memory, opcode,jump_cc),
            0xC3=>run_u32_opcode(self, memory, opcode,jump),
    
            //Memory opcodes
            0x02=>ld_bc_a(self, memory),
            0x0A=>ld_a_bc(self, memory),
            0x12=>ld_de_a(self, memory),
            0x1A=>ld_a_de(self, memory),
            0x22=>ldi_hl_a(self, memory),
            0x2A=>ldi_a_hl(self, memory),
            0x32=>ldd_hl_a(self, memory),
            0x34=>inc_hl(self, memory),
            0x35=>dec_hl(self, memory),
            0x3A=>ldd_a_hl(self, memory),
            0x76=>halt(self, memory),
            0x86=>add_a_hl(self, memory),
            0x8E=>adc_a_hl(self, memory),
            0x96=>sub_a_hl(self, memory),
            0x9E=>sbc_a_hl(self, memory),
            0xA6=>and_a_hl(self, memory),
            0xAE=>xor_a_hl(self, memory),
            0xB6=>or_a_hl(self, memory),
            0xBE=>cp_a_hl(self, memory),
            0xC9=>ret(self, memory),
            0xD9=>reti(self, memory),
            0xE2=>ld_ioport_c_a(self, memory),
            0xF2=>ld_a_ioport_c(self, memory),
    
            //Memory u8 opcodes
            0x46|0x4E|0x56|0x5E|0x66|0x6E|0x7E=>ld_r_hl(self, memory, opcode),
            0x70..=0x75 | 0x77=>ld_hl_r(self, memory, opcode),
            0xC0|0xC8|0xD0|0xD8=>ret_cc(self, memory, opcode),
            0xC1|0xD1|0xE1|0xF1=>pop(self, memory, opcode),
            0xC5|0xD5|0xE5|0xF5=>push(self, memory, opcode),
            0xC7|0xCF|0xD7|0xDF|0xE7|0xEF|0xF7|0xFF=>rst(self, memory, opcode),
    
            //Memory u16 opcodes 
            0x36=>run_u16_memory_opcode(self, memory, opcode, ld_hl_n),
            0xE0=>run_u16_memory_opcode(self, memory, opcode, ld_ioport_n_a),
            0xF0=>run_u16_memory_opcode(self, memory, opcode, ld_a_ioport_n),
    
            //Memory u32 opcodes
            0x08=>run_u32_memory_opcode(self, memory, opcode, ld_nn_sp),
            0xC4|0xCC|0xD4|0xDC=>run_u32_memory_opcode(self, memory, opcode, call_cc),
            0xCD=>run_u32_memory_opcode(self, memory, opcode, call),
            0xEA=>run_u32_memory_opcode(self, memory, opcode, ld_nn_a), 
            0xFA=>run_u32_memory_opcode(self, memory, opcode, ld_a_nn),
    
            //0xCB opcodes
            0xCB=>{
                let next = self.fetch_next_byte(memory);
                let u16_opcode = (opcode as u16) << 8 | next as u16;
                match next{
                    0x00..=0x05 | 0x07=>rlc_r(self, u16_opcode),
                    0x08..=0x0D | 0x0F=>rrc_r(self, u16_opcode),
                    0x10..=0x15 | 0x17=>rl_r(self, u16_opcode),
                    0x18..=0x1D | 0x1F=>rr_r(self, u16_opcode),
                    0x20..=0x25 | 0x27=>sla_r(self, u16_opcode),
                    0x28..=0x2D | 0x2F=>sra_r(self, u16_opcode),
                    0x30..=0x35 | 0x37=>swap_r(self, u16_opcode),
                    0x38..=0x3D | 0x3F=>srl_r(self, u16_opcode),
                    0x40..=0x45 | 0x47..=0x4D | 0x4F..=0x55 | 0x57..=0x5D |
                    0x5F..=0x65 | 0x67..=0x6D | 0x6F..=0x75 | 0x77..=0x7D | 0x7F =>bit_r(self, u16_opcode),
                    0x80..=0x85 | 0x87..=0x8D | 0x8F..=0x95 | 0x97..=0x9D |
                    0x9F..=0xA5 | 0xA7..=0xAD | 0xAF..=0xB5 | 0xB7..=0xBD | 0xBF =>res_r(self, u16_opcode),
                    0xC0..=0xC5 | 0xC7..=0xCD | 0xCF..=0xD5 | 0xD7..=0xDD |
                    0xDF..=0xE5 | 0xE7..=0xED | 0xEF..=0xF5 | 0xF7..=0xFD | 0xFF =>set_r(self, u16_opcode),
    
                    0x06=>rlc_hl(self, memory),
                    0x0E=>rrc_hl(self, memory),
                    0x16=>rl_hl(self, memory),
                    0x1E=>rr_hl(self, memory),
                    0x26=>sla_hl(self, memory),
                    0x2E=>sra_hl(self, memory),
                    0x36=>swap_hl(self, memory),
                    0x3E=>srl_hl(self, memory),
    
                    0x46|0x4E|0x56|0x5E|0x66|0x6E|0x76|0x7E=>bit_hl(self, memory, u16_opcode),
                    0x86|0x8E|0x96|0x9E|0xA6|0xAE|0xB6|0xBE=>res_hl(self, memory, u16_opcode),
                    0xC6|0xCE|0xD6|0xDE|0xE6|0xEE|0xF6|0xFE=>set_hl(self, memory, u16_opcode),
                }
            },
    
            _=>core::panic!("Unsupported opcode:{:#X}", opcode)
        }
    }

    
    fn fetch_next_byte(&mut self, memory: &mut impl Memory)->u8{
        let byte:u8 = memory.read(self.program_counter, 1);
        self.program_counter+=1;
        return byte;
    }
}



fn run_u16_opcode(cpu: &mut GbCpu, memory: &mut impl Memory, opcode:u8, opcode_func:fn(&mut GbCpu, u16)->u8)->u8{
    let u16_opcode = get_u16_opcode(cpu, memory, opcode);
    opcode_func(cpu, u16_opcode)
}

fn run_u16_memory_opcode<T:Memory>(cpu: &mut GbCpu, memory: &mut T, opcode:u8, opcode_func:U16MemoryOpcodeFunc<T>)->u8{
    let u16_opcode = get_u16_opcode(cpu, memory, opcode);
    opcode_func(cpu, memory, u16_opcode)
}

fn run_u32_opcode(cpu: &mut GbCpu, memory: &mut impl Memory, opcode:u8, opcode_func:fn(&mut GbCpu, u32)->u8)->u8{
    let mut u32_opcode:u32 = ((opcode as u32)<<8) | (cpu.fetch_next_byte(memory) as u32);
    u32_opcode <<= 8;
    u32_opcode |= cpu.fetch_next_byte(memory) as u32;

    opcode_func(cpu, u32_opcode)
}

fn run_u32_memory_opcode<T:Memory>(cpu: &mut GbCpu, memory: &mut T, opcode:u8, opcode_func:U32MemoryOpcodeFunc<T>)->u8{
    let mut u32_opcode:u32 = ((opcode as u32)<<8) | (cpu.fetch_next_byte(memory) as u32);
    u32_opcode <<= 8;
    u32_opcode |= cpu.fetch_next_byte(memory) as u32;

    opcode_func(cpu, memory, u32_opcode)
}

fn get_u16_opcode(cpu:&mut GbCpu, memory:&mut impl Memory, opcode:u8)->u16{
    (opcode as u16) << 8 | cpu.fetch_next_byte(memory) as u16
}