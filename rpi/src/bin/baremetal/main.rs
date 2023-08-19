#![no_main]
#![no_std]

mod boot;
mod logging;

use core::panic::PanicInfo;

use magenboy_common::joypad_menu::{joypad_gfx_menu, JoypadMenu, MenuOption};
use magenboy_core::{machine::{gameboy::GameBoy, mbc_initializer::initialize_mbc}, mmu::external_memory_bus::Bootrom, utils::stack_string::StackString};
use magenboy_rpi::{drivers::{GpioJoypadProvider, Ili9341GfxDevice, Fat32, FileEntry}, peripherals::PERIPHERALS, configuration::{display::*, joypad::button_to_bcm_pin, emulation::*}};

const MAX_ROM_SIZE:usize = 0x80_0000;       // 8 MiB, Max size of MBC5 rom

// Allocating as static buffer (on the .bss) because it is a very large buffer and 
// I dont want to cause problems in stack making it overflow and shit (I can increase it when needed but I afraid Id forget)
static mut ROM_BUFFER:[u8; MAX_ROM_SIZE] = [0;MAX_ROM_SIZE];

// This function is no regular main.
// It will not return and will be jumped to from the _start proc in the boot code
// it is unmangled and exposed as a "C" function in order for the _start proc to call it
#[no_mangle]
pub extern "C" fn main()->!{
    unsafe{PERIPHERALS.set_core_clock()};
    logging::UartLogger::init(log::LevelFilter::Debug);
    log::info!("Initialized logger");
    log::info!("running at exec mode: {:#X}", boot::get_cpu_execution_mode());

    let mut fs = Fat32::new();
    let mut gfx = Ili9341GfxDevice::new(RESET_PIN_BCM, LED_PIN_BCM, TURBO, FRAME_LIMITER);
    let mut joypad_provider = GpioJoypadProvider::new(button_to_bcm_pin);
    log::info!("Initialize all drivers succesfully");

    let menu_renderer = joypad_gfx_menu::GfxDeviceMenuRenderer::new(&mut gfx);

    let mut menu_options:[MenuOption<FileEntry, StackString<{FileEntry::FILENAME_SIZE}>>; 255] = [Default::default(); 255];
    let mut menu_options_size = 0;
    let mut root_dir_offset = 0;
    const FILES_PER_LIST:usize = 20;
    'search_dir_loop: loop{
        let dir_entries = fs.root_dir_list::<FILES_PER_LIST>(root_dir_offset);
        for entry in dir_entries{
            let Some(entry) = entry else {break 'search_dir_loop};
            let extension = entry.get_extension();
            if extension.eq_ignore_ascii_case("gb") || extension.eq_ignore_ascii_case("gbc"){
                menu_options[menu_options_size] = MenuOption{ value: entry.clone(), prompt: StackString::from(entry.get_name()) };
                menu_options_size += 1;
                log::debug!("Detected ROM: {}", entry.get_name());
            }
        }
        root_dir_offset += FILES_PER_LIST;
    }

    let mut menu = JoypadMenu::new(&menu_options[0..menu_options_size], StackString::from("Choose ROM"), menu_renderer);
    let selected_rom = menu.get_menu_selection(&mut joypad_provider);
    log::info!("Selected ROM: {}", selected_rom.get_name());
    
    let rom = unsafe{&mut ROM_BUFFER};
    fs.read_file(selected_rom, rom);
    let mbc = initialize_mbc(&rom[0..selected_rom.size as usize], None, None);

    let mut gameboy = GameBoy::new(mbc, joypad_provider, magenboy_rpi::BlankAudioDevice, gfx, Bootrom::None, None);
    log::info!("Initialized gameboy!");
    loop{
        gameboy.cycle_frame();
    }
}

#[panic_handler]
fn panic(info:&PanicInfo)->!{
    log::error!("An error has occoured!");
    log::error!("{}", info);

    unsafe{boot::hang_led()};
}