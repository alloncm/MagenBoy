use lib_gb::ppu::{gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, gfx_device::{GfxDevice, Pixel}};

pub struct MpmcGfxDevice{
    sender: crossbeam_channel::Sender<usize>
}

impl MpmcGfxDevice{
    pub fn new(sender:crossbeam_channel::Sender<usize>)->Self{
        Self{sender}
    }
}

impl GfxDevice for MpmcGfxDevice{
    fn swap_buffer(&mut self, buffer:&[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH]) {
        if self.sender.send(buffer.as_ptr() as usize).is_err(){
            log::debug!("The receiver endpoint has been closed");
        }
    }
}