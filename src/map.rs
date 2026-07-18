use crate::tile::Tile;
use crate::direction::Direction;

pub struct GameMap {
    tiles: Vec<Vec<Tile>>,
    pub w: i32,
    pub h: i32,
}

impl GameMap {
    pub fn get(&self, x: i32, y: i32) -> Tile {
        if x >= 0 && x < self.w && y >= 0 && y < self.h {
            self.tiles[y as usize][x as usize]
        } else {
            Tile::Wall
        }
    }
    pub fn set(&mut self, x: i32, y: i32, t: Tile) {
        if x >= 0 && x < self.w && y >= 0 && y < self.h {
            self.tiles[y as usize][x as usize] = t;
        }
    }
}

pub fn load_map(source: &str) -> (GameMap, i32, Direction, Vec<String>) {
    let mut steps = 30;
    let mut dir = Direction::Right;
    let mut grid_lines: Vec<&str> = Vec::new();
    let mut hub_levels: Vec<String> = Vec::new();

    for line in source.lines() {
        let trim = line.trim();
        if trim.is_empty() || trim.starts_with(';') { continue; }
        if let Some(n) = trim.strip_prefix("steps:") {
            steps = n.trim().parse().unwrap_or(30);
            continue;
        }
        if let Some(d) = trim.strip_prefix("start_dir:") {
            dir = match d.trim() {
                "up" => Direction::Up,
                "down" => Direction::Down,
                "left" => Direction::Left,
                _ => Direction::Right,
            };
            continue;
        }
        if let Some(l) = trim.strip_prefix("levels:") {
            hub_levels = l.split(',').map(|s| s.trim().to_string()).collect();
            continue;
        }
        grid_lines.push(trim);
    }

    let h = grid_lines.len() as i32;
    let w = grid_lines[0].len() as i32;
    let mut tiles = Vec::new();
    for row in grid_lines {
        let mut r = Vec::new();
        for ch in row.chars() {
            r.push(match ch {
                '#' => Tile::Wall,
                '@' => Tile::Start,
                '*' => Tile::Goal,
                '.' => Tile::Floor,
                '0'..='9' => Tile::StepBonus(ch as u8 - b'0'),
                '?' => Tile::RandomBonus,
                'b' => Tile::Block('b'),
                c if c.is_ascii_lowercase() => Tile::Floor,
                _ => Tile::Floor,
            });
        }
        tiles.push(r);
    }
    (GameMap { tiles, w, h }, steps, dir, hub_levels)
}