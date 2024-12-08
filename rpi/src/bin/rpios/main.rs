use std::env;

use magenboy_common::{check_for_terminal_feature_flag, get_terminal_feature_flag_value, init_and_run_gameboy, joypad_menu::*, menu::*, mpmc_gfx_device::MpmcGfxDevice, EMULATOR_STATE};
use magenboy_core::{ppu::{gb_ppu::{BUFFERS_NUMBER, SCREEN_WIDTH, SCREEN_HEIGHT}, gfx_device::{GfxDevice, Pixel}}, keypad::joypad_provider::JoypadProvider};
use magenboy_rpi::{configuration::{display::*, emulation::*, joypad::*}, drivers::*, peripherals::PERIPHERALS, BlankAudioDevice, MENU_PIN_BCM};

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
    init_and_run_gameboy(args, program_name, spsc_gfx_device, joypad_provider, BlankAudioDevice);
}

extern "C" fn sigint_handler(_:std::os::raw::c_int){
    EMULATOR_STATE.running.store(false, std::sync::atomic::Ordering::Relaxed);
    EMULATOR_STATE.exit.store(true, std::sync::atomic::Ordering::Relaxed);
}