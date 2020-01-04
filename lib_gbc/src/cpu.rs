
mod cpu
{
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
    }
}