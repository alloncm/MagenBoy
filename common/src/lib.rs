#![cfg_attr(not(feature = "std"), no_std)]

cfg_if::cfg_if!{ if #[cfg(feature = "std")] {
    pub mod mbc_handler;
    pub mod mpmc_gfx_device;
    pub mod logging;
}}

pub mod menu;
pub mod joypad_menu;
pub mod interpolation;

pub const VERSION:&str = env!("CARGO_PKG_VERSION");