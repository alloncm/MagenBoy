use lib_gb::apu::audio_device::{BUFFER_SIZE, DEFAULT_SAPMPLE, Sample, StereoSample};
use super::audio_resampler::AudioResampler;


pub struct ManualAudioResampler{
    to_skip:u32,
    sampling_buffer:Vec<StereoSample>,
    sampling_counter:u32,
    reminder_steps:f32,
    reminder_counter:f32,
    alternate_to_skip:u32,
    skip_to_use:u32,
}

impl ManualAudioResampler{
    fn interpolate_sample(samples:&[StereoSample])->StereoSample{
        let interpulated_left_sample = samples.iter().fold(DEFAULT_SAPMPLE, |acc, x| acc + x.left_sample) / samples.len() as Sample;
        let interpulated_right_sample = samples.iter().fold(DEFAULT_SAPMPLE, |acc, x| acc + x.right_sample) / samples.len() as Sample;

        return StereoSample{left_sample: interpulated_left_sample, right_sample: interpulated_right_sample};
    }
}

impl AudioResampler for ManualAudioResampler{
    fn new(original_frequency:u32, target_frequency:u32)->Self{
        // Calling round in order to get the nearest integer and resample as precise as possible
        let div = original_frequency as f32 /  target_frequency as f32;

        let lower_to_skip = div.floor() as u32;
        let upper_to_skip = div.ceil() as u32;
        let mut reminder = div.fract();
        let (to_skip, alt_to_skip) = if reminder < 0.5{
            (lower_to_skip, upper_to_skip)
        }
        else{
            reminder = 1.0 - reminder;
            (upper_to_skip, lower_to_skip)
        };

        if lower_to_skip == 0{
            std::panic!("target freqency is too high: {}", target_frequency);
        }

        ManualAudioResampler{
            to_skip:to_skip,
            sampling_buffer:Vec::with_capacity(upper_to_skip as usize),
            sampling_counter: 0,
            reminder_steps:reminder,
            reminder_counter:0.0,
            alternate_to_skip: alt_to_skip,
            skip_to_use:to_skip
        }
    }

    fn resample(&mut self, buffer:&[StereoSample; BUFFER_SIZE])->Vec<StereoSample>{
        let mut output = Vec::new();
        for sample in buffer.into_iter(){
            self.sampling_buffer.push(sample.clone());
            self.sampling_counter += 1;
    
            if self.sampling_counter == self.skip_to_use {
                let interpolated_sample = Self::interpolate_sample(&self.sampling_buffer);
                self.sampling_counter = 0;
                self.sampling_buffer.clear();

                output.push(interpolated_sample);
                if self.reminder_counter >= 1.0{
                    self.skip_to_use = self.alternate_to_skip;
                    self.reminder_counter -= 1.0;
                }
                else{
                    self.skip_to_use = self.to_skip;
                    self.reminder_counter += self.reminder_steps;
                }
            }
        }

        return output;
    }
}