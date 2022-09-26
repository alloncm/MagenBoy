// Checking for rpi
cfg_if::cfg_if!{ if #[cfg(all(target_os = "linux", target_arch = "arm"))]{
    pub mod bcm;
    pub use bcm::BcmHost;
}}