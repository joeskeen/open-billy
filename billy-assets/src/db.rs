//! Decoder for the uncompressed 43-block sprite atlases (`JUNGLE.DB`,
//! `CONSTR.DB`, `CAVE.DB`, `WATER.DB`, `EGYPT.DB`).
//!
//! Each block is a sequence of scanline records.  A record is
//! `[0x83] A:u32 B:u32 C:u32 pixels[B]`, where `A` is the left trim, `B` the
//! run length and `C` is always zero.  A row may contain several runs; the
//! 3-byte trailer belongs to the row, not to each run.

use crate::palette::Palette;

pub const SIGNATURE: [u8; 8] = [0x00, 0x5a, 0x00, 0x00, 0x00, 0x5a, 0x00, 0x00];
pub const SIGNATURE_TO_RECORDS: usize = 28;
pub const SIGNATURE_TO_END: usize = 8;
const MAX_X: u32 = 400;
const MAX_B: u32 = 300;
const MAX_WIDTH: u32 = 400;
const MAX_HEIGHT: u32 = 400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    /// Left trim in pixels.
    pub x: u32,
    /// Palette indices; every stored pixel is opaque.
    pub pixels: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sprite {
    pub width: u32,
    pub height: u32,
    pub rows: Vec<Vec<Run>>,
}

impl Sprite {
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// True for the two non-sprite tile sheets (blocks 39 and 42).
    pub fn is_tileset(&self) -> bool {
        self.width > MAX_WIDTH || self.height >= MAX_HEIGHT
    }

