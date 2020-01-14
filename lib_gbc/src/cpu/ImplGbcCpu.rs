use crate::cpu::GbcCpu::GbcCpu;

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