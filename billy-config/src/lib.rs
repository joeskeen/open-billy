//! Load `GameConfig` from the main project file plus per-world TOML files.
//!
//! The main `open-billy.toml` contains the game/simulation sections and an
//! ordered `worlds = ["worlds/jungle.toml", …]` list. Each world file owns its
//! monster roster, labels, colours, tiles, physics and token allow-list.

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use serde::Deserialize;

pub use billy_core::config::*;

/// The default main configuration, embedded so a native build works without a
/// loose file next to the binary.
pub const DEFAULT_TOML: &str = include_str!("../../open-billy.toml");

/// Embedded copies of the default world files, keyed by the same relative paths
/// used in `open-billy.toml`.
pub const DEFAULT_WORLDS: &[(&str, &str)] = &[
    ("worlds/jungle.toml", include_str!("../../worlds/jungle.toml")),
    (
        "worlds/construction.toml",
        include_str!("../../worlds/construction.toml"),
    ),
    ("worlds/cave.toml", include_str!("../../worlds/cave.toml")),
    ("worlds/water.toml", include_str!("../../worlds/water.toml")),
    ("worlds/egypt.toml", include_str!("../../worlds/egypt.toml")),
];

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Toml(toml::de::Error),
    Json(serde_json::Error),
    UnsupportedExtension(String),
    MissingWorld(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "config I/O error: {err}"),
            Self::Toml(err) => write!(f, "config TOML error: {err}"),
            Self::Json(err) => write!(f, "config JSON error: {err}"),
            Self::UnsupportedExtension(ext) => {
                write!(f, "unsupported config extension {ext:?}; use .toml or .json")
            }
            Self::MissingWorld(path) => write!(f, "world file not found: {path}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        Self::Toml(value)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum WorldsSpec {
    /// `worlds = ["worlds/jungle.toml", …]`
    Files(Vec<String>),
    /// Legacy/self-contained `[[worlds]]` tables.
    Inline(Vec<WorldConfig>),
}

impl Default for WorldsSpec {
    fn default() -> Self {
        Self::Inline(Vec::new())
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ProjectFile {
    game: GameMeta,
    simulation: SimulationConfig,
    global_tokens: Vec<String>,
    worlds: WorldsSpec,
    tokens: Vec<TokenConfig>,
}

/// Parse a main project file, resolving world files through `resolve`.
pub fn from_toml_with_worlds(
    main: &str,
    worlds: &[(&str, &str)],
) -> Result<GameConfig, ConfigError> {
    let map: BTreeMap<&str, &str> = worlds.iter().copied().collect();
    load_project(main, |path| {
        map.get(path)
            .map(|text| (*text).to_string())
            .ok_or_else(|| ConfigError::MissingWorld(path.to_string()))
    })
}

/// Parse a self-contained project file (inline `[[worlds]]` only).
pub fn from_toml(text: &str) -> Result<GameConfig, ConfigError> {
    load_project(text, |path| Err(ConfigError::MissingWorld(path.to_string())))
}

fn load_project(
    main: &str,
    resolve: impl Fn(&str) -> Result<String, ConfigError>,
) -> Result<GameConfig, ConfigError> {
    let project: ProjectFile = toml::from_str(main)?;
    let worlds = match project.worlds {
        WorldsSpec::Files(paths) => paths
            .iter()
            .map(|path| {
                let text = resolve(path)?;
                toml::from_str::<WorldConfig>(&text).map_err(ConfigError::from)
            })
            .collect::<Result<Vec<_>, _>>()?,
        WorldsSpec::Inline(worlds) => worlds,
    };

    Ok(GameConfig {
        game: project.game,
        simulation: project.simulation,
        global_tokens: project.global_tokens,
        worlds,
        tokens: project.tokens,
    }
    .normalize())
}

/// Load a main TOML file and its world files from disk.
pub fn load(path: impl AsRef<Path>) -> Result<GameConfig, ConfigError> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path)?;
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("toml") => {
            let base = path.parent().unwrap_or_else(|| Path::new("."));
            load_project(&text, |relative| {
                let resolved = base.join(relative);
                std::fs::read_to_string(&resolved).map_err(ConfigError::Io)
            })
        }
        Some("json") => from_json(&text),
        Some(other) => Err(ConfigError::UnsupportedExtension(other.to_string())),
        None => load_project(&text, |relative| {
            let base = path.parent().unwrap_or_else(|| Path::new("."));
            std::fs::read_to_string(base.join(relative)).map_err(ConfigError::Io)
        }),
    }
}

