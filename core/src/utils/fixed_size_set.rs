pub struct FixedSizeSet<T, const SIZE:usize>{
    data:[T;SIZE],
    size:usize,
}

impl<T:Default + Copy + PartialEq, const SIZE:usize> FixedSizeSet<T, SIZE>{
    pub fn new()->Self{
        Self{ data: [T::default(); SIZE], size: 0 }
    }

    pub fn add(&mut self, value:T){
        if self.data.contains(&value){
            return;
        }
        self.data[self.size] = value;
        self.size += 1;
    }

    pub fn try_remove(&mut self, value:T)->bool{
        if !self.data.contains(&value) || self.size == 0 {
            return false;
        }
        let mut found = false;
        for i in 0..self.size {
            if found {
                self.data[i-1] = self.data[i];
            }
            if value == self.data[i]{
                found = true;
            }
        }
        self.size -= 1;
        return true;
    }

    pub fn as_slice(&self)->&[T]{&self.data[0..self.size]}
}