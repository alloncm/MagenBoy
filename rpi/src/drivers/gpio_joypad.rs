use magenboy_core::keypad::{joypad::{Joypad, NUM_OF_KEYS},joypad_provider::JoypadProvider, button::Button};

use crate::peripherals::{PERIPHERALS, GpioPull, Trigger, InputGpioPin};

const READ_THRESHOLD:u32 = 0x1000;

pub struct GpioJoypadProvider{
    input_pins: [InputGpioPin; NUM_OF_KEYS],
    read_threshold_counter: u32
}

impl GpioJoypadProvider{
    pub fn new(mapper:impl Fn(Button)->u8)->Self{
        let gpio = unsafe{PERIPHERALS.get_gpio()};
        let input_pins = [
            gpio.take_pin(mapper(Button::A)),
            gpio.take_pin(mapper(Button::B)),
            gpio.take_pin(mapper(Button::Start)),
            gpio.take_pin(mapper(Button::Select)),
            gpio.take_pin(mapper(Button::Up)),
            gpio.take_pin(mapper(Button::Down)),
            gpio.take_pin(mapper(Button::Right)),
            gpio.take_pin(mapper(Button::Left)),
        ];
        
        return Self { input_pins:input_pins.map(|p|{
            let mut p = p.into_input(GpioPull::None);
            p.set_interrupt(Trigger::RisingEdge);
            return p;
        }), read_threshold_counter: 0 };
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

#[cfg(feature = "os")]
impl magenboy_common::joypad_menu::MenuJoypadProvider for GpioJoypadProvider {
    fn poll(&mut self, joypad:&mut Joypad) {
        let gpio = unsafe{PERIPHERALS.get_gpio()};
        gpio.poll_interrupts(&self.input_pins,false);
        
        for i in 0..joypad.buttons.len(){
            joypad.buttons[i] = self.input_pins[i].read_state();
        }
    }
}

impl Clone for GpioJoypadProvider{
    fn clone(&self) -> Self {
        Self { input_pins: self.input_pins.clone(), read_threshold_counter: self.read_threshold_counter.clone() }
    }
}