//! The decoded `.K` level model: a 50×6 object/floor grid plus object lines.

use crate::codec;
use crate::config::WorldId;
use crate::token::is_anchor_char;

/// Playfield width in cells.  The DOS game is fixed at 50 columns.
pub const GRID_W: usize = 50;
/// Playfield height in cells.  The DOS game is fixed at 6 rows.
pub const GRID_H: usize = 6;

pub type Attr = (String, String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelError {
    NoObjects,
    BadObjectLine { line: usize, text: String },
    BadAttribute { line: usize, text: String },
    AnchorNotFound(char),
}

impl std::fmt::Display for LevelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoObjects => write!(f, "no object definitions found"),
            Self::BadObjectLine { line, text } => {
                write!(f, "line {line}: bad object definition: {text:?}")
            }
            Self::BadAttribute { line, text } => {
                write!(f, "line {line}: bad attribute: {text:?}")
            }
            Self::AnchorNotFound(ch) => write!(f, "anchor {ch:?} is not on the grid"),
        }
    }
}

impl std::error::Error for LevelError {}

/// The two grid layers, indexed `[row][column]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    /// Object layer: ` ` empty, `H` ladder, `B` Billy start, `a`–`z`/`0`–`9`
    /// object anchors.
    pub obj: [[u8; GRID_W]; GRID_H],
    /// Floor layer: `x` platform, ` ` air.
    pub floor: [[u8; GRID_W]; GRID_H],
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            obj: [[b' '; GRID_W]; GRID_H],
            floor: [[b'x'; GRID_W]; GRID_H],
        }
    }
}

impl Grid {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn obj_char(&self, col: usize, row: usize) -> u8 {
        self.obj[row][col]
    }

    pub fn floor_char(&self, col: usize, row: usize) -> u8 {
        self.floor[row][col]
    }

    pub fn set_obj(&mut self, col: usize, row: usize, ch: u8) {
        if col < GRID_W && row < GRID_H {
            self.obj[row][col] = ch;
        }
    }

    pub fn set_floor(&mut self, col: usize, row: usize, ch: u8) {
        if col < GRID_W && row < GRID_H {
            self.floor[row][col] = ch;
        }
    }

    pub fn find_char(&self, ch: u8) -> Option<(usize, usize)> {
        for row in 0..GRID_H {
            for col in 0..GRID_W {
                if self.obj[row][col] == ch {
                    return Some((col, row));
                }
            }
        }
        None
    }

    /// All anchor characters currently on the object layer.
    pub fn anchors(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for row in &self.obj {
            for &ch in row {
                if is_anchor_char(ch) && !out.contains(&ch) {
                    out.push(ch);
                }
            }
        }
        out
    }
}

/// One `<token>:<attr>=<value>, …;` definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameObject {
    pub token: String,
    pub attrs: Vec<Attr>,
}

impl GameObject {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            attrs: Vec::new(),
        }
    }

    pub fn with_attrs<K, V>(token: impl Into<String>, attrs: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            token: token.into(),
            attrs: attrs
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }

    pub fn attr(&self, key: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn attr_f64(&self, key: &str) -> Option<f64> {
        self.attr(key).and_then(|v| v.trim().parse().ok())
    }

    pub fn attr_i64(&self, key: &str) -> Option<i64> {
        self.attr(key).and_then(|v| v.trim().parse().ok())
    }

    pub fn pos(&self) -> Option<char> {
        self.attr("pos").and_then(|v| v.chars().next())
    }

    pub fn set_attr(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        if let Some((_, v)) = self.attrs.iter_mut().find(|(k, _)| *k == key) {
            *v = value;
        } else {
            self.attrs.push((key, value));
        }
    }
}

/// A decoded level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level {
    pub grid: Grid,
    pub objects: Vec<GameObject>,
}

impl Level {
    /// A playable empty jungle level with the required `bane` descriptor.
    pub fn empty() -> Self {
        Self {
            grid: Grid::new(),
            objects: vec![GameObject::with_attrs(
                "bane",
                [("nestefil", "jungle3"), ("verden", "jungle"), ("tid", "150")],
            )],
        }
    }

