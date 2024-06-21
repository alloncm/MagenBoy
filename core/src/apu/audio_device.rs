use super::NUMBER_OF_CHANNELS;

pub type Sample = i16;
pub const DEFAULT_SAPMPLE:Sample = 0 as Sample;
const MAX_MASTER_VOLUME:Sample = 8;
pub const SAMPLE_MAX: Sample = Sample::MAX / (MAX_MASTER_VOLUME * NUMBER_OF_CHANNELS as Sample);

pub const BUFFER_SIZE:usize = 0x2000;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct StereoSample{
    pub left_sample:Sample,
    pub right_sample:Sample
}

impl StereoSample{
    pub const fn const_defualt()->Self{
        Self{left_sample:DEFAULT_SAPMPLE, right_sample:DEFAULT_SAPMPLE}
    }

    pub fn interpolate(samples:&[StereoSample])->StereoSample{
        let left_sample = (samples.iter().fold(0, |acc, x| acc + x.left_sample as i64) / samples.len() as i64) as Sample;
        let right_sample = (samples.iter().fold(0, |acc, x| acc + x.right_sample as i64) / samples.len() as i64) as Sample;

        return StereoSample{left_sample, right_sample};
    }
}

pub trait AudioDevice{
    fn push_buffer(&mut self, buffer:&[StereoSample; BUFFER_SIZE]);
}