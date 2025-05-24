use core::{ffi::{c_char, c_int}, fmt::Write};

use log::Log;

use crate::mutex::Mutex;

pub type LogCallback = extern "C" fn(*const c_char, len: c_int) -> ();

struct NxLogCallback{
    cb:LogCallback
}

impl Write for NxLogCallback{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        (self.cb)(s.as_ptr() as *const c_char, s.len() as c_int);
        
        return core::fmt::Result::Ok(());
    }
}

pub struct NxLogger{
    log_cb: Mutex<NxLogCallback>
}


impl NxLogger{
    pub fn init(max_log_level:log::LevelFilter, log_cb: LogCallback){
        static mut LOGGER: Option<NxLogger> = None;
        unsafe{ 
            LOGGER = Some(NxLogger{log_cb: Mutex::new(NxLogCallback{cb: log_cb})});
            log::set_logger(LOGGER.as_ref().unwrap()).expect("Failed to set logger");
        }
        log::set_max_level(max_log_level);
    }
}

impl Log for NxLogger{
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        return metadata.level() <= log::max_level();
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) { return }
        self.log_cb.lock(|d|d.write_fmt(format_args!("{} - {}\r\n", record.level(), record.args())).unwrap());
    }

    fn flush(&self) {}
}