    /// A small original demo level shipped with the shells.  It is our own
    /// design, not an extracted game level: walk right from Billy's start,
    /// climb the ladder, collect the crate and extra time, then reach the
    /// portal on the first platform.
    pub fn demo() -> Self {
        let mut grid = Grid::new();
        for row in 0..GRID_H - 1 {
            grid.floor[row].fill(b' ');
        }
        // The bottom row stays solid.  Above it, a handful of broken
        // platforms and two ladders make a readable vertical slice.
        // Ladders pass through platforms: the floor stays solid at the ladder
        // columns, and the `H` object layer punches the climbable cell.
        let platform = |grid: &mut Grid, row: usize, from: usize, to: usize| {
            for col in from..=to {
                grid.floor[row][col] = b'x';
            }
        };
        platform(&mut grid, 0, 0, 8);
        platform(&mut grid, 0, 41, 49);
        for col in 0..GRID_W {
            grid.floor[1][col] = b'x';
        }
        platform(&mut grid, 2, 0, 10);
        platform(&mut grid, 2, 18, 30);
        platform(&mut grid, 2, 38, 49);
        platform(&mut grid, 3, 0, 20);
        platform(&mut grid, 3, 30, 49);
        platform(&mut grid, 4, 0, 7);
        platform(&mut grid, 4, 15, 35);
        platform(&mut grid, 4, 43, 49);

        grid.obj[GRID_H - 1][3] = b'B';
        for col in [5usize, 25] {
            for row in 1..GRID_H {
                grid.obj[row][col] = b'H';
            }
        }
        grid.obj[GRID_H - 1][8] = b'a';
        grid.obj[1][15] = b'b';
        grid.obj[1][35] = b'c';
        grid.obj[3][10] = b'd';
        grid.obj[4][30] = b'e';
        grid.obj[GRID_H - 1][15] = b'f';
        grid.obj[GRID_H - 1][20] = b'g';

        Self {
            grid,
            objects: vec![
                GameObject::with_attrs(
                    "bane",
                    [("nestefil", "demo"), ("verden", "jungle"), ("tid", "120")],
                ),
                GameObject::with_attrs("crate", [("pos", "a"), ("indhold", "wzz"), ("tid", "0")]),
                GameObject::with_attrs("portal", [("pos", "b"), ("banefil", "demo"), ("tid", "0")]),
                GameObject::with_attrs("xtratid", [("pos", "c"), ("tid", "0")]),
                GameObject::with_attrs("diamant", [("pos", "d"), ("tid", "0")]),
                GameObject::with_attrs("xtraliv", [("pos", "e"), ("tid", "0")]),
                GameObject::with_attrs("banana", [("pos", "f"), ("program", "hv")]),
                GameObject::with_attrs("tryllestav", [("pos", "g"), ("tid", "0")]),
            ],
        }
    }

    /// A flat test level for certifying the HUD.  Walking right collects every
    /// inventory power-up in slot order, then enough extra lives to push the
    /// heart counter past ten.  The shell seeds a maximum score on top; the
    /// level itself contains only ordinary game data.
    pub fn demo_hud() -> Self {
        let mut grid = Grid::new();
        for row in 0..GRID_H {
            grid.floor[row].fill(b' ');
            grid.obj[row].fill(b' ');
        }
        for col in 0..GRID_W {
            grid.floor[1][col] = b'x';
            grid.floor[GRID_H - 1][col] = b'x';
        }
        grid.obj[GRID_H - 1][2] = b'B';

        let mut objects = vec![GameObject::with_attrs(
            "bane",
            [("nestefil", "hudlab"), ("verden", "jungle"), ("tid", "600")],
        )];

        let mut place = |grid: &mut Grid,
                         objects: &mut Vec<GameObject>,
                         col: usize,
                         token: &str,
                         extra: &[(&str, &str)]| {
            let index = objects.len() - 1;
            let anchor = if index < 26 {
                (b'a' + index as u8) as char
            } else {
                (b'0' + (index - 26) as u8) as char
            };
            grid.obj[GRID_H - 1][col] = anchor as u8;
            let mut attrs: Vec<(String, String)> = vec![
                ("pos".to_string(), anchor.to_string()),
                ("tid".to_string(), "0".to_string()),
            ];
            for (key, value) in extra {
                attrs.push((key.to_string(), value.to_string()));
            }
            objects.push(GameObject::with_attrs(token, attrs));
        };

        // One of each inventory power-up, in slot order.  The bomb comes before
        // the jackhammer so the dig-tool slot shows both states while walking.
        let powerups = [
            "tryllestav",
            "tennisbold",
            "regnbue",
            "ketcher",
            "krafthandske",
            "lobesko",
            "ring",
            "bombe",
            "luftbor",
        ];
        for (offset, token) in powerups.iter().enumerate() {
            place(&mut grid, &mut objects, 6 + offset, token, &[]);
        }

        // Fourteen extra lives: 3 starting + 14 = 17 hearts, forcing the
        // "more than ten" display.
        for offset in 0..14 {
            place(&mut grid, &mut objects, 15 + offset, "xtraliv", &[]);
        }

        place(&mut grid, &mut objects, 29, "xtratid", &[]);
        place(&mut grid, &mut objects, 30, "portal", &[]);
        place(&mut grid, &mut objects, 31, "crate", &[("indhold", "www")]);
        place(&mut grid, &mut objects, 33, "diamant", &[]);

        Self { grid, objects }
    }

