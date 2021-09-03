use crate::{
    apu::{audio_device::AudioDevice, gb_apu::GbApu},
    cpu::gb_cpu::GbCpu, 
    keypad::{joypad::Joypad, joypad_provider::JoypadProvider, joypad_register_updater}, 
    mmu::{carts::mbc::Mbc, gb_mmu::{GbMmu, BOOT_ROM_SIZE}, memory::Memory}, 
    ppu::{gb_ppu::CYCLES_PER_FRAME, gfx_device::GfxDevice}
};
use super::interrupts_handler::InterruptsHandler;
use std::boxed::Box;
use log::debug;


pub struct GameBoy<'a, JP: JoypadProvider, AD:AudioDevice, GFX:GfxDevice> {
    cpu: GbCpu,
    mmu: GbMmu::<'a, AD, GFX>,
    interrupts_handler:InterruptsHandler,
    joypad_provider: JP
}

impl<'a, JP:JoypadProvider, AD:AudioDevice, GFX:GfxDevice> GameBoy<'a, JP, AD, GFX>{

    pub fn new_with_bootrom(mbc:&'a mut Box<dyn Mbc>,joypad_provider:JP, audio_device:AD, gfx_device:GFX, boot_rom:[u8;BOOT_ROM_SIZE])->GameBoy<JP, AD, GFX>{
        GameBoy{
            cpu:GbCpu::default(),
            mmu:GbMmu::new_with_bootrom(mbc, boot_rom, GbApu::new(audio_device), gfx_device),
            interrupts_handler: InterruptsHandler::default(),
            joypad_provider: joypad_provider
        }
    }

    pub fn new(mbc:&'a mut Box<dyn Mbc>,joypad_provider:JP, audio_device:AD, gfx_device:GFX)->GameBoy<JP, AD, GFX>{
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
            mmu:GbMmu::new(mbc, GbApu::new(audio_device), gfx_device),
            interrupts_handler: InterruptsHandler::default(),
            joypad_provider: joypad_provider,
        }
    }

    pub fn cycle_frame(&mut self){
        let mut joypad = Joypad::default();

        let mut cycles_counter = 0;

        while cycles_counter < CYCLES_PER_FRAME{
            self.joypad_provider.provide(&mut joypad);
            joypad_register_updater::update_joypad_registers(&joypad, &mut self.mmu);

            //CPU
            let mut cpu_cycles_passed = 1;
            if !self.cpu.halt{
                cpu_cycles_passed = self.execute_opcode();
            }
            
            self.mmu.cycle(cpu_cycles_passed);
            
            //interrupts
            let interrupt_cycles = self.interrupts_handler.handle_interrupts(&mut self.cpu, &mut self.mmu);
            if interrupt_cycles != 0{                
                self.mmu.cycle(interrupt_cycles);
            }

            cycles_counter += cpu_cycles_passed as u32 + interrupt_cycles as u32;
        }
    }

    fn execute_opcode(&mut self)->u8{
        let pc = self.cpu.program_counter;

        //debug
        if self.mmu.io_components.finished_boot{
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

        self.cpu.run_opcode(&mut self.mmu)
    }
}

