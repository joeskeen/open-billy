//! Desktop shell: a thin macroquad window around `billy-core`.
//!
//! World colours, labels, tile paths, asset pack locations and simulation
//! timings come from `open-billy.toml` (or `--config=path`).  There is no
//! hardcoded `match world` in this shell.

use std::env;
use std::fs;
use std::path::Path;

use billy_config::{self, GameConfig};
use billy_core::config::{Rgba8, WorldColors};
use billy_core::i18n::{self, Key, Locale};
use billy_core::mechanics::DigState;
use billy_core::sim::{EntityKind, Facing};
use billy_core::{Input, Level, Sim, Status, CELL_H, CELL_W, VIEW_H, VIEW_W};
use macroquad::prelude::*;

struct Shell {
    sim: Sim,
    locale: Locale,
    level_path: String,
    load_error: Option<String>,
    accumulator: f32,
}

impl Shell {
    fn new(
        locale: Locale,
        level_path: Option<String>,
        config: GameConfig,
        config_error: Option<String>,
    ) -> Self {
        let mut load_error = config_error;
        let (level, name) = match level_path {
            Some(path) => match fs::read(&path) {
                Ok(bytes) => match Level::from_bytes(&bytes) {
                    Ok(level) => (level, path),
                    Err(err) => {
                        load_error = Some(format!("{path}: {err}"));
                        (Level::demo(), "demo".to_string())
                    }
                },
                Err(err) => {
                    load_error = Some(format!("{path}: {err}"));
                    (Level::demo(), "demo".to_string())
                }
            },
            None => (Level::demo(), "demo".to_string()),
        };

        let sim = match Sim::with_config(level, config.clone()) {
            Ok(sim) => sim,
            Err(err) => {
                load_error = Some(format!("demo level: {err}"));
                Sim::with_config(Level::demo(), config)
                    .expect("configured demo must be playable")
            }
        };

        Self {
            sim,
            locale,
            level_path: name,
            load_error,
            accumulator: 0.0,
        }
    }

    fn restart(&mut self) {
        if let Ok(sim) = Sim::with_config(self.sim.level.clone(), self.sim.config.clone()) {
            self.sim = sim;
        }
    }

    fn input(&self) -> Input {
        Input {
            left: is_key_down(KeyCode::Left) || is_key_down(KeyCode::A),
            right: is_key_down(KeyCode::Right) || is_key_down(KeyCode::D),
            up: is_key_down(KeyCode::Up) || is_key_down(KeyCode::W),
            down: is_key_down(KeyCode::Down) || is_key_down(KeyCode::S),
            dig: is_key_down(KeyCode::Space),
            fill: is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift),
            alt: is_key_down(KeyCode::LeftAlt)
                || is_key_down(KeyCode::RightAlt)
                || is_key_down(KeyCode::M),
        }
    }

    fn text(&self, key: Key) -> &'static str {
        i18n::text(self.locale, key)
    }

    fn world_label(&self) -> &str {
        self.sim.world_config.label(self.locale.code())
    }

    fn colors(&self) -> &WorldColors {
        &self.sim.world_colors
    }
}

#[macroquad::main("Open Billy")]
async fn main() {
    request_new_screen_size(VIEW_W as f32, VIEW_H as f32);

    let args: Vec<String> = env::args().skip(1).collect();
    let (config, config_error) = load_config(&args);
    let locale = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--lang="))
        .map(Locale::from_code)
        .or_else(|| env::var("OPEN_BILLY_LANG").ok().map(|value| Locale::from_code(&value)))
        .unwrap_or_else(|| Locale::from_code(&config.game.default_locale));
    let level_path = args
        .iter()
        .find(|arg| !arg.starts_with("--"))
        .map(|arg| arg.to_string());

    let mut shell = Shell::new(locale, level_path, config, config_error);

    loop {
        if is_key_pressed(KeyCode::R) {
            shell.restart();
        }
        if is_key_pressed(KeyCode::L) {
            shell.locale = match shell.locale {
                Locale::English => Locale::Danish,
                Locale::Danish => Locale::Esperanto,
                Locale::Esperanto => Locale::English,
            };
        }
        if is_key_pressed(KeyCode::F1) {
            shell.locale = Locale::English;
        }
        if is_key_pressed(KeyCode::F2) {
            shell.locale = Locale::Danish;
        }
        if is_key_pressed(KeyCode::F3) {
            shell.locale = Locale::Esperanto;
        }

        if shell.sim.status == Status::Playing {
            let tick = 1.0 / shell.sim.config.simulation.tick_hz.max(1) as f32;
            shell.accumulator += get_frame_time().min(0.1);
            let mut steps = 0;
            while shell.accumulator >= tick && steps < 5 {
                shell.sim.step(shell.input());
                shell.accumulator -= tick;
                steps += 1;
            }
            if steps == 5 {
                shell.accumulator = 0.0;
            }
        }

        draw(&shell);
        next_frame().await;
    }
}

