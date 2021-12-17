use lib_gb::keypad::{
    joypad::Joypad,
    joypad_provider::JoypadProvider,
    button::Button
};
use rppal::gpio::{
    Gpio,
    Level
};

pub struct GpioJoypadProvider{
    gpio: Gpio
}

impl GpioJoypadProvider{
    pub fn new()->Self{
        Self{
            gpio: Gpio::new().unwrap()
        }
    }

    fn read_pin(&self, bcm_pin_number:u8)->bool{
        let pin = self.gpio.get(bcm_pin_number).unwrap();
        return match pin.read(){
            Level::High=>false,
            Level::Low=>true
        };
    }
}

impl JoypadProvider for GpioJoypadProvider{
    fn provide(&mut self, joypad:&mut Joypad){
        joypad.buttons[Button::A as usize] = self.read_pin(2);
        joypad.buttons[Button::B as usize] = self.read_pin(3);
    }
}