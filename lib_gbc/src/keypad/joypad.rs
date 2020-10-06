
pub const NUM_OF_KEYS: usize = 8;

pub struct Joypad{
    pub buttons:[bool;NUM_OF_KEYS]
}

impl Default for Joypad{
    fn default()->Self{
        Joypad{
            buttons:[false;NUM_OF_KEYS]
        }
    }
}