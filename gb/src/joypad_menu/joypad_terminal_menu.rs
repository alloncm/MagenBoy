use std::{io::Write};
use crossterm::{QueueableCommand, style::{self, Stylize}, cursor, terminal::{self, ClearType}};
use super::MenuRenderer;

cfg_if::cfg_if!{
    if #[cfg(windows)]{
        const LINE_ENDING:&'static str = "\r\n";
    }
    else{
        const LINE_ENDING:&'static str = "\n";
    }
}

pub struct TerminalMenuRenderer;

impl TerminalMenuRenderer{
    fn clear_screen(&self, menu_len:usize){
        let mut stdout = std::io::stdout();
        stdout.queue(cursor::MoveToPreviousLine(menu_len as u16)).unwrap();
        stdout.queue(terminal::Clear(ClearType::FromCursorDown)).unwrap();
        stdout.flush().unwrap();
    }
}

impl<T> MenuRenderer<T> for TerminalMenuRenderer{
    fn render_menu(&mut self, menu:&Vec<super::MenuOption<T>>, selection:usize) {
        self.clear_screen(menu.len());

        let mut stdout = std::io::stdout();
        for i in 0..menu.len(){
            let option = &menu[i];
            if i == selection{
                stdout.queue(style::PrintStyledContent(option.prompt.as_str().on_white().black())).unwrap();
            }
            else{
                stdout.queue(style::Print(option.prompt.as_str())).unwrap();
            }
            stdout.queue(style::Print(LINE_ENDING)).unwrap();
        }

        stdout.flush().unwrap();
    }
}