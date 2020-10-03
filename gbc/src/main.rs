extern crate lib_gbc;
extern crate stupid_gfx;
use lib_gbc::machine::{
    gameboy::GameBoy,
    joypad::Joypad
};
use std::default::Default;
use std::fs;
use std::env;
use std::result::Result;
use std::vec::Vec;
use lib_gbc::mmu::mbc_initializer::initialize_mbc;
use lib_gbc::mmu::gbc_mmu::BOOT_ROM_SIZE;
use stupid_gfx::{
    event_handler::EventHandler,
    graphics::Graphics,
    initializer::Initializer,
    surface::Surface,
    event::*
};

fn extend_vec(vec:Vec<u32>, scale:usize, w:usize, h:usize)->Vec<u32>{
    let mut new_vec = vec![0;vec.len()*scale*scale];
    for y in 0..h{
        let sy = y*scale;
        for x in 0..w{
            let sx = x*scale;
            for i in 0..scale{
                for j in 0..scale{
                    new_vec[(sy+i)*(w*scale)+sx+j] = vec[y*w+x];
                }
            }
        } 
    }
    return new_vec;
}

fn init_logger()->Result<(), fern::InitError>{
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                //"{}[{}][{}] {}",
                //chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                //record.target(),
                //record.level(),
                "{}",
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}


fn main() {

    let args: Vec<String> = env::args().collect();    

    if args.len() >= 3{
        if args[2].eq(&String::from("--log")){
            match init_logger(){
                Result::Ok(())=>{},
                Result::Err(error)=>std::panic!("error initing logger: {}", error)
            }
        }
    }
    
    let gfx_initializer: Initializer = Initializer::new();
    let mut graphics: Graphics = gfx_initializer.init_graphics("MagenBoy", 800, 600,0, true);
    let mut event_handler: EventHandler = gfx_initializer.init_event_handler();

    let file = match fs::read("Dependencies\\Init\\dmg_boot.bin"){
        Result::Ok(val)=>val,
        Result::Err(why)=>panic!("could not read boot rom {}",why)
    };
    
    let mut bootrom:[u8;BOOT_ROM_SIZE] = [0;BOOT_ROM_SIZE];
    for i in 0..BOOT_ROM_SIZE{
        bootrom[i] = file[i];
    }

    let program = match fs::read(&args[1]){
        Result::Ok(val)=>val,
        Result::Err(why)=>panic!("could not read rom {}\n{}",args[1],why)
    };
    

    let mbc = initialize_mbc(program);    

    let mut joypad = Joypad::default();

    //CPU frequrncy: 1,048,326 / 60 
    let cycles_per_frame = 17556;
    let mut gameboy = GameBoy::new(mbc, bootrom, cycles_per_frame);
    let mut alive = true;
    let scale:u32 = 4;
    while alive {
        graphics.clear();
        joypad.clear();
        for event in event_handler.poll_events(){
            match event{
                Event::Quit=>alive = false,
                Event::KeyPressed(code)=>{
                    match code{
                        Keycode::X=>joypad.a = true,
                        Keycode::Z=>joypad.b = true,
                        Keycode::S=>joypad.start = true,
                        Keycode::A=>joypad.select = true,
                        Keycode::Up=>joypad.up = true,
                        Keycode::Down=>joypad.down = true,
                        Keycode::Right=>joypad.right = true,
                        Keycode::Left=>joypad.left = true,
                        _=>{}
                    }
                }
            }
        }
        
        let vec:Vec<u32> = gameboy.cycle_frame(&joypad).to_vec();
        let other_vec = extend_vec(vec, scale as usize, 160, 144);
        let surface = Surface::new_from_raw(other_vec, 160*scale, 144*scale);

        graphics.draw_surface(0, 0, &surface);
        graphics.update();
    }
}
