//! VGA palette loading.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    rgb: [[u8; 3]; 256],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetError {
    TooShort { what: &'static str, need: usize, got: usize },
    NoSprites,
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort { what, need, got } => {
                write!(f, "{what} is too short: need {need} bytes, got {got}")
            }
            Self::NoSprites => write!(f, "no sprite blocks found"),
        }
    }
}

impl std::error::Error for AssetError {}

impl Palette {
    /// `ALL8.DB` stores the 6-bit VGA DAC values at offset 1.  The game
    /// multiplies each channel by 4 to expand 0..63 into 0..255.
    pub fn from_all8(data: &[u8]) -> Result<Self, AssetError> {
        Self::from_vga_dac(data, 1)
    }

    /// A raw 768-byte VGA DAC table (6-bit channels).
    pub fn from_vga_dac(data: &[u8], offset: usize) -> Result<Self, AssetError> {
        let need = offset + 768;
        if data.len() < need {
            return Err(AssetError::TooShort {
                what: "VGA DAC table",
                need,
                got: data.len(),
            });
        }
        let mut rgb = [[0u8; 3]; 256];
        for (index, color) in rgb.iter_mut().enumerate() {
            let base = offset + index * 3;
            color[0] = data[base].saturating_mul(4);
            color[1] = data[base + 1].saturating_mul(4);
            color[2] = data[base + 2].saturating_mul(4);
        }
        Ok(Self { rgb })
    }

    pub fn rgb(&self, index: u8) -> [u8; 3] {
        self.rgb[index as usize]
    }

    pub fn rgba(&self, index: u8) -> [u8; 4] {
        let [r, g, b] = self.rgb(index);
        [r, g, b, 255]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_six_bit_channels() {
        let mut data = vec![0u8; 769];
        data[1] = 63;
        data[2] = 1;
        data[3] = 0;
        let palette = Palette::from_all8(&data).unwrap();
        assert_eq!(palette.rgb(0), [252, 4, 0]);
        assert_eq!(palette.rgba(0), [252, 4, 0, 255]);
    }

    #[test]
    fn rejects_a_short_file() {
        assert!(matches!(
            Palette::from_all8(&[0u8; 10]),
            Err(AssetError::TooShort { .. })
        ));
    }
}
