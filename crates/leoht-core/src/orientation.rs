//! EXIF orientation: how a stored image must be turned to appear upright.

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// The eight EXIF orientation values (TIFF tag 0x0112), as defined by the EXIF
/// standard. Value 1 means "stored upright"; the others combine a rotation and
/// an optional mirror. Anything outside 1..=8 in a file is treated as 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Orientation(u8);

impl Orientation {
    /// Stored upright (EXIF value 1).
    pub const UPRIGHT: Self = Self(1);

    /// Builds an orientation from its EXIF value; `None` outside 1..=8.
    #[must_use]
    pub fn from_exif(value: u32) -> Option<Self> {
        u8::try_from(value)
            .ok()
            .filter(|v| (1..=8).contains(v))
            .map(Self)
    }

    /// The EXIF value, 1..=8.
    #[must_use]
    pub fn value(self) -> u8 {
        self.0
    }

    /// True when the upright image is the stored one turned a quarter turn
    /// (values 5 to 8), so its width and height swap.
    #[must_use]
    pub fn swaps_dimensions(self) -> bool {
        self.value() >= 5
    }

    /// Upright `(width, height)` for a stored image of `width` x `height`.
    #[must_use]
    pub fn upright_size(self, width: u32, height: u32) -> (u32, u32) {
        if self.swaps_dimensions() {
            (height, width)
        } else {
            (width, height)
        }
    }
}

impl Default for Orientation {
    fn default() -> Self {
        Self::UPRIGHT
    }
}

impl TryFrom<u8> for Orientation {
    type Error = CoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_exif(u32::from(value)).ok_or(CoreError::InvalidOrientation(value))
    }
}

impl From<Orientation> for u8 {
    fn from(value: Orientation) -> Self {
        value.value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_values_one_to_eight_are_orientations() {
        assert_eq!(Orientation::from_exif(0), None);
        assert_eq!(Orientation::from_exif(9), None);
        assert_eq!(Orientation::from_exif(256 + 6), None);
        for v in 1..=8 {
            assert_eq!(u32::from(Orientation::from_exif(v).unwrap().value()), v);
        }
    }

    #[test]
    fn quarter_turn_orientations_swap_width_and_height() {
        for v in 1..=8 {
            let o = Orientation::from_exif(v).unwrap();
            let expected = if v >= 5 { (20, 40) } else { (40, 20) };
            assert_eq!(o.upright_size(40, 20), expected, "orientation {v}");
        }
        assert_eq!(Orientation::default(), Orientation::UPRIGHT);
    }
}