    /// Expand to RGBA8.  Unstored pixels stay transparent; every stored index
    /// is opaque (the DOS art uses black as real shading, not as a key).
    pub fn to_rgba(&self, palette: &Palette) -> Vec<u8> {
        let width = self.width as usize;
        let height = self.height as usize;
        let mut out = vec![0u8; width * height * 4];
        for (y, row) in self.rows.iter().enumerate() {
            if y >= height {
                break;
            }
            for run in row {
                for (i, &index) in run.pixels.iter().enumerate() {
                    let x = run.x as usize + i;
                    if x >= width {
                        continue;
                    }
                    let offset = (y * width + x) * 4;
                    let [r, g, b, a] = palette.rgba(index);
                    out[offset..offset + 4].copy_from_slice(&[r, g, b, a]);
                }
            }
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Atlas {
    pub sprites: Vec<Sprite>,
    pub block_offsets: Vec<usize>,
}

impl Atlas {
    pub fn parse(data: &[u8]) -> Self {
        let offsets = block_offsets(data);
        let mut sprites = Vec::with_capacity(offsets.len());
        for (i, &signature) in offsets.iter().enumerate() {
            let start = signature + SIGNATURE_TO_RECORDS;
            let end = offsets
                .get(i + 1)
                .map(|next| next.saturating_sub(SIGNATURE_TO_END))
                .unwrap_or(data.len());
            if start >= end || end > data.len() {
                continue;
            }
            let rows = parse_block(data, start, end);
            if rows.is_empty() {
                continue;
            }
            let width = rows
                .iter()
                .flat_map(|row| row.iter())
                .map(|run| run.x + run.pixels.len() as u32)
                .max()
                .unwrap_or(0)
                .saturating_add(1);
            let height = rows.len() as u32;
            sprites.push(Sprite { width, height, rows });
        }
        Self { sprites, block_offsets: offsets }
    }

    pub fn sprite(&self, index: usize) -> Option<&Sprite> {
        self.sprites.get(index)
    }

    pub fn game_sprites(&self) -> impl Iterator<Item = &Sprite> {
        self.sprites.iter().filter(|s| !s.is_tileset())
    }
}

pub fn block_offsets(data: &[u8]) -> Vec<usize> {
    let mut out = Vec::new();
    let mut from = 0usize;
    while from + SIGNATURE.len() <= data.len() {
        let Some(relative) = data[from..]
            .windows(SIGNATURE.len())
            .position(|window| window == SIGNATURE)
        else {
            break;
        };
        let at = from + relative;
        out.push(at);
        from = at + 1;
    }
    out
}

#[derive(Debug, Clone, Copy)]
struct Record {
    pos: usize,
    a: u32,
    b: u32,
}

fn read_u32(data: &[u8], at: usize) -> Option<u32> {
    let bytes = data.get(at..at + 4)?;
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn walk_records(data: &[u8], start: usize, end: usize) -> Vec<Record> {
    let mut records = Vec::new();
    let mut pos = start;
    while pos + 16 <= end {
        if data[pos] != 0x83 {
            match data[pos + 1..end].iter().position(|&b| b == 0x83) {
                Some(relative) => {
                    pos = pos + 1 + relative;
                    continue;
                }
                None => break,
            }
        }
        let a = read_u32(data, pos + 1).unwrap_or(u32::MAX);
        let b = read_u32(data, pos + 5).unwrap_or(0);
        let c = read_u32(data, pos + 9).unwrap_or(1);
        if c != 0
            || a > MAX_X
            || !(1..=MAX_B).contains(&b)
            || pos + 16 + b as usize > end
        {
            match data[pos + 1..end].iter().position(|&b| b == 0x83) {
                Some(relative) => {
                    pos = pos + 1 + relative;
                    continue;
                }
                None => break,
            }
        }
        records.push(Record { pos, a, b });
        pos += 16 + b as usize;
    }
    records
}

fn parse_block(data: &[u8], start: usize, end: usize) -> Vec<Vec<Run>> {
    let records = walk_records(data, start, end);
    if records.is_empty() {
        return Vec::new();
    }

    let mut rows = Vec::with_capacity(records.len());
    for (index, record) in records.iter().enumerate() {
        let len = if let Some(next) = records.get(index + 1) {
            next.pos.saturating_sub(record.pos)
        } else {
            let mut len = end.saturating_sub(record.pos).saturating_sub(3);
            if len < 16 {
                len = end.saturating_sub(record.pos);
            }
            len
        };

        let runs = if len >= 3 {
            parse_row(data, record.pos, len)
        } else {
            Vec::new()
        };
        if runs.is_empty() {
            let pixels_start = record.pos + 13;
            let pixels_end = pixels_start.saturating_add(record.b as usize);
            if pixels_end <= data.len() {
                rows.push(vec![Run {
                    x: record.a,
                    pixels: data[pixels_start..pixels_end].to_vec(),
                }]);
            } else {
                rows.push(Vec::new());
            }
        } else {
            rows.push(runs);
        }
    }
    rows
}

fn parse_row(data: &[u8], start: usize, len: usize) -> Vec<Run> {
    if len < 3 || start >= data.len() {
        return Vec::new();
    }
    let header = usize::from(data[start] == 0x83);
    let mut memo = vec![None; len + 1];
    if let Some(runs) = solve_row(data, start, header, len - 3, &mut memo) {
        return runs;
    }
    Vec::new()
}

/// Recover a row's runs, anchored on the 3-byte trailer.  `off` is relative to
/// `start` and points at the trailer.
fn solve_row(
    data: &[u8],
    start: usize,
    header: usize,
    off: usize,
    memo: &mut [Option<Option<Vec<Run>>>],
) -> Option<Vec<Run>> {
    if let Some(cached) = memo.get(off) {
        return cached.clone().flatten();
    }
    if off == header {
        if let Some(slot) = memo.get_mut(off) {
            *slot = Some(Some(Vec::new()));
        }
        return Some(Vec::new());
    }
    if let Some(slot) = memo.get_mut(off) {
        *slot = Some(None);
    }

    let available = off.saturating_sub(header) as u32;
    for b in 1..=available.min(MAX_B) {
        let Some(run_start) = off.checked_sub(b as usize + 12) else {
            break;
        };
        if run_start < header {
            break;
        }
        if read_u32(data, start + run_start + 4) != Some(b) {
            continue;
        }
        if read_u32(data, start + run_start + 8) != Some(0) {
            continue;
        }
        let Some(a) = read_u32(data, start + run_start) else {
            continue;
        };
        if a > MAX_X {
            continue;
        }
        if let Some(mut runs) = solve_row(data, start, header, run_start, memo) {
            let pixels = data[start + off - b as usize..start + off].to_vec();
            runs.insert(0, Run { x: a, pixels });
            if let Some(slot) = memo.get_mut(off) {
                *slot = Some(Some(runs.clone()));
            }
            return Some(runs);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_atlas() -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&SIGNATURE);
        data.extend_from_slice(&[0u8; 20]); // block header padding

        // Row 0: one run, three pixels.
        data.push(0x83);
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&3u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&[1, 2, 3]);
        data.extend_from_slice(&[0, 0, 0]);

        // Row 1: one run, two pixels starting at x=2.
        data.push(0x83);
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&[4, 5]);
        data.extend_from_slice(&[0, 0, 0]);
        data
    }

    #[test]
    fn finds_the_block_signature() {
        let data = synthetic_atlas();
        let offsets = block_offsets(&data);
        assert_eq!(offsets, vec![0]);
    }

    #[test]
    fn decodes_single_run_rows() {
        let atlas = Atlas::parse(&synthetic_atlas());
        assert_eq!(atlas.sprites.len(), 1);
        let sprite = &atlas.sprites[0];
        assert_eq!(sprite.height, 2);
        assert_eq!(sprite.rows[0][0].x, 0);
        assert_eq!(sprite.rows[0][0].pixels, vec![1, 2, 3]);
        assert_eq!(sprite.rows[1][0].x, 2);
        assert_eq!(sprite.rows[1][0].pixels, vec![4, 5]);
        assert_eq!(sprite.width, 5);
    }

    #[test]
    fn expands_to_rgba() {
        let mut dac = vec![0u8; 769];
        dac[1] = 63; // index 0 red
        dac[5] = 63; // index 1 green
        let palette = Palette::from_all8(&dac).unwrap();
        let atlas = Atlas::parse(&synthetic_atlas());
        let rgba = atlas.sprites[0].to_rgba(&palette);
        assert_eq!(rgba.len(), 5 * 2 * 4);
        // Row 0 stores palette indices 1, 2, 3; index 1 is green, and every
        // stored pixel is opaque even when the colour is black.
        assert_eq!(&rgba[0..4], &[0, 252, 0, 255]);
        assert_eq!(&rgba[4..8], &[0, 0, 0, 255]);
        // The right-most column of row 0 is stored; row 1's x=0..1 is empty.
        assert_eq!(&rgba[5 * 4 + 0..5 * 4 + 4], &[0, 0, 0, 0]);
    }
}
