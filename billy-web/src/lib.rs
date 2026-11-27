//! WebAssembly-facing shell.
//!
//! Rendering is intentionally left to JavaScript.  The wasm side owns the
//! deterministic simulation and returns a compact JSON snapshot that the
//! Canvas 2D shell can draw at its own pace.

use wasm_bindgen::prelude::*;

use billy_config::GameConfig;
use billy_core::i18n::{self, Key, Locale};
use billy_core::sim::EntityKind;
use billy_core::{Input, Level, Sim, Status};

/// Injected by `build.sh`; makes a stale browser tab immediately obvious.
const BUILD: &str = match option_env!("OPEN_BILLY_BUILD") {
    Some(value) => value,
    None => "dev",
};

const INPUT_LEFT: u8 = 1 << 0;
const INPUT_RIGHT: u8 = 1 << 1;
const INPUT_UP: u8 = 1 << 2;
const INPUT_DOWN: u8 = 1 << 3;
/// Held Dig action.
const INPUT_DIG: u8 = 1 << 4;
/// Held Fill action.
const INPUT_FILL: u8 = 1 << 5;
/// Held Alt (magic-ladder modifier).
const INPUT_ALT: u8 = 1 << 6;

#[wasm_bindgen]
pub struct Game {
    sim: Sim,
    locale: Locale,
    level_name: String,
    last_events: Vec<String>,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(level_bytes: &[u8], locale: &str, level_name: &str) -> Result<Game, JsValue> {
        Self::build(level_bytes, billy_config::default_config(), locale, level_name)
    }

    /// Construct with a TOML configuration file (`open-billy.toml`).
    #[wasm_bindgen(js_name = newWithConfig)]
    pub fn new_with_config(
        level_bytes: &[u8],
        config_toml: &str,
        locale: &str,
        level_name: &str,
    ) -> Result<Game, JsValue> {
        let config = billy_config::from_toml(config_toml)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        Self::build(level_bytes, config, locale, level_name)
    }

    /// Construct with a main project file plus a JSON array of
    /// `[relative_path, toml_text]` world entries.
    #[wasm_bindgen(js_name = newWithConfigFiles)]
    pub fn new_with_config_files(
        level_bytes: &[u8],
        main_toml: &str,
        worlds_json: &str,
        locale: &str,
        level_name: &str,
    ) -> Result<Game, JsValue> {
        let config = config_from_files(main_toml, worlds_json)?;
        Self::build(level_bytes, config, locale, level_name)
    }

    /// Start the original demo level that ships with the shells.
    #[wasm_bindgen(js_name = demo)]
    pub fn demo(locale: &str) -> Result<Game, JsValue> {
        Self::demo_with_config_inner(billy_config::default_config(), locale)
    }

    /// Start the demo using a project configuration file.
    #[wasm_bindgen(js_name = demoWithConfig)]
    pub fn demo_with_config(config_toml: &str, locale: &str) -> Result<Game, JsValue> {
        let config = billy_config::from_toml(config_toml)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        Self::demo_with_config_inner(config, locale)
    }

    /// Start the demo with a main project file plus per-world TOML entries.
    #[wasm_bindgen(js_name = demoWithConfigFiles)]
    pub fn demo_with_config_files(
        main_toml: &str,
        worlds_json: &str,
        locale: &str,
    ) -> Result<Game, JsValue> {
        let config = config_from_files(main_toml, worlds_json)?;
        Self::demo_with_config_inner(config, locale)
    }

    /// Start the HUD certification level with project configuration files.
    /// Debug/testing only: it grants every power-up and seeds a maximal score.
    #[wasm_bindgen(js_name = demoHudWithConfigFiles)]
    pub fn demo_hud_with_config_files(
        main_toml: &str,
        worlds_json: &str,
        locale: &str,
    ) -> Result<Game, JsValue> {
        let config = config_from_files(main_toml, worlds_json)?;
        Self::demo_hud_inner(config, locale)
    }

    /// Start the HUD certification level with the embedded configuration.
    #[wasm_bindgen(js_name = demoHud)]
    pub fn demo_hud(locale: &str) -> Result<Game, JsValue> {
        Self::demo_hud_inner(billy_config::default_config(), locale)
    }

    fn build(
        level_bytes: &[u8],
        config: GameConfig,
        locale: &str,
        level_name: &str,
    ) -> Result<Game, JsValue> {
        let level =
            Level::from_bytes(level_bytes).map_err(|err| JsValue::from_str(&err.to_string()))?;
        let sim = Sim::with_config(level, config)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        Ok(Self {
            sim,
            locale: Locale::from_code(locale),
            level_name: level_name.to_string(),
            last_events: Vec::new(),
        })
    }

