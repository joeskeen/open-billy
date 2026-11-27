//! Loot groups from the `.K` `indhold` attribute.
//!
//! `indhold` is a `|`-separated list of alternatives.  One alternative is
//! rolled when the level loads.  Each character in the chosen alternative is
//! a **Loot Group** that spawns one or more items.
//!
//! Group `a`–`e` are world-themed monsters/props; `f`–`z` are direct items.
//! The mapping comes from `udloesbonusgruppe` in the original binary.

use crate::program::Lcg;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootSpawn {
    pub token: &'static str,
    /// `None` means "use the configured score for this token".
    pub points: Option<u32>,
}

impl LootSpawn {
    const fn token(token: &'static str) -> Self {
        Self { token, points: None }
    }

    const fn points(points: u32) -> Self {
        Self {
            token: "pointburst",
            points: Some(points),
        }
    }
}

/// Trim and split an `indhold` value, then pick one alternative
/// deterministically from `rng`.
pub fn roll_alternative<'a>(indhold: &'a str, rng: &mut Lcg) -> &'a str {
    let cleaned = indhold.trim().trim_end_matches(';').trim();
    let options: Vec<&str> = cleaned
        .split('|')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if options.is_empty() {
        return "";
    }
    let index = (rng.next() as usize) % options.len();
    options[index]
}

/// The world-themed group `a`–`e`.
fn themed(world: &str, jungle: &'static str, construction: &'static str, cave: &'static str,
          water: &'static str, egypt: &'static str) -> &'static str {
    match world {
        "construction" => construction,
        "cave" => cave,
        "water" => water,
        "egypt" => egypt,
        _ => jungle,
    }
}

/// Every item spawned by one **Loot Group** character.
///
/// Point bursts spawn multiple items, matching the original:
/// `u` = four 5-point items, `v` = four 12-point items, `w` = a
/// 14/8/12/14 mix.
pub fn spawns_for(group: char, world: &str) -> Vec<LootSpawn> {
    match group {
        'a' => vec![LootSpawn::token(themed(
            world, "banana", "beacon", "dynamite", "conkylie", "urn",
        ))],
        'b' => vec![LootSpawn::token(themed(
            world, "pumpkin", "spraycan", "lamp", "musling", "sarkofag",
        ))],
        'c' => vec![LootSpawn::token(themed(
            world, "ananas", "saw", "pick", "star", "torch",
        ))],
        'd' => vec![LootSpawn::token(themed(
            world, "coconut", "hammer", "ghost", "chest", "teabag",
        ))],
        'e' => vec![LootSpawn::token(themed(
            world, "log", "cap", "rock", "tire", "pyramid",
        ))],
        'f' => vec![LootSpawn::token("ring")],
        'g' => vec![LootSpawn::token("lobesko")],
        'h' => vec![LootSpawn::token("krafthandske")],
        'i' => vec![LootSpawn::token("tennisbold")],
        'j' => vec![LootSpawn::token("bombe")],
        'k' => vec![LootSpawn::token("luftbor")],
        'l' => vec![LootSpawn::token("rodpotion")],
        'm' => vec![LootSpawn::token("blaapotion")],
        'n' => vec![LootSpawn::token("gronpotion")],
        'o' => vec![LootSpawn::token("regnbue")],
        'p' => vec![LootSpawn::token("ketcher")],
        // `tryllestav` is the deployable Magic Ladder, not a thrown weapon.
        'q' => vec![LootSpawn::token("tryllestav")],
        'r' => vec![LootSpawn::token("tordensky")],
        's' => vec![LootSpawn::token("xtraliv")],
        't' => vec![LootSpawn::token("xtratid")],
        'u' => vec![
            LootSpawn::points(5),
            LootSpawn::points(5),
            LootSpawn::points(5),
            LootSpawn::points(5),
        ],
        'v' => vec![
            LootSpawn::points(12),
            LootSpawn::points(12),
            LootSpawn::points(12),
            LootSpawn::points(12),
        ],
        'w' => vec![
            LootSpawn::points(14),
            LootSpawn::points(8),
            LootSpawn::points(12),
            LootSpawn::points(14),
        ],
        'x' => vec![LootSpawn::points(500)],
        'y' => vec![LootSpawn::token("tryllebog")],
        'z' => vec![LootSpawn::token("diamant")],
        // The binary handles '1' as a bee spawn, but no shipped `indhold`
        // uses it.  Kept for custom-level fidelity; never encountered in
        // retail play.
        '1' => vec![LootSpawn::token("bi")],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alternatives_are_exclusive_and_trimmed() {
        let mut rng = Lcg::new(7);
        for _ in 0..50 {
            let chosen = roll_alternative("wzz|fuu|rzz;", &mut rng);
            assert!(["wzz", "fuu", "rzz"].contains(&chosen), "{chosen:?}");
        }
    }

    #[test]
    fn single_alternative_is_returned_whole() {
        let mut rng = Lcg::new(1);
        assert_eq!(roll_alternative("zzzzzzzzzz", &mut rng), "zzzzzzzzzz");
        assert_eq!(roll_alternative("", &mut rng), "");
    }

    #[test]
    fn world_groups_map_to_the_right_monster() {
        assert_eq!(spawns_for('a', "jungle")[0].token, "banana");
        assert_eq!(spawns_for('a', "cave")[0].token, "dynamite");
        assert_eq!(spawns_for('e', "egypt")[0].token, "pyramid");
        assert_eq!(spawns_for('j', "jungle")[0].token, "bombe");
        assert_eq!(spawns_for('z', "jungle")[0].token, "diamant");
    }

    #[test]
    fn point_groups_spawn_multiple_items() {
        assert_eq!(spawns_for('u', "jungle").len(), 4);
        assert_eq!(spawns_for('v', "jungle").len(), 4);
        assert_eq!(spawns_for('w', "jungle").len(), 4);
        assert_eq!(spawns_for('u', "jungle")[0].points, Some(5));
        assert_eq!(spawns_for('v', "jungle")[0].points, Some(12));
    }
}
