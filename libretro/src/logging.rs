use std::{ffi::{c_char, CString} ,sync::OnceLock};

use libretro_sys::*;
use log::Log;

pub struct RetroLogger{
    retro_logger: LogCallback
}

impl RetroLogger{
    pub fn init(max_log_level:log::LevelFilter, log_cb: Option<LogCallback>){
        static LOGGER: OnceLock<RetroLogger> = OnceLock::new();
        let logger = match log_cb{
            Some(cb) => LOGGER.get_or_init(||Self{ retro_logger: cb}),
            None => LOGGER.get_or_init(||Self { retro_logger: LogCallback{log: logcb_fallback} }),
        };
        log::set_logger(logger).expect("Failed to set logger");
        log::set_max_level(max_log_level);
    }
}

impl Log for RetroLogger{
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        return metadata.level() <= log::max_level();
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {return}
        let level = match record.level(){
            log::Level::Error   => LogLevel::Error,
            log::Level::Warn    => LogLevel::Warn,
            log::Level::Info    => LogLevel::Info,
            log::Level::Debug |
            log::Level::Trace   => LogLevel::Debug,
        };
        let message = CString::new(format!("{}\n", record.args())).unwrap();
        unsafe{(self.retro_logger.log)(level, message.as_ptr())};
    }

    fn flush(&self) {}
}

// Empty callback in case there is no logbc on the platform
// According to the docs we can also write to the stderr instead
unsafe extern "C" fn logcb_fallback(_: LogLevel, _: *const c_char){}