mod tile;
mod direction;
mod map;
mod game;

use macroquad::prelude::*;
use game::Game;
use map::load_map;
use tile::Tile;
use direction::Direction;
use std::path::PathBuf;

const CELL: f32 = 24.0;
const VIEW_TILES_W: f32 = 28.0;
const VIEW_TILES_H: f32 = 20.0;
const MAP_OX: f32 = 10.0;
const MAP_OY: f32 = 160.0; // below HUD

fn window_conf() -> Conf {
    Conf {
        window_title: "Text Prison".to_string(),
        window_width: (VIEW_TILES_W * CELL + MAP_OX * 2.0) as i32 + 80,
        window_height: (VIEW_TILES_H * CELL + MAP_OY * 2.0 + 180.0) as i32,
        ..Default::default()
    }
}

fn hex(h: u32) -> Color { Color::from_hex(h) }

fn save_path() -> PathBuf {
    let mut p = dirs_next::data_dir().unwrap_or_else(|| PathBuf::from("."));
    p.push("text_prison");
    std::fs::create_dir_all(&p).ok();
    p.push("tutorials_done.flag");
    p
}
fn tutorials_done() -> bool { save_path().exists() }
fn mark_tutorials_done() { std::fs::write(save_path(), "1").ok(); }

const HUB_SRC: &str = include_str!("../maps/hub.txt");
const TUTORIAL_SOURCES: &[&str] = &[
    include_str!("../maps/tutorial_01.txt"),
    include_str!("../maps/tutorial_02.txt"),
    include_str!("../maps/tutorial_03.txt"),
    include_str!("../maps/tutorial_04.txt"),
];
const TUTORIAL_NAMES: &[&str] = &[
    "tutorial_01",
    "tutorial_02",
    "tutorial_03",
    "tutorial_04",
];

enum Screen {
    Tutorial(usize, Game),
    Hub(Game, Vec<String>),
}

struct Camera { x: f32, y: f32 }

impl Camera {
    fn new() -> Self { Self { x: 0.0, y: 0.0 } }
    fn snap_to(&mut self, px: f32, py: f32) {
        self.x = px * CELL - VIEW_TILES_W * CELL / 2.0 + CELL / 2.0;
        self.y = py * CELL - VIEW_TILES_H * CELL / 2.0 + CELL / 2.0;
    }
    fn track(&mut self, px: f32, py: f32) {
        let tx = px * CELL - VIEW_TILES_W * CELL / 2.0 + CELL / 2.0;
        let ty = py * CELL - VIEW_TILES_H * CELL / 2.0 + CELL / 2.0;
        self.x += (tx - self.x) * 0.12;
        self.y += (ty - self.y) * 0.12;
    }
}

fn draw_tiles(g: &Game, cam: &Camera) {
    for y in 0..g.map.h {
        for x in 0..g.map.w {
            let sx = MAP_OX + x as f32 * CELL - cam.x;
            let sy = MAP_OY + y as f32 * CELL - cam.y;
            if sx + CELL < 0.0 || sx > screen_width() || sy + CELL < 0.0 || sy > screen_height() {
                continue;
            }
            draw_rectangle(sx, sy, CELL - 1.0, CELL - 1.0, hex(0x111111));
            let tile = g.map.get(x, y);
            let (ch, col) = match tile {
                Tile::Floor => ('.', hex(0x444444)),
                Tile::Wall => ('#', hex(0x333333)),
                Tile::Block(c) => (c, hex(0xCC4444)),
                Tile::Start => ('.', hex(0x444444)),
                Tile::Goal => ('*', hex(0xFFD700)),
                Tile::Footprint(c) => (c, hex(0x66CC66)),
                Tile::StepBonus(n) => (char::from(b'0' + n), hex(0x33DD33)),
                Tile::RandomBonus => (char::from(b'0' + g.random_at(x, y)), hex(0x33DDFF)),
            };
            draw_text(&ch.to_string(), sx + 4.0, sy + 18.0, 18.0, col);
        }
    }
    // Player on top
    let sx = MAP_OX + g.px as f32 * CELL - cam.x;
    let sy = MAP_OY + g.py as f32 * CELL - cam.y;
    draw_rectangle(sx, sy, CELL - 1.0, CELL - 1.0, hex(0x222222));
    draw_text(&g.dir.arrow().to_string(), sx + 4.0, sy + 18.0, 18.0, hex(0xFFFFFF));
}

fn load_tutorial(idx: usize) -> Game {
    let (m, s, d, _) = load_map(TUTORIAL_SOURCES[idx]);
    Game::new(m, s, d)
}

