pub type Sample = f32;
pub const DEFAULT_SAPMPLE:Sample = 0 as Sample;

#[derive(Copy, Clone)]
pub struct StereoSample{
    pub left_sample:Sample,
    pub right_sample:Sample
}

pub trait AudioDevice{
    fn push_buffer(&mut self, buffer:&[StereoSample]);
}