    fn demo_with_config_inner(config: GameConfig, locale: &str) -> Result<Game, JsValue> {
        let sim = Sim::with_config(Level::demo(), config)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        Ok(Self {
            sim,
            locale: Locale::from_code(locale),
            level_name: "demo".to_string(),
            last_events: Vec::new(),
        })
    }

    fn demo_hud_inner(config: GameConfig, locale: &str) -> Result<Game, JsValue> {
        let mut sim = Sim::with_config(Level::demo_hud(), config)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        // Stress the HUD with a seven-digit score; collecting loot pushes it
        // into eight digits while the level is being played.
        sim.score = 9_999_000;
        Ok(Self {
            sim,
            locale: Locale::from_code(locale),
            level_name: "hudlab".to_string(),
            last_events: Vec::new(),
        })
    }

    /// Debug-only: set score, lives and the level clock from the test panel.
    #[wasm_bindgen(js_name = debugSetState)]
    pub fn debug_set_state(&mut self, score: u32, lives: u32, time_seconds: u32) {
        self.sim.score = score;
        self.sim.lives = lives;
        self.sim.debug_set_time_seconds(time_seconds);
    }

    /// Debug-only: set every inventory item and quantity from the test panel.
    #[allow(clippy::too_many_arguments)]
    #[wasm_bindgen(js_name = debugSetItems)]
    pub fn debug_set_items(
        &mut self,
        bombs: u32,
        magic_ladders: u32,
        tennis_balls: u32,
        rainbow: u32,
        racket: bool,
        tennis_shoes: bool,
        ring: bool,
        power_glove: bool,
        jackhammer: bool,
        jackhammer_permanent: bool,
    ) {
        self.sim.bombs = bombs;
        self.sim.magic_ladders = magic_ladders;
        self.sim.tennis_balls = tennis_balls;
        self.sim.rainbow = rainbow;
        self.sim.racket = racket;
        self.sim.tennis_shoes = tennis_shoes;
        self.sim.ring = ring;
        self.sim.power_glove = power_glove;
        self.sim.jackhammer = jackhammer;
        self.sim.jackhammer_permanent = jackhammer_permanent;
    }

    /// Debug-only: set the Arcane Book permanent flags independently.
    #[wasm_bindgen(js_name = debugSetPermanents)]
    pub fn debug_set_permanents(
        &mut self,
        racket: bool,
        shoes: bool,
        glove: bool,
        ring: bool,
        jackhammer: bool,
    ) {
        self.sim.racket_permanent = racket;
        self.sim.shoes_permanent = shoes;
        self.sim.glove_permanent = glove;
        self.sim.ring_permanent = ring;
        self.sim.jackhammer_permanent = jackhammer;
        // A Permanent implies Billy carries the item.
        if racket {
            self.sim.racket = true;
        }
        if shoes {
            self.sim.tennis_shoes = true;
        }
        if glove {
            self.sim.power_glove = true;
        }
        if ring {
            self.sim.ring = true;
        }
        if jackhammer {
            self.sim.jackhammer = true;
        }
    }

    pub fn step(&mut self, input_bits: u8) {        let input = Input {
            left: input_bits & INPUT_LEFT != 0,
            right: input_bits & INPUT_RIGHT != 0,
            up: input_bits & INPUT_UP != 0,
            down: input_bits & INPUT_DOWN != 0,
            dig: input_bits & INPUT_DIG != 0,
            fill: input_bits & INPUT_FILL != 0,
            alt: input_bits & INPUT_ALT != 0,
        };
        self.sim.step(input);
        self.last_events = self.sim.events().iter().map(event_name).collect();
    }

    pub fn restart(&mut self) -> Result<(), JsValue> {
        self.sim = Sim::with_config(self.sim.level.clone(), self.sim.config.clone())
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        self.last_events.clear();
        Ok(())
    }

    /// Reload the current level with a fresh loot/RNG seed.  A replay of the
    /// same seed is deterministic; a new seed rerolls crate/secret loot.
    #[wasm_bindgen(js_name = newRun)]
    pub fn new_run(&mut self, seed: u32) -> Result<(), JsValue> {
        self.sim = Sim::with_config_seed(
            self.sim.level.clone(),
            self.sim.config.clone(),
            seed,
        )
        .map_err(|err| JsValue::from_str(&err.to_string()))?;
        self.last_events.clear();
        Ok(())
    }

    pub fn set_locale(&mut self, locale: &str) {
        self.locale = Locale::from_code(locale);
    }

    pub fn locale(&self) -> String {
        self.locale.code().to_string()
    }

