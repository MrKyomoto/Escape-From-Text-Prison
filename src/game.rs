use crate::tile::Tile;
use crate::direction::{Direction, DIR_WORDS};
use crate::map::GameMap;
use std::time::Instant;

pub struct Game {
    pub map: GameMap,
    pub px: i32, pub py: i32,
    pub dir: Direction,
    pub steps: i32,
    pub history: Vec<(i32, i32, Direction, Tile, Tile, i32)>, // x, y, dir, orig_here, orig_dest, net_steps
    pub input_buf: String,
    pub game_over: bool,
    pub won: bool,
    pub step_count: u64,
    pub started: bool,
    start_time: Instant,
    finish_time: f32,
}

const COMMON_WORDS: &[&str] = &[
    "a", "an", "as", "at", "am", "be", "by", "do", "go", "he",
    "hi", "if", "in", "is", "it", "me", "my", "no", "of", "oh",
    "on", "or", "so", "to", "up", "us", "we",
    "all", "and", "any", "are", "bad", "big", "but", "can", "car",
    "cat", "cut", "did", "die", "dog", "eat", "end", "far", "few",
    "for", "get", "got", "had", "has", "her", "him", "his", "how",
    "its", "let", "man", "men", "new", "not", "now", "old", "one",
    "our", "out", "own", "put", "ran", "red", "run", "sat", "saw",
    "say", "see", "set", "she", "sit", "six", "ten", "the", "too",
    "top", "try", "two", "use", "war", "was", "way", "who", "why",
    "yes", "yet", "you",
    "able", "also", "back", "call", "come", "dark", "dead", "down",
    "each", "even", "eyes", "face", "fact", "fall", "feel", "find",
    "fire", "fish", "five", "foot", "four", "free", "from", "full",
    "gave", "girl", "give", "good", "hand", "have", "head", "hear",
    "help", "here", "hold", "home", "hope", "just", "keep", "kind",
    "king", "knew", "know", "land", "last", "lead", "left", "life",
    "like", "line", "list", "live", "long", "look", "lost", "love",
    "made", "make", "many", "mark", "mind", "miss", "more", "most",
    "much", "must", "name", "near", "need", "next", "nine", "none",
    "note", "open", "over", "page", "part", "pass", "past", "pick",
    "play", "pull", "push", "rain", "read", "real", "rest", "rich",
    "ride", "ring", "rise", "road", "rock", "room", "rule", "safe",
    "said", "same", "seem", "self", "ship", "show", "shut", "side",
    "sign", "sing", "size", "slow", "song", "soon", "sort", "star",
    "stay", "step", "stop", "such", "suit", "sure", "take", "talk",
    "tell", "them", "then", "they", "thin", "thing", "this", "time",
    "told", "took", "tree", "true", "turn", "upon", "walk", "want",
    "warm", "wash", "wear", "well", "went", "were", "west", "wide",
    "wife", "will", "wind", "wish", "with", "word", "work", "year",
    "young",
];

impl Game {
    pub fn new(map: GameMap, steps: i32, dir: Direction) -> Self {
        for y in 0..map.h {
            for x in 0..map.w {
                if map.get(x, y) == Tile::Start {
                    return Self {
                        map, px: x, py: y, dir, steps,
                        history: Vec::new(),
                        input_buf: String::new(),
                        game_over: false, won: false,
                        step_count: 0,
                        started: false,
                        start_time: Instant::now(),
                        finish_time: 0.0,
                    };
                }
            }
        }
        panic!("no start");
    }

    pub fn random_at(&self, x: i32, y: i32) -> u8 {
        ((x as u64 * 7 + y as u64 * 31 + self.step_count * 13) % 10) as u8
    }

    pub fn elapsed(&self) -> f32 {
        if self.finish_time > 0.0 { self.finish_time }
        else { self.start_time.elapsed().as_secs_f32() }
    }

    pub fn text_score(&self) -> u32 {
        // Count how many common English words appear in input_buf
        let s = self.input_buf.to_lowercase();
        let mut found = 0;
        for w in COMMON_WORDS {
            if s.contains(w) {
                found += 1;
            }
        }
        // Score: 0-100, 1 word = ~10pts, cap at 100
        (found as u32 * 10).min(100)
    }

    pub fn score(&self) -> (u32, u32, u32, u32) {
        let step_score = (self.steps.max(0) as u32) * 5;
        let time_score = 60.0_f32.max(600.0 - self.elapsed() * 2.0) as u32;
        let text_score = self.text_score();
        let total = (step_score as f32 * 0.3 + time_score as f32 * 0.2 + text_score as f32 * 0.5) as u32;
        (step_score, time_score, text_score, total)
    }

    pub fn do_char(&mut self, ch: char) {
        if self.game_over { return; }
        if !ch.is_ascii_lowercase() { return; }

        if !self.started {
            self.start_time = Instant::now();
        }
        self.started = true;
        self.step_count += 1;

        let (dx, dy) = self.dir.as_pair();
        let nx = self.px + dx;
        let ny = self.py + dy;
        let dest = self.map.get(nx, ny);

        // Block tile: only matching char clears it
        if let Tile::Block(e) = dest {
            if ch == e {
                self.map.set(nx, ny, Tile::Floor);
            } else {
                return; // blocked
            }
        } else if matches!(dest, Tile::Wall | Tile::Footprint(_)) {
            return;
        }

        let here = self.map.get(self.px, self.py);
        let old_dir = self.dir;

        let mut step_delta = -1;

        self.map.set(self.px, self.py, Tile::Footprint(ch));

        self.px = nx; self.py = ny;
        self.input_buf.push(ch);
        if self.input_buf.len() > 50 { self.input_buf.drain(..self.input_buf.len() - 50); }

        self.check_direction();

        // Step bonus at destination
        match self.map.get(self.px, self.py) {
            Tile::StepBonus(n) => {
                step_delta += n as i32;
                self.map.set(self.px, self.py, Tile::Floor);
            }
            Tile::RandomBonus => {
                let n = self.random_at(self.px, self.py);
                step_delta += n as i32;
                self.map.set(self.px, self.py, Tile::Floor);
            }
            _ => {}
        }

        self.steps += step_delta;
        // Push history: old_x, old_y, dir, tile we left behind, tile at dest, step_delta
        // Use (nx - dx, ny - dy) to get the old position
        let old_x = nx - dx;
        let old_y = ny - dy;
        self.history.push((old_x, old_y, old_dir, here, dest, step_delta));

        if self.map.get(self.px, self.py) == Tile::Goal {
            self.won = true;
            self.game_over = true;
            self.finish_time = self.start_time.elapsed().as_secs_f32();
        }
        if self.steps <= 0 && !self.game_over { self.game_over = true; }
    }

    fn check_direction(&mut self) {
        for (word, d) in DIR_WORDS {
            if self.input_buf.ends_with(word) { self.dir = *d; return; }
        }
    }

    pub fn undo(&mut self) {
        if self.won { return; }
        if let Some((x, y, d, orig_here, orig_dest, step_delta)) = self.history.pop() {
            self.map.set(x, y, orig_here);
            if self.px != x || self.py != y { self.map.set(self.px, self.py, orig_dest); }
            self.px = x; self.py = y; self.dir = d;
            self.steps -= step_delta; // reverse the exact step change
            self.input_buf.pop();
        }
    }
}