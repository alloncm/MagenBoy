mod mbc_handler;
mod mpmc_gfx_device;
mod joypad_menu;
mod emulation_menu;

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

use crate::{audio::multi_device_audio::*, mbc_handler::*, mpmc_gfx_device::MpmcGfxDevice, emulation_menu::MagenBoyMenu};
use emulation_menu::MagenBoyState;
use joypad_menu::{JoypadMenu, MenuOption, MenuRenderer};
use lib_gb::{keypad::button::Button, apu::audio_device::*, machine::{gameboy::GameBoy, Mode}, ppu::{gb_ppu::{BUFFERS_NUMBER, SCREEN_HEIGHT, SCREEN_WIDTH}, gfx_device::{GfxDevice, Pixel}}, mmu::{GBC_BOOT_ROM_SIZE, external_memory_bus::Bootrom, GB_BOOT_ROM_SIZE}};
use std::{fs, env, result::Result, vec::Vec, path::PathBuf, convert::TryInto};
use log::info;
cfg_if::cfg_if! {if #[cfg(feature = "apu")]{
    use lib_gb::GB_FREQUENCY;
    use crate::audio::audio_resampler::ResampledAudioDevice;
}}
#[cfg(feature = "sdl")]
use sdl2::sys::*;

const TURBO_MUL:u8 = 1;

cfg_if::cfg_if!{ if #[cfg(feature = "rpi")] {
    const RESET_PIN_BCM:u8 = 14;
    const DC_PIN_BCM:u8 = 15;
    const LED_PIN_BCM:u8 = 25;
    const MENU_PIN_BCM:u8 = 3; // This pin is the turn on pin
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
} else if #[cfg(feature = "sdl")] {
    const SCREEN_SCALE:usize = 4;
    use sdl2::sys::SDL_Scancode;
    fn buttons_mapper(button:&Button)->SDL_Scancode{
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
}}

fn init_logger()->Result<(), fern::InitError>{
    let fern_logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S.%f]"),
                record.level(),
                message
            ))
        })
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?);

    fern_logger.apply()?;

    Ok(())
}

fn check_for_terminal_feature_flag(args:&Vec::<String>, flag:&str)->bool{
    args.len() >= 3 && args.contains(&String::from(flag))
}

fn get_terminal_feature_flag_value(args:&Vec<String>, flag:&str, error_message:&str)->String{
    let index = args.iter().position(|v| *v == String::from(flag)).unwrap();
    return args.get(index + 1).expect(error_message).clone();
}

fn get_rom_selection<MR:MenuRenderer<PathBuf, String>>(roms_path:&str, menu_renderer:MR)->String{
    let mut menu_options = Vec::new();
    let dir_entries = std::fs::read_dir(roms_path).expect(std::format!("Error openning the roms directory: {}",roms_path).as_str());
    for entry in dir_entries{
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(extension) = path.as_path().extension().and_then(std::ffi::OsStr::to_str){
            match extension {
                "gb" | "gbc"=>{
                    let filename = String::from(path.file_name().expect("Error should be a file").to_str().unwrap());
                    let option = MenuOption{value: path, prompt: filename};
                    menu_options.push(option);
                },
                _=>{}
            }
        }
    }
    cfg_if::cfg_if!{if #[cfg(feature = "rpi")]{
        let mut provider = GpioJoypadProvider::new(buttons_mapper);
    }
    else{
        let mut provider = sdl::sdl_joypad_provider::SdlJoypadProvider::new(buttons_mapper);
    }}
    let mut menu = JoypadMenu::new(&menu_options, String::from("Choose ROM"), menu_renderer);
    let result = menu.get_menu_selection(&mut provider);

    return String::from(result.to_str().unwrap());
}

// This is static and not local for the unix signal handler to access it
static EMULATOR_STATE:MagenBoyState = MagenBoyState::new();

const VERSION:&str = env!("CARGO_PKG_VERSION");

