mod mbc_handler;
mod mpmc_gfx_device;
mod joypad_terminal_menu;
#[cfg(feature = "rpi")]
mod rpi_gpio;
mod audio{
    pub mod audio_resampler;
    pub mod multi_device_audio;
    #[cfg(feature = "apu")]
    pub mod wav_file_audio_device;
    #[cfg(not(feature = "sdl-resample"))]
    pub mod manual_audio_resampler;
}
#[cfg(feature = "sdl")]
mod sdl{
    pub mod utils;
    #[cfg(not(feature = "u16pixel"))]
    pub mod sdl_gfx_device;
    #[cfg(feature = "sdl-resample")]
    pub mod sdl_audio_resampler;

    #[cfg(feature = "apu")]
    cfg_if::cfg_if!{
        if #[cfg(feature = "push-audio")]{
            pub mod sdl_push_audio_device;
            pub type ChosenAudioDevice<AR> = sdl_push_audio_device::SdlPushAudioDevice<AR>;
        }
        else{
            pub mod sdl_pull_audio_device;
            pub type ChosenAudioDevice<AR> = sdl_pull_audio_device::SdlPullAudioDevice<AR>;
        }
    }
    #[cfg(not(feature = "rpi"))]
    pub mod sdl_joypad_provider;
}

cfg_if::cfg_if!{
    if #[cfg(feature = "sdl-resample")]{
        pub type ChosenResampler  = sdl::sdl_audio_resampler::SdlAudioResampler;
    }
    else{
        pub type ChosenResampler  = audio::manual_audio_resampler::ManualAudioResampler;
    }
}

use crate::{audio::multi_device_audio::*, mbc_handler::*, mpmc_gfx_device::MpmcGfxDevice};
use joypad_terminal_menu::{MenuOption, JoypadTerminalMenu, TerminalRawModeJoypadProvider};
use lib_gb::{keypad::button::Button, apu::audio_device::*, machine::gameboy::GameBoy, mmu::gb_mmu::BOOT_ROM_SIZE, ppu::{gb_ppu::{BUFFERS_NUMBER, SCREEN_HEIGHT, SCREEN_WIDTH}, gfx_device::{GfxDevice, Pixel}}};
use std::{fs, env, result::Result, vec::Vec};
use log::info;
cfg_if::cfg_if! {if #[cfg(feature = "apu")]{
    use lib_gb::GB_FREQUENCY;
    use crate::audio::audio_resampler::ResampledAudioDevice;
}}
#[cfg(feature = "sdl")]
use sdl2::sys::*;

const TURBO_MUL:u8 = 1;

cfg_if::cfg_if!{
    if #[cfg(feature = "rpi")]{
        const RESET_PIN_BCM:u8 = 14;
        const DC_PIN_BCM:u8 = 15;
        const LED_PIN_BCM:u8 = 25;
        use crate::rpi_gpio::gpio_joypad_provider::*;
        fn buttons_mapper(button:&Button)->GpioBcmPin{
            match button{
                Button::A       => 18,
                Button::B       => 17,
                Button::Start   => 22,
                Button::Select  => 23,
                Button::Up      => 19,
                Button::Down    => 16,
                Button::Right   => 20,
                Button::Left    => 21
            }
        }
    }
    else{
        const SCREEN_SCALE:usize = 4;
        use sdl2::sys::SDL_Scancode;
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
    }
}

fn init_logger(debug:bool)->Result<(), fern::InitError>{
    let level = if debug {log::LevelFilter::Debug} else {log::LevelFilter::Info};
    let mut fern_logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S.%f]"),
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

fn menu_buttons_mapper(button:crossterm::event::KeyCode)->Option<Button>{
    match button{
        crossterm::event::KeyCode::Char('x') => Option::Some(Button::A),
        crossterm::event::KeyCode::Up        => Option::Some(Button::Up),
        crossterm::event::KeyCode::Down      => Option::Some(Button::Down),
        _=> Option::None
    }
}

fn check_for_terminal_feature_flag(args:&Vec::<String>, flag:&str)->bool{
    args.len() >= 3 && args.contains(&String::from(flag))
}

fn get_terminal_feature_flag_value(args:&Vec<String>, flag:&str, error_message:&str)->String{
    let index = args.iter().position(|v| *v == String::from(flag)).unwrap();
    return args.get(index + 1).expect(error_message).clone();
}

fn get_rom_selection(roms_path:&str)->String{
    let mut menu_options = Vec::new();
    let dir_entries = std::fs::read_dir(roms_path).expect(std::format!("Error openning the roms directory: {}",roms_path).as_str());
    for entry in dir_entries{
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(extension) = path.as_path().extension(){
            if extension == "gb"{
                let filename = String::from(path.file_name().expect("Error should be a file").to_str().unwrap());
                let option = MenuOption{value: path, prompt: filename};
                menu_options.push(option);
            }
        }
    }

    let mut menu = JoypadTerminalMenu::new(menu_options);
    let mut provider = TerminalRawModeJoypadProvider::new(menu_buttons_mapper);
    let result = menu.get_menu_selection(&mut provider);
    // Removing the file extenstion and casting to String
    let result = String::from(result.with_extension("").to_str().unwrap());
    
    return result;
}

