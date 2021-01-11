pub trait AudioDevice{
    fn push_buffer(&mut self, buffer:&[f32]);
}