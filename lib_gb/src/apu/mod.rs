pub mod audio_device;
pub mod channel;
pub mod frame_sequencer;
pub mod freq_sweep;
pub mod gb_apu;
pub mod noise_sample_producer;
pub mod sample_producer;
pub mod sound_terminal;
pub mod square_sample_producer;
pub mod timer;
pub mod volume_envelop;
pub mod wave_sample_producer;

mod apu_registers_updater;
mod sound_utils;

pub use apu_registers_updater::update_apu_registers;
