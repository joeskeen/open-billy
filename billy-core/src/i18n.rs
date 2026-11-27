//! Small, dependency-free UI vocabulary for the three supported languages.
//!
//! Code identifiers are English.  The original game's Danish token names and
//! `.K` attribute keys remain unchanged because they are file-format data, not
//! source identifiers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    English,
    Danish,
    Esperanto,
}

impl Locale {
    pub const ALL: [Locale; 3] = [Locale::English, Locale::Danish, Locale::Esperanto];

    pub fn from_code(code: &str) -> Self {
        match code.trim().to_ascii_lowercase().as_str() {
            "da" | "dan" | "danish" | "dansk" => Self::Danish,
            "eo" | "epo" | "esperanto" => Self::Esperanto,
            _ => Self::English,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Danish => "da",
            Self::Esperanto => "eo",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Danish => "Dansk",
            Self::Esperanto => "Esperanto",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Title,
    Play,
    Pause,
    Restart,
    LoadLevel,
    Score,
    Time,
    Lives,
    LevelComplete,
    GameOver,
    TimeUp,
    GetReady,
    Bomb,
    Ladder,
    Controls,
    MoveLeft,
    MoveRight,
    Climb,
    Dig,
    Fill,
    InvalidLevel,
    Language,
    World,
    Level,
}

pub fn text(locale: Locale, key: Key) -> &'static str {
    match (locale, key) {
        (Locale::English, Key::Title) => "The Worlds of Billy",
        (Locale::Danish, Key::Title) => "The Worlds of Billy",
        (Locale::Esperanto, Key::Title) => "La Mondoj de Billy",

        (Locale::English, Key::Play) => "Play",
        (Locale::Danish, Key::Play) => "Spil",
        (Locale::Esperanto, Key::Play) => "Ludu",

        (Locale::English, Key::Pause) => "Pause",
        (Locale::Danish, Key::Pause) => "Pause",
        (Locale::Esperanto, Key::Pause) => "Paŭzo",

        (Locale::English, Key::Restart) => "Restart",
        (Locale::Danish, Key::Restart) => "Start forfra",
        (Locale::Esperanto, Key::Restart) => "Restartigi",

        (Locale::English, Key::LoadLevel) => "Load level",
        (Locale::Danish, Key::LoadLevel) => "Indlæs bane",
        (Locale::Esperanto, Key::LoadLevel) => "Ŝargi nivelon",

        (Locale::English, Key::Score) => "Score",
        (Locale::Danish, Key::Score) => "Point",
        (Locale::Esperanto, Key::Score) => "Poentoj",

        (Locale::English, Key::Time) => "Time",
        (Locale::Danish, Key::Time) => "Tid",
        (Locale::Esperanto, Key::Time) => "Tempo",

        (Locale::English, Key::Lives) => "Lives",
        (Locale::Danish, Key::Lives) => "Liv",
        (Locale::Esperanto, Key::Lives) => "Vivoj",

        (Locale::English, Key::LevelComplete) => "Level complete!",
        (Locale::Danish, Key::LevelComplete) => "Banen er klaret!",
        (Locale::Esperanto, Key::LevelComplete) => "Nivelo finita!",

        (Locale::English, Key::GameOver) => "Game over",
        (Locale::Danish, Key::GameOver) => "Spillet er slut",
        (Locale::Esperanto, Key::GameOver) => "Ludo finiĝis",

        (Locale::English, Key::TimeUp) => "Time up",
        (Locale::Danish, Key::TimeUp) => "Tiden er udløbet",
        (Locale::Esperanto, Key::TimeUp) => "Tempo finiĝis",

        (Locale::English, Key::GetReady) => "Get ready!",
        (Locale::Danish, Key::GetReady) => "Gør dig klar!",
        (Locale::Esperanto, Key::GetReady) => "Pretiĝu!",

        (Locale::English, Key::Bomb) => "Bomb",
        (Locale::Danish, Key::Bomb) => "Bombe",
        (Locale::Esperanto, Key::Bomb) => "Bombo",

        (Locale::English, Key::Ladder) => "Ladder",
        (Locale::Danish, Key::Ladder) => "Stige",
        (Locale::Esperanto, Key::Ladder) => "Ŝtuparo",

        (Locale::English, Key::Controls) => {
            "Arrows move · Space dig/fill · Shift fill partial · Alt/M+↑/↓ magic ladder · R restart"
        }
        (Locale::Danish, Key::Controls) => {
            "Piletaster flytter · Mellemrum graver/fylder · Skift fylder delvist · Alt/M+↑/↓ magisk stige · R starter forfra"
        }
        (Locale::Esperanto, Key::Controls) => {
            "Sagoj movas · Spaco fosas/plenigas · Majusklo plenigas parte · Alt/M+↑/↓ magia ŝtuparo · R restartigas"
        }

        (Locale::English, Key::MoveLeft) => "Left",
        (Locale::Danish, Key::MoveLeft) => "Venstre",
        (Locale::Esperanto, Key::MoveLeft) => "Maldekstren",

        (Locale::English, Key::MoveRight) => "Right",
        (Locale::Danish, Key::MoveRight) => "Højre",
        (Locale::Esperanto, Key::MoveRight) => "Dekstren",

        (Locale::English, Key::Climb) => "Up / down",
        (Locale::Danish, Key::Climb) => "Op / ned",
        (Locale::Esperanto, Key::Climb) => "Supren / malsupren",

        (Locale::English, Key::Dig) => "Dig",
        (Locale::Danish, Key::Dig) => "Grav",
        (Locale::Esperanto, Key::Dig) => "Fosi",

        (Locale::English, Key::Fill) => "Fill",
        (Locale::Danish, Key::Fill) => "Fyld",
        (Locale::Esperanto, Key::Fill) => "Plenigi",

        (Locale::English, Key::InvalidLevel) => "This level could not be loaded.",
        (Locale::Danish, Key::InvalidLevel) => "Denne bane kunne ikke indlæses.",
        (Locale::Esperanto, Key::InvalidLevel) => "Ĉi tiu nivelo ne povis esti ŝargita.",

        (Locale::English, Key::Language) => "Language",
        (Locale::Danish, Key::Language) => "Sprog",
        (Locale::Esperanto, Key::Language) => "Lingvo",

        (Locale::English, Key::World) => "World",
        (Locale::Danish, Key::World) => "Verden",
        (Locale::Esperanto, Key::World) => "Mondo",

        (Locale::English, Key::Level) => "Level",
        (Locale::Danish, Key::Level) => "Bane",
        (Locale::Esperanto, Key::Level) => "Nivelo",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_keys_are_filled_in_all_languages() {
        let keys = [
            Key::Title, Key::Play, Key::Pause, Key::Restart, Key::LoadLevel, Key::Score,
            Key::Time, Key::Lives, Key::LevelComplete, Key::GameOver, Key::TimeUp,
            Key::GetReady, Key::Bomb, Key::Ladder, Key::Controls, Key::MoveLeft,
            Key::MoveRight, Key::Climb, Key::Dig, Key::Fill, Key::InvalidLevel,
            Key::Language, Key::World, Key::Level,
        ];
        for locale in Locale::ALL {
            for key in keys {
                assert!(!text(locale, key).is_empty(), "{locale:?} {key:?}");
            }
        }
    }

    #[test]
    fn language_codes_round_trip() {
        for locale in Locale::ALL {
            assert_eq!(Locale::from_code(locale.code()), locale);
        }
        assert_eq!(Locale::from_code("dansk"), Locale::Danish);
        assert_eq!(Locale::from_code("xx"), Locale::English);
    }

}
