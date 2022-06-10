#include<stdint.h>

extern void scale_buffer(const uint16_t* input_buffer, int input_buffer_width, int input_buffer_height, uint8_t* output_buffer, int output_buffer_width, int output_buffer_height){
    const float x_ratio = ((float)input_buffer_width - 1.0f) / (float)output_buffer_width;
    const float y_ratio = ((float)input_buffer_height - 1.0f) / (float)output_buffer_height;

    int output_offset_counter = 0;
    for (int y = 0; y < output_buffer_height; y++){
        const int y_val = (int)(y_ratio * (float)y);
        const float y_diff = (y_ratio * (float)y) - (float)y_val;

        for (int x = 0; x < output_buffer_width; x++){
            const int x_val = (int)(x_ratio * (float)x);
            const float x_diff = (x_ratio * (float)x) - (float)x_val;
            
            const int original_pixel_index = (y_val * input_buffer_width) + x_val;

            const uint16_t pixel_a = input_buffer[original_pixel_index];
            const uint16_t pixel_b = input_buffer[original_pixel_index + 1];
            const uint16_t pixel_c = input_buffer[original_pixel_index + input_buffer_width];
            const uint16_t pixel_d = input_buffer[original_pixel_index + input_buffer_width + 1];

            const float weights[4] = {(1.0f-x_diff) * (1.0f-y_diff), (x_diff)*(1.0f-y_diff), y_diff * (1.0f-x_diff), x_diff * y_diff};

            const float blue = ((float)(pixel_a & 0x1F) * weights[0]) + 
                               ((float)(pixel_b & 0x1F) * weights[1]) + 
                               ((float)(pixel_c & 0x1F) * weights[2]) + 
                               ((float)(pixel_d & 0x1F) * weights[3]);

            const float green = ((float)((pixel_a >> 5)& 0x3F) * weights[0]) + 
                                ((float)((pixel_b >> 5)& 0x3F) * weights[1]) + 
                                ((float)((pixel_c >> 5)& 0x3F) * weights[2]) + 
                                ((float)((pixel_d >> 5)& 0x3F) * weights[3]);

            const float red = ((float)((pixel_a >> 11)& 0x1F) * weights[0]) + 
                              ((float)((pixel_b >> 11)& 0x1F) * weights[1]) + 
                              ((float)((pixel_c >> 11)& 0x1F) * weights[2]) + 
                              ((float)((pixel_d >> 11)& 0x1F) * weights[3]);

            const uint16_t pixel = ((uint16_t)blue) | (((uint16_t)green) << 5) | (((uint16_t)red) << 11);

            output_buffer[output_offset_counter * 2] = (uint8_t)(pixel >> 8);
            output_buffer[(output_offset_counter * 2) + 1] = (uint8_t)(pixel && 0xFF);
            output_offset_counter++;
        }
    }
}