    /// Read and decode a `.K` file.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, LevelError> {
        Self::parse(&latin1_to_string(&codec::decode(bytes)))
    }

    /// Encode this level into a `.K` file.
    pub fn to_bytes(&self) -> Vec<u8> {
        codec::encode(&latin1_from_string(&self.render()))
    }

    pub fn parse(text: &str) -> Result<Self, LevelError> {
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        let lines: Vec<&str> = normalized.split('\n').collect();

        let first_object = lines.iter().position(|line| is_object_line(line));
        let (grid_lines, object_lines): (&[&str], &[&str]) = match first_object {
            Some(index) => (&lines[..index], &lines[index..]),
            None => (&lines[..], &[]),
        };

        let mut grid = Grid::new();
        for row in 0..GRID_H {
            let obj_line = grid_lines.get(2 * row).copied().unwrap_or("");
            let floor_line = grid_lines.get(2 * row + 1).copied().unwrap_or("");
            copy_line(obj_line, &mut grid.obj[row]);
            copy_line(floor_line, &mut grid.floor[row]);
        }

        let mut objects = Vec::new();
        for (offset, raw) in object_lines.iter().enumerate() {
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }
            let line_number = first_object.unwrap_or(0) + offset + 1;
            let Some((token, body)) = line.split_once(':') else {
                return Err(LevelError::BadObjectLine {
                    line: line_number,
                    text: line.to_string(),
                });
            };
            let token = token.trim();
            if token.is_empty() {
                return Err(LevelError::BadObjectLine {
                    line: line_number,
                    text: line.to_string(),
                });
            }
            let mut body = body.trim();
            if let Some(stripped) = body.strip_suffix(';') {
                body = stripped.trim_end();
            }
            let mut attrs = Vec::new();
            for part in body.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                let Some((key, value)) = part.split_once('=') else {
                    return Err(LevelError::BadAttribute {
                        line: line_number,
                        text: part.to_string(),
                    });
                };
                attrs.push((key.trim().to_string(), value.trim().to_string()));
            }
            objects.push(GameObject {
                token: token.to_string(),
                attrs,
            });
        }

        if objects.is_empty() {
            return Err(LevelError::NoObjects);
        }

        Ok(Self { grid, objects })
    }

    pub fn render(&self) -> String {
        let mut lines: Vec<String> = Vec::with_capacity(2 * GRID_H + self.objects.len());
        for row in 0..GRID_H {
            lines.push(bytes_to_line(&self.grid.obj[row]));
            lines.push(bytes_to_line(&self.grid.floor[row]));
        }
        for object in &self.objects {
            let body = object
                .attrs
                .iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join(",");
            lines.push(format!("{}:{};", object.token, body));
        }
        let mut out = lines.join("\r\n");
        out.push_str("\r\n");
        out
    }

    pub fn world(&self) -> WorldId {
        self.objects
            .iter()
            .find(|o| o.token == "bane")
            .and_then(|o| o.attr("verden"))
            .map(WorldId::new)
            .unwrap_or_default()
    }

    /// The `bane:tid=` starting time budget, in seconds.
    pub fn time_budget_seconds(&self) -> f64 {
        self.objects
            .iter()
            .find(|o| o.token == "bane")
            .and_then(|o| o.attr_f64("tid"))
            .unwrap_or(120.0)
    }

    pub fn find_start(&self) -> Option<(usize, usize)> {
        self.grid.find_char(b'B')
    }

    pub fn anchor_position(&self, anchor: char) -> Option<(usize, usize)> {
        if anchor as u32 > 0xff {
            return None;
        }
        self.grid.find_char(anchor as u8)
    }

    pub fn object_anchor(&self, object: &GameObject) -> Option<(usize, usize)> {
        object
            .pos()
            .and_then(|ch| self.anchor_position(ch))
            .or_else(|| {
                object
                    .attr("posh")
                    .and_then(|v| v.chars().next())
                    .and_then(|ch| self.anchor_position(ch))
            })
    }

    pub fn first_object(&self, token: &str) -> Option<&GameObject> {
        self.objects.iter().find(|o| o.token == token)
    }
}

