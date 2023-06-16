#![no_main]
#![no_std]

mod boot;
mod logging;

use core::panic::PanicInfo;

use magenboy_core::{machine::{gameboy::GameBoy, mbc_initializer::initialize_mbc}, mmu::external_memory_bus::Bootrom};

use magenboy_rpi::{drivers::{GpioJoypadProvider, Ili9341GfxDevice}, peripherals::PERIPHERALS, configuration::{display::*, joypad::button_to_bcm_pin, emulation::*}};

// This function is no regular main.
// It will not return and will be jumped to from the _start proc in the boot code
// it is unmangled and exposed as a "C" function in order for the _start proc to call it
#[no_mangle]
pub extern "C" fn main()->!{
    unsafe{PERIPHERALS.set_core_clock()};
    logging::UartLogger::init(log::LevelFilter::Debug);
    log::info!("Initialized logger");
    log::info!("running at exec mode: {:#X}", boot::get_cpu_execution_mode());

    let mbc = initialize_mbc(ROM, None, None);
    let joypad_provider = GpioJoypadProvider::new(button_to_bcm_pin);

    let gfx = Ili9341GfxDevice::new(RESET_PIN_BCM, LED_PIN_BCM, TURBO, FRAME_LIMITER);
    log::info!("Init joypad");
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