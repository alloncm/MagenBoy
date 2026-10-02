use magenboy_core::apu::audio_device::StereoSample;

pub trait AudioResampler{
    fn new(original_frequency:u32, target_frequency:u32)->Self;
    fn resample(&mut self, sample: StereoSample)-> Option<StereoSample>;
}
