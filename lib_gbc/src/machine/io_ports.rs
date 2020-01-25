pub const IO_PORTS_SIZE:usize = 0x80;

pub struct IoPorts{
    pub memory: [u8;IO_PORTS_SIZE]
}