static mut RUNNING:bool = true;

fn main() {
    let args: Vec<String> = env::args().collect();  

    let program_name = if check_for_terminal_feature_flag(&args, "--rom_menu"){
        let roms_path = get_terminal_feature_flag_value(&args, "--rom_menu", "Error! no roms folder specified");
        get_rom_selection(roms_path.as_str())
    }
    else{
        args[1].clone()
    };

    let debug_level = check_for_terminal_feature_flag(&args, "--log");
    
    match init_logger(debug_level){
        Result::Ok(())=>{},
        Result::Err(error)=>std::panic!("error initing logger: {}", error)
    }

    cfg_if::cfg_if!{ if #[cfg(feature = "rpi")]{
        let mut gfx_device:rpi_gpio::ili9341_controller::Ili9341GfxDevice<rpi_gpio::SpiType> = rpi_gpio::ili9341_controller::Ili9341GfxDevice::new(RESET_PIN_BCM, DC_PIN_BCM, LED_PIN_BCM, TURBO_MUL, 0);
    }else{
        let mut gfx_device = sdl::sdl_gfx_device::SdlGfxDevice::new("MagenBoy", SCREEN_SCALE, TURBO_MUL,
        check_for_terminal_feature_flag(&args, "--no-vsync"), check_for_terminal_feature_flag(&args, "--full-screen"));
    }}
    
    let (s,r) = crossbeam_channel::bounded(BUFFERS_NUMBER - 1);
    let mpmc_device = MpmcGfxDevice::new(s);
    
    let emualation_thread = std::thread::Builder::new().name("Emualtion Thread".to_string()).spawn(
        move || emulation_thread_main(args, program_name, mpmc_device)
    ).unwrap();

    unsafe{
        #[cfg(feature = "rpi")]{
            let handler = nix::sys::signal::SigHandler::Handler(sigint_handler);
            nix::sys::signal::signal(nix::sys::signal::Signal::SIGINT, handler).unwrap();
        }

        #[cfg(feature = "sdl")]
        let mut event: std::mem::MaybeUninit<SDL_Event> = std::mem::MaybeUninit::uninit();
        loop{
            #[cfg(feature = "sdl")]
            if SDL_PollEvent(event.as_mut_ptr()) != 0{
                let event: SDL_Event = event.assume_init();
                if event.type_ == SDL_EventType::SDL_QUIT as u32{
                    break;
                }
            }
            
            match r.recv() {
                Result::Ok(buffer) => gfx_device.swap_buffer(&*(buffer as *const [Pixel; SCREEN_WIDTH * SCREEN_HEIGHT])),
                Result::Err(_) => break,
            }

        }

        drop(r);
        RUNNING = false;
        emualation_thread.join().unwrap();

        #[cfg(feature = "sdl")]
        SDL_Quit();
    }
}

#[cfg(feature = "rpi")]
extern "C" fn sigint_handler(_:std::os::raw::c_int){
    unsafe {RUNNING = false};
}

// Receiving usize and not raw ptr cause in rust you cant pass a raw ptr to another thread
fn emulation_thread_main(args: Vec<String>, program_name: String, spsc_gfx_device: MpmcGfxDevice) {
    cfg_if::cfg_if!{ 
        if #[cfg(feature = "apu")]{
            let mut devices: Vec::<Box::<dyn AudioDevice>> = Vec::new();
            let audio_device = sdl::ChosenAudioDevice::<ChosenResampler>::new(44100, TURBO_MUL);
            devices.push(Box::new(audio_device));
            
            if check_for_terminal_feature_flag(&args, "--file-audio"){
                let wav_ad = audio::wav_file_audio_device::WavfileAudioDevice::<ChosenResampler>::new(44100, GB_FREQUENCY, "output.wav");
                devices.push(Box::new(wav_ad));
                log::info!("Writing audio to file: output.wav");
            }
        }
        else{
            let devices: Vec::<Box::<dyn AudioDevice>> = Vec::new();
        }
    }
    let audio_devices = MultiAudioDevice::new(devices);
    let mut mbc = initialize_mbc(&program_name);
    cfg_if::cfg_if!{
        if #[cfg(feature = "rpi")]{
            let joypad_provider = GpioJoypadProvider::new(buttons_mapper);
        }
        else{
            let joypad_provider = sdl::sdl_joypad_provider::SdlJoypadProvider::new(buttons_mapper);
        }
    }
    let bootrom_path = if check_for_terminal_feature_flag(&args, "--bootrom"){
        get_terminal_feature_flag_value(&args, "--bootrom", "Error! you must specify a value for the --bootrom parameter")
    }else{
        String::from("dmg_boot.bin")
    };

    let mut gameboy = match fs::read(bootrom_path){
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

    unsafe{
        while RUNNING{
            gameboy.cycle_frame();
        }
    }
    drop(gameboy);
    release_mbc(&program_name, mbc);
    log::info!("released the gameboy succefully");
}