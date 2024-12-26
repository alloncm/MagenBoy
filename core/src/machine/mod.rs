use core::convert::TryFrom;

use crate::{mmu::carts::{Mbc, CGB_FLAG_ADDRESS}, utils::bit_masks::BIT_7_MASK};

pub mod gameboy;
pub mod mbc_initializer;

#[derive(Clone, Copy, PartialEq)]
pub enum Mode{
    DMG,
    CGB
}

impl TryFrom<&str> for Mode{
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value{
            "DMG"=>Result::Ok(Mode::DMG),
            "CGB"=>Result::Ok(Mode::CGB),
            _=>Result::Err(())
        }
    }
}

impl From<Mode> for &str{
    fn from(mode: Mode) -> &'static str {
        match mode{
            Mode::CGB => "CGB",
            Mode::DMG => "DMG"
        }
    }
}

// for some reason the lifetime is important here for the
// compiler to accept this call on any Mbc lifetine and not just 'static
impl<'a> dyn Mbc + 'a{
    pub fn detect_preferred_mode(&self)->Mode{
        let cart_compatibility_reg = self.read_bank0(CGB_FLAG_ADDRESS as u16);
        return if cart_compatibility_reg & BIT_7_MASK == 0 {Mode::DMG} else{Mode::CGB};
    }
}