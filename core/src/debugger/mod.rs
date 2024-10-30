mod disassembler;

use std::collections::HashSet;

use crate::{*, machine::gameboy::*, cpu::gb_cpu::GbCpu, utils::vec2::Vec2, ppu::{ppu_state::PpuState, gb_ppu::GbPpu}};
use self::disassembler::{OpcodeEntry, disassemble};

#[derive(Clone, Copy)]
pub enum PpuLayer{
    Background,
    Window,
    Sprites
}

pub enum DebuggerCommand{
    Stop,
    Step,
    Continue,
    SkipHalt,
    Registers,
    Break(u16, u16),
    RemoveBreak(u16, u16),
    DumpMemory(u16, u16),
    Disassemble(u16),
    Watch(u16),
    RemoveWatch(u16),
    PpuInfo,
    GetPpuLayer(PpuLayer)
}

pub const PPU_BUFFER_WIDTH:usize = 0x100;
pub const PPU_BUFFER_HEIGHT:usize = 0x100;
pub const PPU_BUFFER_SIZE:usize = PPU_BUFFER_HEIGHT * PPU_BUFFER_WIDTH;

pub enum DebuggerResult{
    Registers(Registers),
    AddedBreak(u16),
    HitBreak(u16, u16),
    RemovedBreak(u16),
    BreakDoNotExist(u16),
    Continuing,
    HaltWakeup,
    Stepped(u16, u16),
    Stopped(u16, u16),
    MemoryDump(u16, u16, Vec<u8>),
    Disassembly(u16, u16, Vec<OpcodeEntry>),
    AddedWatch(u16),
    HitWatch(u16, u16),
    RemovedWatch(u16),
    WatchDoNotExist(u16),
    PpuInfo(PpuInfo),
    PpuLayer(PpuLayer, Box<[Pixel;PPU_BUFFER_SIZE]>)
}

#[derive(Clone, Copy)]
pub struct Registers{
    pub af:u16,
    pub bc:u16,
    pub de: u16,
    pub hl:u16,
    pub pc:u16,
    pub sp:u16,
    pub ime: bool
}

impl Registers{
    fn new(cpu:&GbCpu)->Self{
        Registers { af: cpu.af.value(), bc: cpu.bc.value(), de: cpu.de.value(), hl: cpu.hl.value(), pc: cpu.program_counter, sp: cpu.stack_pointer, ime: cpu.mie }
    }
}

pub struct PpuInfo{
    pub ppu_state:PpuState,
    pub lcdc:u8,
    pub stat:u8,
    pub ly:u8,
    pub window_pos:Vec2<u8>,
    pub background_pos:Vec2<u8>,
    pub vram_bank: u8
}

impl PpuInfo{
    fn new<GFX:GfxDevice>(ppu:&GbPpu<GFX>)->Self{
        Self { 
            ppu_state: ppu.state, lcdc: ppu.lcd_control, stat: ppu.stat_register, 
            ly: ppu.ly_register, window_pos: ppu.window_pos, background_pos: ppu.bg_pos,
            vram_bank: ppu.vram.get_bank_reg()
        }
    }
}

pub trait DebuggerInterface{
    fn should_stop(&self)->bool;
    fn recv_command(&self)->DebuggerCommand;
    fn send_result(&self, result:DebuggerResult);
}

pub struct Debugger<UI:DebuggerInterface>{
    ui:UI,
    breakpoints:HashSet<(u16, u16)>,
    skip_halt: bool
}

impl<UI:DebuggerInterface> Debugger<UI>{
    pub fn new(ui:UI)->Self{
        Self { ui, breakpoints: HashSet::new(), skip_halt: false }
    }

    fn recv(&self)->DebuggerCommand{self.ui.recv_command()}
    fn send(&self, result: DebuggerResult){self.ui.send_result(result)}

    fn should_halt(&self, pc:u16, bank:u16, hit_watch:bool)->bool{
        self.check_for_break(pc, bank) || self.ui.should_stop() || hit_watch
    }

    fn check_for_break(&self, pc:u16, bank:u16)->bool{self.breakpoints.contains(&(pc, bank))}

    fn add_breakpoint(&mut self, address:u16, bank:u16){_ = self.breakpoints.insert((address, bank))}

    fn try_remove_breakpoint(&mut self, address:u16, bank:u16)->bool{self.breakpoints.remove(&(address, bank))}
}

