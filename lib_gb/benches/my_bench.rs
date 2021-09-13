use criterion::*;
use lib_gb::apu::{
    gb_apu::*,
    audio_device::*,
};

pub fn criterion_bench(c: &mut Criterion){
    struct StubApu;
    impl AudioDevice for StubApu{
        fn push_buffer(&mut self, _buffer:&[Sample]){}
    }

    c.bench_function("test apu", |b| b.iter(||{
        let mut apu = GbApu::new(StubApu{});
        apu.enabled = true;
        apu.sweep_tone_channel.enabled = true;
        for _ in 0..100000{
            apu.cycle(255);
        }
    }));
}

criterion_group!(benches, criterion_bench);
criterion_main!(benches);