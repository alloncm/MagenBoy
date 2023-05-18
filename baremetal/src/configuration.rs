pub mod joypad{
    use lib_gb::keypad::button::Button;

    pub const fn button_to_bcm_pin(button:Button)->u8{
        match button{
            Button::A       => 0,
            Button::B       => 7,
            Button::Start   => 6,
            Button::Select  => 5,
            Button::Up      => 22,
            Button::Down    => 17,
            Button::Right   => 4,
            Button::Left    => 27,
        }
    }
}

pub mod display{
    pub const RESET_PIN_BCM:u8 = 13;
    pub const LED_PIN_BCM:u8 = 25;
}

pub mod peripherals{
    pub const CORE_FREQ:u32 = 400_000_000;
    pub const MINI_UART_BAUDRATE:u32 = 115200;
    pub const SPI0_DC_BCM_PIN:u8 = 12;
    pub const DMA_RX_CHANNEL_NUMBER:u8 = 7;
    pub const DMA_TX_CHANNEL_NUMBER:u8 = 1;
    pub const FAST_SPI_CLOCK_DIVISOR:u32 = 6;   // the smaller the faster
    pub const INIT_SPI_CLOCK_DIVISOR:u32 = 34;  // slow clock for verifiying the initialization goes smooth with no corruptions
}

pub mod emulation{
    pub const ROM:&'static [u8] = include_bytes!("../../Dependencies/TetrisDX.gbc"); // Path is relative to the build
    pub const TURBO:u8 = 1;             // Will speed up the emulation * X
    pub const FRAME_LIMITER:u32 = 0;    // Will filter every frame X frames
}