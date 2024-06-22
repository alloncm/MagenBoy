use std::{env, fs};

use magenboy_common::{menu::*, joypad_menu::*, mpmc_gfx_device::MpmcGfxDevice, mbc_handler::{initialize_mbc, release_mbc}};
use magenboy_core::{ppu::{gb_ppu::{BUFFERS_NUMBER, SCREEN_WIDTH, SCREEN_HEIGHT}, gfx_device::{GfxDevice, Pixel}}, mmu::{GBC_BOOT_ROM_SIZE, external_memory_bus::Bootrom, GB_BOOT_ROM_SIZE}, machine::{Mode, gameboy::GameBoy}, keypad::joypad_provider::JoypadProvider};
use log::info;
use magenboy_rpi::{configuration::{emulation::*, display::*, joypad::*}, drivers::*, peripherals::PERIPHERALS, MENU_PIN_BCM};

// This is static and not local for the unix signal handler to access it
static EMULATOR_STATE:MagenBoyState = MagenBoyState::new();

fn main(){
    unsafe{magenboy_rpi::peripherals::PERIPHERALS.set_core_clock()};
    magenboy_common::logging::init_fern_logger().unwrap();
    let mut joypad_provider = GpioJoypadProvider::new(button_to_bcm_pin);
    let mut gfx = Ili9341GfxDevice::new(RESET_PIN_BCM, LED_PIN_BCM, TURBO, FRAME_LIMITER);

    let args: Vec<String> = env::args().collect(); 
    
    let header = std::format!("MagenBoy v{}", magenboy_common::VERSION);

    let mut emulation_menu = MagenBoyMenu::new(joypad_provider.clone(), header.clone());

    while !(EMULATOR_STATE.exit.load(std::sync::atomic::Ordering::Relaxed)){
        
        let program_name = if check_for_terminal_feature_flag(&args, "--rom-menu"){
            let roms_path = get_terminal_feature_flag_value(&args, "--rom-menu", "Error! no roms folder specified");
            let menu_renderer = joypad_gfx_menu::GfxDeviceMenuRenderer::new(&mut gfx);
            get_rom_selection(roms_path.as_str(), menu_renderer, &mut joypad_provider)
        }
        else{
            args[1].clone()
        };

        let (s,r) = crossbeam_channel::bounded(BUFFERS_NUMBER - 1);
        let mpmc_device = MpmcGfxDevice::new(s);

        let joypad_clone = joypad_provider.clone();
        let args_clone = args.clone();
        let emualation_thread = std::thread::Builder::new().name("Emualtion Thread".to_string()).spawn(
            move || emulation_thread_main(args_clone, program_name, mpmc_device, joypad_clone)
        ).unwrap();

        unsafe{
            let handler = nix::sys::signal::SigHandler::Handler(sigint_handler);
            nix::sys::signal::signal(nix::sys::signal::Signal::SIGINT, handler).unwrap();
            let menu_pin = PERIPHERALS.get_gpio().take_pin(MENU_PIN_BCM).into_input(magenboy_rpi::peripherals::GpioPull::PullUp);

            loop{
                if menu_pin.read_state() == false{
                    emulation_menu.pop_game_menu(&EMULATOR_STATE, &mut gfx, r.clone());
                }

                match r.recv() {
                    Result::Ok(buffer) => gfx.swap_buffer(&*(buffer as *const [Pixel; SCREEN_WIDTH * SCREEN_HEIGHT])),
                    Result::Err(_) => break,
                }
            }

            drop(r);
            EMULATOR_STATE.running.store(false, std::sync::atomic::Ordering::Relaxed);
            emualation_thread.join().unwrap();
        }
    }

    if check_for_terminal_feature_flag(&args, "--shutdown-rpi"){
        log::info!("Shuting down the RPi! Goodbye");
        std::process::Command::new("shutdown").arg("-h").arg("now").spawn().expect("Failed to shutdown system");
    }
}

fn emulation_thread_main(args: Vec<String>, program_name: String, spsc_gfx_device: MpmcGfxDevice, joypad_provider:impl JoypadProvider) {
    
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
    let mut gameboy = GameBoy::new(mbc, joypad_provider , magenboy_rpi::BlankAudioDevice, spsc_gfx_device, bootrom, mode);

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

fn check_for_terminal_feature_flag(args:&Vec::<String>, flag:&str)->bool{
    args.len() >= 3 && args.contains(&String::from(flag))
}

fn get_terminal_feature_flag_value(args:&Vec<String>, flag:&str, error_message:&str)->String{
    let index = args.iter().position(|v| *v == String::from(flag)).unwrap();
    return args.get(index + 1).expect(error_message).clone();
}

extern "C" fn sigint_handler(_:std::os::raw::c_int){
    EMULATOR_STATE.running.store(false, std::sync::atomic::Ordering::Relaxed);
    EMULATOR_STATE.exit.store(true, std::sync::atomic::Ordering::Relaxed);
}