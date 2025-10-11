use magenboy_core::{keypad::{button::Button, joypad::Joypad}, JoypadProvider};
use glfw_sys::*;

type GlfwKey = i32;

const fn mapper(button: Button) -> GlfwKey {
    return match button {
        Button::A       => GLFW_KEY_X,
        Button::B       => GLFW_KEY_Z,
        Button::Start   => GLFW_KEY_S,
        Button::Select  => GLFW_KEY_A,
        Button::Up      => GLFW_KEY_UP,
        Button::Down    => GLFW_KEY_DOWN,
        Button::Left    => GLFW_KEY_LEFT,
        Button::Right   => GLFW_KEY_RIGHT,
    };
}
pub struct GlfwJoypadProvider {
    window: *mut GLFWwindow,
}

impl GlfwJoypadProvider {
    pub fn new(window: *mut GLFWwindow) -> Self {
        Self {
            window
        }
    }
}

impl JoypadProvider for GlfwJoypadProvider {
    fn provide(&mut self, joypad:&mut Joypad) { 
        unsafe {
            joypad.buttons[Button::A as usize] = glfwGetKey(self.window, mapper(Button::A)) == GLFW_PRESS;
            joypad.buttons[Button::B as usize] = glfwGetKey(self.window, mapper(Button::B)) == GLFW_PRESS;
            joypad.buttons[Button::Start as usize] = glfwGetKey(self.window, mapper(Button::Start)) == GLFW_PRESS;
            joypad.buttons[Button::Select as usize] = glfwGetKey(self.window, mapper(Button::Select)) == GLFW_PRESS;
            joypad.buttons[Button::Up as usize] = glfwGetKey(self.window, mapper(Button::Up)) == GLFW_PRESS;
            joypad.buttons[Button::Down as usize] = glfwGetKey(self.window, mapper(Button::Down)) == GLFW_PRESS;
            joypad.buttons[Button::Right as usize] = glfwGetKey(self.window, mapper(Button::Right)) == GLFW_PRESS;
            joypad.buttons[Button::Left as usize] = glfwGetKey(self.window, mapper(Button::Left)) == GLFW_PRESS;
        }
    }
}