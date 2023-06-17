mod font;
pub mod joypad_gfx_menu;

use std::path::PathBuf;

use magenboy_core::keypad::{button::Button, joypad::Joypad, joypad_provider::JoypadProvider};

pub struct MenuOption<T, S:AsRef<str>>{
    pub prompt:S,
    pub value:T
}

pub trait MenuRenderer<T, S:AsRef<str>>{
    fn render_menu(&mut self,header:&S, menu:&[MenuOption<T, S>], selection:usize);
}

pub trait MenuJoypadProvider{
    fn poll(&mut self, joypad:&mut Joypad);
}

pub struct JoypadMenu<'a, T, S:AsRef<str>, MR:MenuRenderer<T, S>>{
    header:S,
    options: &'a [MenuOption< T, S>],
    selection: usize,
    renderer:MR,
}

impl<'a, T, S: AsRef<str>, MR:MenuRenderer<T, S>> JoypadMenu<'a, T, S, MR>{
    pub fn new(menu_options:&'a[MenuOption<T, S>], header:S, renderer:MR)->Self{
        JoypadMenu { 
            header,
            options: menu_options,
            selection: 0,
            renderer
        }
    }

    pub fn get_menu_selection<JP:MenuJoypadProvider + JoypadProvider>(&mut self, provider:&mut JP)->&'a T{
        let mut joypad = Joypad::default();
        let mut redraw = true;
        while !joypad.buttons[Button::A as usize]{
            if redraw{
                self.renderer.render_menu(&self.header,&self.options, self.selection);
                redraw = false;
            }
            provider.poll(&mut joypad);
            if joypad.buttons[Button::Up as usize]{
                if self.selection > 0{
                    self.selection -= 1;
                    redraw = true;
                }
            }
            if joypad.buttons[Button::Down as usize]{
                if self.selection < self.options.len() - 1{
                    self.selection += 1;
                    redraw = true;
                }
            }
        }
        // Busy wait untill A is released in order to not leak the button press to the emulation
        while joypad.buttons[Button::A as usize]{
            provider.provide(&mut joypad);
        }
        return &self.options[self.selection].value;
    }
}

pub fn get_rom_selection<MR:MenuRenderer<PathBuf, String>, JP:MenuJoypadProvider + JoypadProvider>(roms_path:&str, menu_renderer:MR, jp:&mut JP)->String{
    let mut menu_options = Vec::new();
    let dir_entries = std::fs::read_dir(roms_path).expect(std::format!("Error openning the roms directory: {}",roms_path).as_str());
    for entry in dir_entries{
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(extension) = path.as_path().extension().and_then(std::ffi::OsStr::to_str){
            match extension {
                "gb" | "gbc"=>{
                    let filename = String::from(path.file_name().expect("Error should be a file").to_str().unwrap());
                    let option = MenuOption{value: path, prompt: filename};
                    menu_options.push(option);
                },
                _=>{}
            }
        }
    }
    let mut menu = JoypadMenu::new(&menu_options, String::from("Choose ROM"), menu_renderer);
    let result = menu.get_menu_selection(jp);

    return String::from(result.to_str().unwrap());
}