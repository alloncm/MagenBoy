pub trait GfxDevice{
    fn swap_buffer(&self, buffer:&[u32]);
}