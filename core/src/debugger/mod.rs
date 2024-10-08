mod disassembler;

use std::collections::HashSet;

use crate::{*, machine::gameboy::*, mmu::Memory, cpu::gb_cpu::GbCpu, utils::vec2::Vec2, ppu::{ppu_state::PpuState, gb_ppu::GbPpu}};
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
    Registers,
    Break(u16),
    RemoveBreak(u16),
    DumpMemory(u16),
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
    HitBreak(u16),
    RemovedBreak(u16),
    BreakDoNotExist(u16),
    Continuing,
    Stepped(u16),
    Stopped(u16),
    MemoryDump(u16, Vec<MemoryEntry>),
    Disassembly(u16, Vec<OpcodeEntry>),
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
    pub sp:u16
}

impl Registers{
    fn new(cpu:&GbCpu)->Self{
        Registers { af: cpu.af.value(), bc: cpu.bc.value(), de: cpu.de.value(), hl: cpu.hl.value(), pc: cpu.program_counter, sp: cpu.stack_pointer }
    }
}

#[derive(Default, Clone, Copy)]
pub struct MemoryEntry{
    pub address:u16,
    pub value:u8
}

pub struct PpuInfo{
    pub ppu_state:PpuState,
    pub lcdc:u8,
    pub stat:u8,
    pub ly:u8,
    pub window_pos:Vec2<u8>,
    pub background_pos:Vec2<u8>
}

impl PpuInfo{
    fn new<GFX:GfxDevice>(ppu:&GbPpu<GFX>)->Self{
        Self { 
            ppu_state: ppu.state, lcdc: ppu.lcd_control, stat: ppu.stat_register, 
            ly: ppu.ly_register, window_pos: ppu.window_pos, background_pos: ppu.bg_pos 
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
    breakpoints:HashSet<u16>
}

impl<UI:DebuggerInterface> Debugger<UI>{
    pub fn new(ui:UI)->Self{
        Self { ui, breakpoints: HashSet::new() }
    }

    fn recv(&self)->DebuggerCommand{self.ui.recv_command()}
    fn send(&self, result: DebuggerResult){self.ui.send_result(result)}

    fn should_halt(&self, pc:u16, hit_watch:bool)->bool{
        self.check_for_break(pc) || self.ui.should_stop() || hit_watch
    }

    fn check_for_break(&self, pc:u16)->bool{self.breakpoints.contains(&pc)}

    fn add_breakpoint(&mut self, address:u16){_ = self.breakpoints.insert(address)}

    fn try_remove_breakpoint(&mut self, address:u16)->bool{self.breakpoints.remove(&address)}
}

impl_gameboy!{{
    pub fn run_debugger(&mut self){
        while self.debugger.should_halt(self.cpu.program_counter, self.mmu.mem_watch.hit_addr.is_some()) {
            if self.debugger.check_for_break(self.cpu.program_counter){
                self.debugger.send(DebuggerResult::HitBreak(self.cpu.program_counter));
            }
            if let Some(addr) = self.mmu.mem_watch.hit_addr{
                self.debugger.send(DebuggerResult::HitWatch(addr, self.cpu.program_counter));
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
                DebuggerCommand::RemoveBreak(address)=>{
                    let result = match self.debugger.try_remove_breakpoint(address) {
                        true => DebuggerResult::RemovedBreak(address),
                        false => DebuggerResult::BreakDoNotExist(address)
                    };
                    self.debugger.send(result);
                },
                DebuggerCommand::DumpMemory(len)=>{
                    let mut buffer = vec![MemoryEntry::default(); len as usize];
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
                    let result = disassemble(&self.cpu, &mut self.mmu, len);
                    self.debugger.send(DebuggerResult::Disassembly(len, result));
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
}}

pub struct MemoryWatcher{
    pub watching_addresses: HashSet<u16>,
    pub hit_addr:Option<u16>,
}

impl MemoryWatcher{
    pub fn new()->Self{Self { watching_addresses: HashSet::new(), hit_addr: None }}
    pub fn add_address(&mut self, address:u16){_ = self.watching_addresses.insert(address)}
    pub fn try_remove_address(&mut self, address:u16)->bool{self.watching_addresses.remove(&address)}
}