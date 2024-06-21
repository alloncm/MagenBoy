pub mod gb_apu;
pub mod channel;
pub mod sample_producer;
pub mod wave_sample_producer;
pub mod audio_device;
pub mod timer;
pub mod frame_sequencer;
pub mod sound_terminal;
pub mod square_sample_producer;
pub mod freq_sweep;
pub mod volume_envelop;
pub mod noise_sample_producer;

mod apu_registers_updater;

pub use apu_registers_updater::*;

pub(self) const NUMBER_OF_CHANNELS:usize = 4;