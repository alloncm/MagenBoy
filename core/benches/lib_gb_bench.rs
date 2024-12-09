use criterion::*;
use magenboy_core::{apu::{
    audio_device::*, channel::Channel, 
    gb_apu::*, sound_terminal::SoundTerminal, 
    square_sample_producer::SquareSampleProducer
}, keypad::{joypad::Joypad, joypad_provider::JoypadProvider, joypad_handler::JoypadHandler}, mmu::interrupts_handler::InterruptsHandler, ppu::{gb_ppu::*, gfx_device::*}, machine::Mode};

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

    let mut joypad_handler = JoypadHandler::new(StubJoypadProvider{set:true});

    c.bench_function("Joypad handler", |b|b.iter(||{
        joypad_handler.poll_joypad_state();
    }));
}

pub fn mmu_interrupt_handler_irq(c:&mut Criterion){
    let mut irh = InterruptsHandler::default();
    irh.interrupt_enable_flag = 1;
    irh.interrupt_flag = 1;

    c.bench_function("Interrupt handler irq", |b|b.iter(||{
        irh.interrupt_enable_flag = irh.interrupt_enable_flag.rotate_left(1);
        irh.interrupt_flag = irh.interrupt_flag.rotate_left(1);

        irh.handle_interrupts(true, 0);
    }));
}


pub fn mmu_interrupt_handler_unhalt(c:&mut Criterion){
    let mut irh = InterruptsHandler::default();
    irh.interrupt_enable_flag = 1;
    irh.interrupt_flag = 1;

    c.bench_function("Interrupt handler unhalt", |b|b.iter(||{
        irh.interrupt_enable_flag = irh.interrupt_enable_flag.rotate_left(1);
        irh.interrupt_flag = irh.interrupt_flag.rotate_left(1);

        irh.handle_interrupts(false, 0);
    }));
}


pub fn mmu_interrupt_handler_early(c:&mut Criterion){
    let mut irh = InterruptsHandler::default();
    irh.interrupt_enable_flag = 1;
    irh.interrupt_flag = 0;

    c.bench_function("Interrupt handler early", |b|b.iter(||{
        std::mem::swap(&mut irh.interrupt_enable_flag, &mut irh.interrupt_flag);
        irh.handle_interrupts(false, 0);
    }));
}

pub fn ppu_gb_ppu(c:&mut Criterion){
    struct StubGfxDevice;
    impl GfxDevice for StubGfxDevice{
        fn swap_buffer(&mut self, _:&[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH]) {}
    }

    let mut ppu = GbPpu::new(StubGfxDevice{}, Mode::DMG);
    ppu.lcd_control = 0xFF;
    ppu.stat_register = 0b111_1000;
    for i in 0..4{
        let y = 16 * (i + 1);
        for j in 0..10{
            let x = 8 * (j + 1);
            ppu.oam[i*j] = y as u8;
            ppu.oam[(i*j) + 1] = x as u8;
            ppu.oam[(i*j) + 2] = 0;
            ppu.oam[(i*j) + 3] = 0b111_0000;

        }
    }

    c.bench_function("Ppu", |b|b.iter(||{
        let mut if_register = 0;
        for _ in 0..100{
            ppu.cycle(10, &mut if_register);
        }
    }));

}
criterion_group!(benches, criterion_bench, apu_sweep_tone_channel, apu_sound_terminal, keypad_joypad_handler, mmu_interrupt_handler_irq, mmu_interrupt_handler_unhalt, mmu_interrupt_handler_early, ppu_gb_ppu);
criterion_main!(benches);