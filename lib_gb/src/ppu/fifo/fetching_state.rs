pub enum FethcingState{
    TileNumber,
    LowTileData(u8),
    HighTileData(u8,u8),
    Push(u8,u8)
}