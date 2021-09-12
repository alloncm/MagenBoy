mod mbc_handler;
mod sdl_joypad_provider;
mod sdl_audio_device;
mod audio_resampler;
mod wav_file_audio_device;
mod multi_device_audio;
mod sdl_gfx_device;

use crate::{mbc_handler::*, sdl_joypad_provider::*, multi_device_audio::*};
use lib_gb::{GB_FREQUENCY, apu::audio_device::*, keypad::button::Button, machine::gameboy::GameBoy, mmu::gb_mmu::BOOT_ROM_SIZE, ppu::{gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, gfx_device::GfxDevice}};
use std::{fs, env, result::Result, vec::Vec};
use log::info;
use sdl2::sys::*;

const FPS:f64 = GB_FREQUENCY as f64 / 70224.0;
const FRAME_TIME_MS:f64 = (1.0 / FPS) * 1000.0;
const SCREEN_SCALE:u8 = 4;

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

fn check_for_terminal_feature_flag(args:&Vec::<String>, flag:&str)->bool{
    args.len() >= 3 && args.contains(&String::from(flag))
}

struct SpscGfxDevice{
    producer: rtrb::Producer<[u32;SCREEN_HEIGHT * SCREEN_WIDTH ]>,
    parker: crossbeam::sync::Parker,
}

impl GfxDevice for SpscGfxDevice{
    fn swap_buffer(&mut self, buffer:&[u32; SCREEN_WIDTH * SCREEN_HEIGHT]) {
        self.producer.push(buffer.clone()).unwrap();
        if self.producer.is_full(){
            self.parker.park();
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();    

    let debug_level = check_for_terminal_feature_flag(&args, "--log");
    
    match init_logger(debug_level){
        Result::Ok(())=>{},
        Result::Err(error)=>std::panic!("error initing logger: {}", error)
    }

    let mut sdl_gfx_device = sdl_gfx_device::SdlGfxDevice::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, "MagenBoy", SCREEN_SCALE);

    let (producer, mut c) = rtrb::RingBuffer::<[u32; SCREEN_HEIGHT * SCREEN_WIDTH]>::new(1).split();
    let parker = crossbeam::sync::Parker::new();
    let unparker = parker.unparker().clone();
    let spsc_gfx_device = SpscGfxDevice{producer, parker: parker};
    

    let program_name = args[1].clone();
    
    std::thread::Builder::new().name("Emualtion Thread".to_string()).spawn(move ||{

        let audio_device = sdl_audio_device::SdlAudioDevie::new(44100);
        let mut devices: Vec::<Box::<dyn AudioDevice>> = Vec::new();
        devices.push(Box::new(audio_device));
        if check_for_terminal_feature_flag(&args, "--file-audio"){
            let wav_ad = wav_file_audio_device::WavfileAudioDevice::new(44100, GB_FREQUENCY, "output.wav");
            devices.push(Box::new(wav_ad));
        }

        let audio_devices = MultiAudioDevice::new(devices);
        let mut mbc = initialize_mbc(&program_name); 
        let joypad_provider = SdlJoypadProvider::new(buttons_mapper);
    
        let mut gameboy = match fs::read("Dependencies/Init/dmg_boot.bin"){
            Result::Ok(file)=>{
                info!("found bootrom!");
    
                let mut bootrom:[u8;BOOT_ROM_SIZE] = [0;BOOT_ROM_SIZE];
                for i in 0..BOOT_ROM_SIZE{
                    bootrom[i] = file[i];
                }
                
                GameBoy::new_with_bootrom(&mut mbc, joypad_provider,audio_devices, spsc_gfx_device, bootrom)
            }
            Result::Err(_)=>{
                info!("could not find bootrom... booting directly to rom");
    
                GameBoy::new(&mut mbc, joypad_provider, audio_devices, spsc_gfx_device)
            }
        };
    
        info!("initialized gameboy successfully!");
    
        loop{
            gameboy.cycle_frame();
        }
        
        drop(gameboy);
        release_mbc(&program_name, mbc);
    }).unwrap();

    unsafe{
        let mut event: std::mem::MaybeUninit<SDL_Event> = std::mem::MaybeUninit::uninit();
        let mut start:u64 = SDL_GetPerformanceCounter();
        loop{

            if SDL_PollEvent(event.as_mut_ptr()) != 0{
                let event: SDL_Event = event.assume_init();
                if event.type_ == SDL_EventType::SDL_QUIT as u32{
                    break;
                }
            }

            if !c.is_empty(){
                let pop = c.pop().unwrap();
                unparker.unpark();
                sdl_gfx_device.swap_buffer(&pop);
            }

            let end = SDL_GetPerformanceCounter();
            let elapsed_ms:f64 = (end - start) as f64 / SDL_GetPerformanceFrequency() as f64 * 1000.0;
            if elapsed_ms < FRAME_TIME_MS{
                SDL_Delay((FRAME_TIME_MS - elapsed_ms).floor() as u32);
            }

            start = SDL_GetPerformanceCounter();
        }

        SDL_Quit();
    }
}