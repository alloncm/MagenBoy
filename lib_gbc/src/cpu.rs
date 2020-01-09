pub struct GbcCpu
{
    pub a:u8,
    pub f:u8,
    pub b:u8,
    pub c:u8,
    pub d:u8,
    pub e:u8,
    pub h:u8,
    pub l:u8,
    pub stack_pointer:u16,
    pub program_counter:u16
}

impl GbcCpu
{
    pub fn af(&self)->u16
    {
        let mut value:u16 = self.a as u16;
        value<<=8;
        value+=self.f as u16;
        return value;
    }

    pub fn bc(&self)->u16
    {
        let mut value:u16 = self.b as u16;
        value<<=8;
        value+=self.c as u16;
        return value;
    }

    pub fn de(&self)->u16
    {
        let mut value:u16 = self.d as u16;
        value<<=8;
        value+=self.e as u16;
        return value;
    }
    
    pub fn hl(&self)->u16
    {
        let mut value:u16 = self.h as u16;
        value<<=8;
        value+=self.l as u16;
        return value;
    }
}   

impl Default for GbcCpu
{
    fn default()->GbcCpu
    {
        GbcCpu{
            a:0,
            f:0,
            b:0,
            c:0,
            d:0,
            e:0,
            h:0,
            l:0,
            stack_pointer:0,
            program_counter:0
        }
    }
}