fn load_config(args: &[String]) -> (GameConfig, Option<String>) {
    if let Some(path) = args.iter().find_map(|arg| arg.strip_prefix("--config=")) {
        return match billy_config::load(path) {
            Ok(config) => (config, None),
            Err(err) => (
                billy_config::default_config(),
                Some(format!("config {path}: {err}; using the embedded default")),
            ),
        };
    }
    if let Ok(path) = env::var("OPEN_BILLY_CONFIG") {
        return match billy_config::load(&path) {
            Ok(config) => (config, None),
            Err(err) => (
                billy_config::default_config(),
                Some(format!("config {path}: {err}; using the embedded default")),
            ),
        };
    }
    let local = Path::new("open-billy.toml");
    if local.exists() {
        return match billy_config::load(local) {
            Ok(config) => (config, None),
            Err(err) => (
                billy_config::default_config(),
                Some(format!("open-billy.toml: {err}; using the embedded default")),
            ),
        };
    }
    (billy_config::default_config(), None)
}

fn draw(shell: &Shell) {
    let camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, VIEW_W as f32, VIEW_H as f32));
    set_camera(&camera);

    let colors = shell.colors();
    clear_background(color(colors.sky));
    draw_rectangle(0.0, 0.0, VIEW_W as f32, VIEW_H as f32, color(colors.sky));

    draw_tiles(shell, colors);
    draw_entities(shell, colors);
    draw_player(shell, colors);
    draw_hud(shell, colors);

    if let Some(error) = &shell.load_error {
        draw_text(
            error,
            18.0,
            VIEW_H as f32 - 38.0,
            18.0,
            Color::from_rgba(255, 120, 100, 255),
        );
    }

    set_default_camera();
}

fn draw_tiles(shell: &Shell, colors: &WorldColors) {
    for row in 0..billy_core::GRID_H {
        for col in 0..billy_core::GRID_W {
            let tile = shell.sim.tiles.at(col, row);
            let x = (col as f32) * CELL_W as f32;
            let y = (row as f32) * CELL_H as f32;
            let hole_depth = match tile.dig_state {
                DigState::Normal => 0.0,
                DigState::PartlyDug => 5.0,
                DigState::MostlyDug => 9.0,
                DigState::FullyDug => 13.0,
            };
            if tile.dig_state != DigState::FullyDug && hole_depth > 0.0 {
                draw_rectangle(
                    x,
                    y + CELL_H as f32 - 32.0,
                    CELL_W as f32,
                    32.0,
                    color(colors.platform),
                );
                draw_circle(
                    x + CELL_W as f32 / 2.0,
                    y + CELL_H as f32 - 44.0 + hole_depth,
                    CELL_W as f32 / 2.0,
                    color(colors.hole),
                );
            } else if tile.solid && tile.dig_state == DigState::FullyDug {
                draw_circle(
                    x + CELL_W as f32 / 2.0,
                    y + CELL_H as f32 - 12.0,
                    CELL_W as f32 / 2.0 - 1.0,
                    color(colors.hole),
                );
            } else if tile.solid {
                draw_rectangle(
                    x,
                    y + CELL_H as f32 - 32.0,
                    CELL_W as f32,
                    32.0,
                    color(colors.platform),
                );
                draw_rectangle(
                    x,
                    y + CELL_H as f32 - 32.0,
                    CELL_W as f32,
                    5.0,
                    color(colors.platform_edge),
                );
            } else if tile.ladder {
                let left = x + 5.0;
                let right = x + CELL_W as f32 - 5.0;
                draw_line(left, y, left, y + CELL_H as f32, 4.0, color(colors.ladder));
                draw_line(right, y, right, y + CELL_H as f32, 4.0, color(colors.ladder));
                let mut rung = 12.0;
                while rung < CELL_H as f32 {
                    draw_line(left, y + rung, right, y + rung, 3.0, color(colors.ladder));
                    rung += 16.0;
                }
            }
        }
    }
}

