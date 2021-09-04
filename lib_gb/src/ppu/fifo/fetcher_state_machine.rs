use super::fetching_state::*;

pub struct FetcherStateMachine{
    pub state:usize,
    pub data:FetchingStateData,
    pub state_machine:[FetchingState;8]
}

impl FetcherStateMachine{
    pub fn advance(&mut self){
        self.state = (self.state + 1) % 8;
    }

    pub fn new(state_machine:[FetchingState;8])->Self{
        Self{
            data:FetchingStateData{low_tile_data:0},
            state:0,
            state_machine
        }
    }

    pub fn reset(&mut self){
        self.state = 0;
        self.data.low_tile_data = 0;
    }

    pub fn current_state(&self)->&FetchingState{
        &self.state_machine[self.state]
    }
}