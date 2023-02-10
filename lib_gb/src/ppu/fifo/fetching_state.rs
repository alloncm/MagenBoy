// Since each operation takes 2 t_cycles I pad them with sleep for my implementation
pub enum FetchingState{
    FetchTileNumber,
    FetchLowTile,
    FetchHighTile,
    Push,
    Sleep
}

pub struct FetchingStateData{
    pub tile_data_address:u16,
    pub tile_data:u8,
    pub low_tile_data:u8,
    pub high_tile_data:u8,
}

impl FetchingStateData{
    pub fn reset(&mut self){
        self.tile_data_address = 0;
        self.high_tile_data = 0;
        self.low_tile_data = 0;
        self.tile_data = 0;
    }
}

pub struct FetcherStateMachine{
    pub data:FetchingStateData,
    state:usize,
    state_machine:[FetchingState;8]
}

impl FetcherStateMachine{
    pub fn advance(&mut self){
        self.state = (self.state + 1) % 8;
    }

    pub fn new(state_machine:[FetchingState;8])->Self{
        Self{
            data:FetchingStateData{tile_data_address:0, high_tile_data:0, low_tile_data:0, tile_data:0},
            state:0,
            state_machine
        }
    }

    pub fn reset(&mut self){
        self.state = 0;
        self.data.reset();
    }

    pub fn current_state(&self)->&FetchingState{
        &self.state_machine[self.state]
    }
}