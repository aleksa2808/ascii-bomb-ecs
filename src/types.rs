#[derive(Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

pub enum PlayerAction {
    Move(Direction),
    DropBomb,
}
