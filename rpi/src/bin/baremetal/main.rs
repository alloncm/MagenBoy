#![no_main]
#![no_std]

mod boot;
mod logging;

use core::panic::PanicInfo;

use magenboy_common::{joypad_menu::{joypad_gfx_menu::{self, GfxDeviceMenuRenderer}, JoypadMenu, }, menu::*, VERSION};
use magenboy_core::{machine::{gameboy::GameBoy, mbc_initializer::initialize_mbc}, mmu::{external_memory_bus::Bootrom, carts::Mbc}, utils::stack_string::StackString};
use magenboy_rpi::{drivers::*, peripherals::{PERIPHERALS, GpioPull, ResetMode, Power}, configuration::{display::*, joypad::button_to_bcm_pin, emulation::*}, MENU_PIN_BCM, delay};

#[panic_handler]
fn panic(info:&PanicInfo)->!{
    log::error!("An error has occoured!: \r\n{}", info);

    unsafe{boot::hang_led()};
}

const MAX_ROM_SIZE:usize = 0x80_0000;       // 8 MiB, Max size of MBC5 rom
const MAX_RAM_SIZE:usize = 0x2_0000;        // 128 KiB

// Allocating as static buffer (on the .bss) because it is a very large buffer and 
// I dont want to cause problems in stack making it overflow and shit (I can increase it when needed but I afraid Id forget)
static mut ROM_BUFFER:[u8; MAX_ROM_SIZE] = [0;MAX_ROM_SIZE];
static mut RAM_BUFFER:[u8; MAX_RAM_SIZE] = [0;MAX_RAM_SIZE];

// This function is no regular main.
// It will not return and will be jumped to from the _start proc in the boot code
// it is unmangled and exposed as a "C" function in order for the _start proc to call it
#[no_mangle]
pub extern "C" fn main()->!{
    unsafe{PERIPHERALS.set_core_clock()};
    logging::UartLogger::init(log::LevelFilter::Debug);
    log::info!("Initialized logger");
    log::info!("running at exec mode: {:#X}", boot::get_cpu_execution_mode());

    let power_manager = unsafe{PERIPHERALS.take_power()};

    let mut fs = Fat32Fs::new();
    let mut gfx = Ili9341GfxDevice::new(RESET_PIN_BCM, LED_PIN_BCM, TURBO, FRAME_LIMITER);
    let mut pause_menu_gfx = gfx.clone();
    let mut joypad_provider = GpioJoypadProvider::new(button_to_bcm_pin);
    let mut pause_menu_joypad_provider = joypad_provider.clone();
    log::info!("Initialize all drivers succesfully");

    let menu_renderer = joypad_gfx_menu::GfxDeviceMenuRenderer::new(&mut gfx);

    let mut menu_options:[MenuOption<FileEntry, StackString<{FileEntry::FILENAME_SIZE}>>; 255] = [Default::default(); 255];
    let menu_options_size = read_menu_options(&mut fs, &mut menu_options);

    let mut menu = JoypadMenu::new(&menu_options[0..menu_options_size], StackString::from("Choose ROM"), menu_renderer);
    let selected_rom = menu.get_menu_selection(&mut joypad_provider);
    log::info!("Selected ROM: {}", selected_rom.get_name());
    
    let rom = unsafe{&mut ROM_BUFFER};
    fs.read_file(selected_rom, rom);
    let save_data = try_read_save_file(selected_rom, &mut fs);
    let mbc = initialize_mbc(&rom[0..selected_rom.size as usize], save_data, None);

    let mut gameboy = GameBoy::new(mbc, joypad_provider, magenboy_rpi::BlankAudioDevice, gfx, Bootrom::None, None);
    log::info!("Initialized gameboy!");

    let menu_pin = unsafe {PERIPHERALS.get_gpio().take_pin(MENU_PIN_BCM).into_input(GpioPull::PullUp)};
    let pause_menu_header:StackString<30> = StackString::from_args(format_args!("MagenBoy bm v{}", VERSION));
    let pause_menu_renderer = GfxDeviceMenuRenderer::new(&mut pause_menu_gfx);
    let mut pause_menu = JoypadMenu::new(&GAME_MENU_OPTIONS, pause_menu_header.as_str(), pause_menu_renderer);
    loop{
        if !menu_pin.read_state(){
            log::info!("Open pause menu");
            match pause_menu.get_menu_selection(&mut pause_menu_joypad_provider){
                EmulatorMenuOption::Resume => {},
                EmulatorMenuOption::Restart => {
                    log::info!("Reseting system");
                    reset_system(mbc, fs, power_manager, ResetMode::Partition0, selected_rom);
                }
                EmulatorMenuOption::Shutdown => {
                    log::info!("Shuting down system");
                    reset_system(mbc, fs, power_manager, ResetMode::Halt, selected_rom);
                }
            }
        }
        gameboy.cycle_frame();
    }
}

fn reset_system<'a>(mbc: &'a dyn Mbc, mut fs: Fat32Fs, mut power_manager: Power, mode: ResetMode, selected_rom: &FileEntry)->!{
    let filename = get_save_filename(selected_rom);
    fs.write_file(filename.as_str(), mbc.get_ram());

    // delaying the reset operation so other low level tasks will have enough time to finish (like uart transmision)
    delay::wait_ms(100);
    power_manager.reset(mode);
}

fn try_read_save_file(selected_rom: &FileEntry, mut fs: &mut Fat32Fs) -> Option<&'static [u8]> {
    let save_filename = get_save_filename(selected_rom);
    let file = search_file(&mut fs, save_filename.as_str())?;
    let ram = unsafe{&mut RAM_BUFFER[0..file.size as usize]};
    fs.read_file(&file, ram);
    log::info!("Found save file for selected rom: {}", file.get_name());
    return Some(ram);
}

fn get_save_filename(selected_rom: &FileEntry) -> StackString<11> {
    StackString::from_args(format_args!("{}SAV",&selected_rom.get_name()[..8]))
}

fn read_menu_options(fs: &mut Fat32Fs, menu_options: &mut [MenuOption<FileEntry, StackString<{FileEntry::FILENAME_SIZE}>>; 255]) -> usize {
    let mut menu_options_size = 0;
    let mut root_dir_offset = 0;
    const FILES_PER_LIST:usize = 20;
    loop{
        let dir_entries = fs.root_dir_list::<FILES_PER_LIST>(root_dir_offset);
        for entry in &dir_entries{
            let extension = entry.get_extension();
            if extension.eq_ignore_ascii_case("gb") || extension.eq_ignore_ascii_case("gbc"){
                menu_options[menu_options_size] = MenuOption{ value: entry.clone(), prompt: StackString::from(entry.get_name()) };
                menu_options_size += 1;
                log::debug!("Detected ROM: {}", entry.get_name());
            }
        }
        // The fact that its not completely full indicatets that there are no more unread entries left
        if dir_entries.remaining_capacity() != 0{
            break;
        }
        root_dir_offset += FILES_PER_LIST;
    }
    return menu_options_size;
}

fn search_file(fs:&mut Fat32Fs, filename: &str)->Option<FileEntry>{
    let mut root_dir_offset = 0;
    const FILES_PER_LIST:usize = 20;
    loop{
        let dir_entries = fs.root_dir_list::<FILES_PER_LIST>(root_dir_offset);
        for entry in &dir_entries{
            if entry.get_name() == filename{
                return Some(entry.clone());
            }
        }
        if dir_entries.remaining_capacity() != 0{
            return None;
        }
        root_dir_offset += FILES_PER_LIST;
    }
}