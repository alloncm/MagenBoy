// Since each operation takes 2 t_cycles I pad them with sleep for my implementation
pub enum FetchingState{
    FetchTileNumber,
    FetchLowTile,
    FetchHighTile,
    Push,
    Sleep
}



pub struct FetchingStateData{
    pub tile_data:u8,
    pub low_tile_data:u8,
    pub high_tile_data:u8,
}

impl FetchingStateData{
    pub fn reset(&mut self){
        self.high_tile_data = 0;
        self.low_tile_data = 0;
        self.tile_data = 0;
    }
}