#![no_std]

use core::{cell::OnceCell, ffi::{c_char, c_ulonglong, c_void}, panic};

use magenboy_core::{machine, GameBoy, JoypadProvider, GfxDevice, AudioDevice, Mode};
use nx::{util, diag::abort};
use nx::diag::log;

struct NxJoypadProvider;

impl JoypadProvider for NxJoypadProvider {
    fn provide(&mut self, joypad: &mut magenboy_core::keypad::joypad::Joypad) {
        // TODO: implement
    }
}

struct NxGfxDevice;

impl GfxDevice for NxGfxDevice{
    fn swap_buffer(&mut self, buffer:&[magenboy_core::Pixel; magenboy_core::ppu::gb_ppu::SCREEN_HEIGHT * magenboy_core::ppu::gb_ppu::SCREEN_WIDTH]) {
        // TODO: implement
    }
}

struct NxAudioDevice;

impl AudioDevice for NxAudioDevice{
    fn push_buffer(&mut self, buffer:&[magenboy_core::apu::audio_device::StereoSample; magenboy_core::apu::audio_device::BUFFER_SIZE]) {
        // TODO: implement
    }
}

// Exported C interface for nx

/// SAFETY: rom size must be the size of rom
#[no_mangle]
pub unsafe extern "C" fn magenboy_init(rom: *const c_char, rom_size: c_ulonglong, ) -> *mut c_void {

    // SAFETY: it is a requirement that rom must be valid for the entire program (static lifetime)
    let rom:&[u8] = unsafe{ core::slice::from_raw_parts(rom as *const u8, rom_size as usize) };
    let mbc = machine::mbc_initializer::initialize_mbc(&rom, None);
    
    // Initialize the GameBoy instance
    let gameboy = GameBoy::new_with_mode(
        mbc,
        NxJoypadProvider,
        NxAudioDevice,
        NxGfxDevice,
        Mode::DMG,
    );

    // Allocate on static memory
    let static_gameboy = magenboy_core::utils::global_static_alloctor::static_alloc(gameboy);
    
    return static_gameboy as *mut _ as *mut c_void;
}


/// SAFETY: ctx is a valid pointer to a GameBoy instance
#[no_mangle]
pub unsafe extern "C" fn magenboy_cycle_frame(ctx: *mut c_void) {
    // SAFETY: ctx is a valid pointer to a GameBoy instance
    unsafe {
        (*(ctx as *mut GameBoy<NxJoypadProvider, NxAudioDevice, NxGfxDevice>)).cycle_frame()
    }
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<log::lm::LmLogger>(info, abort::AbortLevel::FatalThrow())
}