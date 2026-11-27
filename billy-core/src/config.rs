//! Runtime configuration.
//!
//! Nothing in the shells should hardcode a world, palette, tile path or timing
//! value.  The five original worlds are only the *default* configuration; a
//! project can add a sixth world, change its colours and asset paths, or tune
//! the simulation timings in `open-billy.toml`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::token::{AnchorMode, TokenKind, TOKEN_SPECS};

/// A world is an arbitrary string id.  The file format stores it in
/// `bane:verden=…`; it is not limited to the five shipped worlds.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorldId(String);

impl WorldId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Default for WorldId {
    fn default() -> Self {
        Self::new("jungle")
    }
}

impl std::fmt::Display for WorldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for WorldId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for WorldId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// An 8-bit RGBA colour.  TOML accepts `"#rrggbb"` or `"#rrggbbaa"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba8 {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);

    pub fn parse_hex(text: &str) -> Option<Self> {
        let hex = text.trim().strip_prefix('#')?;
        let (r, g, b, a) = match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                (r, g, b, 255)
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                (r, g, b, 255)
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                (r, g, b, a)
            }
            _ => return None,
        };
        Some(Self::new(r, g, b, a))
    }

    pub fn to_hex(self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }
}

impl Default for Rgba8 {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

impl Serialize for Rgba8 {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Rgba8 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse_hex(&text).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "invalid colour {text:?}; expected #rrggbb or #rrggbbaa"
            ))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GameMeta {
    pub title: String,
    pub subtitle: String,
    pub version: String,
    pub default_world: String,
    pub default_locale: String,
}

