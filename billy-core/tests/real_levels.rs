//! Integration tests against a legal local install.
//!
//! Run with:
//! ```text
//! BILLY_GAME_DIR=/path/to/BILLY cargo test -p billy-core -- --ignored
//! ```

use std::fs;
use std::path::Path;

use billy_core::codec;
use billy_core::Level;

#[test]
#[ignore = "requires BILLY_GAME_DIR pointing at a legal game install"]
fn every_shipped_level_survives_codec_and_model_round_trips() {
    let game = std::env::var("BILLY_GAME_DIR")
        .expect("set BILLY_GAME_DIR to the directory containing BANER/");
    let levels_dir = Path::new(&game).join("BANER");
    assert!(levels_dir.is_dir(), "missing {}", levels_dir.display());

    let mut checked = 0usize;
    for entry in fs::read_dir(&levels_dir).expect("read BANER/") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("K") {
            continue;
        }
        let raw = fs::read(&path).expect("read level");
        let plain = codec::decode(&raw);
        assert_eq!(
            codec::encode(&plain),
            raw,
            "cipher round trip failed for {}",
            path.display()
        );

        let level = Level::from_bytes(&raw)
            .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        assert!(
            level.first_object("bane").is_some(),
            "{} has no bane descriptor",
            path.display()
        );

        let rendered = level.render();
        let reparsed = Level::parse(&rendered)
            .unwrap_or_else(|err| panic!("{} reparse: {err}", path.display()));
        assert_eq!(reparsed, level, "model round trip failed for {}", path.display());

        let encoded = level.to_bytes();
        let decoded = Level::from_bytes(&encoded)
            .unwrap_or_else(|err| panic!("{} re-decode: {err}", path.display()));
        assert_eq!(decoded, level, "byte round trip failed for {}", path.display());

        checked += 1;
    }

    assert!(checked > 100, "only found {checked} levels under {}", levels_dir.display());
}

#[test]
#[ignore = "requires BILLY_GAME_DIR pointing at a legal game install"]
fn jungle_easy_1_has_the_known_loot_alternation() {
    let game = std::env::var("BILLY_GAME_DIR")
        .expect("set BILLY_GAME_DIR to the directory containing BANER/");
    let path = Path::new(&game).join("BANER").join("LBANE1.K");
    if !path.exists() {
        return;
    }
    let level = Level::from_bytes(&fs::read(&path).expect("read level")).expect("decode");

    let bane = level.first_object("bane").expect("bane");
    assert_eq!(bane.attr("sverhed"), Some("nem"));
    assert_eq!(bane.attr("nestefil"), Some("ljungl2"));

    let bonus = level.first_object("hulbonus").expect("hulbonus");
    let indhold = bonus.attr("indhold").expect("indhold");
    assert_eq!(indhold, "uvv|ju|www");

    let mut rng = billy_core::Lcg::new(0);
    for _ in 0..20 {
        let chosen = billy_core::loot::roll_alternative(indhold, &mut rng);
        assert!(["uvv", "ju", "www"].contains(&chosen), "{chosen:?}");
    }
    assert_eq!(billy_core::loot::spawns_for('j', "jungle")[0].token, "bombe");
    assert_eq!(billy_core::loot::spawns_for('u', "jungle").len(), 4);

    let crate_obj = level.first_object("crate").expect("crate");
    assert_eq!(crate_obj.attr("indhold"), Some("g"));
}

#[test]
#[ignore = "requires BILLY_GAME_DIR pointing at a legal game install"]
fn known_world_levels_parse_to_expected_world() {
    use billy_core::WorldId;

    let game = std::env::var("BILLY_GAME_DIR")
        .expect("set BILLY_GAME_DIR to the directory containing BANER/");
    let cases = [
        ("BANE1.K", WorldId::new("jungle")),
        ("CONSTR1.K", WorldId::new("construction")),
        ("CAVE1.K", WorldId::new("cave")),
        ("WATER1.K", WorldId::new("water")),
        ("EGYPT1.K", WorldId::new("egypt")),
    ];

    for (name, expected) in cases {
        let path = Path::new(&game).join("BANER").join(name);
        if !path.exists() {
            continue;
        }
        let level = Level::from_bytes(&fs::read(&path).expect("read level")).expect("decode");
        assert_eq!(level.world(), expected, "{name}");
        assert!(level.find_start().is_some(), "{name} has no Billy start");
    }
}
