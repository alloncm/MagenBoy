use super::joypad::Joypad;

pub trait JoypadProvider{
    fn provide(&self, joypad:&mut Joypad);
}