pub trait SampleProducer : Default{
    fn produce(&mut self)->u8;
}