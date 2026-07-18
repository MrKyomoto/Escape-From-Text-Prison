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
const OX: f32 = 20.0;
const OY: f32 = 20.0;

fn save_path() -> PathBuf {
    let mut p = dirs_next::data_dir().unwrap_or_else(|| PathBuf::from("."));
    p.push("text_prison");
    std::fs::create_dir_all(&p).ok();
    p.push("tutorials_done.flag");
    p
}

fn tutorials_done() -> bool {
    save_path().exists()
}

fn mark_tutorials_done() {
    std::fs::write(save_path(), "1").ok();
}

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

fn load_tutorial(idx: usize) -> Game {
    let (m, s, d, _) = load_map(TUTORIAL_SOURCES[idx]);
    Game::new(m, s, d)
}

fn load_hub() -> (Game, Vec<String>) {
    let (m, _, _, levels) = load_map(HUB_SRC);
    (Game::new(m, 9999, Direction::Right), levels)
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Text Prison".to_string(),
        window_width: 900,
        window_height: 600,
        ..Default::default()
    }
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
    let cx = |h: u32| Color::from_hex(h);

    loop {
        clear_background(cx(0x000000));

        if is_key_pressed(KeyCode::Escape) { break; }

        // Transition flag: set this to Some(NewScreen) to switch after this frame
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
                        if g.steps > 0 {
                            g.game_over = false;
                            while get_char_pressed().is_some() {}
                        }
                    }
                    if g.won {
                    if is_key_pressed(KeyCode::R) {
                        *g = load_tutorial(*idx);
                        while get_char_pressed().is_some() {}
                    }
                    if is_key_pressed(KeyCode::H) {
                        let (mut hub_g, levels) = load_hub();
                        if let Some((sx, sy)) = hub_spawn {
                            hub_g.px = sx;
                            hub_g.py = sy;
                        }
                        transition = Some(Screen::Hub(hub_g, levels));
                        while get_char_pressed().is_some() {}
                    }
                    if *idx + 1 < TUTORIAL_SOURCES.len() {
                        if is_key_pressed(KeyCode::N) {
                            *idx += 1;
                            *g = load_tutorial(*idx);
                            while get_char_pressed().is_some() {}
                        }
                    } else if is_key_pressed(KeyCode::N) {
                        mark_tutorials_done();
                        let (mut hub_g, levels) = load_hub();
                        if let Some((sx, sy)) = hub_spawn {
                            hub_g.px = sx;
                            hub_g.py = sy;
                        }
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

                // Draw map
                for y in 0..g.map.h {
                    for x in 0..g.map.w {
                        let tile = g.map.get(x, y);
                        let sx = OX + x as f32 * CELL;
                        let sy = OY + y as f32 * CELL;
                        draw_rectangle(sx, sy, CELL - 1.0, CELL - 1.0, cx(0x111111));
                        let (ch, col) = match tile {
                            Tile::Floor => ('.', cx(0x444444)),
                            Tile::Wall => ('#', cx(0x333333)),
                            Tile::Block(c) => (c, cx(0xCC4444)),
                            Tile::Start => ('.', cx(0x444444)),
                            Tile::Goal => ('*', cx(0xFFD700)),
                            Tile::Footprint(c) => (c, cx(0x66CC66)),
                            Tile::StepBonus(n) => (char::from(b'0' + n), cx(0x33DD33)),
                            Tile::RandomBonus => {
                                let n = g.random_at(x, y);
                                (char::from(b'0' + n), cx(0x33DDFF))
                            }
                        };
                        let mut buf = [0u8; 4];
                        let s = ch.encode_utf8(&mut buf);
                        draw_text(s, sx + 4.0, sy + 18.0, 18.0, col);
                    }
                }

                let sx = OX + g.px as f32 * CELL;
                let sy = OY + g.py as f32 * CELL;
                draw_rectangle(sx, sy, CELL - 1.0, CELL - 1.0, cx(0x222222));
                let mut buf = [0u8; 4];
                let s = g.dir.arrow().encode_utf8(&mut buf);
                draw_text(s, sx + 4.0, sy + 18.0, 18.0, cx(0xFFFFFF));

                let hy = OY + g.map.h as f32 * CELL + 12.0;
                let dir_s = match g.dir {
                    Direction::Up => "^ up", Direction::Down => "v down",
                    Direction::Left => "< left", Direction::Right => "> right",
                };
                let last20: String = g.input_buf.chars().rev().take(20).collect::<Vec<_>>().into_iter().rev().collect();

                draw_text(&format!("Level: {}", TUTORIAL_NAMES[*idx]), OX, hy, 20.0, cx(0x555555));
                draw_text(&format!("Steps: {}", g.steps), OX + 180.0, hy, 20.0, cx(0xFFAA00));
                let elapsed = g.elapsed();
                draw_text(&format!("Time: {}s", elapsed as u32), OX + 360.0, hy, 20.0, cx(0x8888FF));
                draw_text(&format!("Dir: {}", dir_s), OX, hy + 22.0, 20.0, cx(0xAAAAAA));
                draw_text(&format!("Input: {}", last20), OX, hy + 44.0, 18.0, cx(0x88AAFF));
                draw_text(&format!("History: {}", g.history.len()), OX, hy + 66.0, 18.0, cx(0x666666));
                draw_text("a-z: move  Backspace: undo  Esc: quit", OX, hy + 100.0, 16.0, cx(0x444444));

                let ly = hy + 120.0;
                let hint = match *idx {
                    0 => "",
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
                if !hint.is_empty() { draw_text(hint, OX, ly, 16.0, cx(0xFFAA00)); }

                if g.game_over {
                    let bx = 200.0; let by = 130.0; let bw = 500.0;
                    if g.won {
                        let (ss, ts, ws, total) = g.score();
                        let bh = 270.0;
                        draw_rectangle(bx, by, bw, bh, cx(0x222222));
                        draw_rectangle_lines(bx, by, bw, bh, 2.0, cx(0xFFFFFF));
                        draw_text("** WIN! **", 340.0, by + 30.0, 28.0, cx(0xFFD700));
                        draw_text(&format!("Time: {}s", elapsed as u32), bx + 30.0, by + 60.0, 20.0, cx(0x8888FF));
                        draw_text(&format!("Steps remaining: {}", g.steps), bx + 30.0, by + 85.0, 20.0, cx(0xFFAA00));
                        draw_text(&format!("Input: {}", g.input_buf), bx + 30.0, by + 110.0, 16.0, cx(0x88AAFF));
                        draw_text("--- Scores ---", bx + 30.0, by + 135.0, 18.0, cx(0xAAAAAA));
                        draw_text(&format!("Text quality (50%): {}", ws), bx + 30.0, by + 160.0, 20.0, cx(0x88AAFF));
                        draw_text(&format!("Step efficiency (30%): {}", ss), bx + 30.0, by + 185.0, 20.0, cx(0xFFAA00));
                        draw_text(&format!("Time bonus (20%): {}", ts), bx + 30.0, by + 210.0, 20.0, cx(0x8888FF));
                        draw_text(&format!("TOTAL: {}", total), bx + 30.0, by + 245.0, 24.0, cx(0xFFFFFF));
                        if *idx + 1 < TUTORIAL_SOURCES.len() {
                            draw_text("N: next level  H: back to hub  R: retry", 210.0, by + bh - 10.0, 18.0, cx(0xAAAAAA));
                        } else {
                            draw_text("H: back to hub  R: retry", 250.0, by + bh - 10.0, 18.0, cx(0xAAAAAA));
                        }
                    } else {
                        let bh = 140.0;
                        draw_rectangle(bx, by, bw, bh, cx(0x222222));
                        draw_rectangle_lines(bx, by, bw, bh, 2.0, cx(0xFFFFFF));
                        draw_text("Steps exhausted...", 280.0, by + 50.0, 30.0, cx(0xFF4444));
                        draw_text("Press R to restart | Backspace to undo | Esc to quit", 210.0, by + 100.0, 18.0, cx(0xAAAAAA));
                    }
                }
            }

            Screen::Hub(g, levels) => {
                if is_key_pressed(KeyCode::Backspace) { g.undo(); }
                while let Some(ch) = get_char_pressed() {
                    if ch.is_ascii_lowercase() { g.do_char(ch); }
                }

                // Check if standing on a goal tile → load corresponding level
                if g.map.get(g.px, g.py) == Tile::Goal && hub_cooldown == 0 {
                    // Find which goal we're on
                    let mut goal_idx = 0;
                    let mut found = false;
                    for y in 0..g.map.h {
                        for x in 0..g.map.w {
                            if g.map.get(x, y) == Tile::Goal {
                                if x == g.px && y == g.py {
                                    found = true;
                                    break;
                                }
                                goal_idx += 1;
                            }
                        }
                        if found { break; }
                    }
                    if found && goal_idx < levels.len() {
                        let name = &levels[goal_idx];
                        let tut_idx = TUTORIAL_NAMES.iter().position(|n| *n == name.as_str());
                        if let Some(ti) = tut_idx {
                            // Save the tile BEFORE the goal marker as spawn point
                            let (dx, dy) = g.dir.as_pair();
                            hub_spawn = Some((g.px - dx, g.py - dy));
                            transition = Some(Screen::Tutorial(ti, load_tutorial(ti)));
                        }
                    }
                }

                // Draw map
                for y in 0..g.map.h {
                    for x in 0..g.map.w {
                        let tile = g.map.get(x, y);
                        let sx = OX + x as f32 * CELL;
                        let sy = OY + y as f32 * CELL;
                        draw_rectangle(sx, sy, CELL - 1.0, CELL - 1.0, cx(0x111111));
                        let (ch, col) = match tile {
                            Tile::Floor => ('.', cx(0x444444)),
                            Tile::Wall => ('#', cx(0x333333)),
                            Tile::Block(c) => (c, cx(0xCC4444)),
                            Tile::Start => ('.', cx(0x444444)),
                            Tile::Goal => ('*', cx(0xFFD700)),
                            Tile::Footprint(c) => (c, cx(0x66CC66)),
                            _ => ('.', cx(0x444444)),
                        };
                        let mut buf = [0u8; 4];
                        let s = ch.encode_utf8(&mut buf);
                        draw_text(s, sx + 4.0, sy + 18.0, 18.0, col);
                    }
                }

                let sx = OX + g.px as f32 * CELL;
                let sy = OY + g.py as f32 * CELL;
                draw_rectangle(sx, sy, CELL - 1.0, CELL - 1.0, cx(0x222222));
                let mut buf = [0u8; 4];
                let s = g.dir.arrow().encode_utf8(&mut buf);
                draw_text(s, sx + 4.0, sy + 18.0, 18.0, cx(0xFFFFFF));

                let hy = OY + g.map.h as f32 * CELL + 12.0;
                let dir_s = match g.dir {
                    Direction::Up => "^ up", Direction::Down => "v down",
                    Direction::Left => "< left", Direction::Right => "> right",
                };
                draw_text("HUB - Walk to a * to enter a level", OX, hy, 20.0, cx(0x555555));
                draw_text(&format!("Dir: {}", dir_s), OX, hy + 22.0, 20.0, cx(0xAAAAAA));
                draw_text("a-z: move  Esc: quit", OX, hy + 44.0, 16.0, cx(0x444444));
            }
        }

        // Apply screen transition after the match block
        if let Some(new_screen) = transition {
            screen = new_screen;
            hub_cooldown = 3; // wait 3 frames before allowing hub entry again
        }
        if hub_cooldown > 0 { hub_cooldown -= 1; }

        next_frame().await
    }
}