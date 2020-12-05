use super::sample_producer::SampleProducer;

pub struct ToneSweepSampleProducer{
    
}

impl Default for ToneSweepSampleProducer{
    fn default()->Self{
        ToneSweepSampleProducer{}
    }
}

impl SampleProducer for ToneSweepSampleProducer{

    fn produce(&mut self)->u8{

    }
}

