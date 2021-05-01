use super::joypad::Joypad;

pub trait JoypadProvider{
    fn provide(&mut self, joypad:&mut Joypad);
}