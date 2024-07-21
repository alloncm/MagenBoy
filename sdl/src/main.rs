mod audio;
#[cfg(feature = "dbg")]
mod terminal_debugger;
mod sdl{
    pub mod utils;
    pub mod sdl_gfx_device;
    #[cfg(feature = "sdl-resample")]
    pub mod sdl_audio_resampler;

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
    pub mod sdl_joypad_provider;
}

cfg_if::cfg_if!{
    if #[cfg(feature = "sdl-resample")]{
        pub type ChosenResampler  = sdl::sdl_audio_resampler::SdlAudioResampler;
    }
    else{
        pub type ChosenResampler  = magenboy_common::audio::ManualAudioResampler;
    }
}

use magenboy_common::{audio::ResampledAudioDevice, joypad_menu::*, mbc_handler::*, menu::*, mpmc_gfx_device::*};
use magenboy_core::{GB_FREQUENCY, apu::audio_device::*, keypad::joypad::NUM_OF_KEYS, machine::{gameboy::GameBoy, Mode}, mmu::{external_memory_bus::Bootrom, GBC_BOOT_ROM_SIZE, GB_BOOT_ROM_SIZE}, ppu::{gb_ppu::{BUFFERS_NUMBER, SCREEN_HEIGHT, SCREEN_WIDTH}, gfx_device::{GfxDevice, Pixel}}};
use std::{fs, env, result::Result, vec::Vec, convert::TryInto};
use log::info;
use sdl2::sys::*;
#[cfg(feature = "dbg")]
use crate::terminal_debugger::TerminalDebugger;
use crate::{sdl::sdl_gfx_device::SdlGfxDevice, audio::*};

const TURBO_MUL:u8 = 1;

const SCREEN_SCALE:usize = 4;
use sdl2::sys::SDL_Scancode;
const KEYBOARD_MAPPING:[SDL_Scancode; NUM_OF_KEYS] = [
    SDL_Scancode::SDL_SCANCODE_X,
    SDL_Scancode::SDL_SCANCODE_Z,
    SDL_Scancode::SDL_SCANCODE_S,
    SDL_Scancode::SDL_SCANCODE_A,
    SDL_Scancode::SDL_SCANCODE_UP,
    SDL_Scancode::SDL_SCANCODE_DOWN,
    SDL_Scancode::SDL_SCANCODE_RIGHT,
    SDL_Scancode::SDL_SCANCODE_LEFT
];


fn check_for_terminal_feature_flag(args:&Vec::<String>, flag:&str)->bool{
    args.len() >= 3 && args.contains(&String::from(flag))
}

fn get_terminal_feature_flag_value(args:&Vec<String>, flag:&str, error_message:&str)->String{
    let index = args.iter().position(|v| *v == String::from(flag)).unwrap();
    return args.get(index + 1).expect(error_message).clone();
}

// This is static and not local for the unix signal handler to access it
static EMULATOR_STATE:MagenBoyState = MagenBoyState::new();

