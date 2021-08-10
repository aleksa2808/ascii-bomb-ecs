#[derive(Clone, Copy)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

enum PlayerAction {
    Move(Direction),
    DropBomb,
}
