mod mbc_handler;

use lib_gbc::machine::gameboy::GameBoy;
use lib_gbc::ppu::gb_ppu::{
    SCREEN_HEIGHT,
    SCREEN_WIDTH
};
use lib_gbc::keypad::button::Button;
use std::{ffi::c_void, fs};
use std::env;
use std::result::Result;
use std::vec::Vec;
use log::info;
use lib_gbc::mmu::gb_mmu::BOOT_ROM_SIZE;
use crate::mbc_handler::*;
use sdl2::sys::*;
use std::ffi::CString;
use lib_gbc::keypad::{
    joypad_provider::JoypadProvider,
    joypad::Joypad
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
        .level(level);

    if !debug{
        fern_logger = fern_logger.chain(std::io::stdout());
    }
    else{
        fern_logger = fern_logger.chain(fern::log_file("output.log")?);
    }

    fern_logger.apply()?;

    Ok(())
}

struct SdlJoypadProvider<F:Fn(Button)->SDL_Scancode>{
    mapper: F
}

impl<F:Fn(Button)->SDL_Scancode> JoypadProvider for SdlJoypadProvider<F>{
    fn provide(&mut self, joypad:&mut Joypad) {
        let mapper = &(self.mapper);
        unsafe{
            let keyborad_state:*const u8 = SDL_GetKeyboardState(std::ptr::null_mut());

            joypad.buttons[Button::A as usize]      = *keyborad_state.offset(mapper(Button::A) as isize) != 0;
            joypad.buttons[Button::B as usize]      = *keyborad_state.offset(mapper(Button::B) as isize) != 0;
            joypad.buttons[Button::Start as usize]  = *keyborad_state.offset(mapper(Button::Start) as isize) != 0;
            joypad.buttons[Button::Select as usize] = *keyborad_state.offset(mapper(Button::Select) as isize) != 0;
            joypad.buttons[Button::Up as usize]     = *keyborad_state.offset(mapper(Button::Up) as isize) != 0;
            joypad.buttons[Button::Down as usize]   = *keyborad_state.offset(mapper(Button::Down) as isize) != 0;
            joypad.buttons[Button::Right as usize]  = *keyborad_state.offset(mapper(Button::Right) as isize) != 0;
            joypad.buttons[Button::Left as usize]   = *keyborad_state.offset(mapper(Button::Left) as isize) != 0;
        }
    }
}

fn buttons_mapper(button:Button)->SDL_Scancode{
    match button{
        Button::A       => SDL_Scancode::SDL_SCANCODE_X,
        Button::B       => SDL_Scancode::SDL_SCANCODE_Z,
        Button::Start   => SDL_Scancode::SDL_SCANCODE_S,
        Button::Select  => SDL_Scancode::SDL_SCANCODE_A,
        Button::Up      => SDL_Scancode::SDL_SCANCODE_UP,
        Button::Down    => SDL_Scancode::SDL_SCANCODE_DOWN,
        Button::Right   => SDL_Scancode::SDL_SCANCODE_RIGHT,
        Button::Left    => SDL_Scancode::SDL_SCANCODE_LEFT
    }
}

fn main() {

    let screen_scale:u32 = 4;

    let args: Vec<String> = env::args().collect();    

    let debug_level = args.len() >= 3 && args[2].eq(&String::from("--log"));
    
    match init_logger(debug_level){
        Result::Ok(())=>{},
        Result::Err(error)=>std::panic!("error initing logger: {}", error)
    }

    let program_name = &args[1];
    let mut mbc = initialize_mbc(program_name); 

    let mut gameboy = match fs::read("Dependencies\\Init\\dmg_boot.bin"){
        Result::Ok(file)=>{
            info!("found bootrom!");

            let mut bootrom:[u8;BOOT_ROM_SIZE] = [0;BOOT_ROM_SIZE];
            for i in 0..BOOT_ROM_SIZE{
                bootrom[i] = file[i];
            }
            
            GameBoy::new_with_bootrom(&mut mbc, bootrom)
        }
        Result::Err(_)=>{
            info!("could not find bootrom... booting directly to rom");

            GameBoy::new(&mut mbc)
        }
    };

    let buffer_width = SCREEN_WIDTH as u32 * screen_scale;
    let buffer_height = SCREEN_HEIGHT as u32* screen_scale;

    
    let mut fpsc = fps_counter::FPSCounter::new();
    let mut frames = Vec::<usize>::new();
    unsafe{
        SDL_Init(SDL_INIT_VIDEO);
        let window:*mut SDL_Window = SDL_CreateWindow(
            CString::new("MagenBoy").unwrap().as_ptr(),
            SDL_WINDOWPOS_UNDEFINED_MASK as i32, SDL_WINDOWPOS_UNDEFINED_MASK as i32,
            buffer_width as i32, buffer_height as i32, 0);
        
        let renderer: *mut SDL_Renderer = SDL_CreateRenderer(window, -1, 0);

        let texture: *mut SDL_Texture = SDL_CreateTexture(renderer,
            SDL_PixelFormatEnum::SDL_PIXELFORMAT_ARGB8888 as u32, SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING as i32,
             buffer_width as i32, buffer_height as i32);

        let mut event: std::mem::MaybeUninit<SDL_Event> = std::mem::MaybeUninit::uninit();


        info!("initialized gameboy successfully!");


        loop{
            SDL_PumpEvents();
            SDL_RenderClear(renderer);

             if SDL_PollEvent(event.as_mut_ptr()) != 0{
                let event: SDL_Event = event.assume_init();
                if event.type_ == SDL_EventType::SDL_QUIT as u32{
                    break;
                }
            }

            let joypad_provider = SdlJoypadProvider{
                mapper: buttons_mapper
            };

            let mut pixels: *mut c_void = std::ptr::null_mut();
            let mut length: std::os::raw::c_int = 0;

            let vec:Vec<u32> = gameboy.cycle_frame(joypad_provider).to_vec();
            let other_vec = extend_vec(vec, screen_scale as usize, SCREEN_WIDTH, SCREEN_HEIGHT);

            SDL_LockTexture(texture, std::ptr::null(), &mut pixels, &mut length);
            std::ptr::copy_nonoverlapping(other_vec.as_ptr(),pixels as *mut u32,  other_vec.len());
            SDL_UnlockTexture(texture);
            SDL_RenderCopy(renderer, texture, std::ptr::null(), std::ptr::null());
            SDL_RenderPresent(renderer);

            frames.push(fpsc.tick());
        }
    }
    let avarage = {
        let mut counter = 0;
        for frame in &frames{
            counter+=frame;
        }

        counter/frames.len()
    };
    println!("{}", avarage);

    drop(gameboy);
    release_mbc(program_name, mbc);
}
