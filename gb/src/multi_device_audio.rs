use lib_gb::apu::audio_device::*;

pub struct MultiAudioDevice {
    devices: Vec<Box<dyn AudioDevice>>,
}

impl MultiAudioDevice {
    pub fn new(devices: Vec<Box<dyn AudioDevice>>) -> Self {
        MultiAudioDevice { devices }
    }
}

impl AudioDevice for MultiAudioDevice {
    fn push_buffer(&mut self, buffer: &[Sample]) {
        for device in self.devices.iter_mut() {
            device.push_buffer(buffer);
        }
    }
}
