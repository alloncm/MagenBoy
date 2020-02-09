
const LOW_POSITION:isize = 0;
const HIGH_POSITION:isize = 1;

pub struct Reg{
    pub value:u16
}

impl Default for Reg{
    fn default()->Reg{
        Reg{value:0}
    }
}

impl Reg{
    pub fn get_low(&mut self)->&mut u8{
        self.get_offset_byte(LOW_POSITION)
    }

    pub fn get_high(&mut self)->&mut u8{
        self.get_offset_byte(HIGH_POSITION)
    }

    fn get_offset_byte(&mut self, offset:isize)->&mut u8{
        unsafe
        {
            let ptr = (&mut self.value as *mut u16) as *mut u8;
            return &mut *(ptr.offset(offset));
        }
    }
} 