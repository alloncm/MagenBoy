use std::{sync::atomic::{AtomicBool, Ordering}, io::stdin, thread};

use crossbeam_channel::{bounded, Sender, Receiver};
use lib_gb::{machine::debugger::{DebuggerUi, DebuggerCommand, DebuggerResult}};

static ENABLE_FLAG: AtomicBool = AtomicBool::new(false);

pub struct TerminalDebugger{
    command_receiver:Receiver<DebuggerCommand>,
    result_sender:Sender<DebuggerResult>,
}

impl TerminalDebugger{
    pub fn new()->Self{
        let (command_sender, command_receiver) = bounded(0);
        let (result_sender, result_receiver) = bounded(0);
        let (ternimal_input_sender, terminal_input_receiver) = bounded(0);
        thread::spawn(move || Self::get_string_loop(ternimal_input_sender));
        thread::spawn(move || Self::io_loop(command_sender, result_receiver, terminal_input_receiver));
        Self{command_receiver, result_sender}
    }

    fn get_string_loop(sender:Sender<String>){
        loop{
            let mut buffer = String::new();
            stdin().read_line(&mut buffer).unwrap();
            if !buffer.trim().is_empty(){
                sender.send(buffer).unwrap();
            }
        }
    }

    fn io_loop(sender:Sender<DebuggerCommand>, receiver:Receiver<DebuggerResult>, input_receiver:Receiver<String>){
        loop{
            crossbeam_channel::select! {
                recv(input_receiver)-> msg => {
                    let Ok(message) = msg else {break};
                    Self::handle_buffer(&sender, message);
                },
                recv(receiver)-> res =>{ 
                    let Ok(result) = res else {break};
                    Self::handle_debugger_result(result);
                },
            }
        }
        log::info!("Closing the debugger IO loop thread");
    }
    
    fn handle_debugger_result(result:DebuggerResult){
        match result{
            DebuggerResult::Stopped(addr) => println!("Stopped -> {:#X}", addr),
            DebuggerResult::Registers(regs) => println!("AF: 0x{:X}\nBC: 0x{:X}\nDE: 0x{:X}\nHL: 0x{:X}\nSP: 0x{:X}\nPC: 0x{:X}",
                                                            regs.af, regs.bc, regs.de, regs.hl, regs.sp, regs.pc),
            DebuggerResult::HitBreak(addr) =>{
                ENABLE_FLAG.store(true, Ordering::SeqCst);
                println!("Hit break: {:#X}", addr);
            }
            DebuggerResult::AddedBreak(addr)=>println!("Added BreakPoint succesfully at {:#X}", addr),
            DebuggerResult::Continuing=>println!("Contuning execution"),
            DebuggerResult::Stepped(addr)=>println!("-> {:#X}", addr),
            DebuggerResult::RemovedBreak(addr) => println!("Removed breakpoint succesfully at {:#X}", addr),
            DebuggerResult::BreakDoNotExist(addr) => println!("Breakpoint {:#X} does not exist", addr),
        }
    }
    
    fn handle_buffer(sender:&Sender<DebuggerCommand>, buffer: String) {
        let buffer:Vec<&str> = buffer.trim().split(' ').collect();
        match buffer[0]{
            "h"=>{
                ENABLE_FLAG.store(true, Ordering::SeqCst);
                sender.send(DebuggerCommand::Stop).unwrap();
            }
            "c" if ENABLE_FLAG.load(Ordering::SeqCst)=>{
                ENABLE_FLAG.store(false, Ordering::SeqCst);
                sender.send(DebuggerCommand::Continue).unwrap();
            }
            "s"=>sender.send(DebuggerCommand::Step).unwrap(),
            "b"=>match parse_address_string(&buffer) {
                Ok(address) => sender.send(DebuggerCommand::Break(address)).unwrap(),
                Err(msg) => println!("Error setting BreakPoint {}", msg),
            },
            "r"=>sender.send(DebuggerCommand::Registers).unwrap(),
            "d"=>match parse_address_string(&buffer) {
                Ok(address) => sender.send(DebuggerCommand::DeleteBreak(address)).unwrap(),
                Err(msg) => println!("Error deleting BreakPoint {}", msg),
            },
            _=>println!("invalid input: {}", buffer[0])
        }
    }

    fn enabled()->bool{ENABLE_FLAG.load(Ordering::SeqCst)}
}

fn parse_address_string(buffer: &Vec<&str>) -> Result<u16, String> {
    let Some(param) = buffer.get(1) else {
        return Result::Err(String::from("No parameter"));
    };
    let (str, base) = match param.strip_prefix("0x") {
        Some(hex_str) => (hex_str, 16),
        None => (*param, 10),
    };
    let Ok(address) = u16::from_str_radix(str, base) else {
        return Result::Err(format!("param: {} is not a valid address", str));
    };
    return Result::Ok(address);
}

impl DebuggerUi for TerminalDebugger{
    fn should_stop(&self)->bool {Self::enabled()}

    fn recv_command(&self)->DebuggerCommand {
        self.command_receiver.recv().unwrap()
    }

    fn send_result(&self, result:DebuggerResult) {
        self.result_sender.send(result).unwrap()
    }
}