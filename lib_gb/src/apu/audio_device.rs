pub type Sample = f32;
pub const SAMPLE_CONSTANT_DEFAULT:Sample = 0.0;

#[derive(Copy, Clone)]
pub struct StereoSample{
    pub left_sample:Sample,
    pub right_sample:Sample
}

impl StereoSample{
    pub const fn const_defualt()->Self{
        Self{left_sample:SAMPLE_CONSTANT_DEFAULT, right_sample:SAMPLE_CONSTANT_DEFAULT}
    }
}

pub trait AudioDevice{
    fn push_buffer(&mut self, buffer:&[StereoSample]);
}