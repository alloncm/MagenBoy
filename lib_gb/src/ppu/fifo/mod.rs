pub mod background_fetcher;
pub mod sprite_fetcher;
mod fetching_state;
mod fetcher_state_machine;

pub const FIFO_SIZE:usize = 8;
pub const SPRITE_WIDTH:u8 = 8;