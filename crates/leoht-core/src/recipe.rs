//! Edit recipes: an ordered list of non-destructive adjustments.
//!
//! A recipe is *data*, not behaviour: applying it to pixels is the job of
//! `leoht-edit` (Phase 3). The operation set is the one fixed by the plan:
//! crop, rotate, straighten, exposure, contrast, saturation, red-eye.
//!
//! Geometry is stored in *normalised* coordinates (0.0–1.0 of the image
//! width/height) so a recipe stays valid if the original is later re-exported
//! at a different resolution.

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// A circular red-eye correction region, in normalised image coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RedEyeRegion {
    /// Centre x, 0.0–1.0.
    pub cx: f32,
    /// Centre y, 0.0–1.0.
    pub cy: f32,
    /// Radius as a fraction of the image width, > 0.0 and ≤ 0.5.
    pub radius: f32,
}

/// One adjustment. Serialised with an `op` tag, e.g. `{"op":"rotate","quarter_turns":1}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum EditOp {
    /// Crop to a normalised rectangle.
    Crop {
        /// Left edge, 0.0–1.0.
        x: f32,
        /// Top edge, 0.0–1.0.
        y: f32,
        /// Width, > 0.0; `x + width` must not exceed 1.0.
        width: f32,
        /// Height, > 0.0; `y + height` must not exceed 1.0.
        height: f32,
    },
    /// Rotate clockwise in 90° steps (1, 2 or 3).
    Rotate {
        /// Number of clockwise quarter turns.
        quarter_turns: u8,
    },
    /// Fine rotation to level a horizon, in degrees (−45.0 to 45.0).
    Straighten {
        /// Clockwise degrees.
        degrees: f32,
    },
    /// Exposure compensation in stops (−4.0 to 4.0).
    Exposure {
        /// Stops; positive brightens.
        stops: f32,
    },
    /// Contrast adjustment (−1.0 to 1.0).
    Contrast {
        /// Amount; 0.0 is neutral.
        amount: f32,
    },
    /// Saturation adjustment (−1.0 to 1.0; −1.0 is greyscale).
    Saturation {
        /// Amount; 0.0 is neutral.
        amount: f32,
    },
    /// Red-eye correction over one or more regions.
    RedEye {
        /// Regions to correct; must not be empty.
        regions: Vec<RedEyeRegion>,
    },
}

impl EditOp {
    /// The serialised `op` tag, used in error messages.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Crop { .. } => "crop",
            Self::Rotate { .. } => "rotate",
            Self::Straighten { .. } => "straighten",
            Self::Exposure { .. } => "exposure",
            Self::Contrast { .. } => "contrast",
            Self::Saturation { .. } => "saturation",
            Self::RedEye { .. } => "red_eye",
        }
    }

    /// Checks that every parameter is finite and in range.
    ///
    /// # Errors
    /// Returns [`CoreError::InvalidEdit`] describing the first violation.
    pub fn validate(&self) -> Result<(), CoreError> {
        let op = self.name();
        let fail = |reason| Err(CoreError::InvalidEdit { op, reason });
        match self {
            Self::Crop {
                x,
                y,
                width,
                height,
            } => {
                let finite = [x, y, width, height].iter().all(|v| v.is_finite());
                if !finite {
                    return fail("values must be finite");
                }
                if *x < 0.0 || *y < 0.0 || *width <= 0.0 || *height <= 0.0 {
                    return fail("x, y must be >= 0 and width, height must be > 0");
                }
                if x + width > 1.0 || y + height > 1.0 {
                    return fail("rectangle must lie inside the image");
                }
            }
            Self::Rotate { quarter_turns } if !(1..=3).contains(quarter_turns) => {
                return fail("quarter_turns must be 1, 2 or 3");
            }
            Self::Straighten { degrees } if !in_range(*degrees, 45.0) => {
                return fail("degrees must be within -45..=45");
            }
            Self::Exposure { stops } if !in_range(*stops, 4.0) => {
                return fail("stops must be within -4..=4");
            }
            Self::Contrast { amount } | Self::Saturation { amount } if !in_range(*amount, 1.0) => {
                return fail("amount must be within -1..=1");
            }
            Self::RedEye { regions } => {
                if regions.is_empty() {
                    return fail("at least one region is required");
                }
                let ok = regions.iter().all(|r| {
                    (0.0..=1.0).contains(&r.cx)
                        && (0.0..=1.0).contains(&r.cy)
                        && r.radius > 0.0
                        && r.radius <= 0.5
                });
                if !ok {
                    return fail("region centre must be in 0..=1 and radius in (0, 0.5]");
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// True if `value` is finite and within `-limit..=limit`.
fn in_range(value: f32, limit: f32) -> bool {
    value.is_finite() && value.abs() <= limit
}

/// An ordered list of edits applied to the untouched original, first to last.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditRecipe {
    /// The operations, in application order.
    pub ops: Vec<EditOp>,
}

impl EditRecipe {
    /// True if the recipe changes nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Validates every operation.
    ///
    /// # Errors
    /// Returns the first [`CoreError::InvalidEdit`] found.
    pub fn validate(&self) -> Result<(), CoreError> {
        self.ops.iter().try_for_each(EditOp::validate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_ops_pass() {
        let recipe = EditRecipe {
            ops: vec![
                EditOp::Crop {
                    x: 0.1,
                    y: 0.1,
                    width: 0.5,
                    height: 0.5,
                },
                EditOp::Rotate { quarter_turns: 1 },
                EditOp::Straighten { degrees: -2.5 },
                EditOp::Exposure { stops: 0.7 },
                EditOp::Contrast { amount: 0.2 },
                EditOp::Saturation { amount: -1.0 },
                EditOp::RedEye {
                    regions: vec![RedEyeRegion {
                        cx: 0.4,
                        cy: 0.4,
                        radius: 0.02,
                    }],
                },
            ],
        };
        assert!(recipe.validate().is_ok());
    }

    #[test]
    fn invalid_ops_are_rejected() {
        let bad = [
            EditOp::Crop {
                x: 0.8,
                y: 0.0,
                width: 0.5,
                height: 0.5,
            },
            EditOp::Rotate { quarter_turns: 0 },
            EditOp::Rotate { quarter_turns: 4 },
            EditOp::Straighten { degrees: f32::NAN },
            EditOp::Exposure { stops: 9.0 },
            EditOp::Contrast { amount: 1.5 },
            EditOp::Saturation {
                amount: f32::INFINITY,
            },
            EditOp::RedEye { regions: vec![] },
        ];
        for op in bad {
            assert!(op.validate().is_err(), "{op:?} should be invalid");
        }
    }

    #[test]
    fn serialises_with_an_op_tag() {
        let json = serde_json::to_string(&EditOp::Rotate { quarter_turns: 1 }).unwrap();
        assert_eq!(json, r#"{"op":"rotate","quarter_turns":1}"#);
    }
}
