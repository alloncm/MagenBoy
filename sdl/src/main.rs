mod audio;
mod utils;
mod sdl_joypad_provider;
mod window;
#[cfg(feature = "dbg")]
mod terminal_debugger;
#[cfg(feature = "dbg")]
mod dbg_window;

use std::{env, ffi::CString, path::PathBuf, result::Result, vec::Vec};

use sdl2::sys::*;

use magenboy_common::{audio::{ManualAudioResampler, ResampledAudioDevice}, check_for_terminal_feature_flag, get_terminal_feature_flag_value, init_gameboy, joypad_menu::*, mbc_handler::{initialize_mbc, release_mbc}, menu::*, gl_gfx_device::GlGfxDevice};
use magenboy_core::{apu::audio_device::*, keypad::joypad::NUM_OF_KEYS, ppu::gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, GB_FREQUENCY, utils::vec2::Vec2};

use crate::{audio::*, utils::get_sdl_error_message, SdlAudioDevice, window::SdlWindow};

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

fn main() {
    let header = std::format!("MagenBoy v{}", magenboy_common::VERSION);
    let args: Vec<String> = env::args().collect();  
    
    match magenboy_common::logging::init_fern_logger(){
        Result::Ok(())=>{},
        Result::Err(error)=>std::panic!("error initing logger: {}", error)
    }

    init_sdl_subsystem(SDL_INIT_EVENTS);
    init_sdl_subsystem(SDL_INIT_VIDEO);
    init_sdl_subsystem(SDL_INIT_AUDIO);

    let window = SdlWindow::new(header.clone(), Vec2{x: SCREEN_WIDTH, y: SCREEN_HEIGHT}, SCREEN_SCALE);
    let sdl_window = window.window_handle;

    let mut shutdown = false;

    while !shutdown {
        let args = args.clone();

        let mut width: i32 = 0;
        let mut height: i32 = 0;
        unsafe {
            SDL_GetWindowSize(sdl_window, &mut width, &mut height);
        }

        let mut gfx_device = GlGfxDevice::new(width as u32, height as u32, |s|{
            let name = CString::new(s).unwrap();
            unsafe{SDL_GL_GetProcAddress(name.as_ptr())}
        });

        let mut devices: Vec::<Box::<dyn AudioDevice>> = Vec::new();
        let audio_device = SdlAudioDevice::<ManualAudioResampler>::new(44100, TURBO_MUL);
        devices.push(Box::new(audio_device));
        
        if check_for_terminal_feature_flag(&args, "--file-audio"){
            let wav_ad = WavfileAudioDevice::<ManualAudioResampler>::new(44100, GB_FREQUENCY, "output.wav");
            devices.push(Box::new(wav_ad));
            log::info!("Writing audio to file: output.wav");
        }
            
        let audio_devices = MultiAudioDevice::new(devices);

        let mut joypad_provider = sdl_joypad_provider::SdlJoypadProvider::new(KEYBOARD_MAPPING);

        let program_name = if check_for_terminal_feature_flag(&args, "--rom-menu"){
            let roms_path = get_terminal_feature_flag_value(&args, "--rom-menu", "Error! no roms folder specified");

            let rom_options = read_roms_menu_options(&roms_path);
            let mut menu = MagenBoyMenu::new(&header, Some(&rom_options));
            let rom_path: PathBuf;
            loop {
                let mut menu_triggered = false;
                handle_events(&mut shutdown, &mut menu_triggered, sdl_window);
                let joypad = joypad_provider.provide();
                match menu.get_rom_selection(joypad) {
                    MenuResult::Selection(sel) => {
                        rom_path = sel.clone();
                        break;
                    },
                    MenuResult::Frame(frame) => {
                        gfx_device.render(&frame);
                        unsafe{SDL_GL_SwapWindow(sdl_window)};
                    },
                }
            }
            rom_path
        }
        else{
            PathBuf::from(args[1].clone())
        };

        let mut emulation_menu = MagenBoyMenu::new(&header, Option::None );

        #[cfg(feature = "dbg")]
        let (debugger_ppu_layer_sender, debugger_ppu_layer_receiver) = crossbeam_channel::bounded::<terminal_debugger::PpuLayerResult>(0);

        let mbc = initialize_mbc(&program_name);
        
        let mut gameboy = init_gameboy(
            args,
            mbc,
            audio_devices,
            #[cfg(feature = "dbg")] terminal_debugger::TerminalDebugger::new(debugger_ppu_layer_sender)
        );

        let mut game_menu = false;
        loop {
            handle_events(&mut shutdown, &mut game_menu, sdl_window);
            if shutdown {
                break;
            }
            if game_menu {
                let joypad = joypad_provider.poll();
                match emulation_menu.get_game_menu_selection(joypad) {
                    MenuResult::Selection(menu_option) => match menu_option {
                        EmulatorMenuOption::Resume => {
                            game_menu = false;
                            continue;
                        }
                        EmulatorMenuOption::Restart => break,
                        EmulatorMenuOption::Shutdown => {
                            shutdown = true;
                            break;
                        }
                    },
                    MenuResult::Frame(frame) => gfx_device.render(&frame),
                }
            } else {
                let joypad = joypad_provider.provide();
                let buffer = gameboy.cycle_frame(joypad);
                gfx_device.render(buffer);
                #[cfg(feature = "dbg")] 
                {
                    let Ok(result) = debugger_ppu_layer_receiver.try_recv() else {
                        break
                    };
                    let mut window = dbg_window::PpuLayerWindow::new(gfx_device.clone(), result.1);
                    window.run(&result.0);
                }
            };
            // SAFETY: SDL call
            unsafe{SDL_GL_SwapWindow(sdl_window)};
        }

        drop(gameboy);
        release_mbc(&program_name, mbc);
        log::info!("released the gameboy succefully");
    }

    // SAFETY: SDL calls
    unsafe{
        SDL_Quit();
    }
}

