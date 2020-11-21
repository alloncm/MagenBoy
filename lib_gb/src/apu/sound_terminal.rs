pub struct SoundTerminal{
    pub enabled:bool,
    pub volume:u8,
    pub channels:[bool;4]
}

impl Default for SoundTerminal{
    fn default() -> Self {
        SoundTerminal{
            enabled:false,
            channels:[false;4],
            volume:0
        }
    }
}