use lib_gb::apu::audio_device::*;

pub struct MultiAudioDevice<const NUMBER_OF_DEVICES:usize>{
    devices:[Box::<dyn AudioDevice>; NUMBER_OF_DEVICES]
}

impl<const NUMBER_OF_DEVICES:usize> MultiAudioDevice<NUMBER_OF_DEVICES>{
    pub fn new(devices:[Box::<dyn AudioDevice>; NUMBER_OF_DEVICES])->Self{
        MultiAudioDevice{devices}
    }
}

impl<const NUMBER_OF_DEVICES:usize> AudioDevice for MultiAudioDevice<NUMBER_OF_DEVICES>{
    fn push_buffer(&mut self, buffer:&[Sample]) {
        for device in self.devices.iter_mut(){
            device.push_buffer(buffer);
        }
    }
}