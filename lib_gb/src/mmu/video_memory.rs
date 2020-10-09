pub trait ReadOnlyVideoMemory{
    fn read(&self, address:u16)->u8;
}