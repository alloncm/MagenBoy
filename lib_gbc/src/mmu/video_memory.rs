pub trait VideoMemory{
    fn read(&self, address:u16)->u8;
}