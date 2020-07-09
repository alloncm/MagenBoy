
const LOW_POSITION:isize = 0;
const HIGH_POSITION:isize = 1;

pub struct Reg{
    value:u16,
    read_only_mask: u16
}

impl Default for Reg{
    fn default()->Reg{
        Reg{
            value: 0,
            read_only_mask: 0xFFFF}
    }
}

impl Reg{
    pub fn new(romask:u16)->Self{
        Reg{
            value: 0,
            read_only_mask: romask
        }
    }

    pub fn low(&mut self)->&mut u8{
        self.mask();
        self.get_offset_byte(LOW_POSITION)
    }

    pub fn high(&mut self)->&mut u8{
        self.mask();
        self.get_offset_byte(HIGH_POSITION)
    }
    
    pub fn value(&mut self)->&mut u16{
        self.mask();
        return &mut self.value;
    }

    fn get_offset_byte(&mut self, offset:isize)->&mut u8{
        unsafe
        {
            let ptr = (&mut self.value as *mut u16) as *mut u8;
            return &mut *(ptr.offset(offset));
        }
    }

    fn mask(&mut self){
        self.value &= self.read_only_mask;
    }
} 