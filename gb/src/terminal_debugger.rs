use std::{sync::atomic::{AtomicBool, Ordering}, io::stdin};

use crossbeam_channel::{bounded, Sender, Receiver};
use lib_gb::{machine::debugger::{DebuggerUi, DebuggerCommand, DebuggerResult}};

static ENABLE_FLAG: AtomicBool = AtomicBool::new(false);

pub struct TerminalDebugger{
    command_receiver:Receiver<DebuggerCommand>,
    result_sender:Sender<DebuggerResult>
}

impl TerminalDebugger{
    pub fn new()->Self{
        let (command_sender, command_receiver) = bounded(0);
        let (result_sender, result_receiver) = bounded(0);
        std::thread::spawn(move || Self::io_loop(command_sender, result_receiver));
        Self{command_receiver, result_sender}
    }

    fn io_loop(sender:Sender<DebuggerCommand>, receiver:Receiver<DebuggerResult>){
        let send_command = |command:DebuggerCommand|->DebuggerResult{
            sender.send(command).unwrap();
            return receiver.recv().unwrap();
        };
        loop{
            let mut buffer = String::new();
            stdin().read_line(&mut buffer).unwrap();
            let buffer:Vec<&str> = buffer.trim().split(' ').collect();
            match buffer[0]{
                "h"=>{
                    ENABLE_FLAG.store(true, Ordering::SeqCst);
                    match send_command(DebuggerCommand::Stop) {
                        DebuggerResult::Address(adrr) =>println!("Stopped\n->0x{:X}", adrr),
                        _=>std::panic!("Wrong debugger result")
                    }
                }
                "c" if ENABLE_FLAG.load(Ordering::SeqCst)=>{
                    ENABLE_FLAG.store(false, Ordering::SeqCst);
                    match send_command(DebuggerCommand::Continue) {
                        DebuggerResult::Success => {}
                        _=>std::panic!("Wrong debugger result")
                    }
                }
                "s"=>{
                    match send_command(DebuggerCommand::Step) {
                        DebuggerResult::Address(adrr) =>println!("->0x{:X}", adrr),
                        _=>std::panic!("Wrong debugger result"),
                    }
                }
                "b"=>{
                    let Some(param) = buffer.get(1) else {
                        println!("Error setting breakpoint, you must provide an address");
                        continue;
                    };
                    let Some(hex_str) = param.strip_prefix("0x") else{
                        println!("Error setting breakpoint, param {} must start with 0x", param);
                        continue;
                    };
                    let Ok(address) = u16::from_str_radix(hex_str, 16) else {
                        println!("Error setting breakpoint, param {} is not a valid address", param);
                        continue;
                    };
                    match send_command(DebuggerCommand::Break(address)){
                        DebuggerResult::Success => println!("Set break point at: 0x{:X}", address),
                        _=> todo!(),
                    }
                },
                "r"=>{
                    match send_command(DebuggerCommand::Registers){
                        DebuggerResult::Registers(regs)=>
                            println!("AF: 0x{:X}\nBC: 0x{:X}\nDE: 0x{:X}\nHL: 0x{:X}\nSP: 0x{:X}\nPC: 0x{:X}",
                            regs.af, regs.bc, regs.de, regs.hl, regs.sp, regs.pc),
                        _=>std::panic!("Wrong debuger results")
                    }
                }
                _=>println!("invalid input")
            }
        }
    }

    fn enabled()->bool{ENABLE_FLAG.load(Ordering::SeqCst)}
}

impl DebuggerUi for TerminalDebugger{
    fn stop(&self)->bool {Self::enabled()}

    fn recv_command(&self)->DebuggerCommand {
        self.command_receiver.recv().unwrap()
    }

    fn send_result(&self, result:DebuggerResult) {
        self.result_sender.send(result).unwrap()
    }
}