use super::{audio_device::{SAMPLE_CONSTANT_DEFAULT, Sample}, sound_utils::NUMBER_OF_CHANNELS};

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
        let mut mixed_sample:Sample = SAMPLE_CONSTANT_DEFAULT;
        for i in 0..NUMBER_OF_CHANNELS{
            if self.channels[i]{
                mixed_sample += samples[i];
            }
        }

        mixed_sample /= NUMBER_OF_CHANNELS as Sample;

        return mixed_sample * (self.volume + 1) as Sample;
    }
}