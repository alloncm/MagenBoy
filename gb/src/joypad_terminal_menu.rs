use std::{io::Write, time::Duration};
use lib_gb::keypad::{joypad_provider::JoypadProvider, joypad::Joypad, self, button::Button};

use crossterm::{QueueableCommand, style::{self, Stylize}, cursor, terminal::{self, ClearType}, event::{self, KeyCode}};

cfg_if::cfg_if!{
    if #[cfg(windows)]{
        const LINE_ENDING:&'static str = "\r\n";
    }
    else{
        const LINE_ENDING:&'static str = "\n";
    }
}

pub struct TerminalRawModeJoypadProvider<F:Fn(KeyCode)->Option<Button>>{
    mapper:F
}

impl<F:Fn(KeyCode)->Option<Button>> TerminalRawModeJoypadProvider<F> {
    pub fn new(mapper:F)->Self{
        Self{mapper}
    }
} 

impl<F:Fn(KeyCode)->Option<Button>> JoypadProvider for TerminalRawModeJoypadProvider<F>{
    fn provide(&mut self, joypad:&mut Joypad) {
        terminal::enable_raw_mode().unwrap();
        
        joypad.buttons.fill(false);
        // polling so this wont block
        // even tough I can implement the keyboard for this to block I cant implement the rest of the joypads to block
        // so the behavior of this module will have to assume that the input is non blocking
        if event::poll(Duration::from_millis(100)).unwrap(){
            if let event::Event::Key(event) = event::read().unwrap(){
                let mapper = &(self.mapper);
                if let Option::Some(button) = mapper(event.code){
                    joypad.buttons[button as usize] = true;
                }
            }
        }

        terminal::disable_raw_mode().unwrap();
    }
}

impl<F:Fn(KeyCode)->Option<Button>> Drop for TerminalRawModeJoypadProvider<F>{
    fn drop(&mut self) {
        // in case of panic in the middle of the raw mode 
        terminal::disable_raw_mode().unwrap();
    }
}

pub struct MenuOption<T>{
    pub prompt:String,
    pub value:T
}

pub struct JoypadTerminalMenu<T>{
    options: Vec<MenuOption<T>>,
    selection: usize
}

impl<T> JoypadTerminalMenu<T>{
    pub fn new(options:Vec<MenuOption<T>>)->Self{
        JoypadTerminalMenu{options,selection:0}
    }

    pub fn get_menu_selection<JP:JoypadProvider>(&mut self, provider:&mut JP)->&T{
        let mut joypad = Joypad::default();
        let mut redraw = true;
        while !joypad.buttons[keypad::button::Button::A as usize]{
            if redraw{
                self.display_options();
                redraw = false;
            }
            let prev_joypad = Joypad{buttons:joypad.buttons.clone()};
            provider.provide(&mut joypad);
            if chekc_button_down_event(&joypad, &prev_joypad, keypad::button::Button::Up){
                if self.selection > 0{
                    self.selection -= 1;
                    redraw = true;
                }
            }
            else if chekc_button_down_event(&joypad, &prev_joypad, keypad::button::Button::Down){
                if self.selection < self.options.len() - 1{
                    self.selection += 1;
                    redraw = true;
                }
            }
            if redraw{
                self.clear_screen();
            }
        }

        return &self.options[self.selection].value;
    }

    fn display_options(&self){
        let mut stdout = std::io::stdout();
        for i in 0..self.options.len(){
            let option = &self.options[i];
            if i == self.selection{
                stdout.queue(style::PrintStyledContent(option.prompt.as_str().on_white().black())).unwrap();
            }
            else{
                stdout.queue(style::Print(option.prompt.as_str())).unwrap();
            }
            stdout.queue(style::Print(LINE_ENDING)).unwrap();
        }

        stdout.flush().unwrap();
    }

    fn clear_screen(&self){
        let mut stdout = std::io::stdout();
        stdout.queue(cursor::MoveToPreviousLine(self.options.len() as u16)).unwrap();
        stdout.queue(terminal::Clear(ClearType::FromCursorDown)).unwrap();
        stdout.flush().unwrap();
    }
}

// checking the previous state in order to recognize the inital key down event and not the whole time the button is pressed
fn chekc_button_down_event(joypad: &Joypad, prev_joypad: &Joypad, button:Button) -> bool {
    let button_index = button as usize;
    return joypad.buttons[button_index] && !prev_joypad.buttons[button_index];
}