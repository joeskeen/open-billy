//! Pure mechanics primitives for digging, filling, hole spans and fall
//! landing.  Kept independent of `Sim` so the rules can be tested in
//! milliseconds and reused by the shells/tools.

/// A floor tile has four dig states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum DigState {
    Normal = 0,
    PartlyDug = 1,
    MostlyDug = 2,
    FullyDug = 3,
}

impl DigState {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Normal,
            1 => Self::PartlyDug,
            2 => Self::MostlyDug,
            _ => Self::FullyDug,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn is_hole(self) -> bool {
        self != Self::Normal
    }

    pub fn is_full_hole(self) -> bool {
        self == Self::FullyDug
    }

    pub fn advance(self) -> Option<Self> {
        match self {
            Self::Normal => Some(Self::PartlyDug),
            Self::PartlyDug => Some(Self::MostlyDug),
            Self::MostlyDug => Some(Self::FullyDug),
            Self::FullyDug => None,
        }
    }

    pub fn regress(self) -> Option<Self> {
        match self {
            Self::Normal => None,
            Self::PartlyDug => Some(Self::Normal),
            Self::MostlyDug => Some(Self::PartlyDug),
            Self::FullyDug => Some(Self::MostlyDug),
        }
    }
}

/// A hole occupies the dug centre cell and one edge on each side.
pub const HOLE_EDGE_WIDTH: i32 = 1;

/// True when `col` is the centre or an edge of a hole centred at `center`.
pub fn in_hole_span(center: i32, col: i32) -> bool {
    (col - center).abs() <= HOLE_EDGE_WIDTH
}

/// The minimum distance between two non-overlapping hole centres.
pub const HOLE_SPACING: i32 = 4;

/// True when a new hole centred at `candidate` would overlap `existing`.
pub fn holes_overlap(existing: i32, candidate: i32) -> bool {
    (candidate - existing).abs() < HOLE_SPACING
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkKind {
    Dig,
    Fill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    Pickaxe,
    Jackhammer,
    Hammer,
    PowerGlove,
    Bomb,
}

/// Number of strikes needed to advance or reverse one Dig State.
pub fn strikes_per_state(tool: ToolKind) -> u32 {
    match tool {
        ToolKind::Pickaxe | ToolKind::Hammer => 2,
        ToolKind::Jackhammer | ToolKind::PowerGlove => 1,
        ToolKind::Bomb => 1,
    }
}

/// Ticks between strikes.  The live game advances a two-swing state in about
/// 0.97 s, so one swing is roughly half a second at 60 Hz.
pub const DEFAULT_SWING_TICKS: u32 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkProgress {
    pub kind: WorkKind,
    pub strikes_done: u32,
    pub strikes_required: u32,
    pub ticks_into_strike: u32,
    pub swing_ticks: u32,
}

impl WorkProgress {
    pub fn new(kind: WorkKind, tool: ToolKind, swing_ticks: u32) -> Self {
        Self {
            kind,
            strikes_done: 0,
            strikes_required: strikes_per_state(tool),
            ticks_into_strike: 0,
            swing_ticks: swing_ticks.max(1),
        }
    }

    /// Advance one tick.  Returns true when a full state transition is due.
    pub fn tick(&mut self) -> bool {
        self.ticks_into_strike += 1;
        if self.ticks_into_strike < self.swing_ticks {
            return false;
        }
        self.ticks_into_strike = 0;
        self.strikes_done += 1;
        if self.strikes_done < self.strikes_required {
            return false;
        }
        self.strikes_done = 0;
        true
    }

    /// Release the key: the current in-progress strike is lost, but completed
    /// Dig States persist on the tile.
    pub fn release(&mut self) {
        self.strikes_done = 0;
        self.ticks_into_strike = 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landing {
    Land,
    FallThrough,
}

/// Decide whether a falling Billy lands at this row.  `hole_centres` are the
/// centres of any holes on the row; if the trajectory column is inside any
/// hole span (centre or edge), he falls through.
pub fn landing_check(hole_centres: &[i32], trajectory_col: i32) -> Landing {
    if hole_centres
        .iter()
        .any(|center| in_hole_span(*center, trajectory_col))
    {
        Landing::FallThrough
    } else {
        Landing::Land
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dig_states_round_trip() {
        for state in [
            DigState::Normal,
            DigState::PartlyDug,
            DigState::MostlyDug,
            DigState::FullyDug,
        ] {
            assert_eq!(DigState::from_u8(state.as_u8()), state);
        }
    }

    #[test]
    fn pickaxe_needs_six_swings_for_a_full_hole() {
        let mut work = WorkProgress::new(WorkKind::Dig, ToolKind::Pickaxe, 1);
        let mut transitions = 0;
        for _ in 0..6 {
            if work.tick() {
                transitions += 1;
            }
        }
        assert_eq!(transitions, 3);
    }

    #[test]
    fn power_glove_needs_three_pounds_for_a_full_fill() {
        let mut work = WorkProgress::new(WorkKind::Fill, ToolKind::PowerGlove, 1);
        let mut transitions = 0;
        for _ in 0..3 {
            if work.tick() {
                transitions += 1;
            }
        }
        assert_eq!(transitions, 3);
    }

    #[test]
    fn releasing_loses_the_current_strike_but_keeps_state() {
        let mut work = WorkProgress::new(WorkKind::Dig, ToolKind::Pickaxe, 1);
        work.tick();
        work.tick();
        work.tick();
        assert_eq!(work.strikes_done, 1); // state 1 is complete; state 2 has one strike
        work.release();
        assert_eq!(work.strikes_done, 0);
        let mut work = WorkProgress::new(WorkKind::Dig, ToolKind::Pickaxe, 1);
        work.tick();
        work.release();
        assert_eq!(work.strikes_done, 0);
    }

    #[test]
    fn hole_span_is_three_cells() {
        assert!(in_hole_span(10, 9));
        assert!(in_hole_span(10, 10));
        assert!(in_hole_span(10, 11));
        assert!(!in_hole_span(10, 8));
        assert!(!in_hole_span(10, 12));
    }

    #[test]
    fn hole_centres_are_four_cells_apart() {
        assert!(holes_overlap(10, 13));
        assert!(!holes_overlap(10, 14));
        assert!(!holes_overlap(10, 6));
    }

    #[test]
    fn landing_scenarios_1_to_3_fall_through() {
        // centre, left edge, right edge
        for trajectory in [10, 9, 11] {
            assert_eq!(landing_check(&[10], trajectory), Landing::FallThrough);
        }
    }

    #[test]
    fn landing_scenarios_4_to_6_land_on_solid() {
        // A solid cell outside the span, and the one-cell island at 7.
        assert_eq!(landing_check(&[10], 7), Landing::Land);
        assert_eq!(landing_check(&[10, 14], 12), Landing::Land);
    }

    #[test]
    fn scenario_7_falls_through_an_edge() {
        assert_eq!(landing_check(&[10], 9), Landing::FallThrough);
        assert_eq!(landing_check(&[10], 11), Landing::FallThrough);
    }
}
