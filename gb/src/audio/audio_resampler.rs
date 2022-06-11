use lib_gb::apu::audio_device::{Sample, AudioDevice, StereoSample, BUFFER_SIZE};

pub trait AudioResampler{
    fn new(original_frequency:u32, target_frequency:u32)->Self;
    fn resample(&mut self, buffer:&[StereoSample; BUFFER_SIZE])->Vec<StereoSample>;
}

pub trait ResampledAudioDevice<AR:AudioResampler> : AudioDevice{
    const VOLUME:Sample = 10 as Sample;

    fn push_buffer(&mut self, buffer:&[StereoSample; BUFFER_SIZE]){
        let resample = self.get_resampler().resample(buffer);
        for sample in resample{
            let(buffer, index) = self.get_audio_buffer();
            buffer[*index] = sample.left_sample * Self::VOLUME;
            buffer[*index + 1] = sample.left_sample * Self::VOLUME;
            *index += 2;
            if *index == BUFFER_SIZE{
                *index = 0;
                self.full_buffer_callback().unwrap();
            }
        }
    }

    fn get_audio_buffer(&mut self)->(&mut [Sample;BUFFER_SIZE], &mut usize);
    fn get_resampler(&mut self)->&mut AR;
    fn full_buffer_callback(&mut self)->Result<(), String>;
    fn new(frequency:i32, turbo_mul:u8)->Self;
}
