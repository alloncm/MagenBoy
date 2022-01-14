use lib_gb::keypad::{
    joypad::{Joypad, NUM_OF_KEYS},
    joypad_provider::JoypadProvider,
    button::Button
};
use rppal::gpio::{
    Gpio,
    InputPin
};

pub struct GpioJoypadProvider{
    input_pins:[Option<InputPin>;NUM_OF_KEYS]
}

impl GpioJoypadProvider{
    pub fn new()->Self{
        
        let gpio = Gpio::new().unwrap();
        let a_pin = gpio.get(18).unwrap().into_input();
        let b_pin = gpio.get(17).unwrap().into_input();
        let up_pin = gpio.get(16).unwrap().into_input();
        let down_pin = gpio.get(20).unwrap().into_input();
        let right_pin = gpio.get(21).unwrap().into_input();
        let left_pin = gpio.get(19).unwrap().into_input();
        
        let mut pins:[Option<InputPin>;NUM_OF_KEYS] = [None;NUM_OF_KEYS];
        pins[Button::A as u8] = Some(a_pin);
        pins[Button::B as u8] = Some(b_pin);
        pins[Button::Up as u8] = Some(up_pin);
        pins[Button::Down as u8] = Some(down_pin);
        pins[Button::Right as u8] = Some(right_pin);
        pins[Button::Left as u8] = Some(left_pin);

        Self{
            input_pins:pins
        }
    }
}

impl JoypadProvider for GpioJoypadProvider{
    fn provide(&mut self, joypad:&mut Joypad){
        for i in 0..NUM_OF_KEYS{
            if self.input_pins[i] = Some(pin){
                joypad[i] = pin.is_high();
            }
        }
    }
}