fn draw_entities(shell: &Shell, colors: &WorldColors) {
    for entity in &shell.sim.entities {
        if !entity.is_visible() {
            continue;
        }
        let x = entity.col as f32 * CELL_W as f32 + CELL_W as f32 / 2.0;
        let bottom = entity.row as f32 * CELL_H as f32 + CELL_H as f32 - 28.0;
        match entity.kind {
            EntityKind::Portal => {
                draw_circle(x, bottom - 20.0, 12.0, color(colors.portal));
                draw_circle(x, bottom - 20.0, 7.0, color(colors.sky));
            }
            EntityKind::Hazard => {
                draw_triangle(
                    Vec2::new(x, bottom - 30.0),
                    Vec2::new(x + 14.0, bottom),
                    Vec2::new(x - 14.0, bottom),
                    color(colors.hazard),
                );
            }
            EntityKind::Collectible => {
                draw_rectangle(x - 8.0, bottom - 20.0, 16.0, 20.0, color(colors.collectible));
                draw_rectangle_lines(
                    x - 8.0,
                    bottom - 20.0,
                    16.0,
                    20.0,
                    2.0,
                    color(colors.platform_edge),
                );
                let label = entity.token.chars().next().unwrap_or('?');
                draw_text(
                    &label.to_string(),
                    x - 3.0,
                    bottom - 7.0,
                    12.0,
                    color(colors.sky),
                );
            }
            EntityKind::Decoration => {}
        }
    }
}

fn draw_player(shell: &Shell, colors: &WorldColors) {
    let player = &shell.sim.player;
    let x = player.col as f32 * CELL_W as f32 + CELL_W as f32 / 2.0;
    let bottom = player.row as f32 * CELL_H as f32 + CELL_H as f32 - 28.0;
    draw_circle(x, bottom + 2.0, 5.0, color(colors.shadow));

    let bob = (shell.sim.tick as f32 / 7.0).sin() * 1.5;
    let top = bottom - 40.0 + bob;
    draw_rectangle(x - 6.0, top + 8.0, 12.0, 10.0, color(colors.player_skin));
    draw_rectangle(x - 9.0, top + 2.0, 18.0, 7.0, color(colors.player_hat));
    draw_rectangle(x - 8.0, top + 18.0, 16.0, 15.0, color(colors.player));
    draw_rectangle(x - 7.0, top + 33.0, 6.0, 7.0, color(colors.platform));
    draw_rectangle(x + 1.0, top + 33.0, 6.0, 7.0, color(colors.platform));
    draw_rectangle(x - 4.0, top + 12.0, 2.0, 2.0, color(colors.player_eye));
    draw_rectangle(x + 2.0, top + 12.0, 2.0, 2.0, color(colors.player_eye));

    let _ = player.facing == Facing::Left;
}

fn draw_hud(shell: &Shell, colors: &WorldColors) {
    draw_rectangle(10.0, 10.0, 500.0, 40.0, color(colors.panel));
    draw_text(
        &format!(
            "{}: {}   {}: {:.0}   {}: {}   {}: {}   {}",
            shell.text(Key::Score),
            shell.sim.score,
            shell.text(Key::Time),
            shell.sim.time_left_seconds().max(0.0),
            shell.text(Key::Lives),
            shell.sim.lives,
            shell.text(Key::World),
            shell.world_label(),
            shell.level_path,
        ),
        18.0,
        37.0,
        20.0,
        color(colors.text),
    );

    draw_text(
        shell.text(Key::Controls),
        18.0,
        VIEW_H as f32 - 14.0,
        18.0,
        color(colors.platform_edge),
    );

    if shell.sim.status != Status::Playing {
        draw_rectangle(
            0.0,
            0.0,
            VIEW_W as f32,
            VIEW_H as f32,
            color(colors.panel),
        );
        let message = match shell.sim.status {
            Status::Playing => "",
            Status::LevelComplete => shell.text(Key::LevelComplete),
            Status::GameOver => shell.text(Key::GameOver),
            Status::TimeUp => shell.text(Key::TimeUp),
        };
        let size = 44.0;
        let dimensions = measure_text(message, None, size as u16, 1.0);
        draw_text(
            message,
            (VIEW_W as f32 - dimensions.width) / 2.0,
            VIEW_H as f32 / 2.0,
            size,
            color(colors.collectible),
        );
        draw_text(
            shell.text(Key::Restart),
            (VIEW_W as f32 - 120.0) / 2.0,
            VIEW_H as f32 / 2.0 + 48.0,
            24.0,
            color(colors.text),
        );
    }
}

fn color(value: Rgba8) -> Color {
    Color::from_rgba(value.r, value.g, value.b, value.a)
}
