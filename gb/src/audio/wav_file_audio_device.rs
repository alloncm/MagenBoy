use lib_gb::apu::audio_device::*;

use super::audio_resampler::AudioResampler;

pub struct WavfileAudioDevice<AR:AudioResampler>{
    target_frequency:u32,
    resampler: AR,
    filename:&'static str,
    samples_buffer:Vec::<StereoSample>
}

impl<AR:AudioResampler> WavfileAudioDevice<AR>{
    pub fn new(target_freq:u32, original_freq:u32, filename:&'static str)->Self{
        WavfileAudioDevice{
            filename,
            resampler: AudioResampler::new(original_freq, target_freq),
            samples_buffer: Vec::new(),
            target_frequency: target_freq
        }
    }
}

impl<AR:AudioResampler> AudioDevice for WavfileAudioDevice<AR>{
    fn push_buffer(&mut self, buffer:&[StereoSample; BUFFER_SIZE]) {
        self.samples_buffer.append(self.resampler.resample(buffer).as_mut());
    }
}

impl<AR:AudioResampler> Drop for WavfileAudioDevice<AR>{
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