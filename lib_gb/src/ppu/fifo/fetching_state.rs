// Since each operation takes 2 t_cycles I pad them with sleep for my implementation
pub enum FetchingState{
    FetchTileNumber,
    FetchLowTile,
    FetchHighTile,
    Push,
    Sleep
}

pub struct FetchingStateData{
    pub tile_data:Option<u8>,
    pub low_tile_data:Option<u8>,
    pub high_tile_data:Option<u8>,
}

impl FetchingStateData{
    pub fn reset(&mut self){
        self.high_tile_data = None;
        self.low_tile_data = None;
        self.tile_data = None;
    }
}