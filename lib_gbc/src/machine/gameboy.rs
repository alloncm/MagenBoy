use crate::cpu::gbc_cpu::GbcCpu;
use super::joypad::Joypad;
use crate::mmu::memory::Memory;
use crate::mmu::gbc_mmu::{
    GbcMmu,
    BOOT_ROM_SIZE
};
use crate::opcodes::opcode_resolver::*;
use crate::ppu::gbc_ppu::GbcPpu;
use crate::machine::registers_handler::RegisterHandler;
use crate::mmu::carts::mbc::Mbc;
use crate::ppu::gbc_ppu::{
    SCREEN_HEIGHT,
    SCREEN_WIDTH
};
use super::interrupts_handler::InterruptsHandler;
use std::boxed::Box;
use log::info;

pub struct GameBoy {
    cpu: GbcCpu,
    mmu: GbcMmu,
    opcode_resolver:OpcodeResolver,
    ppu:GbcPpu,
    register_handler:RegisterHandler,
    cycles_per_frame:u32,
    interrupts_handler:InterruptsHandler
}

impl GameBoy{

    pub fn new(mbc:Box<dyn Mbc>, boot_rom:[u8;BOOT_ROM_SIZE],cycles:u32)->GameBoy{
        GameBoy{
            cpu:GbcCpu::default(),
            mmu:GbcMmu::new(mbc, boot_rom),
            opcode_resolver:OpcodeResolver::default(),
            ppu:GbcPpu::default(),
            register_handler: RegisterHandler::default(),
            cycles_per_frame:cycles,
            interrupts_handler: InterruptsHandler::default()
        }
    }

    pub fn cycle_frame(&mut self,joypad:&Joypad )->&[u32;SCREEN_HEIGHT*SCREEN_WIDTH]{
        for _ in (0..self.cycles_per_frame).step_by(1){

            if !self.cpu.halt{
                self.execute_opcode();
            }

            self.register_handler.update_registers_state(&mut self.mmu, &mut self.cpu, &mut self.ppu, &mut self.interrupts_handler, joypad, 1);
            //passing in the cycles 1 but in the future when Ill have a cycle accureate cpu ill pass the cycles passed since last time
            self.ppu.update_gb_screen(&self.mmu, 1);
            self.interrupts_handler.handle_interrupts(&mut self.cpu, &mut self.mmu);
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

        //debug
        if self.mmu.finished_boot{
            let a = *self.cpu.af.high();
            let b = *self.cpu.bc.high(); 
            let c = *self.cpu.bc.low();
            let d = *self.cpu.de.high();
            let e = *self.cpu.de.low();
            let f = *self.cpu.af.low();
            let h = *self.cpu.hl.high();
            let l = *self.cpu.hl.low();
            let ly = self.mmu.io_ports.read(0x44);
            info!("A:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} F:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} ({:02X} {:02X} {:02X} {:02X}) LY:{:02X} CS: ({:02X}{:02X} {:02X}{:02X} {:02X}{:02X}) SE {}",
            a, b,c,d,e,f,
            h,l, self.cpu.stack_pointer, pc, self.mmu.read(pc), self.mmu.read(pc+1), self.mmu.read(pc+2), self.mmu.read(pc+3), ly,
            self.mmu.read(self.cpu.stack_pointer+1),self.mmu.read(self.cpu.stack_pointer),self.mmu.read(self.cpu.stack_pointer+3),self.mmu.read(self.cpu.stack_pointer+2),self.mmu.read(self.cpu.stack_pointer+5),self.mmu.read(self.cpu.stack_pointer+4),
            self.ppu.screen_enable);
        }
        
        let opcode_func:OpcodeFuncType = self.opcode_resolver.get_opcode(opcode, &self.mmu, &mut self.cpu.program_counter);
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

