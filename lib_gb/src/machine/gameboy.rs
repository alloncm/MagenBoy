use crate::{cpu::{gb_cpu::GbCpu}, mmu::{self, mmu_register_updater}, ppu::ppu_register_updater, timer::{gb_timer::GbTimer, timer_register_updater}};
use crate::keypad::joypad::Joypad;
use crate::keypad::joypad_provider::JoypadProvider;
use crate::mmu::memory::Memory;
use crate::mmu::gb_mmu::{
    GbMmu,
    BOOT_ROM_SIZE
};
use crate::cpu::opcodes::opcode_resolver::*;
use crate::ppu::gb_ppu::GbPpu;
use crate::mmu::carts::mbc::Mbc;
use crate::ppu::gb_ppu::{
    SCREEN_HEIGHT,
    SCREEN_WIDTH,
    CYCLES_PER_FRAME
};
use super::interrupts_handler::InterruptsHandler;
use std::boxed::Box;
use log::debug;
use mmu::oam_dma_transferer::OamDmaTransferer;


pub struct GameBoy<'a, JP: JoypadProvider> {
    cpu: GbCpu,
    mmu: GbMmu::<'a>,
    opcode_resolver:OpcodeResolver::<GbMmu::<'a>>,
    ppu:GbPpu,
    interrupts_handler:InterruptsHandler,
    cycles_counter:u32, 
    joypad_provider: JP,
    timer: GbTimer,
    dma:OamDmaTransferer
}

impl<'a, JP:JoypadProvider> GameBoy<'a, JP>{

    pub fn new_with_bootrom(mbc:&'a mut Box<dyn Mbc>,joypad_provider:JP, boot_rom:[u8;BOOT_ROM_SIZE])->GameBoy<JP>{
        GameBoy{
            cpu:GbCpu::default(),
            mmu:GbMmu::new_with_bootrom(mbc, boot_rom),
            opcode_resolver:OpcodeResolver::default(),
            ppu:GbPpu::default(),
            interrupts_handler: InterruptsHandler::default(),
            cycles_counter:0,
            joypad_provider: joypad_provider,
            timer:GbTimer::default(),
            dma: OamDmaTransferer::default()
        }
    }

    pub fn new(mbc:&'a mut Box<dyn Mbc>,joypad_provider:JP)->GameBoy<JP>{
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
            interrupts_handler: InterruptsHandler::default(),
            cycles_counter:0,
            joypad_provider: joypad_provider,
            timer: GbTimer::default(),
            dma: OamDmaTransferer::default()
        }
    }

    pub fn cycle_frame(&mut self)->&[u32;SCREEN_HEIGHT*SCREEN_WIDTH]{
        let mut joypad = Joypad::default();

        let mut last_ppu_power_state:bool = self.ppu.screen_enable;

        while self.cycles_counter < CYCLES_PER_FRAME{
            self.joypad_provider.provide(&mut joypad);

            //CPU
            let mut cpu_cycles_passed = 1;
            if !self.cpu.halt{
                cpu_cycles_passed = self.execute_opcode();
            }

            timer_register_updater::update_timer_registers(&mut self.timer, &mut self.mmu.io_ports);
            self.timer.cycle(&mut self.mmu, cpu_cycles_passed);
            self.dma.cycle(&mut self.mmu, cpu_cycles_passed as u8);
            
            mmu_register_updater::update_mmu_registers(&mut self.mmu, &mut self.dma);

            ppu_register_updater::update_ppu_regsiters(&mut self.mmu, &mut self.ppu);
            
            //interrupts
            let interrupt_cycles = self.interrupts_handler.handle_interrupts(&mut self.cpu, &mut self.ppu, &mut self.mmu);
            if interrupt_cycles != 0{
                self.dma.cycle(&mut self.mmu, interrupt_cycles as u8);
                timer_register_updater::update_timer_registers(&mut self.timer, &mut self.mmu.io_ports);
                self.timer.cycle(&mut self.mmu, interrupt_cycles as u8);

                mmu_register_updater::update_mmu_registers(&mut self.mmu, &mut self.dma);
            }

            
            let iter_total_cycles= cpu_cycles_passed as u32 + interrupt_cycles as u32;
            
            //PPU
            ppu_register_updater::update_ppu_regsiters(&mut self.mmu, &mut self.ppu);
            self.ppu.update_gb_screen(&mut self.mmu, iter_total_cycles);
            mmu_register_updater::update_mmu_registers(&mut self.mmu, &mut self.dma);

            //In case the ppu just turned I want to keep it sync with the actual screen and thats why Im reseting the loop to finish
            //the frame when the ppu finishes the frame
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
        
        let opcode_func:OpcodeFuncType<GbMmu> = self.opcode_resolver.get_opcode(opcode, &self.mmu, &mut self.cpu.program_counter);
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

