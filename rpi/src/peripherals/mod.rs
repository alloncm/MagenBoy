mod gpio;
mod mini_uart;
mod mailbox;
mod utils;
mod gpu;
mod spi;
mod dma;
mod timer;
#[cfg(feature = "os")]
mod bcm_host;
cfg_if::cfg_if!{ if #[cfg(not(feature = "os"))]{
    mod emmc;
    mod power;
    pub(crate) use utils::compile_time_size_assert;
    pub use utils::PERIPHERALS_BASE_ADDRESS;
    pub use emmc::Emmc;
    pub use power::*;
}}

pub use gpio::*;
pub use mini_uart::MiniUart;
pub use mailbox::*;
pub use timer::*;
pub use spi::Spi0;

use utils::Peripheral;
use crate::configuration::peripherals::*;

pub struct Peripherals{
    gpio_manager: Peripheral<gpio::GpioManager>,
    mini_uart: Peripheral<mini_uart::MiniUart>,
    mailbox: Peripheral<mailbox::Mailbox>,
    timer: Peripheral<Timer>,
    spi0: Peripheral<Spi0>,
    #[cfg(not(feature = "os"))]
    emmc: Peripheral<emmc::Emmc>,
    #[cfg(not(feature = "os"))]
    power: Peripheral<Power>
}

impl Peripherals{
    pub fn set_core_clock(&mut self){
        const CORE_CLOCK_ID:u32 = 4;
        let mbox = self.get_mailbox();
        let result = mbox.call(Tag::SetClockRate, [CORE_CLOCK_ID, CORE_FREQ, 0]);
        if result[0] != CORE_CLOCK_ID || result[1] != CORE_FREQ{
            core::panic!("Error, set core clock failed, \nfreq: {}, clock id: {}", result[1], result[0]);
        }
    }
    pub fn take_mini_uart(&mut self)->mini_uart::MiniUart{
        self.mini_uart.take(||mini_uart::MiniUart::new(MINI_UART_BAUDRATE))
    }
    pub fn get_gpio(&mut self)->&mut gpio::GpioManager{
        self.gpio_manager.get(||GpioManager::new())
    }
    pub fn get_mailbox(&mut self)->&mut mailbox::Mailbox{
        self.mailbox.get(||mailbox::Mailbox::new())
    }
    pub fn take_timer(&mut self)->timer::Timer{
        self.timer.take(||Timer::new())
    }
    pub fn take_spi0(&mut self)->Spi0{
        self.spi0.take(||spi::Spi0::new(SPI0_DC_BCM_PIN))
    }
    #[cfg(not(feature = "os"))]
    pub fn take_emmc(&mut self)->emmc::Emmc{
        self.emmc.take(||emmc::Emmc::new())
    }
    #[cfg(not(feature = "os"))]
    pub fn take_power(&mut self)->Power{
        self.power.take(||Power::new())
    }
}

pub static mut PERIPHERALS: Peripherals = Peripherals{
    gpio_manager: Peripheral::Uninit,
    mini_uart: Peripheral::Uninit,
    mailbox: Peripheral::Uninit,
    timer: Peripheral::Uninit,
    spi0: Peripheral::Uninit,
    #[cfg(not(feature = "os"))]
    emmc: Peripheral::Uninit,
    #[cfg(not(feature = "os"))]
    power: Peripheral::Uninit
};