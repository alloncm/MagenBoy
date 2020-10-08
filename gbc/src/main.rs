mod mbc_handler;
mod stupid_gfx_joypad_provider;

use lib_gbc::machine::gameboy::GameBoy;
use lib_gbc::ppu::gbc_ppu::{
    SCREEN_HEIGHT,
    SCREEN_WIDTH
};
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

use crate::stupid_gfx_joypad_provider::StupidGfxJoypadProvider;
use crate::mbc_handler::*;

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


//CPU frequrncy: 1,048,326 / 60 
const CYCLES_PER_FRAME:u32 = 17556; 

fn main() {

    let screen_scale:u32 = 4;

    let args: Vec<String> = env::args().collect();    

    let debug_level = args.len() >= 3 && args[2].eq(&String::from("--log"));
    
    match init_logger(debug_level){
        Result::Ok(())=>{},
        Result::Err(error)=>std::panic!("error initing logger: {}", error)
    }
    
    let gfx_initializer: Initializer = Initializer::new();
    let mut graphics: Graphics = gfx_initializer.init_graphics("MagenBoy", SCREEN_WIDTH as u32 * screen_scale, SCREEN_HEIGHT as u32* screen_scale, 0, true);
    let mut event_handler: EventHandler = gfx_initializer.init_event_handler();

    let program_name = &args[1];
    let mut mbc = initialize_mbc(program_name); 

    let mut gameboy = match fs::read("Dependencies\\Init\\dmg_boot.bin"){
        Result::Ok(file)=>{
            info!("found bootrom!");

            let mut bootrom:[u8;BOOT_ROM_SIZE] = [0;BOOT_ROM_SIZE];
            for i in 0..BOOT_ROM_SIZE{
                bootrom[i] = file[i];
            }
            
            GameBoy::new_with_bootrom(&mut mbc, bootrom, CYCLES_PER_FRAME)
        }
        Result::Err(_)=>{
            info!("could not find bootrom... booting directly to rom");

            GameBoy::new(&mut mbc, CYCLES_PER_FRAME)
        }
    };
    
  
    info!("initialized gameboy successfully!");


    let mut alive = true;
    while alive {
        graphics.clear();
        for event in event_handler.get_events(){
            match event{
                Event::Quit=>alive = false,
                _=>{}
            }
        }

        let joypad_provider = StupidGfxJoypadProvider::new(&mut event_handler, buttons_mapper);
        
        let vec:Vec<u32> = gameboy.cycle_frame(joypad_provider).to_vec();
        let other_vec = extend_vec(vec, screen_scale as usize, SCREEN_WIDTH, SCREEN_HEIGHT);
        let surface = Surface::new_from_raw(other_vec, SCREEN_WIDTH as u32 * screen_scale, SCREEN_HEIGHT as u32 * screen_scale);

        graphics.draw_surface(0, 0, &surface);
        graphics.update();
    }

    drop(gameboy);
    release_mbc(program_name, mbc);
}
