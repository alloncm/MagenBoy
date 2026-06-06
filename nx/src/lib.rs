#![no_std]

extern crate alloc;

mod logging;
mod devices;
mod allocator;

use core::{ffi::{c_char, c_ulonglong, c_void, CStr}, panic};
use alloc::{vec::Vec, boxed::Box, string::String};

use magenboy_common::{audio::*, joypad_menu::{JoypadMenu, MenuResult}, menu::{MenuOption, GAME_MENU_OPTIONS}, VERSION};
use magenboy_core::{machine, GameBoy, Mode, GB_FREQUENCY};

use devices::*;
use logging::{LogCallback, NxLogger};

const TURBO: u32 = 2;

struct NxGbContext<'a>{
    gb: GameBoy<'a, NxAudioDevice>,
    joypad_device: NxJoypadProvider,
    renderer: NxGfxDevice,
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
pub unsafe extern "C" fn magenboy_init(
    rom: *const c_char,
    rom_size: c_ulonglong,
    gfx_cb: SwapBufferCallback,
    gl_loader_calback: GlLoaderCallback,
    window_width: core::ffi::c_uint,
    window_height: core::ffi::c_uint,
    joypad_cb: JoypadProviderCallback,
    poll_joypad_cb: PollJoypadProviderCallback,
    audio_cb:AudioDeviceCallback
) -> *mut c_void {
    let rom:&[u8] = unsafe{ core::slice::from_raw_parts(rom as *const u8, rom_size as usize) };
    let mbc = machine::mbc_initializer::initialize_mbc(&rom, None);

    let mode = mbc.detect_preferred_mode();
    log::info!("Detected mode: {}", <Mode as Into<&str>>::into(mode));

    let sram_fat_pointer = (mbc.get_ram().as_mut_ptr(), mbc.get_ram().len());

    // Initialize the GameBoy instance
    let gameboy = GameBoy::new_with_mode(
        mbc,
        NxAudioDevice{cb: audio_cb, resampler: ManualAudioResampler::new(GB_FREQUENCY * TURBO, 48000)},
        mode,
    );

    let joypad_provider = NxJoypadProvider{
        poll_cb: poll_joypad_cb,
        provider_cb: joypad_cb
    };
    let render_device = NxGfxDevice::new(window_width, window_height, gl_loader_calback, gfx_cb, 1);

    let ctx = NxGbContext {
        gb: gameboy,
        renderer: render_device,
        joypad_device: joypad_provider,
        sram_fat_pointer
    };

    // Allocate on static memory
    let gameboy = Box::new(ctx);
    log::info!("Initialized MagenBoy successfully");
    return Box::into_raw(gameboy) as *mut c_void;
}

#[no_mangle]
pub unsafe extern "C" fn magenboy_deinit(ctx: *mut c_void) {
    // SAFETY: ctx is a valid pointer to a GameBoy instance
    if ctx.is_null() { 
        log::warn!("Attempted to deinitialize MagenBoy with a null context pointer");
        return; 
    }

    let _ = unsafe { Box::from_raw(ctx as *mut NxGbContext) }; // Drop the Box to deallocate memory
    log::info!("MagenBoy deinitialized successfully");
}


#[no_mangle]
pub unsafe extern "C" fn magenboy_menu_trigger(
    gfx_cb: SwapBufferCallback,
    joypad_cb: JoypadProviderCallback,
    poll_joypad_cb: PollJoypadProviderCallback,
    gl_loader_calback: GlLoaderCallback,
    window_width: core::ffi::c_uint,
    window_height: core::ffi::c_uint,
    roms: *const *const c_char,
    roms_count: u32
) -> *const c_char {    
    log::info!("Starting ROM menu");

    // SAFETY: roms is a valid c strings array
    let roms: Vec<MenuOption<&CStr, &str>> = unsafe {
        let mut roms_vec = Vec::with_capacity(roms_count as usize);
        for i in 0..roms_count {
            let rom_name = *(roms.add(i as usize));
            let c_str = CStr::from_ptr(rom_name as *mut c_char);
            roms_vec.push(MenuOption{value: c_str, prompt: filename_from_path(c_str.to_str().unwrap())});
        }
        roms_vec
    };

    let mut gfx_device = NxGfxDevice::new(window_width, window_height, gl_loader_calback, gfx_cb, 1);
    let mut provider = NxJoypadProvider{provider_cb: joypad_cb, poll_cb: poll_joypad_cb};
    
    let selection = render_menu(
        &mut provider,
        &mut gfx_device,
        &roms,
        "Choose ROM menu"
    );

    return selection.as_ptr();
}

#[no_mangle]
pub unsafe extern "C" fn magenboy_pause_trigger(ctx: *mut c_void,) -> u32 {
    log::info!("Starting pause menu");
    let header: String = alloc::format!("Magenboy {VERSION}");
    let ctx = ctx as *mut NxGbContext;
    let joypad_provider = &mut (*ctx).joypad_device;
    let gfx_device = &mut (*ctx).renderer;

    let selection= render_menu(
        joypad_provider,
        gfx_device,
        &GAME_MENU_OPTIONS,
        header.as_str()
    );
    return *selection as u32;
}

fn render_menu<'a, T>(
    joypad_provider: &mut NxJoypadProvider,
    gfx_device: &mut NxGfxDevice,
    options: &'a [MenuOption<T, &str>],
    header: &'a str
) -> &'a T {
    let mut menu = JoypadMenu::new(&options, header);

    let menu_selection: &T;
    let mut joypad = joypad_provider.provide();
    loop {
        match menu.try_get_menu_selection(joypad) {
            MenuResult::Selection(sel) => {
                menu_selection = sel;
                break;
            },
            MenuResult::Frame(frame) => {
                gfx_device.swap_buffer(&frame);
            },
        }
        joypad = joypad_provider.poll();
    }
    return menu_selection
}

/// SAFETY: ctx is a valid pointer to a GameBoy instance
#[no_mangle]
pub unsafe extern "C" fn magenboy_cycle_frame(ctx: *mut c_void) {
    // SAFETY: ctx is a valid pointer to a GameBoy instance
    unsafe {
        let ctx = ctx as *mut NxGbContext;
        let joypad = (*ctx).joypad_device.provide();
        let frame = (*ctx).gb.cycle_frame(joypad);
        (*ctx).renderer.swap_buffer(frame);
    }
}

#[no_mangle]
pub unsafe extern "C" fn magenboy_get_sram(ctx: *mut c_void, ptr: *mut *mut u8, size: *mut usize){
    let sram_fat_ptr = (*(ctx as *mut NxGbContext)).sram_fat_pointer;
    *ptr = sram_fat_ptr.0;
    *size = sram_fat_ptr.1;
}

fn filename_from_path(path: &str) -> &str {
    match path.rfind(|c| c == '/') {
        Some(pos) => &path[pos + 1..],
        None => path,
    }
}