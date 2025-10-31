mod audio;
mod utils;
mod sdl_gfx_device;
mod sdl_joypad_provider;
#[cfg(feature = "dbg")]
mod terminal_debugger;

use std::{env, result::Result, vec::Vec};

use sdl2::sys::*;

use magenboy_common::{audio::{ManualAudioResampler, ResampledAudioDevice}, check_for_terminal_feature_flag, get_terminal_feature_flag_value, init_gameboy, joypad_menu::*, menu::*, GfxDevice, JoypadProvider, EMULATOR_STATE};
use magenboy_core::{apu::audio_device::*, keypad::NUM_OF_KEYS, GB_FREQUENCY};

use crate::{sdl_gfx_device::SdlGfxDevice, audio::*, SdlAudioDevice};

const SCREEN_SCALE:usize = 4;
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

fn main() {
    let header = std::format!("MagenBoy v{}", magenboy_common::VERSION);
    let args: Vec<String> = env::args().collect();  
    
    match magenboy_common::logging::init_fern_logger(){
        Result::Ok(())=>{},
        Result::Err(error)=>std::panic!("error initing logger: {}", error)
    }

    // Initialize the gfx first cause it initialize both the screen and the sdl context for the joypad
    let mut gfx_device: SdlGfxDevice = SdlGfxDevice::new(
        header.as_str(), 
        SCREEN_SCALE,
        check_for_terminal_feature_flag(&args, "--no-vsync"),
        check_for_terminal_feature_flag(&args, "--full-screen")
    );

    while !(EMULATOR_STATE.exit.load(std::sync::atomic::Ordering::Relaxed)){
        let mut provider = sdl_joypad_provider::SdlJoypadProvider::new(KEYBOARD_MAPPING, true);

        let program_name = if check_for_terminal_feature_flag(&args, "--rom-menu"){
            let roms_path = get_terminal_feature_flag_value(&args, "--rom-menu", "Error! no roms folder specified");
            let menu_renderer = joypad_gfx_menu::GfxDeviceMenuRenderer::new(&mut gfx_device);
            get_rom_selection(roms_path.as_str(), menu_renderer, &mut provider)
        }
        else{
            args[1].clone()
        };

        let mut emulation_menu = MagenBoyMenu::new(provider, header.clone());

        #[cfg(feature = "dbg")]
        let (debugger_ppu_layer_sender, debugger_ppu_layer_receiver) = crossbeam_channel::bounded::<terminal_debugger::PpuLayerResult>(0);

        let mut devices: Vec::<Box::<dyn AudioDevice>> = Vec::new();
        let audio_device = SdlAudioDevice::<ManualAudioResampler>::new(44100);
        devices.push(Box::new(audio_device));
        
        if check_for_terminal_feature_flag(&args, "--file-audio"){
            let wav_ad = WavfileAudioDevice::<ManualAudioResampler>::new(44100, GB_FREQUENCY, "output.wav");
            devices.push(Box::new(wav_ad));
            log::info!("Writing audio to file: output.wav");
        }
            
        let audio_devices = MultiAudioDevice::new(devices);
        let mut joypad_provider = sdl_joypad_provider::SdlJoypadProvider::new(KEYBOARD_MAPPING, false);
        
        let mut gameboy = init_gameboy(&args, program_name, audio_devices, #[cfg(feature = "dbg")] terminal_debugger::TerminalDebugger::new(debugger_sender));

        unsafe{
            'main:loop{
                while let Some(event) = SdlGfxDevice::poll_event() {
                    if event.type_ == SDL_EventType::SDL_QUIT as u32{
                        EMULATOR_STATE.exit.store(true, std::sync::atomic::Ordering::Relaxed);
                        break 'main;
                    }
                    else if event.type_ == SDL_EventType::SDL_KEYDOWN as u32 && event.key.keysym.scancode == SDL_Scancode::SDL_SCANCODE_ESCAPE{
                        emulation_menu.pop_game_menu(&EMULATOR_STATE, &mut gfx_device);
                    }
                }

                cfg_if::cfg_if! {if #[cfg(feature = "dbg")] {
                    crossbeam_channel::select! {
                        recv(r) -> msg => {
                            let Ok(buffer) = msg else {break};
                            gfx_device.swap_buffer(&*(buffer as *const [Pixel; SCREEN_WIDTH * SCREEN_HEIGHT]));
                        },
                        recv(debugger_ppu_layer_receiver)-> msg => {
                            let Ok(result) = msg else {break};
                            let mut window = sdl_gfx_device::PpuLayerWindow::new(result.1);
                            window.run(&result.0);
                        }
                    }
                } else {
                    let mut joypad = Default::default();
                    joypad_provider.provide(&mut joypad);
                    let frame = gameboy.cycle_frame(joypad);
                    gfx_device.swap_buffer(frame);
                }}
            }
        }
    }

    drop(gfx_device);

    unsafe{SDL_Quit();}
}
