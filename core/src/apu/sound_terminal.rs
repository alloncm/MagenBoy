use super::{audio_device::{DEFAULT_SAPMPLE, Sample}, NUMBER_OF_CHANNELS};

type ChannelMask = u16;

const ENABLE_MASK:ChannelMask = 0xFFFF;
const DISABLE_MASK:ChannelMask = 0x0;

pub struct SoundTerminal{
    pub volume:u8,
    channel_masks:[ChannelMask;NUMBER_OF_CHANNELS]
}

impl Default for SoundTerminal{
    fn default() -> Self {
        SoundTerminal{
            channel_masks:[DISABLE_MASK;NUMBER_OF_CHANNELS],
            volume:0
        }
    }
}

impl SoundTerminal{
    pub fn set_channel_state(&mut self, channel:usize, state:bool){
        self.channel_masks[channel] = state as u16 * ENABLE_MASK;
    }

    // For some reason this function is not inlined on release mode
    #[inline]
    pub fn mix_terminal_samples(&self, samples:&[Sample;NUMBER_OF_CHANNELS])->Sample{
        let mut mixed_sample:Sample = DEFAULT_SAPMPLE;

        // This code should add the samples[i] only if channels[i] it true.
        // After profiling this code is faster than if and since this is a hot spot in the code
        // Im writing it like this.
        // Also unrolling the for loop. 
        // for some reason this increase performance drastically.

        mixed_sample += samples[0] & self.channel_masks[0] as Sample;
        mixed_sample += samples[1] & self.channel_masks[1] as Sample;
        mixed_sample += samples[2] & self.channel_masks[2] as Sample;
        mixed_sample += samples[3] & self.channel_masks[3] as Sample;

        mixed_sample >>= 2; // Divide by 4 in order to normal the sample

        // Adding +1 cause thats how to GB calculates the sound (0 still has volume)
        return mixed_sample * ((self.volume + 1) as Sample);
    }
}