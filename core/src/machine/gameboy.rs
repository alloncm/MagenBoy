use crate::{
    apu::{audio_device::AudioDevice, gb_apu::GbApu},
    cpu::gb_cpu::GbCpu,
    mmu::{carts::Mbc, gb_mmu::GbMmu, memory::Memory, external_memory_bus::Bootrom}, 
    ppu::gfx_device::GfxDevice, keypad::joypad_provider::JoypadProvider
};
use super::Mode;
#[cfg(feature = "dbg")]
use super::debugger::*;

//CPU frequrncy: 4,194,304 / 59.727~ / 4 == 70224 / 4
pub const CYCLES_PER_FRAME:u32 = 17556;

pub struct GameBoy<'a, 
    JP: JoypadProvider, 
    AD:AudioDevice, 
    GFX:GfxDevice, 
    #[cfg(feature = "dbg")] DUI:DebuggerUi> 
{
    cpu: GbCpu,
    mmu: GbMmu::<'a, AD, GFX, JP>,
    #[cfg(feature = "dbg")]
    debugger:Debugger<DUI>
}


// https://stackoverflow.com/questions/72955038/varying-number-of-generic-parameters-based-on-a-feature
macro_rules! impl_gameboy {
    ($implementations:tt) => {
        #[cfg(feature = "dbg")]
        impl<'a, JP:JoypadProvider, AD:AudioDevice, GFX:GfxDevice, DUI:DebuggerUi> GameBoy<'a, JP, AD, GFX, DUI> $implementations
        #[cfg(not(feature = "dbg"))]
        impl<'a, JP:JoypadProvider, AD:AudioDevice, GFX:GfxDevice> GameBoy<'a, JP, AD, GFX> $implementations
    };
}
impl_gameboy! {{
    pub fn new(mbc:&'a mut dyn Mbc, joypad_provider:JP, audio_device:AD, gfx_device:GFX, #[cfg(feature = "dbg")]dui:DUI, boot_rom:Bootrom, mode:Option<Mode>)->Self{
        let mode = mode.unwrap_or(mbc.get_compatibility_mode().into());
        
        let mut cpu = GbCpu::default();
        if boot_rom == Bootrom::None{
            //Values after the bootrom
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
        }
        else {
            // Make sure that the mode and bootrom are compatible
            match mode{
                Mode::DMG => {let Bootrom::Gb(_) = boot_rom else {core::panic!("Bootrom doesnt match mode DMG")};}
                Mode::CGB => {let Bootrom::Gbc(_) = boot_rom else {core::panic!("Bootrom doesnt match mode CGB")};}
            }
        }

        GameBoy{
            cpu:cpu,
            mmu:GbMmu::new(mbc, boot_rom, GbApu::new(audio_device), gfx_device, joypad_provider, mode),
            #[cfg(feature = "dbg")]
            debugger: Debugger::new(dui),
        }
    }

    pub fn cycle_frame(&mut self){
        while self.mmu.m_cycle_counter < CYCLES_PER_FRAME{
            #[cfg(feature = "dbg")]
            self.handle_debugger();
            self.step();
        }

        self.mmu.m_cycle_counter = 0;
    }

    #[cfg(feature = "dbg")]
    fn handle_debugger(&mut self) {
        while self.debugger.should_halt(self.cpu.program_counter, self.mmu.mem_watch.hit_addr.is_some()) {
            if self.debugger.check_for_break(self.cpu.program_counter){
                self.debugger.send(DebuggerResult::HitBreak(self.cpu.program_counter));
            }
            if let Some(addr) = self.mmu.mem_watch.hit_addr{
                self.debugger.send(DebuggerResult::HitWatchPoint(addr, self.cpu.program_counter));
                self.mmu.mem_watch.hit_addr = None;
            }
            match self.debugger.recv(){
                DebuggerCommand::Stop=>self.debugger.send(DebuggerResult::Stopped(self.cpu.program_counter)),
                DebuggerCommand::Step=>{
                    self.step();
                    self.debugger.send(DebuggerResult::Stepped(self.cpu.program_counter));
                }
                DebuggerCommand::Continue=>{
                    self.debugger.send(DebuggerResult::Continuing);
                    break;
                }
                DebuggerCommand::Registers => self.debugger.send(DebuggerResult::Registers(Registers::new(&self.cpu))),
                DebuggerCommand::Break(address) => {
                    self.debugger.add_breakpoint(address);
                    self.debugger.send(DebuggerResult::AddedBreak(address));
                },
                DebuggerCommand::DeleteBreak(address)=>{
                    let result = match self.debugger.try_remove_breakpoint(address) {
                        true => DebuggerResult::RemovedBreak(address),
                        false => DebuggerResult::BreakDoNotExist(address)
                    };
                    self.debugger.send(result);
                },
                DebuggerCommand::DumpMemory(len)=>{
                    let mut buffer = [MemoryEntry{address:0, value:0};0xFF];
                    for i in 0..len as usize{
                        let address = self.cpu.program_counter + i as u16;
                        buffer[i] = MemoryEntry {
                            value: self.mmu.read(address, 0),
                            address,
                        };
                    }
                    self.debugger.send(DebuggerResult::MemoryDump(len, buffer));
                }
                DebuggerCommand::Disassemble(len)=>{
                    let result = crate::cpu::disassembler::disassemble(&self.cpu, &mut self.mmu, len);
                    self.debugger.send(DebuggerResult::Disassembly(len, result));
                },
                DebuggerCommand::AddWatchPoint(address)=>{
                    self.mmu.mem_watch.add_address(address);
                    self.debugger.send(DebuggerResult::SetWatchPoint(address));
                },
                DebuggerCommand::RemoveWatch(address)=>{
                    match self.mmu.mem_watch.try_remove_address(address){
                        true=>self.debugger.send(DebuggerResult::RemovedWatch(address)),
                        false=>self.debugger.send(DebuggerResult::WatchDonotExist(address)),
                    }
                }
            }
        }
    }

    fn step(&mut self) {
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