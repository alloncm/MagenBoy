use lib_gb::keypad::{
    joypad::{Joypad, NUM_OF_KEYS},
    joypad_provider::JoypadProvider,
    button::Button
};
use lib_gb::utils::create_default_array;
use rppal::gpio::{Gpio, InputPin};

pub type GpioPin = u8;

pub struct GpioJoypadProvider{
    input_pins:[Option<InputPin>;NUM_OF_KEYS]
}

impl GpioJoypadProvider{
    pub fn new<F:Fn(Button)->GpioPin>(mapper:F)->Self{
        
        let gpio = Gpio::new().unwrap();
        let a_pin = gpio.get(mapper(Button::A)).unwrap().into_input();
        let b_pin = gpio.get(mapper(Button::B)).unwrap().into_input();
        let up_pin = gpio.get(mapper(Button::Up)).unwrap().into_input();
        let down_pin = gpio.get(mapper(Button::Down)).unwrap().into_input();
        let right_pin = gpio.get(mapper(Button::Right)).unwrap().into_input();
        let left_pin = gpio.get(mapper(Button::Left)).unwrap().into_input();
        
        let mut pins:[Option<InputPin>;NUM_OF_KEYS] = create_default_array();
        pins[Button::A as usize] = Some(a_pin);
        pins[Button::B as usize] = Some(b_pin);
        pins[Button::Up as usize] = Some(up_pin);
        pins[Button::Down as usize] = Some(down_pin);
        pins[Button::Right as usize] = Some(right_pin);
        pins[Button::Left as usize] = Some(left_pin);

        Self{
            input_pins:pins
        }
    }
}

impl JoypadProvider for GpioJoypadProvider{
    fn provide(&mut self, joypad:&mut Joypad){
        for i in 0..NUM_OF_KEYS{
            if let Some(pin) = &self.input_pins[i] {
                joypad.buttons[i] = pin.is_high();
            }
        }
    }
}