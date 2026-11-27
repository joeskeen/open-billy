//! Integration tests against a legal local install.
//!
//! Run with:
//! ```text
//! BILLY_GAME_DIR=/path/to/BILLY cargo test -p billy-assets -- --ignored
//! ```

use std::fs;
use std::path::Path;

use billy_assets::{Atlas, Palette};

#[test]
#[ignore = "requires BILLY_GAME_DIR pointing at a legal game install"]
fn decodes_all_world_atlases_and_the_palette() {
    let game = std::env::var("BILLY_GAME_DIR")
        .expect("set BILLY_GAME_DIR to the directory containing DB/");
    let db = Path::new(&game).join("DB");
    assert!(db.is_dir(), "missing {}", db.display());

    let all8 = fs::read(db.join("ALL8.DB")).expect("read ALL8.DB");
    let palette = Palette::from_all8(&all8).expect("palette");

    for name in ["JUNGLE.DB", "CONSTR.DB", "CAVE.DB", "WATER.DB", "EGYPT.DB"] {
        let path = db.join(name);
        if !path.exists() {
            continue;
        }
        let data = fs::read(&path).expect("read atlas");
        let atlas = Atlas::parse(&data);
        assert!(
            atlas.block_offsets.len() >= 40,
            "{name} decoded only {} block signatures",
            atlas.block_offsets.len()
        );
        assert!(
            atlas.sprites.len() >= 40,
            "{name} decoded only {} sprites",
            atlas.sprites.len()
        );
        let game_sprites = atlas.game_sprites().count();
        assert!(
            game_sprites >= 40,
            "{name} has only {game_sprites} game sprites"
        );

        let first_sprite = atlas.game_sprites().find(|s| !s.is_empty());
        if let Some(sprite) = first_sprite {
            let rgba = sprite.to_rgba(&palette);
            assert_eq!(rgba.len(), sprite.width as usize * sprite.height as usize * 4);
            assert!(rgba.iter().any(|&b| b != 0), "{name} sprite is blank");
        }
    }
}

#[test]
#[ignore = "requires BILLY_GAME_DIR pointing at a legal game install"]
fn jungle_and_construction_share_block_geometry() {
    let game = std::env::var("BILLY_GAME_DIR")
        .expect("set BILLY_GAME_DIR to the directory containing DB/");
    let db = Path::new(&game).join("DB");
    let read = |name: &str| fs::read(db.join(name)).expect("read atlas");

    let jungle = Atlas::parse(&read("JUNGLE.DB"));
    let construction = Atlas::parse(&read("CONSTR.DB"));
    assert_eq!(jungle.block_offsets.len(), construction.block_offsets.len());
    let jungle_sprites: Vec<_> = jungle.game_sprites().collect();
    let construction_sprites: Vec<_> = construction.game_sprites().collect();
    assert_eq!(jungle_sprites.len(), construction_sprites.len());
    for (a, b) in jungle_sprites.iter().zip(construction_sprites.iter()) {
        assert_eq!(a.height, b.height, "sprite row counts differ");
        assert_eq!(a.width, b.width, "sprite widths differ");
    }
}
