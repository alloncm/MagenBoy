use crate::cpu::gbc_cpu::GbcCpu;
use crate::machine::memory::Memory;
use crate::machine::gbc_memory::GbcMmu;
use crate::opcodes::opcode_resolver::*;
use crate::ppu::gbc_ppu::GbcPpu;

pub struct GameBoy<'a> {
    pub cpu: GbcCpu,
    pub mmu: GbcMmu,
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
        match opcode_func{
            OpcodeFuncType::OpcodeFunc(func)=>func(&mut self.cpu),
            OpcodeFuncType::MemoryOpcodeFunc(func)=>func(&mut self.cpu,&mut self.mmu),
            OpcodeFuncType::U8OpcodeFunc(func)=>func(&mut self.cpu, opcode),
            OpcodeFuncType::U8MemoryOpcodeFunc(func)=>func(&mut self.cpu, &mut self.mmu, opcode),
            OpcodeFuncType::MemoryOpcodeFunc2Bytes(func)=>func(&mut self.cpu, &mut self.mmu),
            OpcodeFuncType::U16OpcodeFunc(func)=>{
                let u16_opcode:u16 = ((opcode<<8)as u16) | (self.fetch_next_byte() as u16);
                func(&mut self.cpu, u16_opcode);
            },
            OpcodeFuncType::U16MemoryOpcodeFunc(func)=>{
                let u16_opcode:u16 = ((opcode<<8)as u16) | (self.fetch_next_byte() as u16);
                func(&mut self.cpu, &mut self.mmu, u16_opcode);
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
                func(&mut self.cpu, &mut self.mmu, u32_opcode);
            }
        }
    }

    pub fn new()->GameBoy<'a>{

        let cpu = GbcCpu::default();
        let mmu = GbcMmu::default();
        let mut gb = GameBoy{
            cpu:cpu,
            mmu:mmu,
            opcode_resolver:Option::None,
            ppu:Option::None
        };

        gb.opcode_resolver = Option::Some(OpcodeResolver::<'a>new(&gb.mmu, &gb.cpu.program_counter));
        return gb;
    }
}