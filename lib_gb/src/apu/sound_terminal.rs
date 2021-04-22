pub struct SoundTerminal{
    pub enabled:bool,
    pub volume:u8,
    pub channels:[bool;4]
}

impl Default for SoundTerminal{
    fn default() -> Self {
        SoundTerminal{
            enabled:false,
            channels:[false;4],
            volume:0
        }
    }
}

impl SoundTerminal{
    pub fn mix_terminal_samples(&self, samples:&[f32;4])->f32{
        let mut mixed_sample:f32 = 0.0;
        for i in 0..4{
            if self.channels[i]{
                mixed_sample += samples[i];
            }
        }

        mixed_sample /= 4.0;

        return mixed_sample * (self.volume as f32 + 1.0);
    }
}