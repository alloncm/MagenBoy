use std::{io::stdin, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread};

use crossbeam_channel::{bounded, Sender, Receiver};

use magenboy_core::{debugger::{DebuggerCommand, DebuggerInterface, DebuggerResult, PpuLayer, PPU_BUFFER_SIZE}, Pixel};

const HELP_MESSAGE:&'static str = r"Debugger commands:
- halt(h) - start the debugging session (halt the program execution)
- continue(c) - continue program execution
- step(s) - step 1 instruction
- skip_halt - skip untill CPU is hanlted
- break(b) [address:bank] - set a break point
- remove_break(rb) [address:bank] - delete a breakpoint 
- registers(reg) - print the cpu registers state
- disassemble(di) [number_of_opcodes] - print the disassembly of the next opcodes
- dump(du) [address number_of_bytes] - print memory addresses values from current bank
- watch(w) [address:bank] - set a watch point
- remove_watch(rw) [address:bank] - delete a watch point
- ppu_info(pi) - print info about the ppu execution state
- ppu_layer(pl) [layer] - a debug window with one ppu layer (win, bg, spr)
";

pub struct PpuLayerResult(pub Box<[Pixel; PPU_BUFFER_SIZE]>, pub PpuLayer);

pub struct TerminalDebugger{
    command_receiver:Receiver<DebuggerCommand>,
    result_sender:Sender<DebuggerResult>,
    enabled_flag:Arc<AtomicBool>,
}

