use std::{env, path::PathBuf};
use magenboy_common::{check_for_terminal_feature_flag, get_terminal_feature_flag_value, init_gameboy, joypad_menu::*, mbc_handler::{initialize_mbc, release_mbc}, menu::*};
use magenboy_core::{ppu::gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, apu::audio_device::*, keypad::joypad::NUM_OF_KEYS};
use magenboy_rpi::{configuration::{display::*, emulation::*, joypad::*}, drivers::*, peripherals::PERIPHERALS, BlankAudioDevice, MENU_PIN_BCM};

fn main(){
    unsafe{magenboy_rpi::peripherals::PERIPHERALS.set_core_clock()};
    magenboy_common::logging::init_fern_logger().unwrap();
    let mut joypad_provider = GpioJoypadProvider::new(button_to_bcm_pin);
    let mut gfx = Ili9341GfxDevice::new(RESET_PIN_BCM, LED_PIN_BCM, TURBO, FRAME_LIMITER);

    let args: Vec<String> = env::args().collect(); 
    
    let header = std::format!("MagenBoy v{}", magenboy_common::VERSION);

    let mut shutdown = false;

    while !shutdown {
        let args = args.clone();

        let program_name: PathBuf = if check_for_terminal_feature_flag(&args, "--rom-menu"){
            let roms_path = get_terminal_feature_flag_value(&args, "--rom-menu", "Error! no roms folder specified");
            let rom_options = read_roms_menu_options(&roms_path);
            let mut menu = MagenBoyMenu::new(&header, Some(&rom_options));
            let rom_path: PathBuf;
            loop {
                let joypad = joypad_provider.provide();
                match menu.get_rom_selection(joypad) {
                    MenuResult::Selection(sel) => {
                        rom_path = sel.clone(); 
                        break; 
                    },
                    MenuResult::Frame(frame) => gfx.swap_buffer(&frame),
                }
            }
            rom_path
        }
        else{
            PathBuf::from(args[1].clone())
        };

        let mbc = initialize_mbc(&program_name);

        let mut gameboy = init_gameboy(
            args,
            mbc,
            BlankAudioDevice
        );

        let mut game_menu = false;

        'main: loop {
            if game_menu {
                let joypad = joypad_provider.poll();
                match MagenBoyMenu::new(&header, Option::None).get_game_menu_selection(joypad) {
                    MenuResult::Selection(menu_option) => match *menu_option {
                        EmulatorMenuOption::Resume => { game_menu = false; continue; }
                        EmulatorMenuOption::Restart => break 'main,
                        EmulatorMenuOption::Shutdown => { shutdown = true; break 'main; }
                    },
                    MenuResult::Frame(frame) => gfx.swap_buffer(&frame),
                }
            } else {
                let joypad = joypad_provider.provide();
                let buffer = gameboy.cycle_frame(joypad);
                gfx.swap_buffer(buffer);
            }
        }

        drop(gameboy);
        release_mbc(&program_name, mbc);
        log::info!("released the gameboy succefully");
    }

    if check_for_terminal_feature_flag(&args, "--shutdown-rpi"){
        log::info!("Shuting down the RPi! Goodbye");
        std::process::Command::new("shutdown").arg("-h").arg("now").spawn().expect("Failed to shutdown system");
    }
}