fn handle_events(shutdown: &mut bool, game_menu: &mut bool, sdl_window: *mut SDL_Window) {
    while let Some(event) = poll_event() {
        // SAFETY: type_ is present on all the variants so it is safe to access it
        let event_type = unsafe {event.type_};

        if event_type == SDL_EventType::SDL_QUIT as u32 {
            *shutdown = true;
            return;
        }
        else if event_type == SDL_EventType::SDL_KEYDOWN as u32 {
            // SAFETY: Since event is KEYDOWN the key variant is safe to access
            let key_pressed = unsafe{event.key.keysym.scancode};
            if key_pressed == SDL_Scancode::SDL_SCANCODE_ESCAPE {
                *game_menu = true;
            }
        }
        else if event_type == SDL_EventType::SDL_WINDOWEVENT as u32{
            let mut width: i32 = 0;
            let mut height: i32 = 0;
            // SAFETY: SDL call
            unsafe{SDL_GetWindowSize(sdl_window, &mut width, &mut height)};
            GlGfxDevice::update_viewport(width, height);
        }
    }
}

fn poll_event()->Option<SDL_Event>{
    unsafe{
        let mut event: std::mem::MaybeUninit<SDL_Event> = std::mem::MaybeUninit::uninit();
        // updating the events for the whole app
        SDL_PumpEvents();
        if SDL_PollEvent(event.as_mut_ptr()) != 0{
            return Option::Some(event.assume_init());
        }
        return Option::None;
    }
}

fn init_sdl_subsystem(sdl_subsystem_flag: u32) {
    // SAFETY: SDL call
    unsafe{
        let rv = SDL_InitSubSystem(sdl_subsystem_flag);    
        if rv != 0 {
            std::panic!("Failed to init subsystem, rv: {} flag: {} message: {}", rv, sdl_subsystem_flag, get_sdl_error_message());
        }
    }
}