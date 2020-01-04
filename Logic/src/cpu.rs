
mod cpu
{
    struct GbcCpu
    {
        a:u8,
        f:u8,
        b:u8,
        c:u8,
        d:u8,
        e:u8,
        h:u8,
        l:u8,
        stack_pointer:u16,
        program_counter:u16
    }
}