use crate::cpu::gb_cpu::GbCpu;
use crate::keypad::joypad::Joypad;
use crate::keypad::joypad_provider::JoypadProvider;
use crate::mmu::memory::Memory;
use crate::mmu::gb_mmu::{
    GbMmu,
    BOOT_ROM_SIZE
};
use crate::cpu::opcodes::opcode_resolver::*;
use crate::ppu::gb_ppu::GbPpu;
use crate::machine::registers_handler::RegisterHandler;
use crate::mmu::carts::mbc::Mbc;
use crate::ppu::gb_ppu::{
    SCREEN_HEIGHT,
    SCREEN_WIDTH,
    CYCLES_PER_FRAME
};
use super::interrupts_handler::InterruptsHandler;
use std::boxed::Box;
use log::debug;


pub struct GameBoy<'a> {
    cpu: GbCpu,
    mmu: GbMmu::<'a>,
    opcode_resolver:OpcodeResolver,
    ppu:GbPpu,
    register_handler:RegisterHandler,
    interrupts_handler:InterruptsHandler,
    cycles_counter:u32
}

impl<'a> GameBoy<'a>{

    pub fn new_with_bootrom(mbc:&'a mut Box<dyn Mbc>, boot_rom:[u8;BOOT_ROM_SIZE])->GameBoy{
        GameBoy{
            cpu:GbCpu::default(),
            mmu:GbMmu::new_with_bootrom(mbc, boot_rom),
            opcode_resolver:OpcodeResolver::default(),
            ppu:GbPpu::default(),
            register_handler: RegisterHandler::default(),
            interrupts_handler: InterruptsHandler::default(),
            cycles_counter:0
        }
    }

    pub fn new(mbc:&'a mut Box<dyn Mbc>)->GameBoy{
        let mut cpu = GbCpu::default();
        //Values after the bootrom
        *cpu.af.value() = 0x190;
        *cpu.bc.value() = 0x13;
        *cpu.de.value() = 0xD8;
        *cpu.hl.value() = 0x14D;
        cpu.stack_pointer = 0xFFFE;
        cpu.program_counter = 0x100;

        GameBoy{
            cpu:cpu,
            mmu:GbMmu::new(mbc),
            opcode_resolver:OpcodeResolver::default(),
            ppu:GbPpu::default(),
            register_handler: RegisterHandler::default(),
            interrupts_handler: InterruptsHandler::default(),
            cycles_counter:0
        }
    }

    pub fn cycle_frame(&mut self, mut joypad_provider:impl JoypadProvider )->&[u32;SCREEN_HEIGHT*SCREEN_WIDTH]{
        let mut joypad = Joypad::default();

        let mut last_ppu_power_state:bool = self.ppu.screen_enable;

        while self.cycles_counter < CYCLES_PER_FRAME{
            joypad_provider.provide(&mut joypad);

            //CPU
            let mut cpu_cycles_passed = 1;
            if !self.cpu.halt{
                cpu_cycles_passed = self.execute_opcode();
            }

            //interrupts
            //updating the registers aftrer the CPU
            self.register_handler.update_registers_state(&mut self.mmu, &mut self.cpu, &mut self.ppu, &mut self.interrupts_handler, &joypad, cpu_cycles_passed);
            let interrupt_cycles = self.interrupts_handler.handle_interrupts(&mut self.cpu, &mut self.mmu);
            if interrupt_cycles != 0{
                //updating the register after the interrupts (for timing)
                self.register_handler.update_registers_state(&mut self.mmu, &mut self.cpu, &mut self.ppu, &mut self.interrupts_handler, &joypad, interrupt_cycles);
            }
            
            //PPU
            let iter_total_cycles= cpu_cycles_passed as u32 + interrupt_cycles as u32;
            self.ppu.update_gb_screen(&self.mmu, iter_total_cycles);
            //updating after the PPU
            self.register_handler.update_registers_state(&mut self.mmu, &mut self.cpu, &mut self.ppu, &mut self.interrupts_handler, &joypad, 0);

            if !last_ppu_power_state && self.ppu.screen_enable{
                self.cycles_counter = 0;
            }

            self.cycles_counter += iter_total_cycles;
            last_ppu_power_state = self.ppu.screen_enable;
        }

        if self.cycles_counter >= CYCLES_PER_FRAME{
            self.cycles_counter -= CYCLES_PER_FRAME; 
        }

        return self.ppu.get_frame_buffer();
    }

    fn fetch_next_byte(&mut self)->u8{
        let byte:u8 = self.mmu.read(self.cpu.program_counter);
        self.cpu.program_counter+=1;
        return byte;
    }

    fn execute_opcode(&mut self)->u8{
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
            debug!("A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X})",
            a,f,b,c,d,e,h,l, self.cpu.stack_pointer, pc, self.mmu.read(pc), self.mmu.read(pc+1), self.mmu.read(pc+2), self.mmu.read(pc+3));
        }
        
        let opcode_func:OpcodeFuncType = self.opcode_resolver.get_opcode(opcode, &self.mmu, &mut self.cpu.program_counter);
        match opcode_func{
            OpcodeFuncType::OpcodeFunc(func)=>func(&mut self.cpu),
            OpcodeFuncType::MemoryOpcodeFunc(func)=>func(&mut self.cpu, &mut self.mmu),
            OpcodeFuncType::U8OpcodeFunc(func)=>func(&mut self.cpu, opcode),
            OpcodeFuncType::U8MemoryOpcodeFunc(func)=>func(&mut self.cpu, &mut self.mmu, opcode),
            OpcodeFuncType::U16OpcodeFunc(func)=>{
                let u16_opcode:u16 = ((opcode as u16)<<8) | (self.fetch_next_byte() as u16);
                func(&mut self.cpu, u16_opcode)
            },
            OpcodeFuncType::U16MemoryOpcodeFunc(func)=>{
                let u16_opcode:u16 = ((opcode as u16)<<8) | (self.fetch_next_byte() as u16);
                func(&mut self.cpu, &mut self.mmu, u16_opcode)
            },
            OpcodeFuncType::U32OpcodeFunc(func)=>{
                let mut u32_opcode:u32 = ((opcode as u32)<<8) | (self.fetch_next_byte() as u32);
                u32_opcode <<= 8;
                u32_opcode |= self.fetch_next_byte() as u32;
                func(&mut self.cpu, u32_opcode)
            },
            OpcodeFuncType::U32MemoryOpcodeFunc(func)=>{
                let mut u32_opcode:u32 = ((opcode as u32)<<8) | (self.fetch_next_byte() as u32);
                u32_opcode <<= 8;
                u32_opcode |= self.fetch_next_byte() as u32;
                func(&mut self.cpu, &mut self.mmu, u32_opcode)
            }
        }
    }
}

