use std::convert::TryFrom;

pub mod gameboy;
pub mod mbc_initializer;
#[cfg(feature = "dbg")]
pub mod debugger;

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