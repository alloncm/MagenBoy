pub trait SampleProducer : Default{
    fn produce(&mut self)->i8;
    fn reset(&mut self);
}