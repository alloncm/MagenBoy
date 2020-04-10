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
            let other_vec = extend_vec(vec, 2, 160, 144);
            let surface = Surface::new_from_raw(other_vec, 160*2, 144*2);
            graphics.draw_surface(0, 0, &surface);
        }
        graphics.update();
    }
}
