use lib_gbc::machine::gameboy::GameBoy;
use lib_gbc::keypad::button::Button;
use std::fs;
use std::env;
use std::result::Result;
use std::vec::Vec;
use log::info;
use lib_gbc::mmu::gbc_mmu::BOOT_ROM_SIZE;
use stupid_gfx::{
    event_handler::EventHandler,
    graphics::Graphics,
    initializer::Initializer,
    surface::Surface,
    event::*
};

mod mbc_initializer;
mod stupid_gfx_joypad_provider;
use crate::stupid_gfx_joypad_provider::StupidGfxJoypadProvider;
use crate::mbc_initializer::*;

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

fn init_logger(debug:bool)->Result<(), fern::InitError>{
    let level = if debug {log::LevelFilter::Debug} else {log::LevelFilter::Info};
    let mut fern_logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.level(),
                message
            ))
        })
        .level(level)
        .chain(fern::log_file("output.log")?);

    if !debug{
        fern_logger = fern_logger.chain(std::io::stdout());
    }

    fern_logger.apply()?;

    Ok(())
}

fn buttons_mapper(button:Button)->Scancode{
    match button{
        Button::A => Scancode::X,
        Button::B => Scancode::Z,
        Button::Start => Scancode::S,
        Button::Select =>Scancode::A,
        Button::Up => Scancode::Up,
        Button::Down => Scancode::Down,
        Button::Right => Scancode::Right,
        Button::Left => Scancode::Left
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();    

    let debug_level = args.len() >= 3 && args[2].eq(&String::from("--log"));
    
    match init_logger(debug_level){
        Result::Ok(())=>{},
        Result::Err(error)=>std::panic!("error initing logger: {}", error)
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

    let program_name = &args[1];
    let mut mbc = initialize_mbc(program_name);    

    //CPU frequrncy: 1,048,326 / 60 
    let cycles_per_frame = 17556;
    let mut gameboy = GameBoy::new(&mut mbc, bootrom, cycles_per_frame);

    info!("initialized gameboy successfully");

    let scale:u32 = 4;
    let mut alive = true;
    while alive {
        graphics.clear();
        for event in event_handler.poll_events(){
            match event{
                Event::Quit=>alive = false,
                _=>{}
            }
        }

        let joypad_provider = StupidGfxJoypadProvider::new(&mut event_handler, buttons_mapper);
        
        let vec:Vec<u32> = gameboy.cycle_frame(joypad_provider).to_vec();
        let other_vec = extend_vec(vec, scale as usize, 160, 144);
        let surface = Surface::new_from_raw(other_vec, 160*scale, 144*scale);

        graphics.draw_surface(0, 0, &surface);
        graphics.update();
    }

    drop(gameboy);
    release_mbc(program_name, mbc);
}
