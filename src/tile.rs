#[derive(Clone, Copy, PartialEq)]
pub enum Tile {
    Floor,
    Wall,
    Block(char),
    Start,
    Goal,
    Footprint(char),
    StepBonus(u8),
    RandomBonus,
}