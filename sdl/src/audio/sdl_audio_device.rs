use std::{ffi::c_void, mem::{ManuallyDrop, MaybeUninit}};

use crossbeam_channel::{bounded, Receiver, Sender};
use sdl2::sys::*;

use magenboy_core::{GB_FREQUENCY, apu::audio_device::*};
use magenboy_common::audio::{AudioResampler, ResampledAudioDevice};

use crate::utils::get_sdl_error_message;

const BUFFERS_NUMBER:usize = 3;

struct UserData{
    rx: Receiver<usize>,
    current_buf: Option<usize>,
    current_buf_byte_index:usize,
}

pub struct SdlAudioDevice<AR:AudioResampler>{
    resampler: AR,
    buffers: [[Sample;BUFFER_SIZE];BUFFERS_NUMBER],
    buffer_number_index:usize,
    buffer_index:usize,

    userdata_ptr: *mut UserData,
    device_id:SDL_AudioDeviceID,

    // Needs to be droped manually cause the callback might be blocking on the channel,
    // Closing the channel is only possible by droping and Im closing the callback before the destructor is called
    // So I need to call it before closing the callback
    tarnsmiter: ManuallyDrop<Sender<usize>>,
}

impl<AR:AudioResampler> ResampledAudioDevice<AR> for SdlAudioDevice<AR>{
    fn new(frequency:i32, turbo_mul:u8)->Self{
        // cap of less than 2 hurts the fps
        let(s,r) = bounded(BUFFERS_NUMBER - 1);
        let data = Box::new(UserData{
            current_buf:Option::None,
            current_buf_byte_index:0,
            rx:r
        });

        let mut device = SdlAudioDevice{
            buffers:[[DEFAULT_SAPMPLE;BUFFER_SIZE];BUFFERS_NUMBER],
            buffer_index:0,
            buffer_number_index:0,
            resampler: AudioResampler::new(GB_FREQUENCY * turbo_mul as u32, frequency as u32),
            tarnsmiter: ManuallyDrop::new(s),
            userdata_ptr:Box::into_raw(data),
            device_id:0
        };
        
        let desired_audio_spec = SDL_AudioSpec{
            freq: frequency,
            format: AUDIO_S16SYS as u16,    // assumes Sample type is i16
            channels: 2,
            silence: 0,
            samples: BUFFER_SIZE as u16,
            padding: 0,
            size: 0,
            callback: Option::Some(audio_callback),
            userdata: device.userdata_ptr as *mut c_void
        };

        unsafe{
            SDL_Init(SDL_INIT_AUDIO);
            SDL_ClearError();
            let mut uninit_audio_spec:MaybeUninit<SDL_AudioSpec> = MaybeUninit::uninit();
            let id = SDL_OpenAudioDevice(std::ptr::null(), 0, &desired_audio_spec, uninit_audio_spec.as_mut_ptr() , 0);

            if id == 0{
                std::panic!("{}", get_sdl_error_message());
            }

            let init_audio_spec:SDL_AudioSpec = uninit_audio_spec.assume_init();

            if init_audio_spec.freq != desired_audio_spec.freq {
                std::panic!("Error initializing audio could not use the frequency: {}", desired_audio_spec.freq);
            }

            //This will start the audio processing
            SDL_PauseAudioDevice(id, 0);
            device.device_id = id;
        }

        return device;
    }

    fn full_buffer_callback(&mut self) ->Result<(), String> {
        let result = self.tarnsmiter.send(self.buffers[self.buffer_number_index].as_ptr() as usize).map_err(|e|e.to_string());
        self.buffer_number_index = (self.buffer_number_index + 1) % BUFFERS_NUMBER;

        return result;
    }

    fn get_audio_buffer(&mut self) ->(&mut [Sample;BUFFER_SIZE], &mut usize) {
        (&mut self.buffers[self.buffer_number_index], &mut self.buffer_index)
    }

    fn get_resampler(&mut self) ->&mut AR {
        &mut self.resampler
    }
}

impl<AR:AudioResampler> AudioDevice for SdlAudioDevice<AR>{
    fn push_buffer(&mut self, buffer:&[StereoSample; BUFFER_SIZE]) {
        ResampledAudioDevice::push_buffer(self, buffer);
    }
}

impl<AR:AudioResampler> Drop for SdlAudioDevice<AR>{
    fn drop(&mut self) {
        unsafe{
            // Drops the trasmitter manully since we need it to close the channel before we can close the device
            // if the callback will still wait for more samples we will be in a deadlock since the gameboy will 
            // no longer supply audio samples
            ManuallyDrop::drop(&mut self.tarnsmiter);

            SDL_CloseAudioDevice(self.device_id);
            drop(Box::from_raw(self.userdata_ptr));
        }
    }
}

unsafe extern "C" fn audio_callback(userdata:*mut c_void, buffer:*mut u8, length:i32){
    let length = length as usize;
    let safe_userdata = &mut *(userdata as *mut UserData);

    let Ok(rx_data) = safe_userdata.rx.recv() else {return};
    safe_userdata.current_buf = Some(rx_data);
    safe_userdata.current_buf_byte_index = 0;
    let mut samples_size = copy_buffer_and_update_state(safe_userdata, buffer, length);

    if length > samples_size {
        while let Ok(rx_data) = safe_userdata.rx.try_recv(){
            safe_userdata.current_buf = Some(rx_data);
            safe_userdata.current_buf_byte_index = 0;
            samples_size += copy_buffer_and_update_state(safe_userdata, buffer.add(samples_size), length - samples_size);
            if length <= samples_size{
                return;
            }
        }
        std::ptr::write_bytes(buffer.add(samples_size), 0, length  - samples_size);
    }
}

unsafe fn copy_buffer_and_update_state(safe_userdata: &mut UserData, buffer: *mut u8, length: usize) -> usize {
    let samples = &*((safe_userdata.current_buf.unwrap()) as *const [Sample;BUFFER_SIZE]);
    let samples_size = (samples.len() * std::mem::size_of::<Sample>()) - safe_userdata.current_buf_byte_index;
    let samples_ptr = (samples.as_ptr() as *mut u8).add(safe_userdata.current_buf_byte_index);
    std::ptr::copy_nonoverlapping(samples_ptr, buffer, std::cmp::min(length, samples_size));
    return samples_size;
}