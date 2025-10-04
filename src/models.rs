#[derive(Clone, Copy, PartialEq)]
pub enum Cell {
    Wall,
    Floor,
    Target,
    BoxOnFloor,
    BoxOnTarget,
    PlayerOnFloor,
    PlayerOnTarget,
}

#[derive(Clone, Copy)]
pub struct Vec2 {
    pub i: i32,
    pub j: i32,
}
