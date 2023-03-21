use crate::cpu::gb_cpu::GbCpu;

pub enum DebuggerCommand{
    Stop,
    Step,
    Continue,
    Registers,
    Break(u16),
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
    pub fn new(cpu:&GbCpu)->Self{
        Registers { af: cpu.af.value(), bc: cpu.bc.value(), de: cpu.de.value(), hl: cpu.hl.value(), pc: cpu.program_counter, sp: cpu.stack_pointer }
    }
}

pub enum DebuggerResult{
    None,
    Address(u16),
    Registers(Registers)
}

pub trait DebuggerUi{
    fn stop(&self)->bool;
    fn recv_command(&self)->DebuggerCommand;
    fn send_result(&self, result:DebuggerResult);
}

pub struct Debugger<UI:DebuggerUi>{
    pub ui:UI,
    breakpoints:[u16;0xFF],
    breakpoints_size:usize
}

impl<UI:DebuggerUi> Debugger<UI>{
    pub fn new(ui:UI)->Self{
        Self { ui, breakpoints: [0;0xFF], breakpoints_size:0 }
    }

    pub fn should_break(&self, pc:u16)->bool{
        self.get_breakpoints().contains(&pc)
    }

    pub fn add_breakpoint(&mut self, pc:u16){
        self.breakpoints[self.breakpoints_size] = pc;
        self.breakpoints_size += 1;
    }

    pub fn breakable(&self)->bool{self.breakpoints_size != 0}

    fn get_breakpoints(&self)->&[u16]{
        &self.breakpoints[0..self.breakpoints_size]
    }
}