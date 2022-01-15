use framebuffer::{Framebuffer, KdMode};
use lib_gb::ppu::{gfx_device::GfxDevice, gb_ppu::{SCREEN_WIDTH, SCREEN_HEIGHT}};

pub struct LinuxFbGfxDevice{
    framebuffer: Framebuffer,
    buffer:[u8;1228800]
}

impl LinuxFbGfxDevice{
    pub fn new(framebuffer_path:&str)->Self{
        let mut fb = Framebuffer::new(framebuffer_path).unwrap();

        fb.var_screen_info.yres_virtual = SCREEN_HEIGHT as u32;
        fb.var_screen_info.xres_virtual = SCREEN_WIDTH as u32;
        fb.var_screen_info.yres = SCREEN_HEIGHT as u32;
        fb.var_screen_info.xres = SCREEN_WIDTH as u32;

        Self{
            buffer:[0;1228800],
            framebuffer:fb
        }
    }
}

impl GfxDevice for LinuxFbGfxDevice{
    fn swap_buffer(&mut self, buffer:&[u32; SCREEN_HEIGHT * SCREEN_WIDTH]) {
        unsafe{
            let slice = std::slice::from_raw_parts(buffer as *const u32 as *const u8,SCREEN_HEIGHT*SCREEN_WIDTH*4);
            std::ptr::copy_nonoverlapping(slice.as_ptr(), self.buffer.as_mut_ptr(), slice.len());
            self.framebuffer.write_frame(&self.buffer);
        }
    }
}