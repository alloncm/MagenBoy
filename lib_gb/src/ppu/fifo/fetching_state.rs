pub enum FetchingState{
    FetchTileNumber,
    FetchLowTile,
    FetchHighTile,
    Push,
    Sleep
}

pub union FetchingStateData{
    pub low_tile_data:u8,
    pub high_tile_data:(u8,u8),
    pub push_data:(u8,u8)
}