use criterion::{criterion_group, criterion_main, Criterion};
use image_inter::{scale_bilinear, scale_biliniear_c, scale_nearest};


pub fn interpolation_rust_bench(c: &mut Criterion){
    let input_buffer = [0_u16; 160*144];
    let mut output_buffer = [0_u8; 240*266*2];
    c.bench_function("bench rust inter", |b|b.iter(||{
        unsafe{scale_bilinear::<160, 144, 266, 240>(input_buffer.as_ptr(), output_buffer.as_mut_ptr())};
    }));
}

pub fn interpolation_c_bench(c: &mut Criterion){
    let input_buffer = [0_u16; 160*144];
    let mut output_buffer = [0_u8; 240*266*2];
    c.bench_function("bench c inter", |b|b.iter(||{
        unsafe{scale_biliniear_c::<160, 144, 266, 240>(input_buffer.as_ptr(), output_buffer.as_mut_ptr())};
    }));
}

pub fn neighbor_rust_inter(c: &mut Criterion){
    let input_buffer = [0_u16; 160*144];
    let mut output_buffer = [0_u8; 240*266*2];
    c.bench_function("bench rust neighbor", |b|b.iter(||{
        unsafe{scale_nearest::<160, 144, 266, 240>(input_buffer.as_ptr(), output_buffer.as_mut_ptr(), 5.0/3.0)};
    }));
}

pub fn interpolation_fir_bench(c: &mut Criterion){
    let input_buffer = [0_u8; 160*144*2];
    let mut output_buffer = [0_u8; 240*266*2];
    c.bench_function("bench fir inter", |b|b.iter(||{
        let mut buffer = input_buffer.clone();
        let src_buffer = fast_image_resize::Image::from_slice_u8(
            std::num::NonZeroU32::new(160 as u32).unwrap(),
            std::num::NonZeroU32::new(144 as u32).unwrap(),
            &mut buffer,
            fast_image_resize::PixelType::U16
        ).unwrap();

        let mut dst_buffer = fast_image_resize::Image::from_slice_u8(
            std::num::NonZeroU32::new(266 as u32).unwrap(), 
            std::num::NonZeroU32::new(240 as u32).unwrap(), 
            &mut output_buffer, 
            fast_image_resize::PixelType::U16
        ).unwrap();

        let mut resizer = fast_image_resize::Resizer::new(fast_image_resize::ResizeAlg::Convolution(fast_image_resize::FilterType::Bilinear));
        resizer.resize(&src_buffer.view(), &mut dst_buffer.view_mut()).unwrap();

    }));
}

criterion_group!(benches, interpolation_fir_bench, interpolation_rust_bench, interpolation_c_bench, neighbor_rust_inter);
criterion_main!(benches);