use super::sample_producer::SampleProducer;

pub struct NoiseSampleProducer{
    
}

impl Default for NoiseSampleProducer{
    fn default() -> Self {
        Self{}
    }
}

impl SampleProducer for NoiseSampleProducer{
    fn produce(&mut self)->i8 {
        return 0;
    }

    fn reset(&mut self) {
        
    }
}