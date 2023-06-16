macro_rules! compile_time_size_assert {
    ($t:ty, $size:literal) => {
        const _:[u8;$size] = [0;core::mem::size_of::<$t>()];
    };
}
pub(super) use compile_time_size_assert;

#[repr(transparent)]
pub(super) struct MmioReg32(u32);
impl MmioReg32 {
    #[inline] 
    pub fn read(&self)->u32{
        unsafe{core::ptr::read_volatile(&self.0)}
    }
    #[inline] 
    pub fn write(&mut self, value:u32){
        unsafe{core::ptr::write_volatile(&mut self.0, value)}
    }
}

// According to the docs the raspberrypi requires memory barrier between reads and writes to differnet peripherals 
#[inline] 
pub(super) fn memory_barrier(){
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}

pub enum Peripheral<T>{
    Uninit,
    Init(T),
    Taken
}

impl<T> Peripheral<T>{
    pub(super) fn get(&mut self, init_callback: impl FnOnce()->T)->&mut T{
        if let Self::Uninit = self {
            *self = Self::Init(init_callback());
        }
        return match self{
            Self::Init(t) => t,
            Self::Taken => core::panic!("Peripheral is unavaliable, its been taken "),
            Self::Uninit => core::unreachable!("At this point the peripheral must be initialized"),
        };
    }

    pub(super) fn take(&mut self, init_callback: impl FnOnce()->T)->T{
        let s = core::mem::replace(self, Self::Taken);
        return match s{
            Self::Uninit => init_callback(),
            Self::Init(t) => t,
            Self::Taken => core::panic!("Peripheral is unavaliable, its been taken"),
        };
    }
}


#[cfg(feature = "rpi4")]
pub const PERIPHERALS_BASE_ADDRESS:usize = 0xFE00_0000;
#[cfg(feature = "rpi2")]
pub const PERIPHERALS_BASE_ADDRESS:usize = 0x3F00_0000;

pub(super) fn get_static_peripheral<T>(offset:usize)->&'static mut T{
    #[cfg(feature = "os")]
    unsafe{&mut *(super::bcm_host::BcmHost::get().get_ptr(offset) as *mut T)}
    #[cfg(not(feature = "os"))]
    unsafe{&mut *((offset + PERIPHERALS_BASE_ADDRESS) as *mut T)}
}

#[cfg(feature = "os")]
pub(super) fn libc_abort(message:&str){
    std::io::Result::<&str>::Err(std::io::Error::last_os_error()).expect(message);
}