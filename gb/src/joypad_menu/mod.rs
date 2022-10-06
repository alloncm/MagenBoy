use lib_gb::keypad::{joypad_provider::JoypadProvider, button::Button, joypad::Joypad};

cfg_if::cfg_if!{if #[cfg(feature = "terminal-menu")]{
    pub mod joypad_terminal_menu;
}
else{
    mod font;
    pub mod joypad_gfx_menu;    
}}

pub struct MenuOption<T>{
    pub prompt:String,
    pub value:T
}

pub trait MenuRenderer<T>{
    fn render_menu(&mut self, menu:&Vec<MenuOption<T>>, selection:usize);
}

pub struct JoypadMenu<T, MR:MenuRenderer<T>>{
    options: Vec<MenuOption<T>>,
    selection: usize,
    renderer:MR,
}

impl<T, MR:MenuRenderer<T>> JoypadMenu<T, MR>{
    pub fn new(menu_options:Vec<MenuOption<T>>, renderer:MR)->Self{
        JoypadMenu { 
            options: menu_options,
            selection: 0,
            renderer
        }
    }

    pub fn get_menu_selection<JP:JoypadProvider>(&mut self, provider:&mut JP)->&T{
        let mut joypad = Joypad::default();
        let mut redraw = true;
        while !joypad.buttons[Button::A as usize]{
            if redraw{
                self.renderer.render_menu(&self.options, self.selection);
                redraw = false;
            }
            let prev_joypad = Joypad{buttons:joypad.buttons.clone()};
            provider.provide(&mut joypad);
            if check_button_down_event(&joypad, &prev_joypad, Button::Up){
                if self.selection > 0{
                    self.selection -= 1;
                    redraw = true;
                }
            }
            if check_button_down_event(&joypad, &prev_joypad, Button::Down){
                if self.selection < self.options.len() - 1{
                    self.selection += 1;
                    redraw = true;
                }
            }
        }

        return &&self.options[self.selection].value;
    }
}

// checking the previous state in order to recognize the inital key down event and not the whole time the button is pressed
fn check_button_down_event(joypad: &Joypad, prev_joypad: &Joypad, button:Button) -> bool {
    let button_index = button as usize;
    return joypad.buttons[button_index] && !prev_joypad.buttons[button_index];
}