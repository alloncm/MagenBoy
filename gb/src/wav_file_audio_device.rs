use lib_gb::apu::audio_device::*;

use crate::audio_resampler::AudioResampler;

pub struct WavfileAudioDevice{
    target_frequency:u32,
    resampler: AudioResampler,
    filename:&'static str,
    samples_buffer:Vec::<StereoSample>
}

impl WavfileAudioDevice{
    pub fn new(target_freq:u32, original_freq:u32, filename:&'static str)->Self{
        WavfileAudioDevice{
            filename,
            resampler: AudioResampler::new(original_freq, target_freq),
            samples_buffer: Vec::new(),
            target_frequency: target_freq
        }
    }
}

impl AudioDevice for WavfileAudioDevice{
    fn push_buffer(&mut self, buffer:&[StereoSample]) {
        self.samples_buffer.append(self.resampler.resample(buffer).as_mut());
    }
}

impl Drop for WavfileAudioDevice{
    fn drop(&mut self) {
        let header = wav::header::Header::new(wav::WAV_FORMAT_PCM, 2, self.target_frequency, 16);
        let mut samples = Vec::with_capacity(self.samples_buffer.len() * 2);
        for sample in self.samples_buffer.iter(){
            samples.push(sample.left_sample);
            samples.push(sample.right_sample);
        }

        let data = wav::BitDepth::Sixteen(samples);
        let mut otuput_file = std::fs::File::create(self.filename).unwrap();
        wav::write(header, &data, &mut otuput_file).unwrap();
    }
}