impl TerminalDebugger{
    pub fn new(s: Sender<PpuLayerResult>)->Self{
        let enabled = Arc::new(AtomicBool::new(false));

        let (command_sender, command_receiver) = bounded(0);
        let (result_sender, result_receiver) = bounded(0);
        let (terminal_input_sender, terminal_input_receiver) = bounded(0);
        let _ = thread::Builder::new()
            .name("Debugger input loop".to_string())
            .spawn(move || Self::get_string_loop(terminal_input_sender))
            .unwrap();
        let enabled_flag_clone = enabled.clone();
        let _ = thread::Builder::new()
            .name("Debugger IO loop".to_string())
            .spawn(move || Self::io_loop(command_sender, result_receiver, terminal_input_receiver, s, enabled_flag_clone))
            .unwrap();

        return Self{command_receiver, result_sender, enabled_flag: enabled};
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

    fn io_loop(sender:Sender<DebuggerCommand>, receiver:Receiver<DebuggerResult>, input_receiver:Receiver<String>, ppu_layer_sender:Sender<PpuLayerResult>, enabled:Arc<AtomicBool>){
        loop{
            crossbeam_channel::select! {
                recv(input_receiver)-> msg => {
                    let Ok(message) = msg else {break};
                    Self::handle_buffer(&sender, message, enabled.clone());
                },
                recv(receiver)-> res =>{ 
                    let Ok(result) = res else {break};
                    Self::handle_debugger_result(result, ppu_layer_sender.clone(), enabled.clone());
                },
            }
        }
        log::info!("Closing the debugger IO loop thread");
    }
    
    fn handle_debugger_result(result:DebuggerResult, ppu_layer_sender:Sender<PpuLayerResult>, enabled:Arc<AtomicBool>){
        match result{
            DebuggerResult::Stopped(addr, bank) => println!("Stopped -> {:#X}:{}", addr, bank),
            DebuggerResult::Registers(regs) => println!("AF: 0x{:X}\nBC: 0x{:X}\nDE: 0x{:X}\nHL: 0x{:X}\nSP: 0x{:X}\nPC: 0x{:X}",
                                                            regs.af, regs.bc, regs.de, regs.hl, regs.sp, regs.pc),
            DebuggerResult::HitBreak(addr, bank) =>{
                enabled.store(true, Ordering::SeqCst);
                println!("Hit break: {:#X}:{}", addr, bank);
            }
            DebuggerResult::HaltWakeup => println!("Waked up from halt"),
            DebuggerResult::AddedBreak(addr, bank)=>println!("Added BreakPoint successfully at address: {:#X}:{bank}", addr),
            DebuggerResult::Continuing=>println!("Continuing execution"),
            DebuggerResult::Stepped(addr, bank)=>println!("-> {:#X}:{}", addr, bank),
            DebuggerResult::RemovedBreak(addr, bank) => println!("Removed breakpoint successfully at {:#X}:{}", addr, bank),
            DebuggerResult::BreakDoNotExist(addr, bank) => println!("Breakpoint {:#X}:{} does not exist", addr, bank),
            DebuggerResult::MemoryDump(address, bank, buffer) => {
                const SPACING: usize = 16;
                for i in 0..buffer.len() as usize{
                    if i % SPACING == 0 { 
                        println!();
                        print!("{:#X}:{}: ", address + i as u16 , bank);
                    }
                    print!("{:#04X}, ", buffer[i]);
                }
                println!();
            },
            DebuggerResult::Disassembly(size, bank, opcodes)=>{
                for i in 0..size as usize{
                    println!("{:#X}:{} {}", opcodes[i].address, bank, opcodes[i].string);
                }
            },
            DebuggerResult::AddedWatch(addr, bank)=>println!("Set Watch point at: {addr:#X}:{bank} successfully"),
            DebuggerResult::HitWatch(address, addr_bank, pc, pc_bank, value) => {
                println!("Hit watch point: {address:#X}:{addr_bank} at address: {pc:#X}:{pc_bank} with value: {value:#X}");
                enabled.store(true, Ordering::SeqCst);
            },
            DebuggerResult::RemovedWatch(addr, bank) => println!("Removed watch point {addr:#X}:{bank}"),
            DebuggerResult::WatchDoNotExist(addr, bank) => println!("Watch point {addr:#X}:{bank} do not exist"),
            DebuggerResult::PpuInfo(info) => println!("PpuInfo: \nstate: {} \nlcdc: {:#X} \nstat: {:#X} \nly: {} \nbackground [X: {}, Y: {}] \nwindow [X: {}, Y: {}], \nbank: {}",
                info.ppu_state as u8, info.lcdc, info.stat, info.ly, info.background_pos.x, info.background_pos.y, info.window_pos.x, info.window_pos.y, info.vram_bank),
            DebuggerResult::PpuLayer(layer, buffer) => ppu_layer_sender.send(PpuLayerResult(buffer, layer)).unwrap()
        }
    }
    
    fn handle_buffer(sender:&Sender<DebuggerCommand>, buffer: String, enabled:Arc<AtomicBool>) {
        let buffer:Vec<&str> = buffer.trim().split(' ').collect();
        match buffer[0]{
            "h"|"halt"=>{
                enabled.store(true, Ordering::SeqCst);
                sender.send(DebuggerCommand::Stop).unwrap();
            }
            _ if enabled.load(Ordering::SeqCst)=>{
                match buffer[0]{
                    "c"|"continue"=>{
                        enabled.store(false, Ordering::SeqCst);
                        sender.send(DebuggerCommand::Continue).unwrap();
                    }
                    "s"|"step"=>sender.send(DebuggerCommand::Step).unwrap(),
                    "b"|"break"=>match parse_address_string(&buffer, 1) {
                        Ok((address, bank)) => sender.send(DebuggerCommand::Break(address, bank)).unwrap(),
                        Err(msg) => println!("Error setting BreakPoint {}", msg),
                    },
                    "reg"|"registers"=>sender.send(DebuggerCommand::Registers).unwrap(),
                    "rb"|"remove_break"=>match parse_address_string(&buffer, 1) {
                        Ok((address, bank)) => sender.send(DebuggerCommand::RemoveBreak(address, bank)).unwrap(),
                        Err(msg) => println!("Error deleting BreakPoint {}", msg),
                    },
                    "di"|"disassemble"=>match parse_number_string(&buffer, 1){
                        Ok(num) => sender.send(DebuggerCommand::Disassemble(num)).unwrap(),
                        Err(msg) => println!("Error disassembling: {}", msg),
                    },
                    "du"|"dump"=>match (parse_number_string(&buffer, 1), parse_number_string(&buffer, 2)){
                        (Ok(address), Ok(num)) => sender.send(DebuggerCommand::DumpMemory(address, num)).unwrap(),
                        (Err(msg), _) | 
                        (_, Err(msg)) => println!("Error dumping memory: {}", msg),
                    },
                    "w"|"watch"=> match parse_address_string(&buffer, 1){
                        Ok((addr, bank)) => sender.send(DebuggerCommand::Watch(addr, bank)).unwrap(),
                        Err(msg) => println!("Error setting watch point {}", msg),
                    }
                    "rw"|"remove_watch"=>match parse_address_string(&buffer, 1){
                        Ok((addr, bank)) => sender.send(DebuggerCommand::RemoveWatch(addr, bank)).unwrap(),
                        Err(msg) => println!("Error deleting watch point: {}", msg),
                    },
                    "pi"|"ppu_info"=>sender.send(DebuggerCommand::PpuInfo).unwrap(),
                    "pl"|"ppu_layer"=> match parse_ppu_layer(&buffer){
                        Ok(layer) => sender.send(DebuggerCommand::GetPpuLayer(layer)).unwrap(),
                        Err(msg) => println!("Error getting ppu layer: {}", msg),
                    }
                    "skip_halt"=>sender.send(DebuggerCommand::SkipHalt).unwrap(),
                    "help"=>println!("{}", HELP_MESSAGE),
                    _=>println!("invalid input: {}", buffer[0])
                }
            }
            _=>println!("invalid input: {}", buffer[0])
        }
    }
}

/// Address is "memory_address:bank" format
fn parse_address_string(buffer: &Vec<&str>, index:usize)->Result<(u16, u16), String>{
    let Some(param) = buffer.get(index) else {
        return Result::Err(String::from("No parameter"))
    };
    let strs:Vec<&str> = param.split(":").collect();
    let mem_addr = parse_number_string(&strs, 0)?;
    let bank = parse_number_string(&strs, 1)?;
    return Ok((mem_addr, bank));
}   

fn parse_number_string(buffer: &Vec<&str>, index:usize) -> Result<u16, String> {
    let Some(param) = buffer.get(index) else {
        return Result::Err(String::from("No parameter"))
    };
    let (str, base) = match param.strip_prefix("0x") {
        Some(hex_str) => (hex_str, 16),
        None => (*param, 10),
    };
    return u16::from_str_radix(str, base)
        .map_err(|err|format!("Error parsing string: {}", err));
}

fn parse_ppu_layer(buffer: &Vec<&str>)->Result<PpuLayer, String>{
    let Some(param) = buffer.get(1) else{
        return Result::Err(String::from("No param"))
    };

    return match *param{
        "win" => Ok(PpuLayer::Window),
        "spr" => Ok(PpuLayer::Sprites),
        "bg" => Ok(PpuLayer::Background),
        _=> Err(String::from("No matching layer"))
    };
}

impl DebuggerInterface for TerminalDebugger{
    fn should_stop(&self)->bool {self.enabled_flag.load(Ordering::SeqCst)}

    fn recv_command(&self)->DebuggerCommand {
        self.command_receiver.recv().unwrap()
    }

    fn send_result(&self, result:DebuggerResult) {
        self.result_sender.send(result).unwrap()
    }
}