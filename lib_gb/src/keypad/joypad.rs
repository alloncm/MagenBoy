pub const NUM_OF_KEYS: usize = 8;

pub struct Joypad{
    pub buttons:[bool;NUM_OF_KEYS],
}

impl Default for Joypad{
    fn default()->Self{
        // Since the button pressed state is 0 initializing to unpressed state (1 == true)
        Joypad{
            buttons:[true;NUM_OF_KEYS],
        }
    }
}