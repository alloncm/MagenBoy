use crate::cpu::gbc_cpu::GbcCpu;
use crate::mmu::memory::Memory;
use crate::mmu::gbc_mmu::{
    GbcMmu,
    BOOT_ROM_SIZE
};
use crate::opcodes::opcode_resolver::*;
use crate::ppu::gbc_ppu::GbcPpu;
use crate::machine::registers_handler::update_registers_state;
use crate::mmu::mbc::Mbc;
use crate::ppu::gbc_ppu::{
    SCREEN_HEIGHT,
    SCREEN_WIDTH
};
use super::interrupts_handler::handle_interrupts;
use std::boxed::Box;
use log::info;

pub struct GameBoy {
    cpu: GbcCpu,
    mmu: GbcMmu,
    opcode_resolver:OpcodeResolver,
    ppu:GbcPpu,
    cycles_per_frame:u32
}

impl GameBoy{

    pub fn new(mbc:Box<dyn Mbc>, boot_rom:[u8;BOOT_ROM_SIZE],cycles:u32)->GameBoy{
        GameBoy{
            cpu:GbcCpu::default(),
            mmu:GbcMmu::new(mbc, boot_rom),
            opcode_resolver:OpcodeResolver::default(),
            ppu:GbcPpu::default(),
            cycles_per_frame:cycles
        }
    }

    pub fn cycle_frame(&mut self)->&[u32;SCREEN_HEIGHT*SCREEN_WIDTH]{
        for i in 0..self.cycles_per_frame{
            self.execute_opcode();
            self.ppu.update_gb_screen(&mut self.mmu, i);
            update_registers_state(&mut self.mmu, &mut self.cpu, &mut self.ppu);
            //handle_interrupts(&mut self.cpu);
        }

        return self.ppu.get_frame_buffer();
    }

    fn fetch_next_byte(&mut self)->u8{
        let byte:u8 = self.mmu.read(self.cpu.program_counter);
        self.cpu.program_counter+=1;
        return byte;
    }

    fn execute_opcode(&mut self){
        let pc = self.cpu.program_counter;
        
        let opcode:u8 = self.fetch_next_byte();

        /*
        if pc >= 0xFF{
        info!("Opcode:{:#x} AF:{:#x},BC:{:#x},DE:{:#x},HL:{:#x},PC:{:#x},SP:{:#x}",
            opcode, self.cpu.af.value(),self.cpu.bc.value(),self.cpu.de.value(),self.cpu.hl.value(),self.cpu.program_counter, self.cpu.stack_pointer);
        }
        */
        let opcode_func:OpcodeFuncType = self.opcode_resolver.get_opcode(opcode, &self.mmu, self.cpu.program_counter);
        match opcode_func{
            OpcodeFuncType::OpcodeFunc(func)=>func(&mut self.cpu),
            OpcodeFuncType::MemoryOpcodeFunc(func)=>func(&mut self.cpu, &mut self.mmu),
            OpcodeFuncType::U8OpcodeFunc(func)=>func(&mut self.cpu, opcode),
            OpcodeFuncType::U8MemoryOpcodeFunc(func)=>func(&mut self.cpu, &mut self.mmu, opcode),
            OpcodeFuncType::U16OpcodeFunc(func)=>{
                let u16_opcode:u16 = ((opcode as u16)<<8) | (self.fetch_next_byte() as u16);
                func(&mut self.cpu, u16_opcode);
            },
            OpcodeFuncType::U16MemoryOpcodeFunc(func)=>{
                let u16_opcode:u16 = ((opcode as u16)<<8) | (self.fetch_next_byte() as u16);
                func(&mut self.cpu, &mut self.mmu, u16_opcode);
            },
            OpcodeFuncType::U32OpcodeFunc(func)=>{
                let mut u32_opcode:u32 = ((opcode as u32)<<8) | (self.fetch_next_byte() as u32);
                u32_opcode <<= 8;
                u32_opcode |= self.fetch_next_byte() as u32;
                func(&mut self.cpu, u32_opcode);
            },
            OpcodeFuncType::U32MemoryOpcodeFunc(func)=>{
                let mut u32_opcode:u32 = ((opcode as u32)<<8) | (self.fetch_next_byte() as u32);
                u32_opcode <<= 8;
                u32_opcode |= self.fetch_next_byte() as u32;
                func(&mut self.cpu, &mut self.mmu, u32_opcode);
            }
        }
    }
}

