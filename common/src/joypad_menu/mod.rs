mod font;
mod menu_renderer;

use magenboy_core::{keypad::{button::Button, joypad::Joypad}, FrameBuffer};

use crate::menu::MenuOption;

use menu_renderer::MenuRenderer;

pub enum MenuResult<'a, T> {
    Selection(&'a T),
    Frame(FrameBuffer)
}

pub struct JoypadMenu<'a, T, S:AsRef<str>>{
    header:S,
    options: &'a [MenuOption<T, S>],
    selection: usize,
    renderer: MenuRenderer
}

impl<'a, T, S: AsRef<str>> JoypadMenu<'a, T, S>{
    pub fn new(menu_options:&'a[MenuOption<T, S>], header:S)->Self{
        Self { 
            header,
            options: menu_options,
            selection: 0,
            renderer: MenuRenderer
        }
    }

    pub fn try_get_menu_selection(&mut self, joypad: Joypad) -> MenuResult<'a, T>{
        if !joypad.buttons[Button::A as usize]{
            if joypad.buttons[Button::Up as usize]{
                if self.selection > 0{
                    self.selection -= 1;
                }
            }
            if joypad.buttons[Button::Down as usize]{
                if self.selection < self.options.len() - 1{
                    self.selection += 1;
                }
            }
            let menu_frame = self.renderer.render_menu(&self.header,&self.options, self.selection);
            return MenuResult::Frame(menu_frame);
        }

        return MenuResult::Selection(&self.options[self.selection].value);
    }
}