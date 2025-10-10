use magenboy_core::{keypad::{button::Button, joypad::Joypad}, JoypadProvider};
use glfw_sys::*;

type GlfwKey = i32;

const KEY_MAP: [(GlfwKey, Button); 8] = [
    (GLFW_KEY_X,        Button::A),
    (GLFW_KEY_Z,        Button::B),
    (GLFW_KEY_S,        Button::Start),
    (GLFW_KEY_A,        Button::Select),
    (GLFW_KEY_UP,       Button::Up),
    (GLFW_KEY_DOWN,     Button::Down),
    (GLFW_KEY_LEFT,     Button::Left),
    (GLFW_KEY_RIGHT,    Button::Right),
];

pub struct GlfwJoypadProvider {
    window: *mut GLFWwindow,
}

impl JoypadProvider for GlfwJoypadProvider {
    fn provide(&mut self, joypad:&mut Joypad) { 

    }
}