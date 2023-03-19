pub mod background_fetcher;
pub mod sprite_fetcher;
mod fetching_state;

pub const FIFO_SIZE:usize = 8;
pub const SPRITE_WIDTH:u8 = 8;

fn get_decoded_pixel(index: usize, low_data: u8, high_data: u8) -> u8 {
    let mask = 1 << index;
    let mut pixel = (low_data & mask) >> index;
    pixel |= ((high_data & mask) >> index) << 1;
    return pixel;
}