//! Game-accurate token vocabulary, default attributes and level validation.

use crate::config::GameConfig;
use crate::level::{GameObject, Level, GRID_H, GRID_W};

/// `a`–`z` and `0`–`9` are the only legal anchor characters.
pub const ANCHOR_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

pub fn is_anchor_char(ch: u8) -> bool {
    ANCHOR_CHARS.contains(&ch)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenKind {
    Prop,
    PowerUp,
    Weapon,
    Monster,
    Feature,
}

impl TokenKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Prop => "props",
            Self::PowerUp => "power-ups",
            Self::Weapon => "weapons",
            Self::Monster => "monsters",
            Self::Feature => "features",
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::Prop => 0,
            Self::PowerUp => 1,
            Self::Weapon => 2,
            Self::Monster => 3,
            Self::Feature => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnchorMode {
    /// One `pos=` anchor.
    Pos,
    /// Two `pos=` anchors (`tennisbold`).
    TwoPos,
    /// `posh=` and `posv=`.
    PosHPosV,
    /// No grid anchor (monsters that spawn on their own).
    None,
}

#[derive(Debug, Clone, Copy)]
pub struct TokenSpec {
    pub token: &'static str,
    pub kind: TokenKind,
    pub anchor: AnchorMode,
    pub defaults: &'static [(&'static str, &'static str)],
}

const PROP_PROGRAM: &[(&str, &str)] = &[("program", "mr")];
const POWER_TID: &[(&str, &str)] = &[("tid", "10")];

pub static TOKEN_SPECS: &[TokenSpec] = &[
    // ---- type 0: props -----------------------------------------------------
    TokenSpec { token: "ananas", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "banana", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "beacon", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "cap", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "chest", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "coconut", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "conkylie", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "dynamite", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "ghost", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "hammer", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "lamp", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "log", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "musling", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "pick", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "pumpkin", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "pyramid", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "rock", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "sarkofag", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "saw", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "spraycan", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "star", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "teabag", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "tire", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "torch", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },
    TokenSpec { token: "urn", kind: TokenKind::Prop, anchor: AnchorMode::Pos, defaults: PROP_PROGRAM },

    // ---- type 1: power-ups -------------------------------------------------
    TokenSpec { token: "rodpotion", kind: TokenKind::PowerUp, anchor: AnchorMode::Pos, defaults: POWER_TID },
    TokenSpec { token: "gronpotion", kind: TokenKind::PowerUp, anchor: AnchorMode::Pos, defaults: POWER_TID },
    TokenSpec { token: "blaapotion", kind: TokenKind::PowerUp, anchor: AnchorMode::Pos, defaults: POWER_TID },
    TokenSpec { token: "krafthandske", kind: TokenKind::PowerUp, anchor: AnchorMode::Pos, defaults: POWER_TID },
    TokenSpec { token: "ring", kind: TokenKind::PowerUp, anchor: AnchorMode::Pos, defaults: POWER_TID },
    TokenSpec { token: "luftbor", kind: TokenKind::PowerUp, anchor: AnchorMode::Pos, defaults: POWER_TID },
    TokenSpec { token: "lobesko", kind: TokenKind::PowerUp, anchor: AnchorMode::Pos, defaults: POWER_TID },
    TokenSpec { token: "ketcher", kind: TokenKind::PowerUp, anchor: AnchorMode::Pos, defaults: POWER_TID },
    TokenSpec { token: "tordensky", kind: TokenKind::PowerUp, anchor: AnchorMode::Pos, defaults: POWER_TID },
    TokenSpec { token: "xtratid", kind: TokenKind::PowerUp, anchor: AnchorMode::Pos, defaults: POWER_TID },
    TokenSpec { token: "xtraliv", kind: TokenKind::PowerUp, anchor: AnchorMode::Pos, defaults: POWER_TID },
    TokenSpec { token: "tryllebog", kind: TokenKind::PowerUp, anchor: AnchorMode::Pos, defaults: POWER_TID },

    // ---- type 2: weapons / projectiles ------------------------------------
    TokenSpec { token: "tryllestav", kind: TokenKind::Weapon, anchor: AnchorMode::Pos, defaults: &[("tid", "10"), ("antal", "3")] },
    TokenSpec { token: "bombe", kind: TokenKind::Weapon, anchor: AnchorMode::Pos, defaults: &[("tid", "10"), ("antal", "5")] },
    TokenSpec { token: "regnbue", kind: TokenKind::Weapon, anchor: AnchorMode::Pos, defaults: &[("tid", "10"), ("antal", "1")] },
    TokenSpec { token: "tennisbold", kind: TokenKind::Weapon, anchor: AnchorMode::TwoPos, defaults: &[("tid", "1"), ("antal", "3")] },

    // ---- type 3: monsters --------------------------------------------------
    TokenSpec { token: "bi", kind: TokenKind::Monster, anchor: AnchorMode::None, defaults: &[("tid", "0"), ("hast", "1"), ("interval", "16"), ("forkortelse", "1")] },
    TokenSpec { token: "fishes", kind: TokenKind::Monster, anchor: AnchorMode::None, defaults: &[("tid", "0"), ("hast", "1"), ("interval", "16"), ("forkortelse", "1"), ("stime", "10")] },
    TokenSpec { token: "crusher", kind: TokenKind::Monster, anchor: AnchorMode::None, defaults: &[("tid", "0"), ("frekvens", "1"), ("interval", "7"), ("forkortelse", "8")] },
    TokenSpec { token: "firehead", kind: TokenKind::Monster, anchor: AnchorMode::Pos, defaults: &[("tid", "-8"), ("interval", "14"), ("forkortelse", "0")] },
    TokenSpec { token: "fade", kind: TokenKind::Monster, anchor: AnchorMode::None, defaults: &[("tid", "1"), ("interval", "10")] },
    TokenSpec { token: "ulmeild", kind: TokenKind::Monster, anchor: AnchorMode::None, defaults: &[("interval", "4"), ("tid", "10"), ("forkortelse", "0")] },

    // ---- type 4: level features -------------------------------------------
    TokenSpec { token: "bane", kind: TokenKind::Feature, anchor: AnchorMode::None, defaults: &[] },
    TokenSpec { token: "elevator", kind: TokenKind::Feature, anchor: AnchorMode::Pos, defaults: &[("etager", "12")] },
    TokenSpec { token: "crate", kind: TokenKind::Feature, anchor: AnchorMode::Pos, defaults: &[("indhold", "wzz|fuu|rzz"), ("tid", "30")] },
    TokenSpec { token: "portal", kind: TokenKind::Feature, anchor: AnchorMode::Pos, defaults: &[("banefil", "cave1"), ("tid", "8")] },
    TokenSpec { token: "hulbonus", kind: TokenKind::Feature, anchor: AnchorMode::Pos, defaults: &[("indhold", "uvv|ju|www")] },
    TokenSpec { token: "bundboble", kind: TokenKind::Feature, anchor: AnchorMode::Pos, defaults: &[("frekvens", "5")] },
    TokenSpec { token: "hemmeligivaeg", kind: TokenKind::Feature, anchor: AnchorMode::Pos, defaults: &[("banefil", "secret1"), ("tid", "1")] },
    TokenSpec { token: "diamant", kind: TokenKind::Feature, anchor: AnchorMode::Pos, defaults: &[("tid", "4")] },
    TokenSpec { token: "thrusign", kind: TokenKind::Feature, anchor: AnchorMode::PosHPosV, defaults: &[] },
];

pub fn spec(token: &str) -> Option<&'static TokenSpec> {
    TOKEN_SPECS.iter().find(|s| s.token == token)
}

