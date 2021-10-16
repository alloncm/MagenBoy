use super::{audio_device::{DEFAULT_SAPMPLE, Sample}, sound_utils::NUMBER_OF_CHANNELS};

pub struct SoundTerminal{
    pub enabled:bool,
    pub volume:u8,
    pub channels:[bool;NUMBER_OF_CHANNELS]
}

impl Default for SoundTerminal{
    fn default() -> Self {
        SoundTerminal{
            enabled:false,
            channels:[false;NUMBER_OF_CHANNELS],
            volume:0
        }
    }
}

impl SoundTerminal{
    pub fn mix_terminal_samples(&self, samples:&[Sample;NUMBER_OF_CHANNELS])->Sample{
        let mut mixed_sample:Sample = DEFAULT_SAPMPLE;
        for i in 0..NUMBER_OF_CHANNELS{
            // This code should add the samples[i] only if channels[i] it true.
            // After profiling this code is faster than if and since this is a hot spot in the code
            // Im writing it like this.
            mixed_sample += samples[i] * self.channels[i] as u8 as Sample;
        }

        mixed_sample >>= 2; // Divide by 4 in order to normal the sample

        return mixed_sample * ((self.volume + 1) as Sample);
    }
}