pub type Sample = i16;
pub const DEFAULT_SAPMPLE:Sample = 0 as Sample;

pub const BUFFER_SIZE:usize = 2048;

#[repr(C, packed)]
pub struct StereoSample{
    pub left_sample:Sample,
    pub right_sample:Sample
}

impl StereoSample{
    pub const fn const_defualt()->Self{
        Self{left_sample:DEFAULT_SAPMPLE, right_sample:DEFAULT_SAPMPLE}
    }
}

impl Clone for StereoSample{
    fn clone(&self) -> Self {
        Self{left_sample:self.left_sample,right_sample:self.right_sample}
    }
}

pub trait AudioDevice{
    fn push_buffer(&mut self, buffer:&[StereoSample; BUFFER_SIZE]);
}