impl_gameboy!{{
    pub fn run_debugger(&mut self){
        while self.debugger.should_halt(self.cpu.program_counter, self.get_current_bank(self.cpu.program_counter), self.mmu.mem_watch.hit_addr.is_some()) && 
        !(self.cpu.halt && self.debugger.skip_halt) {
            if !self.cpu.halt && self.debugger.skip_halt{
                self.debugger.send(DebuggerResult::HaltWakeup);
                self.debugger.skip_halt = false;
            }
            if self.debugger.check_for_break(self.cpu.program_counter, self.get_current_bank(self.cpu.program_counter)){
                self.debugger.send(DebuggerResult::HitBreak(self.cpu.program_counter, self.get_current_bank(self.cpu.program_counter)));
            }
            if let Some(addr) = self.mmu.mem_watch.hit_addr{
                self.debugger.send(DebuggerResult::HitWatch(addr, self.cpu.program_counter));
                self.mmu.mem_watch.hit_addr = None;
            }
            match self.debugger.recv(){
                DebuggerCommand::Stop=>self.debugger.send(DebuggerResult::Stopped(self.cpu.program_counter, self.get_current_bank(self.cpu.program_counter))),
                DebuggerCommand::Step=>{
                    self.step();
                    self.debugger.send(DebuggerResult::Stepped(self.cpu.program_counter, self.get_current_bank(self.cpu.program_counter)));
                }
                DebuggerCommand::Continue=>{
                    self.debugger.send(DebuggerResult::Continuing);
                    break;
                },
                DebuggerCommand::SkipHalt => self.debugger.skip_halt = true,
                DebuggerCommand::Registers => self.debugger.send(DebuggerResult::Registers(Registers::new(&self.cpu))),
                DebuggerCommand::Break(address, bank) => {
                    self.debugger.add_breakpoint(address, bank);
                    self.debugger.send(DebuggerResult::AddedBreak(address));
                },
                DebuggerCommand::RemoveBreak(address, bank)=>{
                    let result = match self.debugger.try_remove_breakpoint(address, bank) {
                        true => DebuggerResult::RemovedBreak(address),
                        false => DebuggerResult::BreakDoNotExist(address)
                    };
                    self.debugger.send(result);
                },
                DebuggerCommand::DumpMemory(address, len)=>{
                    let mut buffer = vec![0; len as usize];
                    for i in 0..len {
                        buffer[i as usize] = self.mmu.dbg_read(address + i);
                    }

                    self.debugger.send(DebuggerResult::MemoryDump(address, self.get_current_bank(address), buffer));
                }
                DebuggerCommand::Disassemble(len)=>{
                    let result = disassemble(&self.cpu, &mut self.mmu, len);
                    self.debugger.send(DebuggerResult::Disassembly(len, self.get_current_bank(self.cpu.program_counter), result));
                },
                DebuggerCommand::Watch(address)=>{
                    self.mmu.mem_watch.add_address(address);
                    self.debugger.send(DebuggerResult::AddedWatch(address));
                },
                DebuggerCommand::RemoveWatch(address)=>{
                    match self.mmu.mem_watch.try_remove_address(address){
                        true=>self.debugger.send(DebuggerResult::RemovedWatch(address)),
                        false=>self.debugger.send(DebuggerResult::WatchDoNotExist(address)),
                    }
                },
                DebuggerCommand::PpuInfo=>self.debugger.send(DebuggerResult::PpuInfo(PpuInfo::new(self.mmu.get_ppu()))),
                DebuggerCommand::GetPpuLayer(layer)=>{
                    let buffer = self.mmu.get_ppu().get_layer(layer);
                    self.debugger.send(DebuggerResult::PpuLayer(layer, buffer));
                }
            }
        }
    }

    fn get_current_bank(&self, address:u16)->u16{
        return match address{
            0..=0x3FFF => 0,
            0x4000..=0x7FFF => self.mmu.mem_watch.current_rom_bank_number,
            0x8000..=0x9FFF => self.mmu.get_ppu().vram.get_bank_reg() as u16,
            0xC000..=0xFDFF => self.mmu.mem_watch.current_ram_bank_number as u16,
            _=>0
        };
    }
}}

pub struct MemoryWatcher{
    pub watching_addresses: HashSet<u16>,
    pub hit_addr:Option<u16>,
    pub current_rom_bank_number: u16,
    pub current_ram_bank_number: u8,
}

impl MemoryWatcher{
    pub fn new()->Self{Self { watching_addresses: HashSet::new(), hit_addr: None, current_rom_bank_number: 0, current_ram_bank_number: 0 }}
    pub fn add_address(&mut self, address:u16){_ = self.watching_addresses.insert(address)}
    pub fn try_remove_address(&mut self, address:u16)->bool{self.watching_addresses.remove(&address)}
}