mod mbc_handler;
mod sdl_joypad_provider;
mod sdl_audio_device;

use crate::{mbc_handler::*, sdl_joypad_provider::SdlJoypadProvider};
use lib_gb::{
    machine::gameboy::GameBoy,
    ppu::gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH},
    keypad::button::Button,
    mmu::gb_mmu::BOOT_ROM_SIZE
};
use std::{
    ffi::{c_void, CString},
    fs, env, result::Result, vec::Vec,
    time::{Duration, Instant}
};
use log::info;
use sdl2::sys::*;


const FRAME_RATE:u16 = 60;


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

    let buffer_width = SCREEN_WIDTH as u32 * screen_scale;
    let buffer_height = SCREEN_HEIGHT as u32* screen_scale;
    let program_name = CString::new("MagenBoy").unwrap();
    let (_window, renderer, texture): (*mut SDL_Window, *mut SDL_Renderer, *mut SDL_Texture) = unsafe{
        SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO);
        let wind:*mut SDL_Window = SDL_CreateWindow(
            program_name.as_ptr(),
            SDL_WINDOWPOS_UNDEFINED_MASK as i32, SDL_WINDOWPOS_UNDEFINED_MASK as i32,
            buffer_width as i32, buffer_height as i32, 0);
        
        let rend: *mut SDL_Renderer = SDL_CreateRenderer(wind, -1, 0);

        let tex: *mut SDL_Texture = SDL_CreateTexture(rend,
            SDL_PixelFormatEnum::SDL_PIXELFORMAT_ARGB8888 as u32, SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING as i32,
             buffer_width as i32, buffer_height as i32);
        
        (wind, rend, tex)
    };

    let audio_device = sdl_audio_device::SdlAudioDevie::new(44100, 1);

    let program_name = &args[1];
    let mut mbc = initialize_mbc(program_name); 
    let joypad_provider = SdlJoypadProvider::new(buttons_mapper);

    let mut gameboy = match fs::read("Dependencies\\Init\\dmg_boot.bin"){
        Result::Ok(file)=>{
            info!("found bootrom!");

            let mut bootrom:[u8;BOOT_ROM_SIZE] = [0;BOOT_ROM_SIZE];
            for i in 0..BOOT_ROM_SIZE{
                bootrom[i] = file[i];
            }
            
            GameBoy::new_with_bootrom(&mut mbc, joypad_provider,audio_device, bootrom)
        }
        Result::Err(_)=>{
            info!("could not find bootrom... booting directly to rom");

            GameBoy::new(&mut mbc, joypad_provider, audio_device)
        }
    };

    info!("initialized gameboy successfully!");


    let time:Instant = Instant::now();
    let mut last_time:Duration = time.elapsed();


    let mut s = String::new();
    std::io::stdin().read_line(&mut s).unwrap();

    unsafe{
        let mut event: std::mem::MaybeUninit<SDL_Event> = std::mem::MaybeUninit::uninit();
        loop{
             if SDL_PollEvent(event.as_mut_ptr()) != 0{
                let event: SDL_Event = event.assume_init();
                if event.type_ == SDL_EventType::SDL_QUIT as u32{
                    break;
                }
            }

            let current_time:Duration = time.elapsed();

            if current_time.as_micros() - last_time.as_micros() >= 1000_000 / FRAME_RATE as u128 {

                //log::info!("{}", current_time.as_micros() - last_time.as_micros());
                last_time = current_time;

                let frame_buffer:Vec<u32> = gameboy.cycle_frame().to_vec();
                let scaled_buffer = extend_vec(frame_buffer, screen_scale as usize, SCREEN_WIDTH, SCREEN_HEIGHT);

                let mut pixels: *mut c_void = std::ptr::null_mut();
                let mut length: std::os::raw::c_int = 0;
                SDL_LockTexture(texture, std::ptr::null(), &mut pixels, &mut length);
                std::ptr::copy_nonoverlapping(scaled_buffer.as_ptr(),pixels as *mut u32,  scaled_buffer.len());
                SDL_UnlockTexture(texture);

                SDL_RenderClear(renderer);
                SDL_RenderCopy(renderer, texture, std::ptr::null(), std::ptr::null());
                SDL_RenderPresent(renderer);
            }
        }

        SDL_Quit();
    }

    drop(gameboy);
    release_mbc(program_name, mbc);
}
