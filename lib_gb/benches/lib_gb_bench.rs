use criterion::*;
use lib_gb::{apu::{
    audio_device::*, channel::Channel, 
    gb_apu::*, sound_terminal::SoundTerminal, 
    square_sample_producer::SquareSampleProducer
}, keypad::{joypad::Joypad, joypad_provider::JoypadProvider, joypad_handler::JoypadHandler}};

pub fn criterion_bench(c: &mut Criterion){
    struct StubApu;
    impl AudioDevice for StubApu{
        fn push_buffer(&mut self, _buffer:&[StereoSample; BUFFER_SIZE]){}
    }

    c.bench_function("test apu", |b| b.iter(||{
        let mut apu = GbApu::new(StubApu{});
        apu.enabled = true;
        apu.sweep_tone_channel.enabled = true;
        for _ in 0..100{
            apu.cycle(10);
        }
    }));
}

pub fn apu_sweep_tone_channel(c: &mut Criterion){

    c.bench_function("test square channel", |b|b.iter(||{
        let mut channel = Channel::<SquareSampleProducer>::new(SquareSampleProducer::new_with_sweep());
        channel.sound_length = 63;
        channel.enabled = true;
        channel.length_enable = true;
        while channel.enabled{
            let _ = channel.get_audio_sample();
            channel.update_length_register();
        }
    }));
}

pub fn apu_sound_terminal(c:&mut Criterion){
    let mut sound_terminal = SoundTerminal::default();
    for i in 0..4{
        sound_terminal.set_channel_state(i, true);
    }
    sound_terminal.volume = 8;
    c.bench_function("Sound terminal", |b| b.iter(||{
        let samples:[Sample;4] = [100 as Sample,200 as Sample,5 as Sample,7 as Sample];
        let _ = sound_terminal.mix_terminal_samples(black_box(&samples));        
    }));
}

pub fn keypad_joypad_handler(c:&mut Criterion){
    struct StubJoypadProvider{
        set:bool
    }
    impl JoypadProvider for StubJoypadProvider{
        fn provide(&mut self, joypad:&mut Joypad) {
            joypad.buttons.fill(self.set);
            self.set = !self.set;
        }
    }

    // let mut joypad = Joypad::default();
    let mut joypad_handler = JoypadHandler::new(StubJoypadProvider{set:true});

    c.bench_function("Joypad handler", |b|b.iter(||{
        joypad_handler.poll_joypad_state();
    }));
}

criterion_group!(benches, criterion_bench, apu_sweep_tone_channel, apu_sound_terminal, keypad_joypad_handler);
criterion_main!(benches);