//! The type-0 `program` mini-script.
//!
//! `program` is a mandatory, cyclic string of direction letters.  It is not a
//! pathfinder: `m` is a greedy per-axis step toward Billy’s tile.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramStep {
    Up,
    Down,
    Left,
    Right,
    Chase,
    Random,
    Noop,
}

impl ProgramStep {
    pub fn from_char(ch: char) -> Option<Self> {
        match ch {
            'o' => Some(Self::Up),
            'n' => Some(Self::Down),
            'v' => Some(Self::Left),
            'h' => Some(Self::Right),
            'm' => Some(Self::Chase),
            'r' => Some(Self::Random),
            '.' => Some(Self::Noop),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgramError {
    Unexpected(char),
    Empty,
}

impl std::fmt::Display for ProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unexpected(ch) => write!(f, "unexpected program letter {ch:?}"),
            Self::Empty => write!(f, "program is empty"),
        }
    }
}

impl std::error::Error for ProgramError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    steps: Vec<ProgramStep>,
    pc: usize,
}

impl Program {
    pub fn parse(text: &str) -> Result<Self, ProgramError> {
        let mut steps = Vec::new();
        for ch in text.chars() {
            if ch.is_whitespace() {
                continue;
            }
            steps.push(ProgramStep::from_char(ch).ok_or(ProgramError::Unexpected(ch))?);
        }
        if steps.is_empty() {
            return Err(ProgramError::Empty);
        }
        Ok(Self { steps, pc: 0 })
    }

    pub fn next(&mut self) -> ProgramStep {
        let step = self.steps[self.pc];
        self.pc = (self.pc + 1) % self.steps.len();
        step
    }

    pub fn steps(&self) -> &[ProgramStep] {
        &self.steps
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

/// A deterministic LCG matching the game's `state*0x41C64E6D+0x3039` form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lcg {
    state: u32,
}

impl Lcg {
    pub fn new(seed: u32) -> Self {
        Self { state: seed | 1 }
    }

    pub fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(0x41C6_4E6D).wrapping_add(0x3039);
        self.state % 0x7fff
    }

    /// Random cardinal direction, deterministic for a given seed.
    pub fn direction(&mut self) -> (i32, i32) {
        match self.next() % 4 {
            0 => (0, -1),
            1 => (0, 1),
            2 => (-1, 0),
            _ => (1, 0),
        }
    }
}

/// Greedy per-axis step toward `target`.  Horizontal wins when both axes
/// differ; this is the simple stance the game resolves from the `m` mask.
pub fn chase_step(from: (i32, i32), target: (i32, i32)) -> (i32, i32) {
    if target.0 < from.0 {
        (-1, 0)
    } else if target.0 > from.0 {
        (1, 0)
    } else if target.1 < from.1 {
        (0, -1)
    } else if target.1 > from.1 {
        (0, 1)
    } else {
        (0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_cycles_patrol() {
        let mut program = Program::parse("hv").unwrap();
        assert_eq!(program.next(), ProgramStep::Right);
        assert_eq!(program.next(), ProgramStep::Left);
        assert_eq!(program.next(), ProgramStep::Right);
    }

    #[test]
    fn parses_chase_and_random() {
        let program = Program::parse("mr").unwrap();
        assert_eq!(program.steps(), &[ProgramStep::Chase, ProgramStep::Random]);
    }

    #[test]
    fn rejects_unknown_letters() {
        assert_eq!(Program::parse("mx").unwrap_err(), ProgramError::Unexpected('x'));
        assert_eq!(Program::parse("").unwrap_err(), ProgramError::Empty);
    }

    #[test]
    fn chase_is_greedy_per_axis() {
        assert_eq!(chase_step((5, 5), (8, 5)), (1, 0));
        assert_eq!(chase_step((5, 5), (2, 5)), (-1, 0));
        assert_eq!(chase_step((5, 5), (5, 2)), (0, -1));
        assert_eq!(chase_step((5, 5), (5, 9)), (0, 1));
        assert_eq!(chase_step((5, 5), (5, 5)), (0, 0));
    }

    #[test]
    fn lcg_is_deterministic() {
        let mut a = Lcg::new(12345);
        let mut b = Lcg::new(12345);
        for _ in 0..10 {
            assert_eq!(a.next(), b.next());
        }
    }
}
