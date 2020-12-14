use lib_gb::apu::audio_device::AudioDevice;
use stupid_gfx::audio::Audio;
use stupid_gfx::initializer::Initializer;

pub struct StupidGfxAudioDevie{
    device:Audio,
    frequency:u32
}

impl StupidGfxAudioDevie{
    pub fn new(stupid_initializer:&Initializer,frequency:u32, fps:u8, channels:u8)->Self{
        StupidGfxAudioDevie{
            device:stupid_initializer.init_audio(frequency as i32, channels, fps),
            frequency:frequency
        }
    }
}

impl AudioDevice for StupidGfxAudioDevie{
    fn push_buffer(&self, buffer:&[f32]){
        self.device.push_audio_to_device(buffer).unwrap();
    }
}