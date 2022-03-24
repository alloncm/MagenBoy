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
        
        let mut pins:[Option<InputPin>;NUM_OF_KEYS] = create_default_array();
        pins[Button::A as usize]        = Some(gpio.get(mapper(Button::A)).unwrap().into_input());
        pins[Button::B as usize]        = Some(gpio.get(mapper(Button::B)).unwrap().into_input());
        pins[Button::Start as usize]    = Some(gpio.get(mapper(Button::Start)).unwrap().into_input());
        pins[Button::Select as usize]   = Some(gpio.get(mapper(Button::Select)).unwrap().into_input());
        pins[Button::Up as usize]       = Some(gpio.get(mapper(Button::Up)).unwrap().into_input());
        pins[Button::Down as usize]     = Some(gpio.get(mapper(Button::Down)).unwrap().into_input());
        pins[Button::Right as usize]    = Some(gpio.get(mapper(Button::Right)).unwrap().into_input());
        pins[Button::Left as usize]     = Some(gpio.get(mapper(Button::Left)).unwrap().into_input());

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