pub fn default_attrs(token: &str) -> Vec<(String, String)> {
    spec(token)
        .map(|s| {
            s.defaults
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect()
        })
        .unwrap_or_default()
}

pub fn is_monster(token: &str) -> bool {
    spec(token)
        .map(|s| s.kind == TokenKind::Monster)
        .unwrap_or(false)
}

/// The attribute keys the game's parser knows about.
pub const ATTR_KEYS: &[&str] = &[
    "pos", "posh", "posv", "program", "tid", "hast", "interval", "forkortelse",
    "indhold", "banefil", "nestefil", "etager", "frekvens", "antal", "stor",
    "stime", "spilgennemfort", "deterministisk", "demo", "sverhed", "verden",
    "type",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    pub severity: Severity,
    pub message: String,
}

impl Warning {
    fn new(severity: Severity, message: impl Into<String>) -> Self {
        Self { severity, message: message.into() }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(Severity::Error, message)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, message)
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(Severity::Info, message)
    }
}

/// Validate the structural rules that make a level loadable and playable.
pub fn validate(level: &Level, config: &GameConfig) -> Vec<Warning> {
    let mut out = Vec::new();
    let world = level.world();

    let banes = level.objects.iter().filter(|o| o.token == "bane").count();
    match banes {
        0 => out.push(Warning::error("missing required bane: descriptor")),
        1 => {}
        n => out.push(Warning::error(format!("{n} bane: descriptors; exactly one is expected"))),
    }

    if let Some(bane) = level.first_object("bane") {
        if bane.attr("verden").is_none() {
            out.push(Warning::warning("bane: has no verden=; the jungle world will be used"));
        }
        if bane.attr("nestefil").is_none() && bane.attr("type").is_none() {
            out.push(Warning::warning("bane: has no nestefil= (no next level)"));
        }
    }

    let starts = level.grid.obj.iter().flatten().filter(|&&ch| ch == b'B').count();
    if starts != 1 {
        out.push(Warning::error(format!(
            "Billy start found {starts} \"B\" markers, expected exactly 1"
        )));
    }

    let bottom_gaps = level.grid.floor[GRID_H - 1].iter().filter(|&&ch| ch != b'x').count();
    if bottom_gaps > 0 {
        out.push(Warning::error(format!(
            "bottom row has {bottom_gaps} air cell(s); it must be solid all the way across"
        )));
    }

    let top_ladders = level.grid.obj[0].iter().filter(|&&ch| ch == b'H').count();
    if top_ladders > 0 {
        out.push(Warning::warning(format!(
            "top row has {top_ladders} ladder cell(s); there is nothing above to climb to"
        )));
    }

    let mut anchors = Vec::new();
    for row in 0..GRID_H {
        for col in 0..GRID_W {
            let ch = level.grid.obj[row][col];
            if is_anchor_char(ch) && !anchors.contains(&ch) {
                anchors.push(ch);
            } else if !matches!(ch, b' ' | b'H' | b'B') && !is_anchor_char(ch) {
                out.push(Warning::error(format!(
                    "illegal object-layer character {:?} at column {col}, row {row}",
                    char::from(ch)
                )));
            }
        }
    }
    if anchors.len() > 36 {
        out.push(Warning::error(format!(
            "{} distinct anchors; the game supports at most 36",
            anchors.len()
        )));
    }

    for object in &level.objects {
        let Some(spec) = config.token(&object.token) else {
            out.push(Warning::warning(format!("unknown token {:?}", object.token)));
            continue;
        };

        if !config.allows_token(world.as_str(), &object.token) && object.token != "bane" {
            out.push(Warning::error(format!(
                "token {:?} is not available in the {} world",
                object.token,
                world.as_str()
            )));
        }

        match spec.anchor {
            AnchorMode::Pos | AnchorMode::TwoPos => {
                if object.attr("pos").is_none() {
                    out.push(Warning::error(format!("{}: missing pos=", object.token)));
                } else {
                    for (_, value) in object.attrs.iter().filter(|(k, _)| k == "pos") {
                        for ch in value.chars() {
                            if !ch.is_ascii() || !is_anchor_char(ch as u8) {
                                out.push(Warning::error(format!(
                                    "{}: pos={ch:?} is not a legal anchor",
                                    object.token
                                )));
                            } else if level.anchor_position(ch).is_none() {
                                out.push(Warning::error(format!(
                                    "{}: pos={ch:?} is not on the grid",
                                    object.token
                                )));
                            }
                        }
                    }
                }
            }
            AnchorMode::PosHPosV => {
                for key in ["posh", "posv"] {
                    if object.attr(key).is_none() {
                        out.push(Warning::error(format!("{}: missing {key}=", object.token)));
                    }
                }
            }
            AnchorMode::None => {}
        }

        for (key, value) in &object.attrs {
            if !ATTR_KEYS.contains(&key.as_str()) {
                out.push(Warning::warning(format!(
                    "{}: unknown attribute {key:?}",
                    object.token
                )));
            }
            if matches!(key.as_str(), "tid" | "hast" | "interval" | "forkortelse" | "frekvens") {
                if value.trim().parse::<f64>().is_err() {
                    out.push(Warning::warning(format!(
                        "{}: {key}={value:?} is not a number",
                        object.token
                    )));
                }
            }
        }
    }

    out
}

