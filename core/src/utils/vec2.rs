pub struct Vec2<T>{
    pub x:T,
    pub y:T
}

impl<T:Clone> Clone for Vec2<T>{
    fn clone(&self) -> Self {
        Self { x: self.x.clone(), y: self.y.clone() }
    }
}

impl<T:Copy> Copy for Vec2<T>{}