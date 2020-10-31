pub trait AudioDevice{
    fn push_buffer(&self, buffer:&[f32]);
}