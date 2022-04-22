
pub trait Memory{
    fn read(&mut self, address:u16)->u8;
    fn write(&mut self, address:u16, value:u8);
}