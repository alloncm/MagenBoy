use crate::cpu::gbc_cpu::GbcCpu;
use crate::machine::memory::Memory;
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
type U8MemoryOpcodeFunc = fn(&mut GbcCpu,&mut dyn Memory,u8);
type U32MemoryOpcodeFunc = fn(&mut GbcCpu,&mut dyn Memory,u32);

pub struct GameBoy {
    pub cpu: GbcCpu,
    pub mmu:dyn Memory
}

enum OpcodeType{
    U8(u8),
    U16(u16),
    U24(u32)
}

impl GameBoy{

    fn get_opcode_func_resolver(&self)->fn(OpcodeType)->OpcodeFunc{
        |opcode:OpcodeType|->OpcodeFunc{
            match opcode{
                OpcodeType::U8(op)=>match op{
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
                    _=>std::panic!("no opcode: {}",op)
                },
                OpcodeType::U16(op)=>std::panic!("no opcode type with bigger than u8 opcode: {}",op),
                OpcodeType::U24(op)=>std::panic!("no opcode type with bigger than u8 opcode: {}",op)
            }
        }
    }


    fn fetch_first_byte(&mut self)->u8{
        self.mmu.read(self.cpu.program_counter)
    }

    pub fn cycle(&mut self){
        let opcode = self.fetch_first_byte();
    }
}
