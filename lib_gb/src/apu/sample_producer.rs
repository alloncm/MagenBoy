pub trait SampleProducer : Default{
    fn produce(&mut self)->u8;
    fn get_updated_frequency_ticks(&self, freq:u16)->u16;
    fn reset(&mut self);
}