/// Parse a self-contained JSON `GameConfig` (used by the web bundle).
pub fn from_json(text: &str) -> Result<GameConfig, ConfigError> {
    let config: GameConfig = serde_json::from_str(text)?;
    Ok(config.normalize())
}

/// Parse the embedded default main + world files.
pub fn default_config() -> GameConfig {
    from_toml_with_worlds(DEFAULT_TOML, DEFAULT_WORLDS)
        .unwrap_or_default()
        .normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_project_parses_with_per_world_files() {
        let config = default_config();
        assert_eq!(config.worlds.len(), 5);
        assert_eq!(config.game.title, "Open Billy");
        assert_eq!(config.world("cave").unwrap().label("da"), "Hulen");
        assert!(config.is_monster("jungle", "banana"));
        assert!(config.is_monster("construction", "spraycan"));
        assert!(!config.is_monster("jungle", "crate"));
        assert!(config.allows_token("cave", "thrusign"));
    }

    #[test]
    fn explicit_world_files_are_loaded_in_order() {
        let main = r#"
            global_tokens = ["bane", "portal"]
            worlds = ["a.toml", "b.toml"]
            [game]
            title = "Test"
            default_world = "a"
        "#;
        let a = r#"
            id = "a"
            labels = { en = "A" }
            tokens = ["banana", "portal"]
            monsters = ["banana"]
        "#;
        let b = r#"
            id = "b"
            labels = { en = "B" }
            tokens = ["crusher", "portal"]
            monsters = ["crusher"]
        "#;
        let config = from_toml_with_worlds(main, &[("a.toml", a), ("b.toml", b)]).unwrap();
        assert_eq!(config.world_ids().collect::<Vec<_>>(), vec!["a", "b"]);
        assert!(config.is_monster("a", "banana"));
        assert!(config.is_monster("b", "crusher"));
    }

    #[test]
    fn a_sixth_world_is_fully_configurable() {
        use billy_core::{Level, Sim, WorldId};

        let main = r#"
            global_tokens = ["bane", "portal"]
            worlds = ["moon.toml"]
            [game]
            title = "Moon Test"
            default_world = "moon"
            default_locale = "en"
        "#;
        let moon = r##"
            id = "moon"
            labels = { en = "Moon", da = "Månen" }
            tokens = ["portal", "banana"]
            monsters = ["banana"]
            [colors]
            sky = "#123456"
            platform = "#654321"
            platform_edge = "#abcdef"
            ladder = "#fedcba"
            hole = "#000000b4"
            player = "#3d6fb6"
            hazard = "#eb6446"
            collectible = "#f2c14e"
            portal = "#6ee39b"
            text = "#ffffff"
        "##;
        let config = from_toml_with_worlds(main, &[("moon.toml", moon)]).unwrap();
        assert_eq!(config.worlds.len(), 1);
        assert_eq!(config.game.default_world, "moon");
        assert!(config.is_monster("moon", "banana"));

        let level_text = [
            " B",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "    a",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "bane:verden=moon,tid=60;",
            "portal:pos=a,tid=0;",
        ]
        .join("\r\n")
            + "\r\n";
        let level = Level::parse(&level_text).unwrap();
        let sim = Sim::with_config(level, config).unwrap();
        assert_eq!(sim.world, WorldId::new("moon"));
        assert_eq!(sim.world_config.label("da"), "Månen");
        assert_eq!(sim.world_colors.sky.to_hex(), "#123456");
    }

    #[test]
    fn inline_worlds_still_work() {
        let config = from_toml(
            r#"
            [[worlds]]
            id = "moon"
            labels = { en = "Moon" }
            tokens = ["crate", "portal"]
            monsters = ["crate"]
        "#,
        )
        .unwrap();
        assert_eq!(config.world("moon").unwrap().label("en"), "Moon");
        assert!(config.is_monster("moon", "crate"));
    }
}
