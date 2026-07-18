#[derive(Clone, Copy, PartialEq)]
pub enum Direction { Up, Down, Left, Right }

impl Direction {
    pub fn arrow(self) -> char {
        match self {
            Direction::Up => '^',
            Direction::Down => 'v',
            Direction::Left => '<',
            Direction::Right => '>',
        }
    }
    pub fn as_pair(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

pub const DIR_WORDS: &[(&str, Direction)] = &[
    ("up", Direction::Up),
    ("down", Direction::Down),
    ("left", Direction::Left),
    ("right", Direction::Right),
];