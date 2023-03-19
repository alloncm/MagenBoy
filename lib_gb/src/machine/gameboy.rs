use crate::{
    apu::{audio_device::AudioDevice, gb_apu::GbApu},
    cpu::gb_cpu::GbCpu,
    mmu::{carts::Mbc, gb_mmu::GbMmu, memory::Memory, external_memory_bus::Bootrom}, 
    ppu::gfx_device::GfxDevice, keypad::joypad_provider::JoypadProvider
};
use super::Mode;

//CPU frequrncy: 4,194,304 / 59.727~ / 4 == 70224 / 4
pub const CYCLES_PER_FRAME:u32 = 17556;

pub struct GameBoy<'a, JP: JoypadProvider, AD:AudioDevice, GFX:GfxDevice> {
    cpu: GbCpu,
    mmu: GbMmu::<'a, AD, GFX, JP>
}

impl<'a, JP:JoypadProvider, AD:AudioDevice, GFX:GfxDevice> GameBoy<'a, JP, AD, GFX>{
    pub fn new(mbc:&'a mut Box<dyn Mbc>,joypad_provider:JP, audio_device:AD, gfx_device:GFX, boot_rom:Bootrom, mode:Option<Mode>)->GameBoy<JP, AD, GFX>{
        let mode = mode.unwrap_or_else(||mbc.get_compatibility_mode().into());
        
        let mut cpu = GbCpu::default();
        if boot_rom == Bootrom::None{
            //Values after the bootrom
            match mode{
                Mode::DMG=>{
                    *cpu.af.value() = 0x190;
                    *cpu.bc.value() = 0x13;
                    *cpu.de.value() = 0xD8;
                    *cpu.hl.value() = 0x14D;
                },
                Mode::CGB=>{
                    *cpu.af.value() = 0x1180;
                    *cpu.bc.value() = 0x0;
                    *cpu.de.value() = 0xFF56;
                    *cpu.hl.value() = 0xD;
                }
            }
            cpu.stack_pointer = 0xFFFE;
            cpu.program_counter = 0x100;
        }
        else {
            // Make sure that the mode and bootrom are compatible
            match mode{
                Mode::DMG => {let Bootrom::Gb(_) = boot_rom else {std::panic!("Bootrom doesnt match mode DMG")};}
                Mode::CGB => {let Bootrom::Gbc(_) = boot_rom else {std::panic!("Bootrom doesnt match mode CGB")};}
            }
        }

        GameBoy{
            cpu:cpu,
            mmu:GbMmu::new(mbc, boot_rom, GbApu::new(audio_device), gfx_device, joypad_provider, mode),
        }
    }

    pub fn cycle_frame(&mut self){
        while self.mmu.m_cycle_counter < CYCLES_PER_FRAME{
            self.mmu.poll_joypad_state();

            //CPU
            let mut cpu_cycles_passed = 1;
            if !self.cpu.halt && !self.mmu.dma_block_cpu(){
                cpu_cycles_passed = self.execute_opcode();
            }
            if cpu_cycles_passed != 0{
                self.mmu.cycle(cpu_cycles_passed);
            }
            
            //interrupts
            let interrupt_request = self.mmu.handle_interrupts(self.cpu.mie);
            let interrupt_cycles = self.cpu.execute_interrupt_request(&mut self.mmu, interrupt_request);
            if interrupt_cycles != 0{
                self.mmu.cycle(interrupt_cycles);
            }
        }

        self.mmu.m_cycle_counter = 0;
    }

    fn execute_opcode(&mut self)->u8{
        let pc = self.cpu.program_counter;

        log::trace!("A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X})",
            {*self.cpu.af.high()}, *self.cpu.af.low(),
            {*self.cpu.bc.high()}, *self.cpu.bc.low(),
            {*self.cpu.de.high()}, *self.cpu.de.low(),
            {*self.cpu.hl.high()}, *self.cpu.hl.low(),
            self.cpu.stack_pointer, pc,
            self.mmu.read(pc,0), self.mmu.read(pc+1,0), self.mmu.read(pc+2,0), self.mmu.read(pc+3,0)
        );
    
        self.cpu.run_opcode(&mut self.mmu)
    }
}