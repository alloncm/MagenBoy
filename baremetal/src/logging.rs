use core::fmt::Write;

use crate::{peripherals::{MiniUart, PERIPHERALS}, syncronization::Mutex};

use log::{Record, Metadata, Log, LevelFilter};

struct UartDevice(MiniUart);
impl Write for UartDevice{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.send(s.as_bytes());
        return core::fmt::Result::Ok(());
    }
}

pub struct UartLogger{
    uart_mutex:Mutex<UartDevice>
}

static mut LOGGER:Option<UartLogger> = None;

impl UartLogger{
    pub fn init(max_log_level:LevelFilter){
        let uart = unsafe{PERIPHERALS.take_mini_uart()};
        let logger:&'static UartLogger = unsafe{
            LOGGER = Some(UartLogger{uart_mutex:Mutex::new(UartDevice(uart))});
            LOGGER.as_ref().unwrap()
        };

        // On armv7a calling set_logger corrupt the program and make it unresponsive (not even panicking) 
        // I dont know why but I believe its due to the MMU not being turned on
        // Now that I turned it on I believe this should work, in case it hangs again use log::set_logger_racy
        if let Err(error_message) = log::set_logger(logger){
            logger.uart_mutex.lock(|u|u.write_fmt(format_args!("{}", error_message)).unwrap());
            core::panic!("Error initializng logger");
        }
        log::set_max_level(max_log_level);
    }
}

impl Log for UartLogger{
    fn enabled(&self, metadata: &Metadata) -> bool {
        unsafe{LOGGER.is_some() && metadata.level() >= log::max_level()}
    }

    fn log(&self, record: &Record) {
        let level = record.level();
        let log_message = *record.args();
        self.uart_mutex.lock(|uart|uart.write_fmt(format_args!("{} - {}\n\r", level, log_message)).unwrap());
    }

    fn flush(&self) {}
}