impl Default for GameMeta {
    fn default() -> Self {
        Self {
            title: "Open Billy".to_string(),
            subtitle: "unofficial fan remake".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            default_world: "jungle".to_string(),
            default_locale: "en".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PointConfig {
    pub prop: u32,
    pub power_up: u32,
    pub weapon: u32,
    pub feature: u32,
    pub monster: u32,
    #[serde(rename = "crate")]
    pub crate_points: u32,
    pub diamond: u32,
    pub extra_life: u32,
}

impl Default for PointConfig {
    fn default() -> Self {
        Self {
            prop: 10,
            power_up: 25,
            weapon: 15,
            feature: 25,
            monster: 0,
            crate_points: 50,
            diamond: 100,
            extra_life: 25,
        }
    }
}

impl PointConfig {
    pub fn for_kind(&self, kind: TokenKind) -> u32 {
        match kind {
            TokenKind::Prop => self.prop,
            TokenKind::PowerUp => self.power_up,
            TokenKind::Weapon => self.weapon,
            TokenKind::Feature => self.feature,
            TokenKind::Monster => self.monster,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SimulationConfig {
    pub tick_hz: u32,
    pub move_ticks: u32,
    pub fall_ticks: u32,
    /// Ticks per ladder cell while climbing.
    pub climb_ticks: u32,
    pub monster_move_ticks: u32,
    pub dig_cooldown_ticks: u32,
    /// Ticks Billy must stand still on a **Pause Pickup**.
    pub pause_pickup_ticks: u32,
    /// Lifetime of one deployed **Magic Ladder** cell.
    pub magic_ladder_ticks: u32,
    pub extra_time_ticks: i64,
    pub default_time_seconds: f64,
    pub lives: u32,
    pub points: PointConfig,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            tick_hz: 60,
            move_ticks: 8,
            fall_ticks: 4,
            climb_ticks: 48,
            monster_move_ticks: 30,
            dig_cooldown_ticks: 12,
            pause_pickup_ticks: 18,
            magic_ladder_ticks: 180,
            extra_time_ticks: 1800,
            default_time_seconds: 120.0,
            lives: 3,
            points: PointConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldColors {
    #[serde(default)]
    pub sky: Rgba8,
    #[serde(default)]
    pub platform: Rgba8,
    #[serde(default)]
    pub platform_edge: Rgba8,
    #[serde(default)]
    pub ladder: Rgba8,
    #[serde(default)]
    pub hole: Rgba8,
    #[serde(default)]
    pub player: Rgba8,
    #[serde(default)]
    pub player_skin: Rgba8,
    #[serde(default)]
    pub player_hat: Rgba8,
    #[serde(default)]
    pub player_eye: Rgba8,
    #[serde(default)]
    pub hazard: Rgba8,
    #[serde(default)]
    pub collectible: Rgba8,
    #[serde(default)]
    pub portal: Rgba8,
    #[serde(default)]
    pub text: Rgba8,
    #[serde(default)]
    pub shadow: Rgba8,
    #[serde(default)]
    pub grid: Rgba8,
    #[serde(default)]
    pub panel: Rgba8,
}

impl WorldColors {
    pub fn jungle() -> Self {
        Self {
            sky: Rgba8::rgb(10, 26, 18),
            platform: Rgba8::rgb(107, 74, 42),
            platform_edge: Rgba8::rgb(176, 124, 62),
            ladder: Rgba8::rgb(210, 170, 90),
            hole: Rgba8::new(0, 0, 0, 180),
            player: Rgba8::rgb(61, 111, 182),
            player_skin: Rgba8::rgb(232, 195, 158),
            player_hat: Rgba8::rgb(122, 74, 38),
            player_eye: Rgba8::rgb(21, 21, 21),
            hazard: Rgba8::rgb(235, 100, 70),
            collectible: Rgba8::rgb(242, 193, 78),
            portal: Rgba8::rgb(110, 230, 155),
            text: Rgba8::rgb(232, 240, 247),
            shadow: Rgba8::new(0, 0, 0, 90),
            grid: Rgba8::new(255, 255, 255, 9),
            panel: Rgba8::new(5, 9, 13, 190),
        }
    }

    pub fn construction() -> Self {
        Self {
            sky: Rgba8::rgb(20, 24, 32),
            platform: Rgba8::rgb(85, 95, 107),
            platform_edge: Rgba8::rgb(152, 164, 179),
            ladder: Rgba8::rgb(214, 190, 120),
            ..Self::jungle()
        }
    }

    pub fn cave() -> Self {
        Self {
            sky: Rgba8::rgb(16, 15, 24),
            platform: Rgba8::rgb(74, 64, 88),
            platform_edge: Rgba8::rgb(143, 127, 174),
            ladder: Rgba8::rgb(190, 165, 220),
            ..Self::jungle()
        }
    }

    pub fn water() -> Self {
        Self {
            sky: Rgba8::rgb(4, 26, 36),
            platform: Rgba8::rgb(45, 100, 117),
            platform_edge: Rgba8::rgb(101, 199, 216),
            ladder: Rgba8::rgb(120, 220, 230),
            ..Self::jungle()
        }
    }

    pub fn egypt() -> Self {
        Self {
            sky: Rgba8::rgb(30, 20, 8),
            platform: Rgba8::rgb(138, 90, 43),
            platform_edge: Rgba8::rgb(224, 180, 92),
            ladder: Rgba8::rgb(240, 200, 110),
            ..Self::jungle()
        }
    }
}

impl WorldColors {
    /// Fill colours that were omitted from a partial TOML table.  A fully
    /// transparent colour is treated as "not set"; use an explicit alpha for a
    /// genuinely invisible effect.
    pub fn fill_missing(&mut self, fallback: &WorldColors) {
        macro_rules! fill {
            ($($field:ident),* $(,)?) => {
                $(
                    if self.$field.a == 0 {
                        self.$field = fallback.$field;
                    }
                )*
            };
        }
        fill!(
            sky,
            platform,
            platform_edge,
            ladder,
            hole,
            player,
            player_skin,
            player_hat,
            player_eye,
            hazard,
            collectible,
            portal,
            text,
            shadow,
            grid,
            panel,
        );
    }
}

impl Default for WorldColors {
    fn default() -> Self {
        Self::jungle()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldTiles {
    pub platform_odd: Option<String>,
    pub platform_even: Option<String>,
    pub ladder: Option<String>,
    pub background: Option<String>,
    /// Decorative overlay drawn along the top of the playfield.
    pub top_adornment: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldPhysics {
    pub move_ticks: Option<u32>,
    pub fall_ticks: Option<u32>,
    pub climb_ticks: Option<u32>,
    pub dig_cooldown_ticks: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldConfig {
    pub id: String,
    /// Locale code → display name.  `en` is used as the fallback.
    pub labels: BTreeMap<String, String>,
    /// Tokens allowed in this world, including monsters and loot.
    pub tokens: Vec<String>,
    /// Tokens that behave as hostile actors when placed in this world.
    pub monsters: Vec<String>,
    /// `None` inherits the built-in palette for this world id.
    pub colors: Option<WorldColors>,
    pub tiles: WorldTiles,
    pub physics: WorldPhysics,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            id: "jungle".to_string(),
            labels: BTreeMap::new(),
            tokens: Vec::new(),
            monsters: Vec::new(),
            colors: None,
            tiles: WorldTiles::default(),
            physics: WorldPhysics::default(),
        }
    }
}

impl WorldConfig {
    pub fn resolved_colors(&self) -> WorldColors {
        self.colors.clone().unwrap_or_default()
    }

    pub fn label(&self, locale: &str) -> &str {
        self.labels
            .get(locale)
            .or_else(|| self.labels.get("en"))
            .or_else(|| self.labels.values().next())
            .map(String::as_str)
            .unwrap_or(&self.id)
    }

    pub fn allows_token(&self, token: &str) -> bool {
        self.tokens.iter().any(|t| t == token)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityBehavior {
    Collectible,
    Portal,
    Hazard,
    Decoration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickupMode {
    /// Collected immediately by walking over the cell.
    Instant,
    /// Collected only after Billy stands still on the cell briefly.
    Pause,
}

impl Default for PickupMode {
    fn default() -> Self {
        Self::Instant
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TokenConfig {
    pub id: String,
    pub kind: TokenKind,
    pub anchor: AnchorMode,
    /// What the simulation does when Billy enters this object's cell.
    /// `None` means the default for `kind` (monsters are hazards, everything
    /// else is collectible).
    pub behavior: Option<EntityBehavior>,
    pub pickup: PickupMode,
    pub defaults: BTreeMap<String, String>,
    pub points: Option<u32>,
    pub sprite: Option<String>,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            kind: TokenKind::Prop,
            anchor: AnchorMode::None,
            behavior: None,
            pickup: PickupMode::Instant,
            defaults: BTreeMap::new(),
            points: None,
            sprite: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GameConfig {
    pub game: GameMeta,
    pub simulation: SimulationConfig,
    /// Tokens legal in every world (`bane`, shared pickups, …).
    pub global_tokens: Vec<String>,
    pub worlds: Vec<WorldConfig>,
    pub tokens: Vec<TokenConfig>,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            game: GameMeta::default(),
            simulation: SimulationConfig::default(),
            global_tokens: default_global_tokens(),
            worlds: Vec::new(),
            tokens: default_tokens(),
        }
    }
}

impl GameConfig {
    /// Fill generic defaults for fields a world file omitted.  The world list
    /// is authoritative: core never injects a world list.
    pub fn normalize(mut self) -> Self {
        for world in &mut self.worlds {
            if let Some(colors) = &mut world.colors {
                colors.fill_missing(&WorldColors::default());
            } else {
                world.colors = Some(WorldColors::default());
            }
        }

        if self.global_tokens.is_empty() {
            self.global_tokens = default_global_tokens();
        }

        let mut tokens = default_tokens();
        for token in self.tokens.drain(..) {
            if let Some(existing) = tokens.iter_mut().find(|t| t.id == token.id) {
                *existing = token;
            } else {
                tokens.push(token);
            }
        }
        self.tokens = tokens;

        if self.game.default_world.is_empty()
            || self.world(&self.game.default_world).is_none()
        {
            self.game.default_world = self
                .worlds
                .first()
                .map(|w| w.id.clone())
                .unwrap_or_else(|| "jungle".to_string());
        }

        self
    }

    pub fn world(&self, id: &str) -> Option<&WorldConfig> {
        self.worlds.iter().find(|w| w.id == id)
    }

    pub fn world_or_default(&self, id: &str) -> &WorldConfig {
        self.world(id)
            .or_else(|| self.worlds.first())
            .expect("GameConfig::normalize must leave at least one world")
    }

    pub fn token(&self, id: &str) -> Option<&TokenConfig> {
        self.tokens.iter().find(|t| t.id == id)
    }

    pub fn allows_token(&self, world: &str, token: &str) -> bool {
        self.global_tokens.iter().any(|t| t == token)
            || self
                .world(world)
                .map(|w| w.allows_token(token))
                .unwrap_or(false)
    }

    /// True when a placed token is a hostile actor in this world.  The same
    /// token can be harmless loot when it is spawned from a crate.
    pub fn is_monster(&self, world: &str, token: &str) -> bool {
        self.world(world)
            .map(|w| w.monsters.iter().any(|m| m == token))
            .unwrap_or(false)
    }

    pub fn monster_roster(&self, world: &str) -> &[String] {
        self.world(world)
            .map(|w| w.monsters.as_slice())
            .unwrap_or(&[])
    }

    pub fn world_ids(&self) -> impl Iterator<Item = &str> {
        self.worlds.iter().map(|w| w.id.as_str())
    }

    pub fn points_for(&self, token: &str) -> u32 {
        if let Some(config) = self.token(token) {
            if let Some(points) = config.points {
                return points;
            }
            return match token {
                "crate" => self.simulation.points.crate_points,
                "diamant" => self.simulation.points.diamond,
                "xtraliv" => self.simulation.points.extra_life,
                _ => self.simulation.points.for_kind(config.kind),
            };
        }
        self.simulation.points.prop
    }

    pub fn sprite_for(&self, token: &str) -> Option<&str> {
        self.token(token).and_then(|t| t.sprite.as_deref())
    }

    pub fn token_ids(&self) -> impl Iterator<Item = &str> {
        self.tokens.iter().map(|t| t.id.as_str())
    }
}

pub fn default_global_tokens() -> Vec<String> {
    [
        "bane",
        "bombe",
        "crate",
        "diamant",
        "hemmeligivaeg",
        "hulbonus",
        "krafthandske",
        "lobesko",
        "luftbor",
        "portal",
        "regnbue",
        "ring",
        "tryllebog",
        "xtraliv",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

pub fn default_tokens() -> Vec<TokenConfig> {
    TOKEN_SPECS
        .iter()
        .map(|spec| TokenConfig {
            id: spec.token.to_string(),
            kind: spec.kind,
            anchor: spec.anchor,
            behavior: default_behavior(spec.token, spec.kind),
            pickup: default_pickup(spec.token),
            defaults: spec
                .defaults
                .iter()
                .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
                .collect(),
            points: default_score(spec.token, spec.kind),
            sprite: None,
        })
        .collect()
}

/// Per-token scores recovered from the v1.25 binary.  Type-3 monsters and
/// power-ups score through separate tables not yet wired here.
fn default_score(token: &str, kind: TokenKind) -> Option<u32> {
    if kind != TokenKind::Prop {
        return None;
    }
    Some(match token {
        "banana" => 250,
        "log" => 500,
        "ananas" => 350,
        "coconut" => 400,
        "pumpkin" => 300,
        "cap" => 550,
        "beacon" => 250,
        "hammer" => 450,
        "saw" => 350,
        "spraycan" => 350,
        "musling" => 450,
        "chest" => 650,
        "conkylie" => 350,
        "star" => 550,
        "tire" => 800,
        "dynamite" => 400,
        "lamp" => 450,
        "pick" => 500,
        "ghost" => 600,
        "rock" => 700,
        "torch" => 600,
        "sarkofag" => 500,
        "urn" => 450,
        "pyramid" => 950,
        "teabag" => 750,
        _ => return None,
    })
}

fn default_pickup(token: &str) -> PickupMode {
    match token {
        "bombe" | "crate" | "portal" => PickupMode::Pause,
        _ => PickupMode::Instant,
    }
}

fn default_behavior(token: &str, kind: TokenKind) -> Option<EntityBehavior> {
    match token {
        "portal" => Some(EntityBehavior::Portal),
        "hulbonus" | "elevator" | "bundboble" | "thrusign" | "hemmeligivaeg" => {
            Some(EntityBehavior::Decoration)
        }
        _ => match kind {
            TokenKind::Monster => Some(EntityBehavior::Hazard),
            _ => None,
        },
    }
}


/// Test-only configuration with two worlds.  Production code never injects a
/// world list; this helper keeps the core unit tests independent of the
/// project TOML files.
#[cfg(test)]
pub(crate) fn test_config() -> GameConfig {
    let tokens = |list: &[&str]| list.iter().map(|t| (*t).to_string()).collect::<Vec<_>>();
    let jungle = WorldConfig {
        id: "jungle".to_string(),
        labels: [("en".to_string(), "Jungle".to_string())].into(),
        tokens: tokens(&[
            "ananas", "banana", "bi", "bombe", "coconut", "crate", "diamant",
            "gronpotion", "hemmeligivaeg", "hulbonus", "lobesko", "log", "portal",
            "pumpkin", "regnbue", "rodpotion", "tennisbold", "tordensky",
            "tryllebog", "tryllestav", "xtraliv", "xtratid",
        ]),
        monsters: tokens(&["banana", "pumpkin", "ananas", "coconut", "log", "bi"]),
        ..WorldConfig::default()
    };
    let cave = WorldConfig {
        id: "cave".to_string(),
        labels: [("en".to_string(), "Cave".to_string())].into(),
        tokens: tokens(&[
            "crate", "dynamite", "fade", "ghost", "hulbonus", "lamp", "lobesko",
            "luftbor", "pick", "portal", "regnbue", "rock", "thrusign", "xtratid",
        ]),
        monsters: tokens(&["dynamite", "lamp", "pick", "ghost", "rock", "fade"]),
        ..WorldConfig::default()
    };
    GameConfig {
        game: GameMeta {
            default_world: "jungle".to_string(),
            ..GameMeta::default()
        },
        worlds: vec![jungle, cave],
        ..GameConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_have_no_builtin_worlds_but_all_tokens() {
        let config = GameConfig::default();
        assert!(config.worlds.is_empty());
        assert!(config.token("thrusign").is_some());
    }

    #[test]
    fn colours_parse_hex() {
        assert_eq!(Rgba8::parse_hex("#0a1a12"), Some(Rgba8::rgb(10, 26, 18)));
        assert_eq!(Rgba8::parse_hex("#123"), Some(Rgba8::rgb(17, 34, 51)));
        assert_eq!(Rgba8::parse_hex("#11223344"), Some(Rgba8::new(17, 34, 51, 68)));
        assert_eq!(Rgba8::parse_hex("nope"), None);
        assert_eq!(Rgba8::rgb(10, 26, 18).to_hex(), "#0a1a12");
    }

    #[test]
    fn normalize_keeps_explicit_worlds_and_overrides_tokens() {
        let config = GameConfig {
            worlds: vec![WorldConfig {
                id: "moon".to_string(),
                labels: [("en".to_string(), "Moon".to_string())].into(),
                tokens: vec!["portal".to_string()],
                ..WorldConfig::default()
            }],
            tokens: vec![TokenConfig {
                id: "crate".to_string(),
                points: Some(999),
                ..TokenConfig::default()
            }],
            ..GameConfig::default()
        }
        .normalize();
        assert_eq!(config.world("moon").unwrap().label("en"), "Moon");
        assert!(config.world("jungle").is_none());
        assert_eq!(config.game.default_world, "moon");
        assert_eq!(config.points_for("crate"), 999);
        assert!(config.token("thrusign").is_some());
    }

    #[test]
    fn world_labels_fall_back() {
        let mut world = WorldConfig::default();
        world.labels.insert("en".to_string(), "Jungle".to_string());
        world.labels.insert("da".to_string(), "Junglen".to_string());
        assert_eq!(world.label("da"), "Junglen");
        assert_eq!(world.label("xx"), "Jungle");
    }
}
