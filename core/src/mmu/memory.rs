
pub trait Memory{
    fn read(&mut self, address:u16, m_cycles:u8)->u8;
    fn write(&mut self, address:u16, value:u8, m_cycles:u8);
    fn set_double_speed_mode(&mut self, state:bool);
}