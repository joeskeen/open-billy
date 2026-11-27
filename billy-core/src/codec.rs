//! The `.K` file cipher.
//!
//! The on-disk bytes are obfuscated with a 107-byte keystream plus a running
//! previous-plaintext delta.  The implicit NUL at index 107 is part of the
//! cycle, so the effective period is 108 bytes.
//!
//! ```text
//! plain[i]  = ((cipher[i] - plain[i-1]) & 0xff) ^ key[(i+1) mod 108]
//! cipher[i] = ((plain[i]  ^ key[(i+1) mod 108]) + plain[i-1]) & 0xff
//! ```

/// The 107-byte keystream embedded in `BILLY.EXE` (Ghidra `0x81def`).
pub const KEY: [u8; 107] = [
    0x54, 0x6f, 0x6d, 0x31, 0x31, 0x61, 0x6c, 0x91, 0x6c, 0x6b, 0x73, 0x91, 0x6a, 0x6f, 0x6e,
    0x65, 0x73, 0x38, 0x5b, 0x7b, 0x20, 0x4d, 0x67, 0x79, 0x75, 0x62, 0x6f, 0x69, 0x37, 0x20,
    0x45, 0x45, 0x45, 0x52, 0x20, 0x54, 0x59, 0x79, 0x69, 0x75, 0x86, 0x70, 0x65, 0x6e, 0x6c,
    0x75, 0x64, 0x65, 0x72, 0x66, 0x6f, 0x72, 0x48, 0x61, 0x33, 0x33, 0x34, 0x33, 0x4e, 0x70,
    0x35, 0x24, 0x24, 0x36, 0x36, 0x24, 0x40, 0x75, 0x74, 0x74, 0x65, 0x72, 0x53, 0x4f, 0x4b,
    0x4b, 0x45, 0x52, 0x49, 0x53, 0x49, 0x4e, 0x45, 0x42, 0x55, 0x6b, 0x73, 0x65, 0x72, 0x63,
    0x40, 0x9c, 0x24, 0x36, 0x37, 0x39, 0x39, 0x6b, 0x67, 0x6c, 0x6a, 0x2f, 0x2f, 0x36, 0x24,
    0x9c, 0x32,
];

/// Number of key bytes, including the implicit NUL terminator.
pub const KEY_CYCLE: usize = KEY.len() + 1;

#[inline]
fn key_at(index: usize) -> u8 {
    KEY.get(index).copied().unwrap_or(0)
}

/// Decode an obfuscated `.K` byte stream into its plaintext body.
pub fn decode(cipher: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(cipher.len());
    let mut previous = 0u8;
    let mut position = 0usize;

    for &byte in cipher {
        if key_at(position) == 0 {
            position = 0;
        } else {
            position += 1;
        }
        let plain = byte.wrapping_sub(previous) ^ key_at(position);
        out.push(plain);
        previous = plain;
    }

    out
}

/// Encode a plaintext level body into the on-disk `.K` byte stream.
pub fn encode(plain: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(plain.len());
    let mut previous = 0u8;
    let mut position = 0usize;

    for &byte in plain {
        if key_at(position) == 0 {
            position = 0;
        } else {
            position += 1;
        }
        out.push((byte ^ key_at(position)).wrapping_add(previous));
        previous = byte;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_arbitrary_bytes() {
        for len in [0usize, 1, 2, 107, 108, 109, 4096] {
            let plain: Vec<u8> = (0..len)
                .map(|i| ((i.wrapping_mul(37) ^ 0x5a) & 0xff) as u8)
                .collect();
            let cipher = encode(&plain);
            assert_eq!(decode(&cipher), plain, "length {len}");
            assert_eq!(encode(&decode(&cipher)), cipher, "length {len}");
        }
    }

    #[test]
    fn key_cycle_includes_nul() {
        assert_eq!(KEY_CYCLE, 108);
        assert_eq!(key_at(107), 0);
        assert_eq!(key_at(108), 0);
    }

    #[test]
    fn first_key_bytes_are_stable() {
        // Guards against accidental edits to the reversed key table.
        assert_eq!(&KEY[..8], b"Tom11al\x91");
        assert_eq!(KEY[106], 0x32);
    }
}