fn is_object_line(line: &str) -> bool {
    let trimmed = line.trim();
    let Some((token, rest)) = trimmed.split_once(':') else {
        return false;
    };
    if token.is_empty() {
        return false;
    }
    token
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !rest.trim().is_empty()
}

fn copy_line(line: &str, out: &mut [u8; GRID_W]) {
    out.fill(b' ');
    for (slot, byte) in out.iter_mut().zip(line.as_bytes()) {
        *slot = *byte;
    }
}

fn bytes_to_line(bytes: &[u8; GRID_W]) -> String {
    let mut text = bytes
        .iter()
        .map(|&b| char::from(b))
        .collect::<String>();
    while text.ends_with(' ') {
        text.pop();
    }
    text
}

fn latin1_to_string(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| char::from(b)).collect()
}

fn latin1_from_string(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| if (c as u32) <= 0xff { c as u8 } else { b'?' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_text() -> String {
        [
            "  B",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "                                                  ",
            "                                                  ",
            "                                                  ",
            "                                                  ",
            "                                                  ",
            "                                                  ",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "  H                                               ",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "bane:nestefil=jungle2,verden=jungle,tid=120;",
            "crate:pos=a,indhold=wzz|fuu;",
        ]
        .join("\r\n")
            + "\r\n"
    }

    #[test]
    fn parses_grid_and_objects() {
        let level = Level::parse(&sample_text()).unwrap();
        assert_eq!(level.find_start(), Some((2, 0)));
        assert_eq!(level.grid.find_char(b'H'), Some((2, 5)));
        assert_eq!(level.objects.len(), 2);
        assert_eq!(level.world(), WorldId::new("jungle"));
        assert_eq!(level.time_budget_seconds(), 120.0);
        assert_eq!(level.objects[1].attr("indhold"), Some("wzz|fuu"));
    }

    #[test]
    fn render_round_trips_through_parse() {
        let level = Level::parse(&sample_text()).unwrap();
        let text = level.render();
        let again = Level::parse(&text).unwrap();
        assert_eq!(level, again);
        assert!(text.ends_with("\r\n"));
    }

    #[test]
    fn bytes_round_trip_through_cipher() {
        let level = Level::parse(&sample_text()).unwrap();
        let bytes = level.to_bytes();
        let again = Level::from_bytes(&bytes).unwrap();
        assert_eq!(level, again);
    }

    #[test]
    fn missing_objects_is_an_error() {
        assert_eq!(Level::parse("x\n"), Err(LevelError::NoObjects));
    }

    #[test]
    fn short_grid_lines_are_padded() {
        let level = Level::parse("B\nx\nbane:tid=1;\n").unwrap();
        assert_eq!(level.grid.obj[0][0], b'B');
        assert_eq!(level.grid.floor[0][0], b'x');
        assert_eq!(level.grid.obj[0][49], b' ');
        assert_eq!(level.grid.floor[0][49], b' ');
    }
}
