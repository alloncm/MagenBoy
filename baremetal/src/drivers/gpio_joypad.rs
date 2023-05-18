use lib_gb::keypad::{joypad::{Joypad, NUM_OF_KEYS},joypad_provider::JoypadProvider, button::Button};

use crate::peripherals::{PERIPHERALS, GpioPin, Mode, GpioPull};

const READ_THRESHOLD:u32 = 0x1000;

pub struct GpioJoypadProvider{
    input_pins: [GpioPin; NUM_OF_KEYS],
    read_threshold_counter: u32
}

impl GpioJoypadProvider{
    pub fn new(mapper:impl Fn(Button)->u8)->Self{
        let gpio = unsafe{PERIPHERALS.get_gpio()};
        let mut input_pins = [
            gpio.take_pin(mapper(Button::A), Mode::Input),
            gpio.take_pin(mapper(Button::B), Mode::Input),
            gpio.take_pin(mapper(Button::Start), Mode::Input),
            gpio.take_pin(mapper(Button::Select), Mode::Input),
            gpio.take_pin(mapper(Button::Up), Mode::Input),
            gpio.take_pin(mapper(Button::Down), Mode::Input),
            gpio.take_pin(mapper(Button::Right), Mode::Input),
            gpio.take_pin(mapper(Button::Left), Mode::Input),
        ];
        for i in &mut input_pins{
            i.set_pull(GpioPull::None);
        }
        return Self { input_pins, read_threshold_counter: 0 };
    }
}

impl JoypadProvider for GpioJoypadProvider{
    fn provide(&mut self, joypad:&mut Joypad){
        self.read_threshold_counter = (self.read_threshold_counter + 1) % READ_THRESHOLD;
        if self.read_threshold_counter != 0 {
            return;
        }
        for i in 0..joypad.buttons.len(){
            joypad.buttons[i] = self.input_pins[i].read_state();
        }
    }
}