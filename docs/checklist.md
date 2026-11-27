# Open Billy feature checklist

This is the certification list. **A box is ticked only after the project owner
has confirmed the item is 100 % correct in the running game.** "Implemented" is
not the same as "certified".

Status notes used on unchecked items:

- `implemented, awaiting certification` — code exists, owner has not signed off.
- `partial` — some of the behaviour exists, but a known gap remains.
- `not implemented` — nothing yet.

The implementation-level status lives in `README.md`; this file tracks what the
owner has actually accepted.

## HUD and presentation

- [x] HUD sits at the top of the screen — certified 2026-11-27
- [ ] World name is shown under the game screen — current placement is a debug convenience only; needs a spec grilling before certification
- [x] Time is a progress bar, never a numeric countdown — certified 2026-11-27
- [x] Score uses the original colourful digit art — certified 2026-11-27
- [x] Score keeps a fixed right edge, about one heart slot before the ten-heart group — certified 2026-11-26
- [x] Lives use animated hearts, 10 or fewer; above 10 show one heart plus "xN" (e.g. ❤x17), up to 3 digits — certified 2026-11-27
- [x] Inventory uses native item icons in fixed slots — certified 2026-11-27
- [x] Inventory counts are outlined and readable — certified 2026-11-27
- [x] Inventory order is ladder, ball, rainbow, racket, glove, shoe, ring, dig tool — certified 2026-11-27
- [x] Dig-tool slot shows Bomb → Jackhammer → Pickaxe correctly — certified 2026-11-27
- [x] **P** badge appears only on items made Permanent by the Arcane Book — certified 2026-11-27
- [x] Top adornments appear on Jungle and Construction only — certified 2026-11-27
- [x] Cave, Lagoon and Egypt have no top adornment — certified 2026-11-27
- [x] Bottom platform is only about half visible — certified 2026-11-27
- [x] All images use the same 1:1 scale relative to the background — certified 2026-11-27
- [x] Original ALL16 bitmap font used for HUD, status card and world labels — certified 2026-11-27

## Billy movement and animation

- [ ] Horizontal walking speed is consistent in px/tick — implemented, awaiting certification
- [ ] Horizontal walking animates smoothly (no cell stepping) — implemented, awaiting certification
- [ ] Walking and climbing use the same px/tick rate — implemented, awaiting certification
- [ ] Walking direction mirrors the sprite correctly — implemented, awaiting certification
- [ ] Billy stops mid-climb without snapping to the next row — implemented, awaiting certification
- [ ] Billy resumes a stopped climb when the key is held again — implemented, awaiting certification
- [ ] Billy reverses direction mid-climb instead of teleporting — implemented, awaiting certification
- [ ] Climbing through a platform cell works without jumping — implemented, awaiting certification
- [ ] Tennis Shoes speed up climbing by the correct factor — partial (currently 2/3; exact factor needs RE)
- [ ] Climb animation frames are correct and paced correctly — implemented, awaiting certification
- [ ] Dig animation plays while digging — implemented, awaiting certification
- [ ] Drill (jackhammer) animation plays while drilling — implemented, awaiting certification
- [ ] Hammer animation plays while filling — implemented, awaiting certification
- [ ] Giggle animation plays on level completion — implemented, awaiting certification
- [ ] Ouch animation triggers where the original does — not implemented
- [ ] Throw animation triggers where the original does — not implemented
- [ ] Falling behaviour matches the original — partial
- [ ] Death/respawn animation and timing match the original — partial
- [ ] Exact original walk and climb frame timing — partial (needs RE)

## Digging, holes and filling

- [ ] Dig target is the hole centre two cells ahead — implemented, awaiting certification
- [ ] Four Dig States persist correctly — implemented, awaiting certification
- [ ] Partial holes block horizontal entry — implemented, awaiting certification
- [ ] Three-stage hole art aligned to the platform top — implemented, awaiting certification
- [ ] World-specific partial-hole masks (`01bc`/`01bd`) are correct — implemented, awaiting certification
- [ ] Hole masks cut the platform with alpha, never drawn as red art — implemented, awaiting certification
- [ ] Dig = Space; Fill = Space on a full hole — implemented, awaiting certification
- [ ] Shift fills a partial hole — implemented, awaiting certification
- [ ] Fill-drops and push-through drops behave like the original — implemented, awaiting certification
- [ ] Buried loot is released on the first dig — implemented, awaiting certification

## Monsters

