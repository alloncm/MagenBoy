use alloc::vec::Vec;

use magenboy_core::apu::audio_device::{BUFFER_SIZE, StereoSample};

use super::audio_resampler::AudioResampler;

pub struct ManualAudioResampler{
    target_frequency:u32,
    original_frequency:u32,

    to_skip:u32,
    sampling_buffer:Vec<StereoSample>,
    sampling_counter:u32,
    reminder_steps:f32,
    reminder_counter:f32,
    alternate_to_skip:u32,
    skip_to_use:u32,
}

impl AudioResampler for ManualAudioResampler{
    fn new(original_frequency:u32, target_frequency:u32)->Self{
        
        let mut resampler = ManualAudioResampler{
            target_frequency,
            original_frequency: 0, // Will be set in set_original_frequency
            to_skip: 0,
            sampling_buffer: Vec::new(),
            sampling_counter: 0,
            reminder_steps: 0.0,
            reminder_counter: 0.0,
            alternate_to_skip: 0,
            skip_to_use: 0,
        };

        resampler.set_original_frequency(original_frequency);

        return resampler;
    }

    fn resample(&mut self, buffer:&[StereoSample; BUFFER_SIZE])->Vec<StereoSample>{
        let mut output = Vec::new();
        for sample in buffer.into_iter(){
            self.sampling_buffer.push(sample.clone());
            self.sampling_counter += 1;
    
            if self.sampling_counter == self.skip_to_use {
                let interpolated_sample = StereoSample::interpolate(&self.sampling_buffer);
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
    
    fn set_original_frequency(&mut self, original_frequency:u32) {
        if original_frequency == self.original_frequency{
            // No need to reconfigure resampler parameters
            return;
        }

        log::info!("Reconfiguring audio resampler from {} to {}", self.original_frequency, original_frequency);
        
        // Calling round in order to get the nearest integer and resample as precise as possible
        let div = original_frequency as f32 /  self.target_frequency as f32;

        // Sicne we dont have many f32 methods without std we are implementing them ourself
        let lower_to_skip = libm::floorf(div) as u32;
        let upper_to_skip = libm::ceilf(div) as u32;
        let mut reminder = div - (div as u32 as f32);       // Acts as f32::fracts (since inputs are unsigned)
        
        let (to_skip, alt_to_skip) = if reminder < 0.5{
            (lower_to_skip, upper_to_skip)
        }
        else{
            reminder = 1.0 - reminder;
            (upper_to_skip, lower_to_skip)
        };

        if lower_to_skip == 0{
            core::panic!("target freqency is too high: {} for original frequency: {}", self.target_frequency, original_frequency);
        }

        self.to_skip = to_skip;
        self.alternate_to_skip = alt_to_skip;
        self.skip_to_use = to_skip;
        self.reminder_steps = reminder;
        self.original_frequency = original_frequency;
        // Reset buffers to avoid issues
        self.sampling_buffer.clear();
        self.sampling_counter = 0;
        self.reminder_counter = 0.0;
    }
}