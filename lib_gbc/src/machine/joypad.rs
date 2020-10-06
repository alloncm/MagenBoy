pub struct Joypad{
    pub up:bool,
    pub down:bool,
    pub left:bool,
    pub right:bool,
    pub start:bool,
    pub select:bool,
    pub a:bool,
    pub b:bool
}

impl Default for Joypad{
    fn default()->Self{
        Joypad{
            a:false, 
            b:false, 
            down:false, 
            left:false, 
            right:false, 
            up:false, 
            select:false, 
            start:false, 
        }
    }
}