- [ ] Banana starts on idle frame `02c8` — implemented, awaiting certification
- [ ] Banana uses the exact 12-frame walk loop — implemented, awaiting certification
- [ ] Banana walk frames face left by default and mirror when walking right — implemented, awaiting certification
- [ ] Banana walking is smooth and has no spin or jitter — implemented, awaiting certification
- [ ] Banana climb frames `02da`/`02db` — implemented, awaiting certification
- [ ] Banana hole-wiggle frames `02d5`/`02dc` — implemented, awaiting certification
- [ ] Generic trapped-monster hands `00ac`/`00ad`/`00ae` match hole stage — implemented, awaiting certification
- [ ] Monster walk animation is tied to movement, not a free-running tick — implemented, awaiting certification
- [ ] Monster movement is interpolated (no cell teleporting) — implemented, awaiting certification
- [ ] Pumpkin walk cycle matches the original — partial (frames guessed)
- [ ] Ananas walk cycle matches the original — partial (frames guessed)
- [ ] Coconut walk cycle matches the original — partial (frames guessed)
- [ ] Log walk cycle matches the original — partial (frames guessed)
- [ ] Bee (`bi`) behaviour matches the original — partial
- [ ] Type-0 `program` movement matches the original — implemented, awaiting certification
- [ ] Patrol-until-edge behaviour matches the original — implemented, awaiting certification
- [ ] Type-3 monster behaviours — not implemented

## Items, pickups and scoring

- [ ] Magic Ladder grants 3 charges and deploys with Alt/M + Up/Down — implemented, awaiting certification
- [ ] Magic Ladder art and lifetime match the original — implemented, awaiting certification
- [ ] `thrusign` is Cave-only edge wrap, not a magic ladder — implemented, awaiting certification
- [ ] Instant and Pause pickups behave correctly — implemented, awaiting certification
- [ ] Loot group `indhold` alternatives (`|`) pick the right item — implemented, awaiting certification
- [ ] Crate loot bursts match the original — implemented, awaiting certification
- [ ] Bombs are consumable and cleared on death — implemented, awaiting certification
- [ ] Non-permanent items are cleared on death — implemented, awaiting certification
- [ ] Permanents from the Arcane Book survive death — implemented, awaiting certification
- [ ] Arcane Book grants Ring → Tennis Shoes → Power Glove → Jackhammer → Tennis Racket; extra books do nothing — implemented, awaiting certification
- [ ] Score values per token match the original binary — implemented, awaiting certification
- [ ] Extra-life threshold and timing — partial
- [ ] Weapons and projectiles — not implemented
- [ ] Elevators and bubbles — not implemented
- [ ] Secret walls — not implemented

## Worlds, levels and platform

- [ ] Worlds are configuration files, no hardcoded enum — implemented, awaiting certification
- [ ] All five original worlds load and render — implemented, awaiting certification
- [ ] All original levels load and round-trip through the `.K` codec — implemented, awaiting certification
- [ ] Per-world palettes and tiles are correct — partial
- [ ] Portals behave like the original — implemented, awaiting certification
- [ ] Self-fill and grace timing match the original — partial
- [ ] Original-art asset pack loader — partial
- [ ] Audio — not implemented

## Shells and tooling

- [ ] Browser shell plays the vertical slice end-to-end — implemented, awaiting certification
- [ ] Native desktop shell matches the browser shell — partial
- [ ] Keyboard, touch and gamepad input all work — partial
- [ ] English, Danish and Esperanto UI strings — implemented, awaiting certification
- [ ] Build stamp visible at the page bottom — implemented, awaiting certification
- [ ] Deterministic input replay / record & verify — not implemented

## Known defects

- Alarm clock pickup added a small amount of time instead of resetting the bar; fixed to refill to the level start value, awaiting re-test. (Found while certifying the time bar.)

## Certification log

| Date | Item | Certified by |
| --- | --- | --- |
| 2026-11-26 | Score right anchor, digit overlap and vertical offsets | owner |
| 2026-11-27 | Inventory counts (overlap and per-item alignment) | owner |
| 2026-11-27 | HUD at the top of the screen | owner |
| 2026-11-27 | Time is a progress bar (alarm clock refill) | owner |
| 2026-11-27 | Score colourful digit art | owner |
| 2026-11-27 | Animated hearts and three-digit overflow | owner |
| 2026-11-27 | Inventory native icons in fixed slots | owner |
| 2026-11-27 | Inventory order | owner |
| 2026-11-27 | Dig-tool slot order (Bomb, Jackhammer, Pickaxe) | owner |
| 2026-11-27 | P badge only for Arcane Book Permanents | owner |
| 2026-11-27 | Top adornments on Jungle and Construction | owner |
| 2026-11-27 | No adornments on Cave, Lagoon or Egypt | owner |
| 2026-11-27 | Bottom platform half visible | owner |
| 2026-11-27 | All art shares the background 1:1 scale | owner |
| 2026-11-27 | ALL16 bitmap font at designed size | owner |
