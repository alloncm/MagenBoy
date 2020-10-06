extern crate stupid_gfx;
extern crate lib_gbc;

use lib_gbc::machine::{
    joypad_provider::JoypadProvider,
    joypad::Joypad
};
use stupid_gfx::{
    event_handler::EventHandler,
    event::{Event,Scancode}
};

pub struct StupidGfxJoypadProvider<'a>{
    event_handler:&'a mut EventHandler
}

impl<'a> StupidGfxJoypadProvider<'a>{
    pub fn new(handler:&'a mut EventHandler)->Self{
        StupidGfxJoypadProvider{
            event_handler:handler
        }
    }
}

impl<'a> JoypadProvider for StupidGfxJoypadProvider<'a>{
    fn provide(&mut self, joypad:&mut Joypad) {
        joypad.a        = self.event_handler.has_event_occurred(Event::KeyPressed(Scancode::X));
        joypad.b        = self.event_handler.has_event_occurred(Event::KeyPressed(Scancode::Z));
        joypad.start    = self.event_handler.has_event_occurred(Event::KeyPressed(Scancode::S));
        joypad.select   = self.event_handler.has_event_occurred(Event::KeyPressed(Scancode::A));
        joypad.up       = self.event_handler.has_event_occurred(Event::KeyPressed(Scancode::Up));
        joypad.down     = self.event_handler.has_event_occurred(Event::KeyPressed(Scancode::Down));
        joypad.right    = self.event_handler.has_event_occurred(Event::KeyPressed(Scancode::Right));
        joypad.left     = self.event_handler.has_event_occurred(Event::KeyPressed(Scancode::Left));
    }
}