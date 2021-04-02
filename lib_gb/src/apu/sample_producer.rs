pub trait SampleProducer : Default{
    fn produce(&mut self)->i8;
    fn get_updated_frequency_ticks(freq:u16)->u16;
    fn reset(&mut self);
}