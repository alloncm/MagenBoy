#![cfg_attr(not(feature = "std"), no_std)]

cfg_if::cfg_if!{ if #[cfg(feature = "std")] {
    pub mod mbc_handler;
    pub mod logging;
    pub mod initialization;
    pub use initialization::*;

    pub use log;
}}

cfg_if::cfg_if!{ if #[cfg(feature = "alloc")] {
    extern crate alloc;
    
    pub mod audio{
        mod audio_resampler;
        mod manual_audio_resampler;
        pub use audio_resampler::*;
        pub use manual_audio_resampler::*;
    }
}}

pub mod menu;
pub mod joypad_menu;
pub mod interpolation;
pub mod synchronization;

pub const VERSION:&str = env!("MAGENBOY_VERSION");