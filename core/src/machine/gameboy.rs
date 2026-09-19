use crate::{apu::gb_apu::GbApu, cpu::gb_cpu::GbCpu, keypad::joypad::Joypad, mmu::{carts::Mbc, external_memory_bus::Bootrom, gb_mmu::GbMmu, Memory}, *};
use super::Mode;
#[cfg(feature = "dbg")]
use crate::debugger::*;

pub struct GameBoy<'a, AD:AudioDevice, #[cfg(feature = "dbg")] DI:DebuggerInterface>{
    pub(crate) cpu: GbCpu,       
    pub(crate) mmu:GbMmu<'a, AD>,
    #[cfg(feature = "dbg")] pub(crate) debugger:Debugger<DI>
}

// https://stackoverflow.com/questions/72955038/varying-number-of-generic-parameters-based-on-a-feature
// In order to conditionally add the debugger based on the dbg feature Im having this macro
macro_rules! impl_gameboy {
    ($implementations:tt) => {
        #[cfg(feature = "dbg")]
        impl<'a, AD:AudioDevice, DUI:DebuggerInterface> GameBoy<'a, AD, DUI> $implementations
        #[cfg(not(feature = "dbg"))]
        impl<'a, AD:AudioDevice> GameBoy<'a, AD> $implementations
    };
}
pub(crate) use impl_gameboy;

impl_gameboy! {{
    pub fn new_with_mode(mbc:&'a mut dyn Mbc, audio_device:AD, mode:Mode, #[cfg(feature = "dbg")]dui:DUI)->Self{
        let mut cpu = GbCpu::default();
        match mode{
            Mode::DMG=>{
                *cpu.af.value_mut() = 0x190;
                *cpu.bc.value_mut() = 0x13;
                *cpu.de.value_mut() = 0xD8;
                *cpu.hl.value_mut() = 0x14D;
            },
            Mode::CGB=>{
                *cpu.af.value_mut() = 0x1180;
                *cpu.bc.value_mut() = 0x0;
                *cpu.de.value_mut() = 0xFF56;
                *cpu.hl.value_mut() = 0xD;
            }
        }
        cpu.stack_pointer = 0xFFFE;
        cpu.program_counter = 0x100;

        return Self{
            cpu: cpu,
            mmu: GbMmu::new(mbc, None, GbApu::new(audio_device), mode),
            #[cfg(feature = "dbg")]
            debugger: Debugger::new(dui),
        };
    }

    pub fn new_with_bootrom(mbc:&'a mut dyn Mbc, audio_device:AD, bootrom:Bootrom, #[cfg(feature = "dbg")]dui:DUI)->Self{
        let mode = match bootrom{
            Bootrom::Gb(_) => Mode::DMG,
            Bootrom::Gbc(_) => Mode::CGB
        };

        return Self{
            cpu: GbCpu::default(),
            mmu: GbMmu::new(mbc, Some(bootrom), GbApu::new(audio_device), mode),
            #[cfg(feature = "dbg")]
            debugger: Debugger::new(dui),
        };
    }

    pub fn cycle_frame(&mut self, joypad: Joypad) -> &FrameBuffer {
        self.mmu.set_joypad_state(joypad);

        while !self.mmu.consume_vblank_event() {
            #[cfg(feature = "dbg")]
            self.run_debugger();
            self.step();
        }

        return self.mmu.get_frame_buffer();
    }

    pub(crate) fn step(&mut self) {
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
}}