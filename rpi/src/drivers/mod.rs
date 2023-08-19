mod gpio_joypad;
mod ili9341_gfx_device;
cfg_if::cfg_if!{ if #[cfg(not(feature = "os"))]{
    pub(super) mod disk;
    mod fat32;
    pub use fat32::*;
}}

pub use gpio_joypad::*;
pub use ili9341_gfx_device::*;


#[cfg(not(feature = "os"))]
pub(crate) unsafe fn as_mut_buffer<'a, T>(t:&'a mut T)->&'a mut [u8]{
    let buffer = &mut *core::ptr::slice_from_raw_parts_mut(t as *mut T as *mut _, core::mem::size_of::<T>());
    return buffer;
}