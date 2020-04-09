extern crate sdl2;
use sdl2::event::Event as SdlEvent;
use sdl2::EventPump;
use sdl2::Sdl;
use std::option::Option;
use crate::event::Event;

pub struct EventHandler{
    event_pump:EventPump,
    func_event_handler: Option<fn(Event)>
}

impl EventHandler{
    pub fn init(context:&Sdl)->Self{
        let event_handler = context.event_pump().unwrap();
        return EventHandler{
            event_pump: event_handler,
            func_event_handler:Option::None
        };
    }

    pub fn register_event_handler(&mut self, callback:fn(Event)){
        self.func_event_handler = Option::Some(callback);
    }

    pub fn handle_events(&mut self)->bool{
        let mut alive:bool = true;
        for sdl_event in self.event_pump.poll_iter(){
            
            if self.func_event_handler.is_some(){
                let callback = self.func_event_handler.unwrap();
                let event = Self::sdlevent_into_event(&sdl_event);
                if event.is_some(){
                    let event = event.unwrap();
                    callback(event);
                }
            }
            match sdl_event{
                SdlEvent::Quit{timestamp:_}=>alive = false,
                _=>{}
            }
        }

        return alive;
    }

    fn sdlevent_into_event(sdl_event: &SdlEvent)->Option<Event>{
        return match sdl_event{
            SdlEvent::Quit{timestamp:_}=>Option::Some(Event::Quit),
            SdlEvent::KeyDown{timestamp:_, window_id:_, keycode:key, scancode:_,keymod:_, repeat:_}=>Option::Some(Event::KeyPressed(key.unwrap() as u8)),
            _=>Option::None
        }
    }
}