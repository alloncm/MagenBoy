use lib_gb::apu::audio_device::AudioDevice;
use stupid_gfx::audio::Audio;

pub struct StupidGfxAudioDevie{
    device:Audio
}

impl StupidGfxAudioDevie{
    pub fn new(audio_device: Audio)->Self{
        StupidGfxAudioDevie{
            device:audio_device
        }
    }
}

impl AudioDevice for StupidGfxAudioDevie{
    fn push_buffer(&self, buffer:&[f32]){
        self.device.push_audio_to_device(buffer).unwrap();
    }
}