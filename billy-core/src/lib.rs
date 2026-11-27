//! Pure, deterministic core for Open Billy.
//!
//! No rendering, audio, file dialogs or platform APIs live here.  A shell
//! feeds [`sim::Input`] into [`sim::Sim::step`] once per 60 Hz tick and reads
//! [`sim::Event`] values back out.

pub mod codec;
pub mod config;
pub mod i18n;
pub mod level;
pub mod loot;
pub mod mechanics;
pub mod program;
pub mod sim;
pub mod token;

pub use config::{EntityBehavior, GameConfig, Rgba8, WorldConfig, WorldId};
pub use i18n::Locale;
pub use level::{GameObject, Grid, Level, LevelError, GRID_H, GRID_W};
pub use mechanics::{DigState, Landing, ToolKind, WorkKind, WorkProgress};
pub use program::{chase_step, Lcg, Program, ProgramStep};
pub use sim::{Event, Input, Sim, SimError, Status};

/// Version of the core contract.  Bump when a replay-breaking rule changes.
pub const CORE_VERSION: &str = "0.1.0";

/// The original game's frame is 800×600 with a 50×6 playfield of 16×96 cells.
pub const VIEW_W: u32 = 800;
pub const VIEW_H: u32 = 600;
pub const CELL_W: u32 = 16;
pub const CELL_H: u32 = 96;
