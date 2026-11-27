//! Deterministic fixed-tick simulation.
//!
//! This is a clean-room approximation of the DOS rules, not a frame-identical
//! port.  It deliberately uses integer ticks and cell coordinates so a replay
//! on any platform produces the same result.
//!
//! The model follows the documented level semantics:
//! * the floor layer marks the platform Billy stands on;
//! * `H` is a ladder (no gravity, up/down to climb);
//! * digging removes an adjacent platform cell for a few seconds;
//! * `tid=` is the number of seconds of level time before an object appears;
//! * the bottom row is solid, so nothing can fall out of the playfield.

use std::collections::BTreeMap;

use crate::config::{EntityBehavior, GameConfig, PickupMode, WorldColors, WorldConfig, WorldId};
use crate::level::{GameObject, Level, GRID_H, GRID_W};
use crate::loot;
use crate::mechanics::{
    holes_overlap, DigState, ToolKind, WorkKind, WorkProgress, DEFAULT_SWING_TICKS,
};
use crate::program::{chase_step, Lcg, Program, ProgramStep};

/// Default logic rate.  The shells render at whatever rate they like and call
/// `step` once per tick; the actual rate comes from `GameConfig`.
pub const TICKS_PER_SECOND: u32 = 60;

pub const MOVE_TICKS: u32 = 8;
pub const FALL_TICKS: u32 = 4;
pub const DIG_COOLDOWN_TICKS: u32 = 12;
pub const HOLE_LIFETIME_TICKS: u32 = 3 * TICKS_PER_SECOND;
pub const EXTRA_TIME_TICKS: i64 = 30 * TICKS_PER_SECOND as i64;
pub const DEFAULT_TIME_SECONDS: f64 = 120.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facing {
    Left,
    Right,
}

