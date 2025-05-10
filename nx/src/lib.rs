#![no_std]

extern crate alloc;

mod mutex;
mod logging;
mod devices;
mod allocator;

use core::{ffi::{c_char, c_ulonglong, c_void}, panic};

use magenboy_common::audio::*;
use magenboy_core::{machine, GameBoy, Mode, GB_FREQUENCY};

use devices::*;
use logging::{LogCallback, NxLogger};

#[global_allocator]
static ALLOCATOR: allocator::NxAllocator = allocator::NxAllocator{};

// Exported C interface for nx

/// SAFETY: rom size must be the size of rom
#[no_mangle]
pub unsafe extern "C" fn magenboy_init(rom: *const c_char, rom_size: c_ulonglong, gfx_cb: GfxDeviceCallback, joypad_cb: JoypadProviderCallback, audio_cb:AudioDeviceCallback, log_cb: LogCallback) -> *mut c_void {
    NxLogger::init(log::LevelFilter::Debug, log_cb);

    let rom:&[u8] = unsafe{ core::slice::from_raw_parts(rom as *const u8, rom_size as usize) };
    let mbc = machine::mbc_initializer::initialize_mbc(&rom, None);
    
    // Initialize the GameBoy instance
    let gameboy = GameBoy::new_with_mode(
        mbc,
        NxJoypadProvider{cb: joypad_cb},
        NxAudioDevice{cb: audio_cb, resampler: ManualAudioResampler::new(GB_FREQUENCY, 48000)},
        NxGfxDevice {cb: gfx_cb},
        Mode::DMG,
    );

    // Allocate on static memory
    let static_gameboy = magenboy_core::utils::global_static_alloctor::static_alloc(gameboy);
    log::info!("Initialized MagenBoy successfully");
    return static_gameboy as *mut _ as *mut c_void;
}


/// SAFETY: ctx is a valid pointer to a GameBoy instance
#[no_mangle]
pub unsafe extern "C" fn magenboy_cycle_frame(ctx: *mut c_void) {
    // SAFETY: ctx is a valid pointer to a GameBoy instance
    unsafe {
        (*(ctx as *mut GameBoy<devices::NxJoypadProvider, devices::NxAudioDevice, devices::NxGfxDevice>)).cycle_frame()
    }
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    log::error!("Panic: {}", info);
    loop{}
}