fn main() {
    let header = std::format!("MagenBoy v{}", VERSION);
    let args: Vec<String> = env::args().collect();  
    
    match init_logger(){
        Result::Ok(())=>{},
        Result::Err(error)=>std::panic!("error initing logger: {}", error)
    }

    // Initialize the gfx first cause it initialize both the screen and the sdl context for the joypad
    cfg_if::cfg_if!{ if #[cfg(feature = "rpi")]{
        let mut gfx_device:rpi_gpio::ili9341_controller::Ili9341GfxDevice<rpi_gpio::SpiType> = rpi_gpio::ili9341_controller::Ili9341GfxDevice::new(RESET_PIN_BCM, DC_PIN_BCM, LED_PIN_BCM, TURBO_MUL, 0);
    }else{
        let mut gfx_device = sdl::sdl_gfx_device::SdlGfxDevice::new(header.as_str(), SCREEN_SCALE, TURBO_MUL,
        check_for_terminal_feature_flag(&args, "--no-vsync"), check_for_terminal_feature_flag(&args, "--full-screen"));
    }}

    cfg_if::cfg_if!{if #[cfg(feature = "rpi")]{
        let provider = GpioJoypadProvider::new(buttons_mapper);
    }
    else{
        let provider = sdl::sdl_joypad_provider::SdlJoypadProvider::new(buttons_mapper);
    }} 
    let mut emulation_menu = MagenBoyMenu::new(provider, header);

    while !(EMULATOR_STATE.exit.load(std::sync::atomic::Ordering::Relaxed)){
        let program_name = if check_for_terminal_feature_flag(&args, "--rom-menu"){
            let roms_path = get_terminal_feature_flag_value(&args, "--rom-menu", "Error! no roms folder specified");
            let menu_renderer = joypad_menu::joypad_gfx_menu::GfxDeviceMenuRenderer::new(&mut gfx_device);
            get_rom_selection(roms_path.as_str(), menu_renderer)
        }
        else{
            args[1].clone()
        };

        let (s,r) = crossbeam_channel::bounded(BUFFERS_NUMBER - 1);
        let mpmc_device = MpmcGfxDevice::new(s);

        let args_clone = args.clone();
        let emualation_thread = std::thread::Builder::new().name("Emualtion Thread".to_string()).spawn(
            move || emulation_thread_main(args_clone, program_name, mpmc_device)
        ).unwrap();

        unsafe{
            cfg_if::cfg_if!{ if #[cfg(feature = "rpi")]{
                let handler = nix::sys::signal::SigHandler::Handler(sigint_handler);
                nix::sys::signal::signal(nix::sys::signal::Signal::SIGINT, handler).unwrap();
                let menu_pin = rppal::gpio::Gpio::new().unwrap().get(MENU_PIN_BCM).unwrap().into_input_pullup();
            } else if #[cfg(feature = "sdl")]{
                let mut event: std::mem::MaybeUninit<SDL_Event> = std::mem::MaybeUninit::uninit();
            }}

            loop{
                cfg_if::cfg_if!{ if #[cfg(feature = "sdl")]{
                    if SDL_PollEvent(event.as_mut_ptr()) != 0{
                        let event: SDL_Event = event.assume_init();
                        if event.type_ == SDL_EventType::SDL_QUIT as u32{
                            EMULATOR_STATE.exit.store(true, std::sync::atomic::Ordering::Relaxed);
                            break;
                        }
                        else if event.type_ == SDL_EventType::SDL_KEYDOWN as u32 && event.key.keysym.scancode == SDL_Scancode::SDL_SCANCODE_ESCAPE{
                            emulation_menu.pop_game_menu(&EMULATOR_STATE, &mut gfx_device, r.clone());
                        }
                    }
                } else if #[cfg(feature = "rpi")]{
                    if menu_pin.is_low(){
                        emulation_menu.pop_game_menu(&EMULATOR_STATE, &mut gfx_device, r.clone());
                    }
                }}

                match r.recv() {
                    Result::Ok(buffer) => gfx_device.swap_buffer(&*(buffer as *const [Pixel; SCREEN_WIDTH * SCREEN_HEIGHT])),
                    Result::Err(_) => break,
                }
            }

            drop(r);
            EMULATOR_STATE.running.store(false, std::sync::atomic::Ordering::Relaxed);
            emualation_thread.join().unwrap();
        }
    }

    drop(gfx_device);

    #[cfg(feature = "sdl")]
    unsafe{SDL_Quit();}

    #[cfg(feature = "rpi")]
    if check_for_terminal_feature_flag(&args, "--shutdown-rpi"){
        log::info!("Shuting down the RPi! Goodbye");
        std::process::Command::new("shutdown").arg("-h").arg("now").spawn().expect("Failed to shutdown system");
    }
}

#[cfg(feature = "rpi")]
extern "C" fn sigint_handler(_:std::os::raw::c_int){
    EMULATOR_STATE.running.store(false, std::sync::atomic::Ordering::Relaxed);
    EMULATOR_STATE.exit.store(true, std::sync::atomic::Ordering::Relaxed);
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

    let bootrom = match fs::read(&bootrom_path){
        Result::Ok(file)=>{
            info!("found bootrom!");
            if file.len() == GBC_BOOT_ROM_SIZE{
                Bootrom::Gbc(file.try_into().unwrap())
            }
            else if file.len() == GB_BOOT_ROM_SIZE{
                Bootrom::Gb(file.try_into().unwrap())
            }
            else{
                std::panic!("Error! bootrom: \"{}\" length is invalid", bootrom_path);
            }
        }
        Result::Err(_)=>{
            info!("Could not find bootrom... booting directly to rom");
            Bootrom::None
        }
    };

    let mode = match bootrom{
        Bootrom::Gb(_) => Some(Mode::DMG),
        Bootrom::Gbc(_)=> Some(Mode::CGB),
        Bootrom::None=>{
            if check_for_terminal_feature_flag(&args, "--mode"){
                let mode = get_terminal_feature_flag_value(&args, "--mode", "Error: Must specify a mode");
                let mode = mode.as_str().try_into().expect(format!("Error! mode cannot be: {}", mode.as_str()).as_str());
                Some(mode)
            }
            else{
                Option::None
            }
        }
    };

    let mut mbc = initialize_mbc(&program_name, mode);
    let mut gameboy = GameBoy::new(&mut mbc, joypad_provider, audio_devices, spsc_gfx_device, bootrom, mode);

    info!("initialized gameboy successfully!");

    EMULATOR_STATE.running.store(true, std::sync::atomic::Ordering::Relaxed);
    while EMULATOR_STATE.running.load(std::sync::atomic::Ordering::Relaxed){
        if !EMULATOR_STATE.pause.load(std::sync::atomic::Ordering::SeqCst){
            let state = &EMULATOR_STATE;
            let _mutex_ctx = state.state_mutex.lock().unwrap();
            gameboy.cycle_frame();
        }
    }
    drop(gameboy);
    release_mbc(&program_name, mbc);
    log::info!("released the gameboy succefully");
}