impl Facing {
    pub fn step(self) -> isize {
        match self {
            Self::Left => -1,
            Self::Right => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    /// Held work key (**Space**).  It digs a Normal or partial tile, and fills
    /// a **Full Hole** (which cannot be dug deeper).
    pub dig: bool,
    /// Held explicit **Fill** (**Shift**), used to reverse a **Partial Hole**
    /// instead of digging it deeper.
    pub fill: bool,
    /// Held **Alt**: with Up/Down, deploys a **Magic Ladder**.
    pub alt: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Playing,
    LevelComplete,
    GameOver,
    TimeUp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Moved { col: usize, row: usize },
    Fell { col: usize, row: usize },
    Dug { col: usize, row: usize },
    Filled { col: usize, row: usize },
    MagicLadder { col: usize, row: usize },
    Collected { token: String, points: u32 },
    ObjectAppeared { token: String, col: usize, row: usize },
    Hurt { token: String },
    LevelComplete,
    GameOver,
    TimeUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tile {
    /// The level's platform texture is present here.  A dug tile keeps its
    /// platform and records the **Dig State** separately so it can be filled.
    pub solid: bool,
    pub ladder: bool,
    pub dig_state: DigState,
    /// Ticks left on a deployed **Magic Ladder** in this cell.
    pub temp_ladder_ticks: u32,
}

impl Tile {
    fn new(solid: bool, ladder: bool) -> Self {
        Self {
            solid,
            ladder,
            dig_state: DigState::Normal,
            temp_ladder_ticks: 0,
        }
    }

    pub fn is_hole(&self) -> bool {
        self.dig_state.is_hole()
    }
}

#[derive(Debug, Clone)]
pub struct Tiles {
    pub cells: [[Tile; GRID_W]; GRID_H],
}

impl Tiles {
    pub fn from_level(level: &Level) -> Self {
        let mut cells = [[Tile::new(false, false); GRID_W]; GRID_H];
        for row in 0..GRID_H {
            for col in 0..GRID_W {
                cells[row][col] = Tile::new(
                    level.grid.floor[row][col] == b'x',
                    level.grid.obj[row][col] == b'H',
                );
            }
        }
        Self { cells }
    }

    pub fn at(&self, col: usize, row: usize) -> Tile {
        self.cells[row][col]
    }

    /// A **Full Hole** is open: nothing may stand on it except a locked hole.
    pub fn is_full_hole(&self, col: usize, row: usize) -> bool {
        self.cells[row][col].dig_state == DigState::FullyDug
    }

    /// Platforms in the Normal, Partly Dug or Mostly Dug states still support
    /// weight.  A Fully Dug centre opens its whole three-cell **Hole Span**.
    pub fn is_solid(&self, col: usize, row: usize) -> bool {
        if !self.cells[row][col].solid {
            return false;
        }
        let first = col.saturating_sub(1);
        let last = (col + 1).min(GRID_W - 1);
        !(first..=last).any(|centre| self.is_full_hole(centre, row))
    }

    pub fn is_ladder(&self, col: usize, row: usize) -> bool {
        let tile = self.cells[row][col];
        tile.ladder || tile.temp_ladder_ticks > 0
    }

    pub fn dig_state(&self, col: usize, row: usize) -> DigState {
        self.cells[row][col].dig_state
    }
}

/// A **Hole** located by its centre.  `locked` is reserved for a hole that a
/// **Trapped Monster** is self-filling; self-fill timing is still open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Climb {
    from: usize,
    to: usize,
    progress: u32,
    total: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hole {
    pub centre: usize,
    pub state: DigState,
    pub locked: bool,
}

/// The hole span covering `col`, if any, without consulting entities.
fn hole_span_in_tiles(tiles: &Tiles, row: usize, col: usize) -> Option<(usize, DigState)> {
    let first = col.saturating_sub(1);
    let last = (col + 1).min(GRID_W - 1);
    for centre in first..=last {
        let state = tiles.cells[row][centre].dig_state;
        if state.is_hole() {
            return Some((centre, state));
        }
    }
    None
}

/// Whether a monster may move one cell in `(dx, dy)`.
///
/// Horizontal movement follows platforms, turns at ladders and partial holes,
/// and may step into an empty full hole (it is trapped next tick).  Vertical
/// movement uses air or a ladder and cannot pass through a bare platform.
fn monster_can_enter(
    tiles: &Tiles,
    col: usize,
    row: usize,
    dx: i32,
    dy: i32,
) -> Option<(usize, usize)> {
    let next_col = col as i32 + dx;
    let next_row = row as i32 + dy;
    if next_col < 0
        || next_col >= GRID_W as i32
        || next_row < 0
        || next_row >= GRID_H as i32
    {
        return None;
    }
    let (next_col, next_row) = (next_col as usize, next_row as usize);
    let tile = tiles.at(next_col, next_row);
    let hole = hole_span_in_tiles(tiles, next_row, next_col);
    let full = matches!(hole, Some((_, DigState::FullyDug)));
    let partial = matches!(
        hole,
        Some((_, DigState::PartlyDug | DigState::MostlyDug))
    );

    if dx != 0 {
        if tile.ladder || partial || (!tile.solid && !full) {
            return None;
        }
    } else if tile.solid && !tile.ladder && !full {
        return None;
    }
    Some((next_col, next_row))
}

/// Candidate directions for one program letter.  Chase tries the other axis
/// when its greedy axis is blocked; random tries the opposite direction.
fn monster_candidates(
    step: ProgramStep,
    rng: &mut Lcg,
    from: (i32, i32),
    player: (i32, i32),
) -> Vec<(i32, i32)> {
    match step {
        ProgramStep::Up => vec![(0, -1)],
        ProgramStep::Down => vec![(0, 1)],
        ProgramStep::Left => vec![(-1, 0)],
        ProgramStep::Right => vec![(1, 0)],
        ProgramStep::Random => {
            let direction = rng.direction();
            vec![direction, (-direction.0, -direction.1)]
        }
        ProgramStep::Chase => {
            let primary = chase_step(from, player);
            let mut out = Vec::new();
            if primary != (0, 0) {
                out.push(primary);
            }
            let tx = (player.0 - from.0).signum();
            let ty = (player.1 - from.1).signum();
            if primary.0 != 0 && ty != 0 {
                out.push((0, ty));
            }
            if primary.1 != 0 && tx != 0 {
                out.push((tx, 0));
            }
            out
        }
        ProgramStep::Noop => Vec::new(),
    }
}

/// Advance a **Pause Pickup** while Billy stands still; reset otherwise.
fn pickup_ready(entity: &mut Entity, standing_still: bool, pause_ticks: u32) -> bool {
    match entity.pickup {
        PickupMode::Instant => true,
        PickupMode::Pause => {
            if !standing_still {
                entity.pickup_ticks = 0;
                false
            } else {
                entity.pickup_ticks = entity.pickup_ticks.saturating_add(1);
                entity.pickup_ticks >= pause_ticks
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Collectible,
    Portal,
    Hazard,
    Decoration,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub token: String,
    pub kind: EntityKind,
    pub col: usize,
    pub row: usize,
    pub points: u32,
    pub activate_at_tick: u64,
    pub active: bool,
    pub alive: bool,
    /// Type-0 movement script, if this placed object is a Monster.
    pub program: Option<Program>,
    pub program_cooldown: u32,
    /// The direction the monster is currently committed to; it keeps moving
    /// until blocked or a ladder, then consumes the next program letter.
    pub patrol_dir: Option<(i32, i32)>,
    /// Last committed direction, used to reverse at a dead end.
    pub last_dir: Option<(i32, i32)>,
    pub rng: Lcg,
    /// Stable animation phase so same-type monsters do not animate in lockstep.
    pub anim_phase: u32,
    /// A **Trapped Monster** occupying a **Hole**.  It is harmless there.
    pub trapped: bool,
    pub pickup: PickupMode,
    /// Ticks Billy has stood still on a **Pause Pickup**.
    pub pickup_ticks: u32,
    /// The rolled `indhold` alternative for a Crate, or `None`.
    pub loot: Option<String>,
}

impl Entity {
    pub fn is_visible(&self) -> bool {
        self.active && self.alive && self.kind != EntityKind::Decoration
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub col: usize,
    pub row: usize,
    pub facing: Facing,
    pub alive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimError {
    MissingStart,
    InvalidStart,
    NoWorldConfig,
}

impl std::fmt::Display for SimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingStart => write!(f, "level has no Billy start (\"B\") marker"),
            Self::InvalidStart => write!(f, "Billy start is outside the 50×6 grid"),
            Self::NoWorldConfig => write!(f, "configuration has no worlds"),
        }
    }
}

impl std::error::Error for SimError {}

#[derive(Debug, Clone)]
pub struct Sim {
    pub level: Level,
    pub config: GameConfig,
    pub world: WorldId,
    pub world_config: WorldConfig,
    pub world_colors: WorldColors,
    pub tiles: Tiles,
    pub player: Player,
    pub entities: Vec<Entity>,
    pub tick: u64,
    /// Seed for loot rolls and monster RNG; fixed for a deterministic replay.
    pub seed: u32,
    pub time_left_ticks: i64,
    time_total_ticks: i64,
    /// Continuous ladder climb: row progress is interpolated, so Billy can
    /// stop mid-cell and reverse without finishing the climb.
    climbing: Option<Climb>,
    pub score: u32,
    pub lives: u32,
    pub status: Status,
    /// **Get Ready**: every level load starts frozen until Billy's first input.
    get_ready: bool,
    start: (usize, usize),
    move_cooldown: u32,
    fall_cooldown: u32,
    dig_cooldown: u32,
    /// The active **Work State**, if Billy is holding Dig or Fill.
    work: Option<WorkProgress>,
    work_target: Option<(usize, usize)>,
    /// Consumable Bomb charges; while non-zero they replace the Pickaxe.
    pub bombs: u32,
    /// Consumable **Magic Ladder** charges (token `tryllestav`).
    pub magic_ladders: u32,
    pub tennis_balls: u32,
    pub rainbow: u32,
    pub racket: bool,
    pub tennis_shoes: bool,
    pub ring: bool,
    pub power_glove: bool,
    pub jackhammer: bool,
    pub jackhammer_permanent: bool,
    /// The other four **Permanents**, granted only by an **Arcane Book**.
    pub racket_permanent: bool,
    pub shoes_permanent: bool,
    pub glove_permanent: bool,
    pub ring_permanent: bool,
    magic_cooldown: u32,
    /// One-shot **Buried Loot** rolled from each `hulbonus` object.
    buried_loot: BTreeMap<(usize, usize), String>,
    /// A `thrusign` **Edge Wrap** feature is present in this level.
    edge_wrap: bool,
    events: Vec<Event>,
}

impl Sim {
    pub fn with_config(level: Level, config: GameConfig) -> Result<Self, SimError> {
        Self::with_config_seed(level, config, 0)
    }

    /// Same as [`Sim::with_config`], but mixes `seed` into loot rolls and
    /// monster RNG.  A replay is exact for a fixed seed.
    pub fn with_config_seed(
        level: Level,
        config: GameConfig,
        seed: u32,
    ) -> Result<Self, SimError> {
        let (start_col, start_row) = level.find_start().ok_or(SimError::MissingStart)?;
        if start_col >= GRID_W || start_row >= GRID_H {
            return Err(SimError::InvalidStart);
        }

        let config = config.normalize();
        if config.worlds.is_empty() {
            return Err(SimError::NoWorldConfig);
        }
        let world = level.world();
        let world_config = config.world_or_default(world.as_str()).clone();
        let world_colors = world_config.resolved_colors();
        let tiles = Tiles::from_level(&level);
        let budget = level.time_budget_seconds();
        let time_seconds = if budget.is_finite() && budget > 0.0 {
            budget
        } else {
            config.simulation.default_time_seconds
        };
        let time_left_ticks =
            (time_seconds * config.simulation.tick_hz.max(1) as f64).round() as i64;

        let edge_wrap = level.objects.iter().any(|object| object.token == "thrusign");
        let mut entities = Vec::new();
        let mut buried_loot = BTreeMap::new();
        for object in &level.objects {
            if object.token == "bane" {
                continue;
            }
            if object.token == "hulbonus" {
                if let Some(text) = object.attr("indhold") {
                    if let Some((col, row)) = level.object_anchor(object) {
                        if col < GRID_W && row < GRID_H {
                            let mut rng = Lcg::new(
                                object_seed(object, col, row) ^ seed.rotate_left(16),
                            );
                            let chosen = loot::roll_alternative(text, &mut rng);
                            if !chosen.is_empty() {
                                buried_loot.insert((col, row), chosen.to_string());
                            }
                        }
                    }
                }
                continue;
            }
            if let Some(entity) = entity_from_object(&level, object, &config, seed) {
                entities.push(entity);
            }
        }
        let lives = config.simulation.lives;

        Ok(Self {
            level,
            config,
            world,
            world_config,
            world_colors,
            tiles,
            player: Player {
                col: start_col,
                row: start_row,
                facing: Facing::Right,
                alive: true,
            },
            entities,
            tick: 0,
            seed,
            time_left_ticks,
            time_total_ticks: time_left_ticks,
            climbing: None,
            score: 0,
            lives,
            status: Status::Playing,
            get_ready: true,
            start: (start_col, start_row),
            move_cooldown: 0,
            fall_cooldown: 0,
            dig_cooldown: 0,
            work: None,
            work_target: None,
            bombs: 0,
            magic_ladders: 0,
            tennis_balls: 0,
            rainbow: 0,
            racket: false,
            tennis_shoes: false,
            ring: false,
            power_glove: false,
            jackhammer: false,
            jackhammer_permanent: false,
            racket_permanent: false,
            shoes_permanent: false,
            glove_permanent: false,
            ring_permanent: false,
            magic_cooldown: 0,
            buried_loot,
            edge_wrap,
            events: Vec::new(),
        })
    }

    fn tick_hz(&self) -> u32 {
        self.config.simulation.tick_hz.max(1)
    }

    fn move_ticks(&self) -> u32 {
        self.world_config
            .physics
            .move_ticks
            .unwrap_or(self.config.simulation.move_ticks)
    }

    fn fall_ticks(&self) -> u32 {
        self.world_config
            .physics
            .fall_ticks
            .unwrap_or(self.config.simulation.fall_ticks)
    }

    fn climb_ticks(&self) -> u32 {
        let base = self
            .world_config
            .physics
            .climb_ticks
            .unwrap_or(self.config.simulation.climb_ticks);
        // Tennis Shoes speed the climb up; exact factor still needs RE.
        if self.tennis_shoes {
            (base * 2 / 3).max(1)
        } else {
            base
        }
    }

    fn dig_cooldown_ticks(&self) -> u32 {
        self.world_config
            .physics
            .dig_cooldown_ticks
            .unwrap_or(self.config.simulation.dig_cooldown_ticks)
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }

    pub fn is_get_ready(&self) -> bool {
        self.get_ready
    }

    /// True while Billy is in the short walk-step animation window.
    pub fn is_walking(&self) -> bool {
        self.move_cooldown > 0
    }

    pub fn is_climbing(&self) -> bool {
        self.climbing.is_some()
    }

    /// 0.0 at the starting row, 1.0 at the destination row.
    pub fn climb_progress(&self) -> f64 {
        self.climbing
            .map(|climb| {
                climb.progress as f64 / climb.total.max(1) as f64
            })
            .unwrap_or(0.0)
    }

    /// Fractional row for rendering a mid-climb Billy.
    pub fn render_row(&self) -> f64 {
        match self.climbing {
            Some(climb) => {
                let t = climb.progress as f64 / climb.total.max(1) as f64;
                climb.from as f64 + (climb.to as f64 - climb.from as f64) * t
            }
            None => self.player.row as f64,
        }
    }

    pub fn move_ticks_per_cell(&self) -> u32 {
        self.move_ticks()
    }

    pub fn climb_ticks_per_cell(&self) -> u32 {
        self.climb_ticks()
    }

    pub fn monster_move_ticks_per_cell(&self) -> u32 {
        self.config.simulation.monster_move_ticks.max(1)
    }

    /// Which held-work animation to show, if any.
    pub fn work_animation(&self) -> Option<&'static str> {
        match self.work?.kind {
            WorkKind::Dig => Some(if self.jackhammer { "drill" } else { "dig" }),
            WorkKind::Fill => Some("hammer"),
        }
    }

    pub fn time_left_seconds(&self) -> f64 {
        self.time_left_ticks as f64 / self.tick_hz() as f64
    }

    /// Fraction of the level clock that has elapsed, `0.0..=1.0`.
    pub fn time_progress(&self) -> f64 {
        if self.time_total_ticks <= 0 {
            1.0
        } else {
            (1.0 - self.time_left_ticks as f64 / self.time_total_ticks as f64)
                .clamp(0.0, 1.0)
        }
    }

    pub fn player_cell(&self) -> (usize, usize) {
        (self.player.col, self.player.row)
    }

    /// Advance exactly one 60 Hz tick.
    pub fn step(&mut self, input: Input) -> &[Event] {
        self.events.clear();
        if self.status != Status::Playing {
            return &self.events;
        }

        if self.get_ready {
            let started = input.left
                || input.right
                || input.up
                || input.down
                || input.dig
                || input.fill;
            if !started {
                return &self.events;
            }
            self.get_ready = false;
        }

        self.tick += 1;
        self.time_left_ticks -= 1;

        if self.time_left_ticks <= 0 {
            self.time_left_ticks = 0;
            self.status = Status::TimeUp;
            self.events.push(Event::TimeUp);
            return &self.events;
        }

        if self.move_cooldown > 0 {
            self.move_cooldown -= 1;
        }
        if self.fall_cooldown > 0 {
            self.fall_cooldown -= 1;
        }
        if self.dig_cooldown > 0 {
            self.dig_cooldown -= 1;
        }
        if self.magic_cooldown > 0 {
            self.magic_cooldown -= 1;
        }

        self.activate_entities();
        self.expire_magic_ladders();
        self.handle_magic_ladder(input);
        self.handle_vertical(input);
        self.handle_horizontal(input);
        self.handle_work(input);
        self.handle_gravity();
        self.step_monsters();
        self.handle_entities(input);

        &self.events
    }

    fn activate_entities(&mut self) {
        let tick = self.tick;
        let mut appeared = Vec::new();
        for entity in &mut self.entities {
            if !entity.active && tick >= entity.activate_at_tick && entity.activate_at_tick > 0 {
                entity.active = true;
                appeared.push(Event::ObjectAppeared {
                    token: entity.token.clone(),
                    col: entity.col,
                    row: entity.row,
                });
            }
        }
        self.events.extend(appeared);
    }

    /// Debug-only: refill the level clock to `seconds` and make that the maximum
    /// so the time bar reads correctly.
    pub fn debug_set_time_seconds(&mut self, seconds: u32) {
        let ticks = seconds as i64 * self.tick_hz() as i64;
        self.time_left_ticks = ticks;
        self.time_total_ticks = ticks;
    }

    /// Ticks Billy must stand still on a **Pause Pickup**.
    fn pause_pickup_ticks(&self) -> u32 {
        self.config.simulation.pause_pickup_ticks.max(1)
    }

    fn expire_magic_ladders(&mut self) {
        for row in 0..GRID_H {
            for col in 0..GRID_W {
                let tile = &mut self.tiles.cells[row][col];
                if tile.temp_ladder_ticks > 0 {
                    tile.temp_ladder_ticks -= 1;
                }
            }
        }
    }

    fn magic_ladder_ticks(&self) -> u32 {
        self.config.simulation.magic_ladder_ticks.max(1)
    }

    /// **Alt + Up/Down** deploys one **Magic Ladder** cell above or below
    /// Billy, consuming one `tryllestav` charge.
    fn handle_magic_ladder(&mut self, input: Input) {
        if self.climbing.is_some()
            || !input.alt
            || self.magic_cooldown > 0
            || self.magic_ladders == 0
        {
            return;
        }
        let direction = match (input.up, input.down) {
            (true, false) => -1,
            (false, true) => 1,
            _ => return,
        };
        let (col, row) = (self.player.col, self.player.row);
        let target = row as isize + direction;
        if target < 0 || target >= GRID_H as isize {
            return;
        }
        let target = target as usize;
        if self.tiles.is_ladder(col, target) {
            return;
        }
        // A magic ladder is a two-cell segment: Billy's own cell plus the
        // chosen direction.  Existing platforms are allowed in between.
        let ticks = self.magic_ladder_ticks();
        self.tiles.cells[row][col].temp_ladder_ticks = ticks;
        self.tiles.cells[target][col].temp_ladder_ticks = ticks;
        self.magic_ladders -= 1;
        self.magic_cooldown = self.dig_cooldown_ticks();
        self.events.push(Event::MagicLadder { col, row: target });
    }

    fn handle_vertical(&mut self, input: Input) {
        if self.work.is_some() || input.alt || self.move_cooldown > 0 {
            return;
        }
        let (col, row) = (self.player.col, self.player.row);

        // Continue or reverse an in-progress climb.  The row only changes when
        // progress reaches the end; releasing the key leaves Billy mid-rung.
        if let Some(mut climb) = self.climbing {
            let direction = if input.up {
                -1
            } else if input.down {
                1
            } else {
                0
            };
            if direction == 0 {
                self.climbing = Some(climb);
                return;
            }
            let climb_dir = if climb.to < climb.from { -1 } else { 1 };
            if direction == climb_dir {
                climb.progress += 1;
                if climb.progress >= climb.total {
                    self.player.row = climb.to;
                    self.climbing = None;
                    self.events.push(Event::Moved {
                        col,
                        row: climb.to,
                    });
                } else {
                    self.climbing = Some(climb);
                }
            } else {
                climb.progress = climb.progress.saturating_sub(1);
                if climb.progress == 0 {
                    self.player.row = climb.from;
                    self.climbing = None;
                } else {
                    self.climbing = Some(climb);
                }
            }
            return;
        }

        // Billy must already be on a ladder.  There is no jumping and no
        // teleporting onto a ladder from underneath it.
        if !self.tiles.is_ladder(col, row) {
            return;
        }
        let direction = if input.up {
            -1
        } else if input.down {
            1
        } else {
            0
        };
        if direction == 0 {
            return;
        }
        let target = row as isize + direction;
        if target < 0 || target >= GRID_H as isize {
            return;
        }
        let target = target as usize;
        // A ladder may pass through a solid platform cell.
        if !self.tiles.is_solid(col, target) || self.tiles.is_ladder(col, target) {
            self.climbing = Some(Climb {
                from: row,
                to: target,
                progress: 0,
                total: self.climb_ticks(),
            });
        }
    }

    fn handle_horizontal(&mut self, input: Input) {
        if self.work.is_some() || self.move_cooldown > 0 || self.climbing.is_some() {
            return;
        }
        let direction = match (input.left, input.right) {
            (true, false) => Facing::Left,
            (false, true) => Facing::Right,
            _ => return,
        };
        // Facing is chosen even when the step is blocked, so Billy can turn
        // and dig the other way.
        self.player.facing = direction;
        let next = self.player.col as isize + direction.step();
        // A `thrusign` **Edge Wrap** continues at the opposite edge.
        let next = if next < 0 {
            if !self.edge_wrap {
                return;
            }
            GRID_W - 1
        } else if next >= GRID_W as isize {
            if !self.edge_wrap {
                return;
            }
            0
        } else {
            next as usize
        };
        if !self.can_enter_horizontally(next, self.player.row) {
            return;
        }
        self.player.col = next;
        self.move_cooldown = self.move_ticks();
        self.events.push(Event::Moved {
            col: self.player.col,
            row: self.player.row,
        });
    }

    /// Horizontal movement may not cross a **Partial Hole**.  An empty **Full
    /// Hole** may be entered (Billy then drops through it); a monster-filled or
    /// locked hole blocks entry from the same level.
    fn can_enter_horizontally(&self, col: usize, row: usize) -> bool {
        match self.hole_span_at(row, col) {
            None => true,
            Some(hole) => match hole.state {
                DigState::FullyDug => !self.hole_has_monster(hole.centre, row) && !hole.locked,
                _ => false,
            },
        }
    }

    /// The hole (if any) whose three-cell **Hole Span** covers `col`.
    fn hole_span_at(&self, row: usize, col: usize) -> Option<Hole> {
        let first = col.saturating_sub(1);
        let last = (col + 1).min(GRID_W - 1);
        for centre in first..=last {
            let state = self.tiles.cells[row][centre].dig_state;
            if state.is_hole() {
                return Some(Hole {
                    centre,
                    state,
                    locked: false,
                });
            }
        }
        None
    }

    fn hole_has_monster(&self, centre: usize, row: usize) -> bool {
        self.entities
            .iter()
            .any(|entity| entity.trapped && entity.col == centre && entity.row == row)
    }

    /// The **Dig Target**: the **Hole Center** two cells ahead.  The bottom
    /// boundary row is never worked.
    fn dig_target(&self) -> Option<(usize, usize)> {
        let row = self.player.row;
        if row == GRID_H - 1 {
            // The bottom boundary row is never dug or filled.
            return None;
        }
        if !self.tiles.is_solid(self.player.col, row) {
            // Billy must be standing on a platform to work.
            return None;
        }
        let col = self.player.col as isize + self.player.facing.step() * 2;
        if col < 0 || col >= GRID_W as isize {
            return None;
        }
        Some((col as usize, row))
    }

    fn handle_work(&mut self, input: Input) {
        if self.climbing.is_some() {
            return;
        }
        if !(input.dig || input.fill) {
            if let Some(mut work) = self.work.take() {
                work.release();
            }
            self.work_target = None;
            return;
        }
        if self.dig_cooldown > 0 {
            return;
        }
        // Shift is always Fill.  Space continues the current action, or picks
        // Fill for a Full Hole and Dig otherwise.
        let kind = if input.fill {
            WorkKind::Fill
        } else if let Some(work) = self.work {
            work.kind
        } else {
            let Some((col, row)) = self.dig_target() else {
                return;
            };
            if self.tiles.cells[row][col].dig_state == DigState::FullyDug {
                WorkKind::Fill
            } else {
                WorkKind::Dig
            }
        };
        if self.work.map(|work| work.kind) != Some(kind) {
            self.work = None;
            self.work_target = None;
        }
        if self.work.is_none() {
            let Some((col, row)) = self.dig_target() else {
                return;
            };
            if !self.target_allows(kind, col, row) {
                return;
            }
            if kind == WorkKind::Dig && self.bombs > 0 {
                self.use_bomb(col, row);
                return;
            }
            let tool = self.tool_for(kind);
            self.work = Some(WorkProgress::new(
                kind,
                tool,
                DEFAULT_SWING_TICKS,
            ));
            self.work_target = Some((col, row));
        }

        let Some((col, row)) = self.work_target else {
            return;
        };
        let Some(mut work) = self.work.take() else {
            return;
        };
        let transitioned = work.tick();
        self.work = Some(work);
        if transitioned && self.apply_transition(kind, col, row) {
            // Keep the held action running into the next state.
        } else if transitioned {
            self.work = None;
            self.work_target = None;
            self.dig_cooldown = self.dig_cooldown_ticks();
        }
    }

    fn tool_for(&self, kind: WorkKind) -> ToolKind {
        match kind {
            WorkKind::Dig => {
                if self.jackhammer {
                    ToolKind::Jackhammer
                } else {
                    ToolKind::Pickaxe
                }
            }
            WorkKind::Fill => {
                if self.power_glove {
                    ToolKind::PowerGlove
                } else {
                    ToolKind::Hammer
                }
            }
        }
    }

    fn target_allows(&self, kind: WorkKind, col: usize, row: usize) -> bool {
        let hole = self.hole_span_at(row, col);
        match kind {
            WorkKind::Dig => {
                let Some(tile) = self.tiles.cells.get(row).and_then(|r| r.get(col)) else {
                    return false;
                };
                if !tile.solid {
                    return false;
                }
                match hole {
                    Some(hole) => {
                        hole.centre == col
                            && hole.state != DigState::FullyDug
                            && !hole.locked
                            && !self.hole_has_monster(col, row)
                    }
                    None => !self
                        .tiles
                        .cells[row]
                        .iter()
                        .enumerate()
                        .any(|(centre, tile)| {
                            tile.dig_state.is_hole() && holes_overlap(centre as i32, col as i32)
                        }),
                }
            }
            WorkKind::Fill => match hole {
                Some(hole) => {
                    hole.centre == col
                        && hole.state != DigState::Normal
                        && !hole.locked
                }
                None => false,
            },
        }
    }

    /// Returns true when work can continue on the same target.
    fn apply_transition(&mut self, kind: WorkKind, col: usize, row: usize) -> bool {
        match kind {
            WorkKind::Dig => {
                let current = self.tiles.cells[row][col].dig_state;
                let Some(next) = current.advance() else {
                    return false;
                };
                self.tiles.cells[row][col].dig_state = next;
                self.events.push(Event::Dug { col, row });
                if current == DigState::Normal && next == DigState::PartlyDug {
                    // **Buried Loot** is released on the first dig only.
                    if let Some(content) = self.buried_loot.remove(&(col, row)) {
                        let seed = (self.tick as u32) ^ ((col as u32) << 8) ^ row as u32;
                        self.spawn_loot_content(&content, col, row, seed);
                    }
                }
                if next == DigState::FullyDug {
                    self.trap_monster_at(col, row);
                }
                next != DigState::FullyDug
            }
            WorkKind::Fill => {
                let current = self.tiles.cells[row][col].dig_state;
                let Some(next) = current.regress() else {
                    return false;
                };
                self.tiles.cells[row][col].dig_state = next;
                self.events.push(Event::Filled { col, row });
                if next == DigState::Normal {
                    self.release_monster_from(col, row);
                }
                next != DigState::Normal
            }
        }
    }

    fn use_bomb(&mut self, col: usize, row: usize) {
        self.bombs -= 1;
        self.tiles.cells[row][col].dig_state = DigState::FullyDug;
        self.events.push(Event::Dug { col, row });
        self.trap_monster_at(col, row);
        self.dig_cooldown = self.dig_cooldown_ticks();
    }

    fn trap_monster_at(&mut self, col: usize, row: usize) {
        for entity in &mut self.entities {
            if entity.alive
                && entity.active
                && entity.kind == EntityKind::Hazard
                && !entity.trapped
                && entity.col == col
                && entity.row == row
            {
                entity.trapped = true;
            }
        }
    }

    fn release_monster_from(&mut self, col: usize, row: usize) {
        for entity in &mut self.entities {
            if entity.trapped && entity.col == col && entity.row == row {
                entity.trapped = false;
                entity.row = (entity.row + 1).min(GRID_H - 1);
            }
        }
    }

    fn push_through(&mut self, col: usize, row: usize) {
        for entity in &mut self.entities {
            if entity.trapped && entity.col == col && entity.row == row {
                entity.trapped = false;
                entity.row = (entity.row + 1).min(GRID_H - 1);
            }
        }
    }

    /// Spawn every item in a rolled `indhold` alternative, scattering them
    /// across nearby platform cells.
    fn spawn_loot_content(&mut self, content: &str, col: usize, row: usize, seed: u32) {
        let world = self.world.as_str().to_string();
        let mut rng = Lcg::new(seed | 1);
        for group in content.chars() {
            for item in loot::spawns_for(group, &world) {
                let points = item
                    .points
                    .unwrap_or_else(|| self.config.points_for(item.token));
                let is_monster = self.config.is_monster(&world, item.token)
                    || item.token == "bi";
                let kind = if is_monster {
                    EntityKind::Hazard
                } else {
                    EntityKind::Collectible
                };
                let pickup = self
                    .config
                    .token(item.token)
                    .map(|token| token.pickup)
                    .unwrap_or(PickupMode::Instant);
                let target = self.scatter_cell(row, col, &mut rng);
                let program = if is_monster {
                    Program::parse("r").ok()
                } else {
                    None
                };
                self.entities.push(Entity {
                    token: item.token.to_string(),
                    kind,
                    col: target,
                    row,
                    points,
                    activate_at_tick: 0,
                    active: true,
                    alive: true,
                    program,
                    program_cooldown: 0,
                    patrol_dir: None,
                    last_dir: None,
                    rng: Lcg::new(seed ^ (group as u32) ^ target as u32),
                    anim_phase: (seed ^ (group as u32) ^ target as u32) % 1009,
                    trapped: false,
                    pickup,
                    pickup_ticks: 0,
                    loot: None,
                });
            }
        }
    }

    /// Random nearby cell for a piece of loot.  Avoids Billy's cell so a crate
    /// monster cannot appear underneath him.
    fn scatter_cell(&self, row: usize, base_col: usize, rng: &mut Lcg) -> usize {
        for _ in 0..8 {
            let dx = (rng.next() % 9) as isize - 4;
            let col = (base_col as isize + dx).clamp(0, GRID_W as isize - 1) as usize;
            if self.tiles.is_solid(col, row)
                && !(self.player.col == col && self.player.row == row)
            {
                return col;
            }
        }
        for dx in 1..=4isize {
            for signed in [-dx, dx] {
                let col = (base_col as isize + signed).clamp(0, GRID_W as isize - 1) as usize;
                if self.tiles.is_solid(col, row)
                    && !(self.player.col == col && self.player.row == row)
                {
                    return col;
                }
            }
        }
        base_col
    }

    fn handle_gravity(&mut self) {
        if let Some(climb) = self.climbing {
            let col = self.player.col;
            if self.tiles.is_ladder(col, climb.from)
                || self.tiles.is_ladder(col, climb.to)
            {
                return;
            }
            // The magic ladder expired under him.
            self.climbing = None;
        }
        let (col, row) = (self.player.col, self.player.row);
        // Dropping onto a **Monster-Filled Hole** pushes the monster down one
        // layer; Billy keeps falling through the hole.
        if let Some((centre, DigState::FullyDug)) = hole_span_in_tiles(&self.tiles, row, col) {
            if self.hole_has_monster(centre, row) {
                self.push_through(centre, row);
            }
        }
        if self.tiles.is_ladder(col, row) || self.tiles.is_solid(col, row) {
            return;
        }
        if self.fall_cooldown > 0 {
            return;
        }
        if row + 1 >= GRID_H {
            self.hurt("fall");
            return;
        }
        self.player.row = row + 1;
        self.fall_cooldown = self.fall_ticks();
        self.events.push(Event::Fell {
            col,
            row: self.player.row,
        });
    }

    fn step_monsters(&mut self) {
        let player = (self.player.col as i32, self.player.row as i32);
        let cooldown = self.config.simulation.monster_move_ticks.max(1);
        let tiles = &self.tiles;
        for entity in &mut self.entities {
            if !entity.active || !entity.alive || entity.kind != EntityKind::Hazard {
                continue;
            }
            if entity.trapped {
                // A **Trapped Monster** is harmless and does not move until a
                // **Push-through**, **Fill**, or **Self-fill** releases it.
                continue;
            }
            // A monster stepping into an empty **Full Hole** falls to its
            // centre and becomes trapped.
            if let Some((centre, DigState::FullyDug)) =
                hole_span_in_tiles(tiles, entity.row, entity.col)
            {
                entity.col = centre;
                entity.trapped = true;
                continue;
            }
            // **Monster Drop**: a monster without support falls one layer.
            if !tiles.is_solid(entity.col, entity.row)
                && !tiles.is_ladder(entity.col, entity.row)
            {
                if entity.row + 1 < GRID_H {
                    entity.row += 1;
                }
                continue;
            }
            if entity.program_cooldown > 0 {
                entity.program_cooldown -= 1;
                continue;
            }
            if entity.program.is_none() {
                continue;
            }
            entity.program_cooldown = cooldown;

            // Keep moving in the committed direction until blocked or a
            // ladder.  Only then consume the next program letter.
            let mut moved = false;
            if let Some((dx, dy)) = entity.patrol_dir {
                if let Some((col, row)) =
                    monster_can_enter(tiles, entity.col, entity.row, dx, dy)
                {
                    entity.col = col;
                    entity.row = row;
                    moved = true;
                } else {
                    entity.last_dir = Some((dx, dy));
                    entity.patrol_dir = None;
                }
            }

            if !moved {
                let steps = entity
                    .program
                    .as_ref()
                    .map(|program| program.len())
                    .unwrap_or(0)
                    .max(1);
                for _ in 0..=steps {
                    let step = match entity.program.as_mut() {
                        Some(program) => program.next(),
                        None => break,
                    };
                    let candidates = monster_candidates(
                        step,
                        &mut entity.rng,
                        (entity.col as i32, entity.row as i32),
                        player,
                    );
                    for (dx, dy) in candidates {
                        if let Some((col, row)) =
                            monster_can_enter(tiles, entity.col, entity.row, dx, dy)
                        {
                            entity.patrol_dir = Some((dx, dy));
                            entity.last_dir = Some((dx, dy));
                            entity.col = col;
                            entity.row = row;
                            moved = true;
                            break;
                        }
                    }
                    if moved {
                        break;
                    }
                }
            }

            // Dead end: reverse the last committed direction.
            if !moved {
                if let Some((dx, dy)) = entity.last_dir {
                    if let Some((col, row)) =
                        monster_can_enter(tiles, entity.col, entity.row, -dx, -dy)
                    {
                        entity.patrol_dir = Some((-dx, -dy));
                        entity.last_dir = Some((-dx, -dy));
                        entity.col = col;
                        entity.row = row;
                    }
                }
            }
        }
    }

    fn handle_entities(&mut self, input: Input) {
        let (col, row) = (self.player.col, self.player.row);
        let pause_ticks = self.pause_pickup_ticks();
        let standing_still = !(input.left || input.right || input.up || input.down)
            && self.work.is_none();
        let mut complete = false;
        let mut hurt_by: Option<String> = None;
        let mut score = 0;
        let mut extra_life = false;
        let mut extra_time = false;
        let mut bombs = 0;
        let mut magic_ladders = 0;
        let mut tennis_balls = 0;
        let mut rainbow = 0;
        let mut racket = false;
        let mut tennis_shoes = false;
        let mut ring = false;
        let mut power_glove = false;
        let mut jackhammer = false;
        let mut arcane_book = false;
        let mut collected = Vec::new();
        let mut loot_bursts: Vec<(String, usize, usize, u32)> = Vec::new();

        for entity in &mut self.entities {
            if !entity.active || !entity.alive {
                continue;
            }
            if entity.col != col || entity.row != row {
                entity.pickup_ticks = 0;
                continue;
            }
            match entity.kind {
                EntityKind::Collectible => {
                    if !pickup_ready(entity, standing_still, pause_ticks) {
                        continue;
                    }
                    entity.alive = false;
                    score += entity.points;
                    match entity.token.as_str() {
                        "xtraliv" => extra_life = true,
                        "xtratid" => extra_time = true,
                        "bombe" => bombs += 5,
                        "tryllestav" => magic_ladders += 3,
                        "tennisbold" => tennis_balls += 3,
                        "regnbue" => rainbow += 1,
                        "ketcher" => racket = true,
                        "lobesko" => tennis_shoes = true,
                        "ring" => ring = true,
                        "krafthandske" => power_glove = true,
                        "luftbor" => jackhammer = true,
                        "tryllebog" => arcane_book = true,
                        "crate" => {
                            if let Some(content) = entity.loot.take() {
                                let seed = entity.rng.next();
                                loot_bursts.push((
                                    content,
                                    entity.col,
                                    entity.row,
                                    seed,
                                ));
                            }
                        }
                        _ => {}
                    }
                    collected.push(Event::Collected {
                        token: entity.token.clone(),
                        points: entity.points,
                    });
                }
                EntityKind::Portal => {
                    if pickup_ready(entity, standing_still, pause_ticks) {
                        complete = true;
                    }
                }
                EntityKind::Hazard => {
                    if !entity.trapped {
                        hurt_by = Some(entity.token.clone());
                    }
                }
                EntityKind::Decoration => {}
            }
        }

        for (content, burst_col, burst_row, seed) in loot_bursts {
            self.spawn_loot_content(&content, burst_col, burst_row, seed);
        }

        self.events.extend(collected);
        self.bombs = self.bombs.saturating_add(bombs);
        self.magic_ladders = self.magic_ladders.saturating_add(magic_ladders);
        self.tennis_balls = self.tennis_balls.saturating_add(tennis_balls);
        self.rainbow = self.rainbow.saturating_add(rainbow);
        self.racket |= racket;
        self.tennis_shoes |= tennis_shoes;
        self.ring |= ring;
        self.power_glove |= power_glove;
        self.jackhammer |= jackhammer;
        if arcane_book {
            self.grant_next_permanent();
        }
        self.score = self.score.saturating_add(score);
        if extra_life {
            self.lives = self.lives.saturating_add(1);
        }
        if extra_time {
            // The alarm clock restores the level's starting time, it does not
            // merely add a few seconds.
            self.time_left_ticks = self.time_total_ticks;
        }
        if complete {
            self.status = Status::LevelComplete;
            self.events.push(Event::LevelComplete);
        } else if let Some(token) = hurt_by {
            self.hurt(&token);
        }
    }

    /// **Arcane Book**: grants the next **Permanent** in the fixed order,
    /// Ring → Tennis Shoes → Power Glove → Jackhammer → Tennis Racket.
    /// Extra books have no effect.
    fn grant_next_permanent(&mut self) {
        if !self.ring_permanent {
            self.ring = true;
            self.ring_permanent = true;
        } else if !self.shoes_permanent {
            self.tennis_shoes = true;
            self.shoes_permanent = true;
        } else if !self.glove_permanent {
            self.power_glove = true;
            self.glove_permanent = true;
        } else if !self.jackhammer_permanent {
            self.jackhammer = true;
            self.jackhammer_permanent = true;
        } else if !self.racket_permanent {
            self.racket = true;
            self.racket_permanent = true;
        }
    }

    fn hurt(&mut self, token: &str) {
        if self.status != Status::Playing {
            return;
        }
        self.lives = self.lives.saturating_sub(1);
        // Consumables are lost on death.  Only **Permanents** granted by an
        // Arcane Book survive; everything else is a Temporary.
        self.bombs = 0;
        self.magic_ladders = 0;
        self.tennis_balls = 0;
        self.rainbow = 0;
        self.climbing = None;
        if !self.racket_permanent {
            self.racket = false;
        }
        if !self.shoes_permanent {
            self.tennis_shoes = false;
        }
        if !self.glove_permanent {
            self.power_glove = false;
        }
        if !self.ring_permanent {
            self.ring = false;
        }
        if !self.jackhammer_permanent {
            self.jackhammer = false;
        }
        self.events.push(Event::Hurt {
            token: token.to_string(),
        });
        if self.lives == 0 {
            self.player.alive = false;
            self.status = Status::GameOver;
            self.events.push(Event::GameOver);
        } else {
            self.player.col = self.start.0;
            self.player.row = self.start.1;
            self.player.alive = true;
            self.move_cooldown = self.move_ticks();
            self.fall_cooldown = self.fall_ticks();
            self.get_ready = true;
        }
    }
}

fn entity_from_object(
    level: &Level,
    object: &GameObject,
    config: &GameConfig,
    run_seed: u32,
) -> Option<Entity> {
    let spec = config.token(&object.token)?;
    let is_monster = config.is_monster(level.world().as_str(), &object.token)
        || spec.kind == crate::token::TokenKind::Monster
        || matches!(spec.behavior, Some(EntityBehavior::Hazard));
    let kind = if is_monster {
        EntityKind::Hazard
    } else {
        match spec.behavior {
            Some(EntityBehavior::Collectible) => EntityKind::Collectible,
            Some(EntityBehavior::Portal) => EntityKind::Portal,
            Some(EntityBehavior::Hazard) => EntityKind::Hazard,
            Some(EntityBehavior::Decoration) => EntityKind::Decoration,
            None => match spec.kind {
                crate::token::TokenKind::Monster => EntityKind::Hazard,
                _ => EntityKind::Collectible,
            },
        }
    };

    let position = match spec.anchor {
        crate::token::AnchorMode::None => None,
        crate::token::AnchorMode::Pos | crate::token::AnchorMode::TwoPos => {
            level.object_anchor(object)
        }
        crate::token::AnchorMode::PosHPosV => object
            .attr("posh")
            .and_then(|v| v.chars().next())
            .and_then(|ch| level.anchor_position(ch)),
    };

    let (col, row) = position?;
    let seconds = object.attr_f64("tid").unwrap_or(0.0).max(0.0);
    let activate_at_tick =
        (seconds * config.simulation.tick_hz.max(1) as f64).round() as u64;
    let program = if is_monster {
        object
            .attr("program")
            .and_then(|text| Program::parse(text).ok())
    } else {
        None
    };
    let seed = object_seed(object, col, row) ^ run_seed.rotate_left(16);
    let loot = object.attr("indhold").map(|text| {
        let mut rng = Lcg::new(seed);
        loot::roll_alternative(text, &mut rng).to_string()
    });

    Some(Entity {
        token: object.token.clone(),
        kind,
        col,
        row,
        points: config.points_for(&object.token),
        activate_at_tick,
        active: activate_at_tick == 0,
        alive: true,
        program,
        program_cooldown: 0,
        patrol_dir: None,
        last_dir: None,
        rng: Lcg::new(seed),
        anim_phase: seed % 1009,
        trapped: false,
        pickup: spec.pickup,
        pickup_ticks: 0,
        loot: loot.filter(|text| !text.is_empty()),
    })
}

/// Stable per-object seed for loot rolls and movement RNG.
fn object_seed(object: &GameObject, col: usize, row: usize) -> u32 {
    object
        .token
        .bytes()
        .fold(0u32, |hash, byte| hash.wrapping_mul(31).wrapping_add(byte as u32))
        ^ ((col as u32) << 8)
        ^ row as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sim(level: Level) -> Sim {
        Sim::with_config(level, crate::config::test_config()).unwrap()
    }

    /// Leave **Get Ready** without moving Billy (the test levels have no
    /// ladder under the start cell).
    fn begin(sim: &mut Sim) {
        sim.step(Input { up: true, ..Input::default() });
    }

    fn level_text() -> String {
        [
            " B",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "                                                  ",
            "                                                  ",
            "x xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "                                                  ",
            "                                                  ",
            "                                                  ",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "                                                  ",
            "                                                  ",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "bane:nestefil=cave2,verden=cave,tid=60;",
            "portal:pos=a,tid=0;",
        ]
        .join("\r\n")
            + "\r\n"
    }

    fn level_with_portal() -> Level {
        let mut level = Level::parse(&level_text()).unwrap();
        level.grid.set_obj(3, 0, b'a');
        level.grid.set_floor(3, 0, b'x');
        level.objects.push(GameObject::with_attrs(
            "portal",
            [("pos", "a"), ("tid", "0"), ("banefil", "next")],
        ));
        level
    }

    #[test]
    fn demo_level_starts_and_is_playable() {
        let sim = sim(Level::demo());
        assert_eq!(sim.player_cell(), (3, 5));
        assert!(sim.entities.iter().any(|e| e.kind == EntityKind::Portal));
        assert!(sim.entities.iter().any(|e| e.kind == EntityKind::Collectible));
    }

    #[test]
    fn starts_at_billy_marker() {
        let sim = sim(level_with_portal());
        assert_eq!(sim.player_cell(), (1, 0));
        assert_eq!(sim.world, WorldId::new("cave"));
        assert!((sim.time_left_seconds() - 60.0).abs() < 0.01);
    }

    fn magic_ladder_level() -> Level {
        let rows = [
            "",
            "",
            "",
            "",
            " B",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "",
            "",
            "",
            "",
            "",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "bane:verden=jungle,tid=60;",
        ];
        Level::parse(&(rows.join("\r\n") + "\r\n")).unwrap()
    }

    #[test]
    fn alt_up_deploys_a_temporary_magic_ladder() {
        let mut sim = sim(magic_ladder_level());
        begin(&mut sim);
        sim.magic_ladders = 1;
        sim.step(Input { alt: true, up: true, ..Input::default() });
        assert_eq!(sim.magic_ladders, 0);
        assert!(sim.tiles.is_ladder(1, 1));
        assert!(sim.tiles.is_ladder(1, 2));
        assert!(sim.tiles.cells[1][1].temp_ladder_ticks > 0);

        sim.step(Input::default());
        sim.step(Input { up: true, ..Input::default() });
        assert_eq!(sim.player.row, 2);
        assert!(sim.is_climbing());
        for _ in 0..sim.climb_ticks_per_cell() {
            sim.step(Input { up: true, ..Input::default() });
        }
        assert_eq!(sim.player.row, 1);

        let lifetime = sim.tiles.cells[1][1].temp_ladder_ticks;
        for _ in 0..(lifetime + 1) {
            sim.step(Input::default());
        }
        assert!(!sim.tiles.is_ladder(1, 1));
    }

    #[test]
    fn alt_down_deploys_below_without_charges_it_does_nothing() {
        let mut sim = sim(magic_ladder_level());
        begin(&mut sim);
        sim.step(Input { alt: true, down: true, ..Input::default() });
        assert!(!sim.tiles.is_ladder(1, 3));

        sim.magic_ladders = 1;
        sim.step(Input { alt: true, down: true, ..Input::default() });
        assert_eq!(sim.magic_ladders, 0);
        assert!(sim.tiles.is_ladder(1, 3));
        assert!(sim.tiles.is_ladder(1, 2));
    }

    #[test]
    fn collecting_tryllestav_grants_three_magic_ladders() {
        let mut level = magic_ladder_level();
        // Billy starts one cell to the right of the staff.
        level.grid.set_obj(1, 2, b'a');
        level.grid.set_obj(2, 2, b'B');
        level.objects.push(GameObject::with_attrs("tryllestav", [("pos", "a")]));
        let mut sim = sim(level);
        begin(&mut sim);
        sim.step(Input { left: true, ..Input::default() });
        assert_eq!(sim.player.col, 1);
        assert_eq!(sim.magic_ladders, 3);
    }

    fn patrol_level() -> Level {
        let rows = [
            "     B       a",
            "     xxxxxxxxxx",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "bane:verden=jungle,tid=60;",
            "banana:pos=a,program=h;",
        ];
        Level::parse(&(rows.join("\r\n") + "\r\n")).unwrap()
    }

    #[test]
    fn a_patrol_continues_until_the_edge_then_turns() {
        let mut sim = sim(patrol_level());
        begin(&mut sim);
        let mut cols = vec![sim
            .entities
            .iter()
            .find(|entity| entity.token == "banana")
            .map(|entity| entity.col)
            .unwrap()];
        for _ in 0..4 {
            for _ in 0..(sim.config.simulation.monster_move_ticks + 1) {
                sim.step(Input::default());
            }
            cols.push(
                sim.entities
                    .iter()
                    .find(|entity| entity.token == "banana")
                    .map(|entity| entity.col)
                    .unwrap(),
            );
        }
        // The platform is cols 5..=14, so the banana goes 14, 13, 12, ...
        assert_eq!(cols[0], 14);
        assert!(cols.windows(2).any(|pair| pair[0] == 14 && pair[1] == 13));
    }

    fn ladder_level() -> Level {
        let rows = [
            "   H",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "   H",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "  BH",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "",
            "",
            "",
            "",
            "",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "bane:verden=jungle,tid=60;",
        ];
        Level::parse(&(rows.join("\r\n") + "\r\n")).unwrap()
    }

    #[test]
    fn ladders_pass_through_platforms_without_teleporting_from_below() {
        let mut sim = sim(ladder_level());
        begin(&mut sim);
        assert_eq!(sim.player_cell(), (2, 2));

        // Not standing on the ladder yet: up must do nothing.
        sim.step(Input { up: true, ..Input::default() });
        assert_eq!(sim.player_cell(), (2, 2));

        // Move onto the ladder cell, then climb through both platforms.
        sim.step(Input { right: true, ..Input::default() });
        assert_eq!(sim.player_cell(), (3, 2));
        for _ in 0..(MOVE_TICKS + 1) {
            sim.step(Input::default());
        }
        sim.step(Input { up: true, ..Input::default() });
        assert_eq!(sim.player_cell(), (3, 2));
        assert!(sim.is_climbing());
        for _ in 0..sim.climb_ticks_per_cell() {
            sim.step(Input { up: true, ..Input::default() });
        }
        assert_eq!(sim.player_cell(), (3, 1));

        sim.step(Input { up: true, ..Input::default() });
        for _ in 0..sim.climb_ticks_per_cell() {
            sim.step(Input { up: true, ..Input::default() });
        }
        assert_eq!(sim.player_cell(), (3, 0));
    }

    #[test]
    fn thrusign_wraps_at_the_level_edges() {
        let mut level = level_with_portal();
        level.objects.push(GameObject::new("thrusign"));
        let mut sim = sim(level);
        begin(&mut sim);

        let left = Input { left: true, ..Input::default() };
        for _ in 0..40 {
            sim.step(left);
            if sim.player.col == GRID_W - 1 {
                break;
            }
        }
        assert_eq!(sim.player.col, GRID_W - 1);

        for _ in 0..(MOVE_TICKS + 1) {
            sim.step(Input::default());
        }
        let right = Input { right: true, ..Input::default() };
        sim.step(right);
        assert_eq!(sim.player.col, 0);
    }

    #[test]
    fn get_ready_freezes_until_the_first_input() {
        let mut sim = sim(level_with_portal());
        let time = sim.time_left_ticks;
        for _ in 0..10 {
            sim.step(Input::default());
        }
        assert!(sim.is_get_ready());
        assert_eq!(sim.tick, 0);
        assert_eq!(sim.time_left_ticks, time);
        begin(&mut sim);
        assert!(!sim.is_get_ready());
        assert_eq!(sim.tick, 1);
    }

    #[test]
    fn arcane_book_grants_the_five_permanents_in_order() {
        let mut sim = sim(level_with_portal());
        begin(&mut sim);

        sim.grant_next_permanent();
        assert!(sim.ring && sim.ring_permanent);
        sim.grant_next_permanent();
        assert!(sim.tennis_shoes && sim.shoes_permanent);
        sim.grant_next_permanent();
        assert!(sim.power_glove && sim.glove_permanent);
        sim.grant_next_permanent();
        assert!(sim.jackhammer && sim.jackhammer_permanent);
        sim.grant_next_permanent();
        assert!(sim.racket && sim.racket_permanent);

        // A sixth book changes nothing.
        sim.grant_next_permanent();
        assert!(
            sim.ring && sim.tennis_shoes && sim.power_glove && sim.jackhammer && sim.racket
        );

        // All five survive death.
        sim.hurt("banana");
        assert!(sim.ring && sim.ring_permanent);
        assert!(sim.tennis_shoes && sim.shoes_permanent);
        assert!(sim.power_glove && sim.glove_permanent);
        assert!(sim.jackhammer && sim.jackhammer_permanent);
        assert!(sim.racket && sim.racket_permanent);
    }

    #[test]
    fn temporaries_are_lost_on_death() {
        let mut sim = sim(level_with_portal());
        begin(&mut sim);
        sim.racket = true;
        sim.ring = true;
        sim.power_glove = true;
        sim.tennis_shoes = true;
        sim.jackhammer = true;
        sim.bombs = 5;

        sim.hurt("banana");

        assert!(!sim.racket);
        assert!(!sim.ring);
        assert!(!sim.power_glove);
        assert!(!sim.tennis_shoes);
        assert!(!sim.jackhammer);
        assert_eq!(sim.bombs, 0);
    }

    #[test]
    fn arcane_book_pickup_starts_the_permanent_chain() {
        let mut level = Level::demo();
        level.objects.push(GameObject::with_attrs(
            "tryllebog",
            [("pos", "z"), ("tid", "0")],
        ));
        level.grid.set_obj(4, 5, b'z');

        let mut sim = sim(level);
        begin(&mut sim);
        for _ in 0..(MOVE_TICKS + 2) {
            sim.step(Input { right: true, ..Input::default() });
        }

        assert!(sim.ring);
        assert!(sim.ring_permanent);
        assert!(!sim.shoes_permanent);
    }

    #[test]
    fn extra_time_refills_to_the_level_start_value() {
        let mut level = Level::parse(&level_text()).unwrap();
        let (start_col, start_row) = level.find_start().unwrap();
        level.grid.set_obj(start_col + 1, start_row, b'a');
        level.grid.set_floor(start_col + 1, start_row, b'x');
        level.objects.push(GameObject::with_attrs(
            "xtratid",
            [("pos", "a"), ("tid", "0")],
        ));

        let mut sim = sim(level);
        begin(&mut sim);
        let total = sim.time_total_ticks;
        for _ in 0..60 {
            sim.step(Input::default());
        }
        assert!(sim.time_left_ticks < total);

        let mut best = sim.time_left_ticks;
        for _ in 0..(MOVE_TICKS + 4) {
            sim.step(Input { right: true, ..Input::default() });
            best = best.max(sim.time_left_ticks);
        }
        assert_eq!(best, total);
    }

    #[test]
    fn walks_right_one_cell_then_waits_for_cooldown() {
        let mut sim = sim(level_with_portal());
        let input = Input { right: true, ..Input::default() };
        sim.step(input);
        assert_eq!(sim.player.col, 2);
        for _ in 0..=MOVE_TICKS {
            sim.step(input);
        }
        assert_eq!(sim.player.col, 3);
    }

    #[test]
    fn falls_when_standing_over_air() {
        let mut level = level_with_portal();
        level.grid.set_floor(1, 0, b' ');
        level.grid.set_floor(1, 1, b'x');
        let mut sim = sim(level);
        begin(&mut sim);
        for _ in 0..(FALL_TICKS + 1) {
            sim.step(Input::default());
        }
        assert_eq!(sim.player_cell(), (1, 1));
    }

    fn dig_ticks(states: u32) -> u32 {
        DEFAULT_SWING_TICKS
            * crate::mechanics::strikes_per_state(ToolKind::Pickaxe)
            * states
            + 2
    }

    #[test]
    fn digging_advances_three_states_two_cells_ahead() {
        let mut sim = sim(level_with_portal());
        let dig = Input { dig: true, ..Input::default() };
        for _ in 0..dig_ticks(1) {
            sim.step(dig);
        }
        assert_eq!(sim.tiles.dig_state(3, 0), DigState::PartlyDug);
        // The centre is two cells ahead; the near edge and Billy's own cell
        // are untouched.
        assert_eq!(sim.tiles.dig_state(2, 0), DigState::Normal);
        assert_eq!(sim.tiles.dig_state(1, 0), DigState::Normal);

        let mut saw_dug = false;
        for _ in 0..dig_ticks(2) {
            saw_dug |= sim
                .step(dig)
                .iter()
                .any(|e| matches!(e, Event::Dug { col: 3, .. }));
        }
        assert_eq!(sim.tiles.dig_state(3, 0), DigState::FullyDug);
        assert!(!sim.tiles.is_solid(3, 0));
        // The full span (edge, centre, edge) is open; the next cell is not.
        assert!(!sim.tiles.is_solid(2, 0));
        assert!(!sim.tiles.is_solid(4, 0));
        assert!(sim.tiles.is_solid(5, 0));
        assert!(saw_dug);
    }

    #[test]
    fn a_partial_hole_blocks_horizontal_movement() {
        let mut sim = sim(level_with_portal());
        let dig = Input { dig: true, ..Input::default() };
        for _ in 0..dig_ticks(1) {
            sim.step(dig);
        }
        sim.step(Input::default());
        let right = Input { right: true, ..Input::default() };
        for _ in 0..(MOVE_TICKS * 2) {
            sim.step(right);
        }
        assert_eq!(sim.player.col, 1);
    }

    #[test]
    fn filling_reverses_the_dig_state() {
        let mut sim = sim(level_with_portal());
        let dig = Input { dig: true, ..Input::default() };
        for _ in 0..dig_ticks(1) {
            sim.step(dig);
        }
        assert_eq!(sim.tiles.dig_state(3, 0), DigState::PartlyDug);

        let fill = Input { fill: true, ..Input::default() };
        // Hammer: two pounds per state.
        let mut saw_filled = false;
        for _ in 0..dig_ticks(1) {
            saw_filled |= sim
                .step(fill)
                .iter()
                .any(|e| matches!(e, Event::Filled { col: 3, .. }));
        }
        assert_eq!(sim.tiles.dig_state(3, 0), DigState::Normal);
        assert!(saw_filled);
    }

    #[test]
    fn a_monster_falls_into_a_full_hole_and_a_fill_drops_it_one_floor() {
        let rows = [
            " B a",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "bane:verden=jungle,tid=60;",
            "banana:pos=a,program=.;",
        ];
        let level = Level::parse(&(rows.join("\r\n") + "\r\n")).unwrap();
        let mut sim = sim(level);
        let dig = Input { dig: true, ..Input::default() };
        for _ in 0..dig_ticks(3) {
            sim.step(dig);
        }
        let banana = sim.entities.iter().find(|e| e.token == "banana").unwrap();
        assert!(banana.trapped);
        assert_eq!((banana.col, banana.row), (3, 0));

        let fill = Input { fill: true, ..Input::default() };
        // Wait out the post-dig cooldown, then six hammer pounds.
        for _ in 0..(dig_ticks(3) + DIG_COOLDOWN_TICKS) {
            sim.step(fill);
        }
        let banana = sim.entities.iter().find(|e| e.token == "banana").unwrap();
        assert!(!banana.trapped);
        assert_eq!(banana.row, 1);
        assert_eq!(sim.tiles.dig_state(3, 0), DigState::Normal);
    }

    #[test]
    fn space_fills_a_full_hole() {
        let mut sim = sim(level_with_portal());
        for _ in 0..dig_ticks(3) {
            sim.step(Input { dig: true, ..Input::default() });
        }
        assert_eq!(sim.tiles.dig_state(3, 0), DigState::FullyDug);
        for _ in 0..DIG_COOLDOWN_TICKS {
            sim.step(Input::default());
        }

        let mut saw_filled = false;
        for _ in 0..dig_ticks(3) {
            saw_filled |= sim
                .step(Input { dig: true, ..Input::default() })
                .iter()
                .any(|e| matches!(e, Event::Filled { col: 3, .. }));
        }
        assert_eq!(sim.tiles.dig_state(3, 0), DigState::Normal);
        assert!(saw_filled);
    }

    fn hulbonus_level(indhold: &str) -> Level {
        let rows = [
            " B a",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "bane:verden=jungle,tid=60;",
            &format!("hulbonus:pos=a,indhold={indhold};"),
        ];
        Level::parse(&(rows.join("\r\n") + "\r\n")).unwrap()
    }

    #[test]
    fn buried_loot_is_released_on_the_first_dig_only() {
        let mut sim = sim(hulbonus_level("g"));
        assert!(sim.entities.iter().all(|entity| entity.token != "lobesko"));
        for _ in 0..dig_ticks(1) {
            sim.step(Input { dig: true, ..Input::default() });
        }
        assert_eq!(sim.tiles.dig_state(3, 0), DigState::PartlyDug);
        assert!(sim.entities.iter().any(|entity| entity.token == "lobesko"));
        assert!(sim.buried_loot.is_empty());

        // Digging deeper must not release it again.
        let before = sim.entities.iter().filter(|e| e.token == "lobesko").count();
        for _ in 0..dig_ticks(2) {
            sim.step(Input { dig: true, ..Input::default() });
        }
        let after = sim.entities.iter().filter(|e| e.token == "lobesko").count();
        assert_eq!(before, after);
    }

    #[test]
    fn a_crate_explodes_its_rolled_loot_into_several_cells() {
        let mut level = level_with_portal();
        level.grid.set_obj(2, 0, b'b');
        level.objects.push(GameObject::with_attrs(
            "crate",
            [("pos", "b"), ("indhold", "zz")],
        ));
        let mut sim = sim(level);
        sim.step(Input { right: true, ..Input::default() });
        assert_eq!(sim.player.col, 2);
        for _ in 0..(sim.pause_pickup_ticks() + 2) {
            sim.step(Input::default());
        }
        let diamonds: Vec<_> = sim
            .entities
            .iter()
            .filter(|entity| entity.token == "diamant")
            .collect();
        assert_eq!(diamonds.len(), 2);
        assert!(diamonds.iter().all(|entity| entity.points == 100));
    }

    #[test]
    fn a_bomb_instantly_completes_the_hole() {
        let mut sim = sim(level_with_portal());
        sim.bombs = 1;
        sim.step(Input { dig: true, ..Input::default() });
        assert_eq!(sim.tiles.dig_state(3, 0), DigState::FullyDug);
        assert_eq!(sim.bombs, 0);
    }

    #[test]
    fn a_pause_pickup_needs_standing_still() {
        let mut level = level_with_portal();
        level.grid.set_obj(2, 0, b'b');
        level.objects.push(GameObject::with_attrs(
            "crate",
            [("pos", "b"), ("indhold", "wzz")],
        ));
        let mut sim = sim(level);
        sim.step(Input { right: true, ..Input::default() });
        assert_eq!(sim.player.col, 2);
        // One tick is not enough, even standing still.
        sim.step(Input::default());
        assert!(sim.entities.iter().any(|e| e.token == "crate" && e.alive));
        for _ in 0..sim.pause_pickup_ticks() {
            sim.step(Input::default());
        }
        assert!(sim.entities.iter().all(|e| !(e.token == "crate" && e.alive)));
    }

    #[test]
    fn portal_completes_the_level_after_a_pause() {
        let mut sim = sim(level_with_portal());
        let right = Input { right: true, ..Input::default() };
        while sim.player.col < 3 {
            sim.step(right);
        }
        for _ in 0..(sim.pause_pickup_ticks() + 2) {
            sim.step(Input::default());
        }
        assert_eq!(sim.status, Status::LevelComplete);
    }

    #[test]
    fn timed_object_appears_after_tid_seconds() {
        let mut level = level_with_portal();
        level.grid.set_obj(5, 0, b'b');
        level.objects.push(GameObject::with_attrs(
            "crate",
            [("pos", "b"), ("tid", "1"), ("indhold", "wzz")],
        ));
        let mut sim = sim(level);
        let crate_entity = sim.entities.iter().find(|e| e.token == "crate").unwrap();
        assert!(!crate_entity.active);
        begin(&mut sim);
        for _ in 0..(TICKS_PER_SECOND + 1) {
            sim.step(Input::default());
        }
        assert!(sim.entities.iter().find(|e| e.token == "crate").unwrap().active);
    }

    #[test]
    fn time_runs_out() {
        let rows = [
            " B",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "bane:verden=jungle,tid=1;",
        ];
        let level = Level::parse(&(rows.join("\r\n") + "\r\n")).unwrap();
        let mut sim = sim(level);
        begin(&mut sim);
        for _ in 0..(TICKS_PER_SECOND + 1) {
            sim.step(Input::default());
        }
        assert_eq!(sim.status, Status::TimeUp);
    }

    #[test]
    fn replay_is_deterministic() {
        let run = || {
            let mut sim = sim(level_with_portal());
            let mut checksum = 0u64;
            for i in 0..300 {
                let input = Input {
                    right: i % 20 < 10,
                    left: i % 20 >= 15,
                    ..Input::default()
                };
                sim.step(input);
                checksum = checksum
                    .wrapping_mul(31)
                    .wrapping_add(sim.player.col as u64)
                    .wrapping_add((sim.player.row as u64) << 8)
                    .wrapping_add(sim.score as u64);
            }
            checksum
        };
        assert_eq!(run(), run());
    }

    fn monster_level(program: &str) -> Level {
        let banana = format!("banana:pos=a,program={program};");
        let rows = [
            " B   a",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "bane:verden=jungle,tid=60;",
            banana.as_str(),
        ];
        Level::parse(&(rows.join("\r\n") + "\r\n")).unwrap()
    }

    #[test]
    fn program_patrol_moves_the_banana() {
        let mut sim = sim(monster_level("h"));
        begin(&mut sim);
        let start = sim
            .entities
            .iter()
            .find(|entity| entity.token == "banana")
            .map(|entity| entity.col)
            .unwrap();
        for _ in 0..(sim.config.simulation.monster_move_ticks + 1) {
            sim.step(Input::default());
        }
        let after = sim
            .entities
            .iter()
            .find(|entity| entity.token == "banana")
            .map(|entity| entity.col)
            .unwrap();
        assert_eq!(after, start + 1);
    }

    #[test]
    fn program_chase_moves_toward_billy() {
        let mut sim = sim(monster_level("m"));
        begin(&mut sim);
        let start = sim
            .entities
            .iter()
            .find(|entity| entity.token == "banana")
            .map(|entity| entity.col)
            .unwrap();
        for _ in 0..(sim.config.simulation.monster_move_ticks + 1) {
            sim.step(Input::default());
        }
        let after = sim
            .entities
            .iter()
            .find(|entity| entity.token == "banana")
            .map(|entity| entity.col)
            .unwrap();
        assert_eq!(after, start - 1);
    }
}
