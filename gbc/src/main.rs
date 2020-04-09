extern crate lib_gbc;
extern crate stupid_gfx;
use lib_gbc::machine::gameboy::GameBoy;
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


fn main() {
    let gfx_initializer: Initializer = Initializer::new();
    let mut graphics: Graphics = gfx_initializer.init_graphics("GbcEmul", 800, 600,0);
    let mut event_handler: EventHandler = gfx_initializer.init_event_handler();

    let args: Vec<String> = env::args().collect();
    let file = match fs::read("Dependencies\\Init\\dmg_boot.bin"){
        Result::Ok(val)=>val,
        Result::Err(why)=>panic!("could not read file {}",why)
    };
    
    let mut bootrom:[u8;BOOT_ROM_SIZE] = [0;BOOT_ROM_SIZE];
    for i in 0..BOOT_ROM_SIZE{
        bootrom[i] = file[i];
    }

    let program = match fs::read(&args[1]){
        Result::Ok(val)=>val,
        Result::Err(why)=>panic!("could not read file {}",why)
    };
    

    let mbc = initialize_mbc(program);    

    let mut gameboy = GameBoy::new(mbc, bootrom, 17556);
    let mut alive = true;
    while alive {
        graphics.clear();
        for event in event_handler.poll_events(){
            match event{
                Event::Quit=>alive = false,
                _=>{}
            }
        }
        if alive{
            let vec = gameboy.cycle_frame();
            let surface = Surface::new_from_raw(vec, 160, 144);
            graphics.draw_surface(0, 0, &surface);
        }
        graphics.update();
    }
}
