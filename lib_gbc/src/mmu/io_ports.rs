pub const IO_PORTS_SIZE:usize = 0x80;

pub struct IoPorts{
    pub memory: [u8;IO_PORTS_SIZE]
}

impl Default for IoPorts{
    fn default()->IoPorts{
        IoPorts{
            memory:[0;IO_PORTS_SIZE]
        }
    }
}