/// Max value for digital sample according to the Pandocs
pub const MAX_DIGITAL_SAMPLE:u8 = 0xF;

pub trait SampleProducer{
    /// Produces a digital sample in range of 0 to 0xF 
    fn produce(&mut self)->u8;
    fn get_updated_frequency_ticks(&self, freq:u16)->u16;
    fn reset(&mut self);
}