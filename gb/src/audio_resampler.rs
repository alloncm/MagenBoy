use lib_gb::apu::audio_device::{DEFAULT_SAPMPLE, Sample, StereoSample};

pub struct AudioResampler{
    to_skip:u32,
    sampling_buffer:Vec<StereoSample>,
    sampling_counter:u32
}

impl AudioResampler{
    pub fn new(original_frequency:u32, target_frequency:u32)->Self{
        // Calling round in order to get the nearest integer and resample as precise as possible
        let to_skip = (original_frequency as f32 / target_frequency as f32).round() as u32;
        if to_skip == 0{
            std::panic!("target freqency is too high: {}", target_frequency);
        }

        AudioResampler{
            to_skip:to_skip,
            sampling_buffer:Vec::with_capacity(to_skip as usize),
            sampling_counter: 0
        }
    }

    pub fn resample(&mut self, buffer:&[StereoSample])->Vec<StereoSample>{
        let mut output = Vec::new();
        for sample in buffer.into_iter(){
            self.sampling_buffer.push(*sample);
            self.sampling_counter += 1;
    
            if self.sampling_counter == self.to_skip {
                let interpolated_sample = Self::interpolate_sample(&self.sampling_buffer);
                self.sampling_counter = 0;
                self.sampling_buffer.clear();

                output.push(interpolated_sample);
            }
        }

        return output;
    }
    
    fn interpolate_sample(samples:&[StereoSample])->StereoSample{

        let interpulated_left_sample = samples.iter().fold(DEFAULT_SAPMPLE, |acc, x| acc + x.left_sample) / samples.len() as Sample;
        let interpulated_right_sample = samples.iter().fold(DEFAULT_SAPMPLE, |acc, x| acc + x.right_sample) / samples.len() as Sample;

        return StereoSample{left_sample: interpulated_left_sample, right_sample: interpulated_right_sample};
    }
}