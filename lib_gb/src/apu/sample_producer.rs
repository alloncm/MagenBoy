pub trait SampleProducer : Default{
    fn produce(&mut self)->i8;
}