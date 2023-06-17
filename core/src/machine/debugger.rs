use crate::{cpu::{gb_cpu::GbCpu, disassembler::OpcodeEntry}, utils::fixed_size_set::FixedSizeSet};

pub enum DebuggerCommand{
    Stop,
    Step,
    Continue,
    Registers,
    Break(u16),
    DeleteBreak(u16),
    DumpMemory(u8),
    Disassemble(u8),
    AddWatchPoint(u16),
    RemoveWatch(u16),
}

pub enum DebuggerResult{
    Registers(Registers),
    HitBreak(u16),
    AddedBreak(u16),
    RemovedBreak(u16),
    BreakDoNotExist(u16),
    Continuing,
    Stepped(u16),
    Stopped(u16),
    MemoryDump(u8, [MemoryEntry;0xFF]),
    Disassembly(u8, [OpcodeEntry;0xFF]),
    SetWatchPoint(u16),
    HitWatchPoint(u16, u16),
    RemovedWatch(u16),
    WatchDonotExist(u16),
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

#[derive(Clone, Copy)]
pub struct MemoryEntry{
    pub address:u16,
    pub value:u8
}

impl Registers{
    pub fn new(cpu:&GbCpu)->Self{
        Registers { af: cpu.af.value(), bc: cpu.bc.value(), de: cpu.de.value(), hl: cpu.hl.value(), pc: cpu.program_counter, sp: cpu.stack_pointer }
    }
}

pub enum DebuggerPush{
    Breakpoint(u16),
}

pub trait DebuggerUi{
    fn should_stop(&self)->bool;
    fn recv_command(&self)->DebuggerCommand;
    fn send_result(&self, result:DebuggerResult);
}

pub struct Debugger<UI:DebuggerUi>{
    ui:UI,
    breakpoints:FixedSizeSet<u16, 0xFF>
}

impl<UI:DebuggerUi> Debugger<UI>{
    pub fn new(ui:UI)->Self{
        Self { ui, breakpoints: FixedSizeSet::<u16, 0xFF>::new() }
    }

    pub fn recv(&self)->DebuggerCommand{self.ui.recv_command()}
    pub fn send(&self, result: DebuggerResult){self.ui.send_result(result)}

    pub fn should_halt(&self, pc:u16, hit_watch:bool)->bool{
        self.check_for_break(pc) || self.ui.should_stop() || hit_watch
    }

    pub fn check_for_break(&self, pc:u16)->bool{
        self.get_breakpoints().contains(&pc)
    }

    pub fn add_breakpoint(&mut self, address:u16){self.breakpoints.add(address)}

    pub fn try_remove_breakpoint(&mut self, address:u16)->bool{self.breakpoints.try_remove(address)}

    fn get_breakpoints(&self)->&[u16]{self.breakpoints.as_slice()}
}