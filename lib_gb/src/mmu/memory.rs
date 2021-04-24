
pub trait Memory{
    fn read(&self, address:u16)->u8;
    fn write(&mut self, address:u16, value:u8);
}

pub trait UnprotectedMemory{
    fn read_unprotected(&self, address:u16)->u8;
    fn write_unprotected(&mut self, address:u16, value:u8);
}