pub fn default_object(token: &str) -> Option<GameObject> {
    let spec = spec(token)?;
    let mut object = GameObject::new(spec.token);
    for (key, value) in spec.defaults {
        object.set_attr(*key, *value);
    }
    Some(object)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::Level;

    fn base_level() -> Level {
        let rows = [
            " B",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "                                                  ",
            "                                                  ",
            "                                                  ",
            "                                                  ",
            "                                                  ",
            "                                                  ",
            "                                                  ",
            "                                                  ",
            "                                                  ",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "bane:nestefil=cave2,verden=cave,tid=60;",
            "portal:pos=a,banefil=lagoon1,tid=130;",
        ];
        let mut level = Level::parse(&(rows.join("\r\n") + "\r\n")).unwrap();
        // The portal's anchor has to exist in the level for the test to be valid.
        level.grid.set_obj(3, 1, b'a');
        level
    }

    #[test]
    fn registry_has_the_six_monsters() {
        let monsters: Vec<_> = TOKEN_SPECS
            .iter()
            .filter(|s| s.kind == TokenKind::Monster)
            .map(|s| s.token)
            .collect();
        assert_eq!(monsters.len(), 6);
        for token in ["bi", "fishes", "crusher", "firehead", "fade", "ulmeild"] {
            assert!(monsters.contains(&token));
        }
    }

    #[test]
    fn defaults_match_the_editor() {
        assert_eq!(default_attrs("crate"), vec![
            ("indhold".to_string(), "wzz|fuu|rzz".to_string()),
            ("tid".to_string(), "30".to_string()),
        ]);
        assert_eq!(default_attrs("bi").len(), 4);
    }

    #[test]
    fn catches_bottom_row_and_missing_anchor() {
        let rows = [
            "B",
            "  ",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "bane:verden=jungle;",
            "portal:pos=z;",
        ];
        let level = Level::parse(&(rows.join("\r\n") + "\r\n")).unwrap();
        let warnings = validate(&level, &crate::config::test_config());
        assert!(warnings.iter().any(|w| w.message.contains("bottom row")));
        assert!(warnings.iter().any(|w| w.message.contains("not on the grid")));
    }

    #[test]
    fn accepts_a_valid_level() {
        let warnings = validate(&base_level(), &crate::config::test_config());
        assert!(
            warnings.iter().all(|w| w.severity != Severity::Error),
            "{warnings:#?}"
        );
    }

    #[test]
    fn demo_level_has_no_errors() {
        let warnings = validate(&Level::demo(), &crate::config::test_config());
        assert!(
            warnings.iter().all(|w| w.severity != Severity::Error),
            "{warnings:#?}"
        );
    }

    #[test]
    fn rejects_a_cross_world_token() {
        let rows = [
            " Ba",
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
            "bane:verden=cave;",
            "thrusign:posh=a,posv=a;",
            "firehead:pos=a;",
        ];
        let level = Level::parse(&(rows.join("\r\n") + "\r\n")).unwrap();
        let warnings = validate(&level, &crate::config::test_config());
        assert!(warnings.iter().any(|w| w.message.contains("not available in the cave world")));
    }
}
