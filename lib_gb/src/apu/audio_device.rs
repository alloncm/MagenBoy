pub type Sample = i16;
pub const DEFAULT_SAPMPLE:Sample = 0 as Sample;

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