    pub fn level_name(&self) -> String {
        self.level_name.clone()
    }

    pub fn to_level_bytes(&self) -> Vec<u8> {
        self.sim.level.to_bytes()
    }

    /// A compact JSON snapshot for the Canvas renderer and HUD.
    pub fn snapshot(&self) -> String {
        let sim = &self.sim;
        let mut grid = Vec::with_capacity(billy_core::GRID_H);
        for row in 0..billy_core::GRID_H {
            let mut line = String::with_capacity(billy_core::GRID_W);
            for col in 0..billy_core::GRID_W {
                let tile = sim.tiles.at(col, row);
                line.push(if tile.temp_ladder_ticks > 0 {
                    // `M`/`N` = animated Magic Ladder, in air / through a platform.
                    if tile.solid { 'N' } else { 'M' }
                } else if tile.ladder {
                    // `L` = ladder through a platform cell.
                    if tile.solid { 'L' } else { 'H' }
                } else {
                    match tile.dig_state {
                        billy_core::mechanics::DigState::FullyDug => 'o',
                        billy_core::mechanics::DigState::MostlyDug => '2',
                        billy_core::mechanics::DigState::PartlyDug => '1',
                        billy_core::mechanics::DigState::Normal if tile.solid => '#',
                        billy_core::mechanics::DigState::Normal => ' ',
                    }
                });
            }
            grid.push(format!("\"{}\"", escape(&line)));
        }

        let entities = sim
            .entities
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_visible())
            .map(|(id, e)| {
                let direction = e.patrol_dir.or(e.last_dir);
                let facing = direction
                    .map(|(dx, _)| if dx < 0 { "left" } else { "right" })
                    .unwrap_or("right");
                let (dir_x, dir_y) = direction.unwrap_or((0, 0));
                format!(
                    "{{\"id\":{},\"token\":\"{}\",\"kind\":\"{}\",\"col\":{},\"row\":{},\"points\":{},\"facing\":\"{}\",\"phase\":{},\"trapped\":{},\"moving\":{},\"dirX\":{},\"dirY\":{}}}",
                    id,
                    escape(&e.token),
                    kind_name(e.kind),
                    e.col,
                    e.row,
                    e.points,
                    facing,
                    e.anim_phase,
                    e.trapped,
                    e.patrol_dir.is_some(),
                    dir_x,
                    dir_y
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        let events = self
            .last_events
            .iter()
            .map(|e| format!("\"{}\"", escape(e)))
            .collect::<Vec<_>>()
            .join(",");

        let status = match sim.status {
            Status::Playing => "playing",
            Status::LevelComplete => "complete",
            Status::GameOver => "gameover",
            Status::TimeUp => "timeup",
        };
        let status_text = match sim.status {
            Status::Playing => "",
            Status::LevelComplete => i18n::text(self.locale, Key::LevelComplete),
            Status::GameOver => i18n::text(self.locale, Key::GameOver),
            Status::TimeUp => i18n::text(self.locale, Key::TimeUp),
        };

        let colors = &sim.world_colors;
        let world_colors = format!(
            concat!(
                "{{\"sky\":\"{sky}\",\"platform\":\"{platform}\",",
                "\"platformEdge\":\"{platform_edge}\",\"ladder\":\"{ladder}\",",
                "\"hole\":\"{hole}\",\"player\":\"{player}\",",
                "\"playerSkin\":\"{player_skin}\",\"playerHat\":\"{player_hat}\",",
                "\"playerEye\":\"{player_eye}\",\"hazard\":\"{hazard}\",",
                "\"collectible\":\"{collectible}\",\"portal\":\"{portal}\",",
                "\"text\":\"{text}\",\"shadow\":\"{shadow}\",\"grid\":\"{grid}\",",
                "\"panel\":\"{panel}\"}}"
            ),
            sky = colors.sky.to_hex(),
            platform = colors.platform.to_hex(),
            platform_edge = colors.platform_edge.to_hex(),
            ladder = colors.ladder.to_hex(),
            hole = colors.hole.to_hex(),
            player = colors.player.to_hex(),
            player_skin = colors.player_skin.to_hex(),
            player_hat = colors.player_hat.to_hex(),
            player_eye = colors.player_eye.to_hex(),
            hazard = colors.hazard.to_hex(),
            collectible = colors.collectible.to_hex(),
            portal = colors.portal.to_hex(),
            text = colors.text.to_hex(),
            shadow = colors.shadow.to_hex(),
            grid = colors.grid.to_hex(),
            panel = colors.panel.to_hex(),
        );
        let tile_assets = format!(
            concat!(
                "{{\"platformOdd\":{odd},\"platformEven\":{even},",
                "\"ladder\":{ladder},\"background\":{background},",
                "\"topAdornment\":{top_adornment}}}"
            ),
            odd = option_json(sim.world_config.tiles.platform_odd.as_deref()),
            even = option_json(sim.world_config.tiles.platform_even.as_deref()),
            ladder = option_json(sim.world_config.tiles.ladder.as_deref()),
            background = option_json(sim.world_config.tiles.background.as_deref()),
            top_adornment = option_json(
                sim.world_config.tiles.top_adornment.as_deref()
            ),
        );
        let token_sprites = sim
            .config
            .tokens
            .iter()
            .filter_map(|token| {
                token
                    .sprite
                    .as_ref()
                    .map(|sprite| format!("\"{}\":\"{}\"", escape(&token.id), escape(sprite)))
            })
            .collect::<Vec<_>>()
            .join(",");
        let token_sprites = format!("{{{token_sprites}}}");

        format!(
            concat!(
                "{{",
                "\"tick\":{tick},",
                "\"build\":\"{build}\",",
                "\"seed\":{seed},",
                "\"score\":{score},",
                "\"time\":{time:.1},",
                "\"timeProgress\":{time_progress:.4},",
                "\"lives\":{lives},",
                "\"bombs\":{bombs},",
                "\"magicLadders\":{magic_ladders},",
                "\"tennisBalls\":{tennis_balls},",
                "\"rainbow\":{rainbow},",
                "\"racket\":{racket},",
                "\"tennisShoes\":{tennis_shoes},",
                "\"ring\":{ring},",
                "\"powerGlove\":{power_glove},",
                "\"jackhammer\":{jackhammer},",
                "\"jackhammerPermanent\":{jackhammer_permanent},",
                "\"racketPermanent\":{racket_permanent},",
                "\"shoesPermanent\":{shoes_permanent},",
                "\"glovePermanent\":{glove_permanent},",
                "\"ringPermanent\":{ring_permanent},",
                "\"status\":\"{status}\",",
                "\"statusText\":\"{status_text}\",",
                "\"getReady\":{get_ready},",
                "\"world\":\"{world}\",",
                "\"worldLabel\":\"{world_label}\",",
                "\"level\":\"{level}\",",
                "\"locale\":\"{locale}\",",
                "\"title\":\"{title}\",",
                "\"subtitle\":\"{subtitle}\",",
                "\"worldColors\":{world_colors},",
                "\"tileAssets\":{tile_assets},",
                "\"tokenSprites\":{token_sprites},",
                "\"moveTicks\":{move_ticks},",
                "\"climbTicks\":{climb_ticks},",
                "\"monsterMoveTicks\":{monster_move_ticks},",
                "\"player\":{{\"col\":{col},\"row\":{row},\"renderRow\":{render_row:.5},\"climbProgress\":{climb_progress:.5},\"facing\":\"{facing}\",\"walking\":{walking},\"climbing\":{climbing},\"work\":\"{work}\"}},",
                "\"grid\":[{grid}],",
                "\"entities\":[{entities}],",
                "\"events\":[{events}],",
                "\"strings\":{{",
                "\"score\":\"{score_label}\",",
                "\"time\":\"{time_label}\",",
                "\"lives\":\"{lives_label}\",",
                "\"getReady\":\"{get_ready_label}\",",
                "\"bomb\":\"{bomb_label}\",",
                "\"ladder\":\"{ladder_label}\",",
                "\"controls\":\"{controls}\"",
                "}}",
                "}}"
            ),
            tick = sim.tick,
            build = BUILD,
            seed = sim.seed,
            score = sim.score,
            time = sim.time_left_seconds(),
            time_progress = sim.time_progress(),
            lives = sim.lives,
            bombs = sim.bombs,
            magic_ladders = sim.magic_ladders,
            tennis_balls = sim.tennis_balls,
            rainbow = sim.rainbow,
            racket = sim.racket,
            tennis_shoes = sim.tennis_shoes,
            ring = sim.ring,
            power_glove = sim.power_glove,
            jackhammer = sim.jackhammer,
            jackhammer_permanent = sim.jackhammer_permanent,
            racket_permanent = sim.racket_permanent,
            shoes_permanent = sim.shoes_permanent,
            glove_permanent = sim.glove_permanent,
            ring_permanent = sim.ring_permanent,
            status = status,
            status_text = escape(status_text),
            get_ready = sim.is_get_ready(),
            world = sim.world.as_str(),
            world_label = escape(sim.world_config.label(self.locale.code())),
            level = escape(&self.level_name),
            locale = self.locale.code(),
            title = escape(&sim.config.game.title),
            subtitle = escape(&sim.config.game.subtitle),
            world_colors = world_colors,
            tile_assets = tile_assets,
            token_sprites = token_sprites,
            col = sim.player.col,
            row = sim.player.row,
            facing = match sim.player.facing {
                billy_core::sim::Facing::Left => "left",
                billy_core::sim::Facing::Right => "right",
            },
            walking = sim.is_walking(),
            climbing = sim.is_climbing(),
            render_row = sim.render_row(),
            climb_progress = sim.climb_progress(),
            work = sim.work_animation().unwrap_or(""),
            move_ticks = sim.move_ticks_per_cell(),
            climb_ticks = sim.climb_ticks_per_cell(),
            monster_move_ticks = sim.monster_move_ticks_per_cell(),
            grid = grid.join(","),
            entities = entities,
            events = events,
            score_label = escape(i18n::text(self.locale, Key::Score)),
            time_label = escape(i18n::text(self.locale, Key::Time)),
            lives_label = escape(i18n::text(self.locale, Key::Lives)),
            get_ready_label = escape(i18n::text(self.locale, Key::GetReady)),
            bomb_label = escape(i18n::text(self.locale, Key::Bomb)),
            ladder_label = escape(i18n::text(self.locale, Key::Ladder)),
            controls = escape(i18n::text(self.locale, Key::Controls)),
        )
    }

    /// Validation output as JSON: `[{"severity":"error","message":"…"}]`.
    pub fn validate(&self) -> String {
        let warnings = billy_core::token::validate(&self.sim.level, &self.sim.config);
        let body = warnings
            .iter()
            .map(|warning| {
                format!(
                    "{{\"severity\":\"{}\",\"message\":\"{}\"}}",
                    match warning.severity {
                        billy_core::token::Severity::Info => "info",
                        billy_core::token::Severity::Warning => "warning",
                        billy_core::token::Severity::Error => "error",
                    },
                    escape(&warning.message)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("[{body}]")
    }
}

fn config_from_files(main_toml: &str, worlds_json: &str) -> Result<GameConfig, JsValue> {
    let pairs: Vec<(String, String)> = serde_json::from_str(worlds_json)
        .map_err(|err| JsValue::from_str(&err.to_string()))?;
    let refs: Vec<(&str, &str)> = pairs
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect();
    billy_config::from_toml_with_worlds(main_toml, &refs)
        .map_err(|err| JsValue::from_str(&err.to_string()))
}

/// Localized UI label for the JS shell.  Unknown keys fall back to the title.
#[wasm_bindgen(js_name = label)]
pub fn label(locale: &str, key: &str) -> String {
    let key = match key {
        "language" => Key::Language,
        "load" => Key::LoadLevel,
        "restart" => Key::Restart,
        "score" => Key::Score,
        "time" => Key::Time,
        "lives" => Key::Lives,
        "world" => Key::World,
        "level" => Key::Level,
        "getready" => Key::GetReady,
        "bomb" => Key::Bomb,
        "ladder" => Key::Ladder,
        "dig" => Key::Dig,
        "fill" => Key::Fill,
        "controls" => Key::Controls,
        _ => Key::Title,
    };
    i18n::text(Locale::from_code(locale), key).to_string()
}

fn event_name(event: &billy_core::Event) -> String {
    match event {
        billy_core::Event::Moved { .. } => "moved".to_string(),
        billy_core::Event::Fell { .. } => "fell".to_string(),
        billy_core::Event::Dug { .. } => "dug".to_string(),
        billy_core::Event::Filled { .. } => "filled".to_string(),
        billy_core::Event::MagicLadder { .. } => "magic-ladder".to_string(),
        billy_core::Event::Collected { token, .. } => format!("collected:{token}"),
        billy_core::Event::ObjectAppeared { token, .. } => format!("appeared:{token}"),
        billy_core::Event::Hurt { token } => format!("hurt:{token}"),
        billy_core::Event::LevelComplete => "level-complete".to_string(),
        billy_core::Event::GameOver => "game-over".to_string(),
        billy_core::Event::TimeUp => "time-up".to_string(),
    }
}

fn kind_name(kind: EntityKind) -> &'static str {
    match kind {
        EntityKind::Collectible => "collectible",
        EntityKind::Portal => "portal",
        EntityKind::Hazard => "hazard",
        EntityKind::Decoration => "decoration",
    }
}

fn option_json(value: Option<&str>) -> String {
    value
        .map(|text| format!("\"{}\"", escape(text)))
        .unwrap_or_else(|| "null".to_string())
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push(' '),
            c => out.push(c),
        }
    }
    out
}
