#[derive(Copy, Clone)]
pub struct Sample{
    pub left_sample:f32,
    pub right_sample:f32
}

pub trait AudioDevice{
    fn push_buffer(&mut self, buffer:&[Sample]);
}