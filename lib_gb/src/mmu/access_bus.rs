pub enum AccessBus{
    External,
    Video
}

impl Clone for AccessBus{
    fn clone(&self) -> Self {
        match *self{
            AccessBus::External=>AccessBus::External,
            AccessBus::Video=>AccessBus::Video
        }
    }
}

impl Copy for AccessBus{

}