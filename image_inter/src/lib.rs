use libc::c_int;

extern "C" {
    fn scale_buffer(
        input_buffer:*const u16,
        input_buffer_width:c_int, 
        input_buffer_height:c_int, 
        
        output_buffer: *mut u8, 
        output_buffer_width:c_int, 
        output_buffer_height:c_int
    );
}

// This function implements bilinear interpolation scaling according to this article - http://tech-algorithm.com/articles/bilinear-image-scaling/
pub unsafe fn scale_bilinear<const INPUT_WIDTH:usize,const INPUT_HEIGHT:usize, const OUTPUT_WIDTH:usize, const OUTPUT_HEIGHT:usize>(input_buffer: *const u16, output_buffer: *mut u8){
    // not sure why the -1.0
    let x_ratio = (INPUT_WIDTH as f32 - 1.0) / OUTPUT_WIDTH as f32;
    let y_ratio = (INPUT_HEIGHT as f32 - 1.0) / OUTPUT_HEIGHT as f32;

    let mut offset_counter = 0;
    for y in 0..OUTPUT_HEIGHT{
        let y_val = (y_ratio * y as f32) as u32;            // y value of a point in this ratio between o and y
        let y_diff = (y_ratio * y as f32) - y_val as f32;

        for x in 0..OUTPUT_WIDTH{
            let x_val = (x_ratio * x as f32) as u32;            // x value of a point in this ratio between 0 and x
            let x_diff = (x_ratio * x as f32) - x_val as f32;   

            let original_pixel_index = (y_val as usize * INPUT_WIDTH) + x_val as usize;
            // Get the pixel and 3 surounding pixels
            let pixel_a = *input_buffer.add(original_pixel_index);
            let pixel_b = *input_buffer.add(original_pixel_index + 1);
            let pixel_c = *input_buffer.add(original_pixel_index + INPUT_WIDTH);
            let pixel_d = *input_buffer.add(original_pixel_index + INPUT_WIDTH + 1);
            
            let weights = [ (1.0-x_diff) * (1.0-y_diff), (x_diff)*(1.0-y_diff), y_diff * (1.0-x_diff), x_diff * y_diff];

            let blue:f32 = ((pixel_a & 0x1F) as f32 * weights[0]) + 
                           ((pixel_b & 0x1F) as f32 * weights[1]) + 
                           ((pixel_c & 0x1F) as f32 * weights[2]) + 
                           ((pixel_d & 0x1F) as f32 * weights[3]);
            let green:f32 = (((pixel_a >> 5) & 0x3F) as f32 * weights[0]) + 
                            (((pixel_b >> 5) & 0x3F) as f32 * weights[1]) + 
                            (((pixel_c >> 5) & 0x3F) as f32 * weights[2]) + 
                            (((pixel_d >> 5) & 0x3F) as f32 * weights[3]);
            let red:f32 = (((pixel_a >> 11) & 0x1F) as f32 * weights[0]) + 
                          (((pixel_b >> 11) & 0x1F) as f32 * weights[1]) + 
                          (((pixel_c >> 11) & 0x1F) as f32 * weights[2]) + 
                          (((pixel_d >> 11) & 0x1F) as f32 * weights[3]);

            let pixel = blue as u16 | ((green as u16) << 5) | ((red as u16) << 11);
            *output_buffer.add(offset_counter * 2) = (pixel >> 8) as u8;
            *output_buffer.add((offset_counter * 2) + 1) = (pixel & 0xFF) as u8;
            offset_counter += 1;
        }
    }
}

// implemented based on this article - https://kwojcicki.github.io/blog/NEAREST-NEIGHBOUR
pub unsafe fn scale_nearest<const INPUT_WIDTH:usize,const INPUT_HEIGHT:usize, const OUTPUT_WIDTH:usize, const OUTPUT_HEIGHT:usize>(input_buffer: *const u16, output_buffer: *mut u8, scale:f32){
    for y in 0..OUTPUT_HEIGHT{
        for x in 0..OUTPUT_WIDTH{
            let proj_x = ((1.0 / scale) * x as f32) as usize;
            let proj_y = ((1.0 / scale) * y as f32) as usize;
            let pixel = *input_buffer.add((proj_y * INPUT_WIDTH) + proj_x);
            let output_index = (y * OUTPUT_WIDTH) + x;
            *output_buffer.add(output_index * 2) = (pixel >> 8) as u8;
            *output_buffer.add((output_index * 2) + 1) = (pixel & 0xFF) as u8;
        }
    }
}

pub unsafe fn scale_biliniear_c<const INPUT_WIDTH:usize,const INPUT_HEIGHT:usize, const OUTPUT_WIDTH:usize, const OUTPUT_HEIGHT:usize>(input_buffer: *const u16, output_buffer: *mut u8){
    scale_buffer(
        input_buffer, 
        INPUT_WIDTH as c_int, 
        INPUT_HEIGHT as c_int, 
        output_buffer, 
        OUTPUT_WIDTH as c_int, 
        OUTPUT_HEIGHT as c_int
    );
}
