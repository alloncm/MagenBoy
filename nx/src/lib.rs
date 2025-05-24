#![no_std]

extern crate alloc;

mod mutex;
mod logging;
mod devices;
mod allocator;

use core::{ffi::{c_char, c_ulonglong, c_void, CStr}, panic};
use alloc::vec::Vec;

use magenboy_common::{audio::*, joypad_menu::{joypad_gfx_menu::GfxDeviceMenuRenderer, JoypadMenu}, menu::MenuOption};
use magenboy_core::{machine, GameBoy, Mode, GB_FREQUENCY};

use devices::*;
use logging::{LogCallback, NxLogger};

struct NxGbContext<'a>{
    gb: GameBoy<'a, NxJoypadProvider, NxAudioDevice, NxGfxDevice>,
    sram_fat_pointer: (*mut u8, usize)
}

#[global_allocator]
static ALLOCATOR: allocator::NxAllocator = allocator::NxAllocator{};

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    log::error!("Panic: {}", info);
    loop{}
}

// Exported C interface for nx

#[no_mangle]
pub unsafe extern "C" fn magenboy_init_logger(log_cb: LogCallback) {
    // SAFETY: log_cb is a valid c function pointer
    NxLogger::init(log::LevelFilter::Debug, log_cb);
}

/// SAFETY: rom size must be the size of rom
#[no_mangle]
pub unsafe extern "C" fn magenboy_init(rom: *const c_char, rom_size: c_ulonglong, gfx_cb: GfxDeviceCallback, joypad_cb: JoypadProviderCallback, 
    poll_joypad_cb: PollJoypadProviderCallback, audio_cb:AudioDeviceCallback) -> *mut c_void {

    let rom:&[u8] = unsafe{ core::slice::from_raw_parts(rom as *const u8, rom_size as usize) };
    let mbc = machine::mbc_initializer::initialize_mbc(&rom, None);

    let mode = mbc.detect_preferred_mode();
    log::info!("Detected mode: {}", <Mode as Into<&str>>::into(mode));

    let sram_fat_pointer = (mbc.get_ram().as_mut_ptr(), mbc.get_ram().len());

    // Initialize the GameBoy instance
    let gameboy = GameBoy::new_with_mode(
        mbc,
        NxJoypadProvider{provider_cb: joypad_cb, poll_cb: poll_joypad_cb},
        NxAudioDevice{cb: audio_cb, resampler: ManualAudioResampler::new(GB_FREQUENCY, 48000)},
        NxGfxDevice {cb: gfx_cb},
        mode,
    );

    let ctx = NxGbContext {gb: gameboy, sram_fat_pointer };

    // Allocate on static memory
    let static_gameboy = magenboy_core::utils::global_static_alloctor::static_alloc(ctx);
    log::info!("Initialized MagenBoy successfully");
    return static_gameboy as *mut _ as *mut c_void;
}

#[no_mangle]
pub unsafe extern "C" fn magenboy_menu_trigger(gfx_cb: GfxDeviceCallback, joypad_cb: JoypadProviderCallback, poll_joypad_cb: PollJoypadProviderCallback, 
    roms: *const *const c_char, roms_count: u32) -> *const c_char {
    
    log::info!("Starting ROM menu");

    // SAFETY: roms is a valid c strings array
    let roms: Vec<MenuOption<&CStr, &str>> = unsafe {
        let mut roms_vec = Vec::with_capacity(roms_count as usize);
        for i in 0..roms_count {
            let rom_name = *(roms.add(i as usize));
            let c_str = CStr::from_ptr(rom_name as *mut c_char);
            roms_vec.push(MenuOption{value: c_str, prompt: c_str.to_str().unwrap()});
        }
        roms_vec
    };
    
    let mut gfx_device = NxGfxDevice {cb: gfx_cb};
    let menu_renderer = GfxDeviceMenuRenderer::new(&mut gfx_device);
    let mut provider = NxJoypadProvider{provider_cb: joypad_cb, poll_cb: poll_joypad_cb};
    let mut menu = JoypadMenu::new(&roms, "Choose ROM", menu_renderer);
    let selection= menu.get_menu_selection(&mut provider);

    return selection.as_ptr();
}

/// SAFETY: ctx is a valid pointer to a GameBoy instance
#[no_mangle]
pub unsafe extern "C" fn magenboy_cycle_frame(ctx: *mut c_void) {
    // SAFETY: ctx is a valid pointer to a GameBoy instance
    unsafe {
        (*(ctx as *mut NxGbContext)).gb.cycle_frame()
    }
}

#[no_mangle]
pub unsafe extern "C" fn magenboy_get_dimensions(width: *mut u32, height: *mut u32) {
    // SAFETY: width and height are valid pointers to uint32_t
    unsafe {
        *width = magenboy_core::ppu::gb_ppu::SCREEN_WIDTH as u32;
        *height = magenboy_core::ppu::gb_ppu::SCREEN_HEIGHT as u32;
    }
}

#[no_mangle]
pub unsafe extern "C" fn magenboy_get_sram(ctx: *mut c_void, ptr: *mut *mut u8, size: *mut usize){
    let sram_fat_ptr = (*(ctx as *mut NxGbContext)).sram_fat_pointer;
    *ptr = sram_fat_ptr.0;
    *size = sram_fat_ptr.1;
}