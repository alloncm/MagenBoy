use super::fetching_state::*;

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
            data:FetchingStateData{high_tile_data:0, low_tile_data:0, tile_data:0},
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