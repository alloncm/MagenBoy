#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", any(feature = "rpi4",feature = "rpi2")))]
core::compile_error!("rpiX features cant be combined with the std feature");

pub mod configuration;
pub mod peripherals;
pub mod drivers;
pub mod syncronization;