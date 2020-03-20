use crate::cpu::gbc_cpu::GbcCpu;
use crate::machine::memory::Memory;
use crate::machine::gbc_memory::GbcMmu;
use crate::opcodes::opcode_resolver::*;
use crate::ppu::gbc_ppu::GbcPpu;

pub struct GameBoy<'a> {
    pub cpu: &'a mut GbcCpu,
    pub mmu: &'a mut GbcMmu,
    opcode_resolver:Option<OpcodeResolver<'a>>,
    pub ppu:Option<GbcPpu<'a>>
}

impl<'a> GameBoy<'a>{

    fn fetch_next_byte(&mut self)->u8{
        let byte:u8 = self.mmu.read(self.cpu.program_counter);
        self.cpu.program_counter+=1;
        return byte;
    }

    pub fn cycle(&mut self){
        let opcode:u8 = self.fetch_next_byte();
        let opcode_func:OpcodeFuncType = self.opcode_resolver.as_ref().unwrap().get_opcode(opcode);
        let memory:&'a mut dyn Memory = &mut *self.mmu;
        match opcode_func{
            OpcodeFuncType::OpcodeFunc(func)=>func(&mut self.cpu),
            OpcodeFuncType::MemoryOpcodeFunc(func)=>func(&mut self.cpu, memory),
            OpcodeFuncType::U8OpcodeFunc(func)=>func(&mut self.cpu, opcode),
            OpcodeFuncType::U8MemoryOpcodeFunc(func)=>func(&mut self.cpu, memory, opcode),
            OpcodeFuncType::MemoryOpcodeFunc2Bytes(func)=>func(&mut self.cpu, memory),
            OpcodeFuncType::U16OpcodeFunc(func)=>{
                let u16_opcode:u16 = ((opcode<<8)as u16) | (self.fetch_next_byte() as u16);
                func(&mut self.cpu, u16_opcode);
            },
            OpcodeFuncType::U16MemoryOpcodeFunc(func)=>{
                let u16_opcode:u16 = ((opcode<<8)as u16) | (self.fetch_next_byte() as u16);
                func(&mut self.cpu, memory, u16_opcode);
            },
            OpcodeFuncType::U32OpcodeFunc(func)=>{
                let mut u32_opcode:u32 = ((opcode<<8)as u32) | (self.fetch_next_byte() as u32);
                u32_opcode <<= 8;
                u32_opcode |= self.fetch_next_byte() as u32;
                func(&mut self.cpu, u32_opcode);
            },
            OpcodeFuncType::U32MemoryOpcodeFunc(func)=>{
                let mut u32_opcode:u32 = ((opcode<<8)as u32) | (self.fetch_next_byte() as u32);
                u32_opcode <<= 8;
                u32_opcode |= self.fetch_next_byte() as u32;
                func(&mut self.cpu, memory, u32_opcode);
            }
        }
    }

    pub fn new(cpu:&'a GbcCpu, mmu:&'a GbcMmu, resolver:OpcodeResolver<'a>, ppu:GbcPpu<'a>)->GameBoy<'a>{
        let gb = GameBoy{
            cpu:cpu,
            mmu:mmu,
            opcode_resolver:Option::Some(resolver),
            ppu:Option::Some(ppu)
        };

        return gb;
    }
}