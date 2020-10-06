extern crate stupid_gfx;
extern crate lib_gbc;
use lib_gbc::keypad::{
    joypad_provider::JoypadProvider,
    joypad::Joypad,
    button::Button
};
use stupid_gfx::{
    event_handler::EventHandler,
    event::{Event,Scancode}
};

pub struct StupidGfxJoypadProvider<'a, F: Fn(Button)->Scancode>{
    event_handler:&'a mut EventHandler,
    mapper:F
}

impl<'a, F: Fn(Button)->Scancode> StupidGfxJoypadProvider<'a, F>{
    pub fn new(handler:&'a mut EventHandler, mapper:F)->Self{
        StupidGfxJoypadProvider{
            event_handler:handler,
            mapper:mapper
        }
    }
}

impl<'a, F:Fn(Button)->Scancode> JoypadProvider for StupidGfxJoypadProvider<'a, F>{
    fn provide(&mut self, joypad:&mut Joypad) {
        let mapper = &(self.mapper);
        joypad.buttons[Button::A as usize]      = self.event_handler.has_event_occurred(Event::KeyPressed(mapper(Button::A)));
        joypad.buttons[Button::B as usize]      = self.event_handler.has_event_occurred(Event::KeyPressed(mapper(Button::B)));
        joypad.buttons[Button::Start as usize]  = self.event_handler.has_event_occurred(Event::KeyPressed(mapper(Button::Start)));
        joypad.buttons[Button::Select as usize] = self.event_handler.has_event_occurred(Event::KeyPressed(mapper(Button::Select)));
        joypad.buttons[Button::Up as usize]     = self.event_handler.has_event_occurred(Event::KeyPressed(mapper(Button::Up)));
        joypad.buttons[Button::Down as usize]   = self.event_handler.has_event_occurred(Event::KeyPressed(mapper(Button::Down)));
        joypad.buttons[Button::Right as usize]  = self.event_handler.has_event_occurred(Event::KeyPressed(mapper(Button::Right)));
        joypad.buttons[Button::Left as usize]   = self.event_handler.has_event_occurred(Event::KeyPressed(mapper(Button::Left)));
    }
}