fn load_hub() -> (Game, Vec<String>) {
    let (m, _, _, levels) = load_map(HUB_SRC);
    (Game::new(m, 9999, Direction::Right), levels)
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut screen = if tutorials_done() {
        let (hub_g, hub_levels) = load_hub();
        Screen::Hub(hub_g, hub_levels)
    } else {
        Screen::Tutorial(0, load_tutorial(0))
    };
    let mut hub_spawn: Option<(i32, i32)> = None;
    let mut hub_cooldown: u8 = 0;
    let mut cam = Camera::new();

    loop {
        clear_background(hex(0x000000));
        if is_key_pressed(KeyCode::Escape) { break; }

        let mut transition: Option<Screen> = None;

        match &mut screen {
            Screen::Tutorial(idx, g) => {
                if g.game_over {
                    if is_key_pressed(KeyCode::R) {
                        *g = load_tutorial(*idx);
                        while get_char_pressed().is_some() {}
                    }
                    if !g.won && is_key_pressed(KeyCode::Backspace) {
                        g.undo();
                        if g.steps > 0 { g.game_over = false; while get_char_pressed().is_some() {} }
                    }
                    if g.won {
                        if is_key_pressed(KeyCode::R) {
                            *g = load_tutorial(*idx);
                            while get_char_pressed().is_some() {}
                        }
                        // H to hub only from tutorial_02 onward (after learning to turn)
                        if *idx >= 1 && is_key_pressed(KeyCode::H) {
                            let (mut hub_g, levels) = load_hub();
                            if let Some((sx, sy)) = hub_spawn { hub_g.px = sx; hub_g.py = sy; }
                            transition = Some(Screen::Hub(hub_g, levels));
                            while get_char_pressed().is_some() {}
                        }
                        if *idx + 1 < TUTORIAL_SOURCES.len() {
                            if is_key_pressed(KeyCode::N) {
                                *idx += 1; *g = load_tutorial(*idx);
                                while get_char_pressed().is_some() {}
                            }
                        } else if is_key_pressed(KeyCode::N) {
                            mark_tutorials_done();
                            let (mut hub_g, levels) = load_hub();
                            if let Some((sx, sy)) = hub_spawn { hub_g.px = sx; hub_g.py = sy; }
                            transition = Some(Screen::Hub(hub_g, levels));
                            while get_char_pressed().is_some() {}
                        }
                    }
                } else {
                    if is_key_pressed(KeyCode::Backspace) { g.undo(); }
                    while let Some(ch) = get_char_pressed() {
                        if ch.is_ascii_lowercase() { g.do_char(ch); }
                    }
                }
            }

            Screen::Hub(g, levels) => {
                if is_key_pressed(KeyCode::Backspace) { g.undo(); }
                while let Some(ch) = get_char_pressed() {
                    if ch.is_ascii_lowercase() { g.do_char(ch); }
                }
                if g.map.get(g.px, g.py) == Tile::Goal && hub_cooldown == 0 {
                    let mut goal_idx = 0;
                    let mut found = false;
                    for y in 0..g.map.h { for x in 0..g.map.w {
                        if g.map.get(x, y) == Tile::Goal {
                            if x == g.px && y == g.py { found = true; break; }
                            goal_idx += 1;
                        }
                    } if found { break; }}
                    if found && goal_idx < levels.len() {
                        let name = &levels[goal_idx];
                        if let Some(ti) = TUTORIAL_NAMES.iter().position(|n| *n == name.as_str()) {
                            let (dx, dy) = g.dir.as_pair();
                            hub_spawn = Some((g.px - dx, g.py - dy));
                            transition = Some(Screen::Tutorial(ti, load_tutorial(ti)));
                        }
                    }
                }
            }
        }

        // ── Render ──
        let (g, _is_tutorial): (&Game, bool) = match &screen {
            Screen::Tutorial(_, g) => (g, true),
            Screen::Hub(g, _) => (g, false),
        };
        cam.track(g.px as f32, g.py as f32);
        draw_tiles(g, &cam);

        // HUD (no background bar — map starts below it)
        let hy = 10.0;
        let dir_s = match g.dir {
            Direction::Up => "^ up", Direction::Down => "v down",
            Direction::Left => "< left", Direction::Right => "> right",
        };
        let last20: String = g.input_buf.chars().rev().take(20).collect::<Vec<_>>().into_iter().rev().collect();
        let level_name = match &screen {
            Screen::Tutorial(idx, _) => TUTORIAL_NAMES[*idx],
            Screen::Hub(_, _) => "Hub",
        };

        draw_text(&format!("Level: {}", level_name), MAP_OX, hy + 20.0, 20.0, hex(0x555555));
        draw_text(&format!("Steps: {}", g.steps), MAP_OX + 180.0, hy + 20.0, 20.0, hex(0xFFAA00));
        draw_text(&format!("Time: {}s", g.elapsed() as u32), MAP_OX + 360.0, hy + 20.0, 20.0, hex(0x8888FF));
        draw_text(&format!("Dir: {}", dir_s), MAP_OX, hy + 44.0, 20.0, hex(0xAAAAAA));
        draw_text(&format!("Input: {}", last20), MAP_OX, hy + 66.0, 18.0, hex(0x88AAFF));
        draw_text(&format!("History: {}", g.history.len()), MAP_OX, hy + 88.0, 18.0, hex(0x666666));
        draw_text("a-z: move  Backspace: undo  Esc: quit", MAP_OX, hy + 120.0, 16.0, hex(0x444444));

        // Tutorial hints
        let ly = hy + 140.0;
        if let Screen::Tutorial(idx, g) = &screen {
            let hint = match idx {
                1 => {
                    if g.won { "Direction keys: 'right' 'left' 'up' 'down' : type them to turn" }
                    else if g.input_buf.contains("down") { "Direction keys: 'right' 'left' 'up' 'down' : type them to turn which way you face" }
                    else if g.game_over { "Tip: press Backspace to undo, then try going down and right around the wall" }
                    else if g.step_count >= 8 && !g.input_buf.contains("down") { "Tip: you hit a wall! Press Backspace, then try 'down' to face down" }
                    else { "" }
                }
                2 => {
                    if g.game_over && !g.won { "Tip: you need more steps! Try walking over the green numbers to collect them" }
                    else { "" }
                }
                3 => {
                    let (dx, dy) = g.dir.as_pair();
                    let fwd = g.map.get(g.px + dx, g.py + dy);
                    if matches!(fwd, Tile::Footprint(_)) { "Your own footprints block the way: try a different route" }
                    else if g.game_over && !g.won { "Your own footprints block the way: try a different route" }
                    else if matches!(fwd, Tile::Block(_)) { "The red 'b' blocks the path: type b on it to break through" }
                    else { "" }
                }
                _ => "",
            };
            if !hint.is_empty() { draw_text(hint, MAP_OX, ly, 16.0, hex(0xFFAA00)); }
        }

        // Game over overlay
        if g.game_over {
            let bx = 200.0; let by = 130.0; let bw = 500.0;
            if g.won {
                let (ss, ts, ws, total) = g.score();
                let bh = 270.0;
                draw_rectangle(bx, by, bw, bh, hex(0x222222));
                draw_rectangle_lines(bx, by, bw, bh, 2.0, hex(0xFFFFFF));
                draw_text("** WIN! **", 340.0, by + 30.0, 28.0, hex(0xFFD700));
                draw_text(&format!("Time: {}s", g.elapsed() as u32), bx + 30.0, by + 60.0, 20.0, hex(0x8888FF));
                draw_text(&format!("Steps remaining: {}", g.steps), bx + 30.0, by + 85.0, 20.0, hex(0xFFAA00));
                draw_text(&format!("Input: {}", g.input_buf), bx + 30.0, by + 110.0, 16.0, hex(0x88AAFF));
                draw_text("--- Scores ---", bx + 30.0, by + 135.0, 18.0, hex(0xAAAAAA));
                draw_text(&format!("Text quality (50%): {}", ws), bx + 30.0, by + 160.0, 20.0, hex(0x88AAFF));
                draw_text(&format!("Step efficiency (30%): {}", ss), bx + 30.0, by + 185.0, 20.0, hex(0xFFAA00));
                draw_text(&format!("Time bonus (20%): {}", ts), bx + 30.0, by + 210.0, 20.0, hex(0x8888FF));
                draw_text(&format!("TOTAL: {}", total), bx + 30.0, by + 245.0, 24.0, hex(0xFFFFFF));
                if let Screen::Tutorial(idx, _) = &screen {
                    let controls = if *idx >= 1 { "N: next level  H: back to hub  R: retry" } else { "N: next level  R: retry" };
                    if *idx + 1 < TUTORIAL_SOURCES.len() {
                        draw_text(controls, 210.0, by + bh - 10.0, 18.0, hex(0xAAAAAA));
                    } else {
                        draw_text("H: back to hub  R: retry", 250.0, by + bh - 10.0, 18.0, hex(0xAAAAAA));
                    }
                }
            } else {
                draw_rectangle(bx, by, bw, 140.0, hex(0x222222));
                draw_rectangle_lines(bx, by, bw, 140.0, 2.0, hex(0xFFFFFF));
                draw_text("Steps exhausted...", 280.0, by + 50.0, 30.0, hex(0xFF4444));
                draw_text("Press R to restart | Backspace to undo | Esc to quit", 210.0, by + 100.0, 18.0, hex(0xAAAAAA));
            }
        }

        if let Some(new_screen) = transition {
            screen = new_screen;
            hub_cooldown = 3;
            match &screen {
                Screen::Tutorial(_, g) => cam.snap_to(g.px as f32, g.py as f32),
                Screen::Hub(g, _) => cam.snap_to(g.px as f32, g.py as f32),
            }
        }
        if hub_cooldown > 0 { hub_cooldown -= 1; }

        next_frame().await
    }
}