fn main() {
    let header = std::format!("MagenBoy v{}", magenboy_common::VERSION);
    let args: Vec<String> = env::args().collect();  
    
    match magenboy_common::logging::init_fern_logger(){
        Result::Ok(())=>{},
        Result::Err(error)=>std::panic!("error initing logger: {}", error)
    }

    // Initialize the gfx first cause it initialize both the screen and the sdl context for the joypad
    let mut gfx_device: SdlGfxDevice = SdlGfxDevice::new(header.as_str(), SCREEN_SCALE, TURBO_MUL,
    check_for_terminal_feature_flag(&args, "--no-vsync"), check_for_terminal_feature_flag(&args, "--full-screen"));

    while !(EMULATOR_STATE.exit.load(std::sync::atomic::Ordering::Relaxed)){
        let mut provider = sdl::sdl_joypad_provider::SdlJoypadProvider::new(KEYBOARD_MAPPING, true);

        let program_name = if check_for_terminal_feature_flag(&args, "--rom-menu"){
            let roms_path = get_terminal_feature_flag_value(&args, "--rom-menu", "Error! no roms folder specified");
            let menu_renderer = joypad_gfx_menu::GfxDeviceMenuRenderer::new(&mut gfx_device);
            get_rom_selection(roms_path.as_str(), menu_renderer, &mut provider)
        }
        else{
            args[1].clone()
        };

        let mut emulation_menu = MagenBoyMenu::new(provider, header.clone());

        let (s,r) = crossbeam_channel::bounded(BUFFERS_NUMBER - 1);
        let mpmc_device = MpmcGfxDevice::new(s);

        #[cfg(feature = "dbg")]
        let (debugger_s, debugger_r) = crossbeam_channel::bounded::<terminal_debugger::PpuLayerResult>(0);

        let args_clone = args.clone();
        let emualation_thread = std::thread::Builder::new()
            .name("Emualtion Thread".to_string())
            .stack_size(0x100_0000)
            .spawn(move || emulation_thread_main(args_clone, program_name, mpmc_device, #[cfg(feature = "dbg")]debugger_s))
            .unwrap();

        unsafe{
            'main:loop{
                while let Some(event) = gfx_device.poll_event(){
                    if event.type_ == SDL_EventType::SDL_QUIT as u32{
                        EMULATOR_STATE.exit.store(true, std::sync::atomic::Ordering::Relaxed);
                        break 'main;
                    }
                    else if event.type_ == SDL_EventType::SDL_KEYDOWN as u32 && event.key.keysym.scancode == SDL_Scancode::SDL_SCANCODE_ESCAPE{
                        emulation_menu.pop_game_menu(&EMULATOR_STATE, &mut gfx_device, r.clone());
                    }
                }

                cfg_if::cfg_if! {if #[cfg(feature = "dbg")] {
                    crossbeam_channel::select! {
                        recv(r) -> msg => {
                            let Ok(buffer) = msg else {break};
                            gfx_device.swap_buffer(&*(buffer as *const [Pixel; SCREEN_WIDTH * SCREEN_HEIGHT]));
                        },
                        recv(debugger_r)-> msg => {
                            let Ok(result) = msg else {break};
                            let mut window = sdl::sdl_gfx_device::PpuLayerWindow::new(result.1);
                            window.run(result.0);
                        }
                    }
                }else{
                    let Ok(buffer) = r.recv() else {break};
                    gfx_device.swap_buffer(&*(buffer as *const [Pixel; SCREEN_WIDTH * SCREEN_HEIGHT]));
                }}
            }

            drop(r);
            EMULATOR_STATE.running.store(false, std::sync::atomic::Ordering::Relaxed);
            emualation_thread.join().unwrap();
        }
    }

    drop(gfx_device);

    unsafe{SDL_Quit();}
}

// Receiving usize and not raw ptr cause in rust you cant pass a raw ptr to another thread
fn emulation_thread_main(args: Vec<String>, program_name: String, spsc_gfx_device: MpmcGfxDevice, #[cfg(feature = "dbg")] debugger_sender: crossbeam_channel::Sender<terminal_debugger::PpuLayerResult>) {
    let mut devices: Vec::<Box::<dyn AudioDevice>> = Vec::new();
    let audio_device = sdl::ChosenAudioDevice::<ChosenResampler>::new(44100, TURBO_MUL);
    devices.push(Box::new(audio_device));
    
    if check_for_terminal_feature_flag(&args, "--file-audio"){
        let wav_ad = WavfileAudioDevice::<ChosenResampler>::new(44100, GB_FREQUENCY, "output.wav");
        devices.push(Box::new(wav_ad));
        log::info!("Writing audio to file: output.wav");
    }
        
    let audio_devices = MultiAudioDevice::new(devices);
    let joypad_provider = sdl::sdl_joypad_provider::SdlJoypadProvider::new(KEYBOARD_MAPPING, false);
    
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

    let mbc = initialize_mbc(&program_name, mode);
    let mut gameboy = GameBoy::new(
        mbc, joypad_provider, audio_devices, spsc_gfx_device, 
        #[cfg(feature = "dbg")]
        TerminalDebugger::new(debugger_sender),
        bootrom, mode);

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