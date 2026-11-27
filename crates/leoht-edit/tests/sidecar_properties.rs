//! Property tests: any valid recipe must survive a sidecar write/read unchanged,
//! and the on-disk layout of a representative sidecar must not drift silently.

use camino::Utf8PathBuf;
use leoht_core::{EditOp, EditRecipe, RedEyeRegion};
use leoht_edit::Sidecar;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed};

/// Fixed seed: every run explores the same cases, so a failure always reproduces.
/// Change it on purpose (and say so in the PR) to explore new cases.
const PROPTEST_SEED: u64 = 0x1E07_5EED;

/// Strategy producing only *valid* operations (inside the documented ranges).
fn arb_op() -> impl Strategy<Value = EditOp> {
    prop_oneof![
        (0.0f32..0.5, 0.0f32..0.5, 0.01f32..0.5, 0.01f32..0.5).prop_map(|(x, y, width, height)| {
            EditOp::Crop {
                x,
                y,
                width,
                height,
            }
        }),
        (1u8..=3).prop_map(|quarter_turns| EditOp::Rotate { quarter_turns }),
        (-45.0f32..=45.0).prop_map(|degrees| EditOp::Straighten { degrees }),
        (-4.0f32..=4.0).prop_map(|stops| EditOp::Exposure { stops }),
        (-1.0f32..=1.0).prop_map(|amount| EditOp::Contrast { amount }),
        (-1.0f32..=1.0).prop_map(|amount| EditOp::Saturation { amount }),
        prop::collection::vec((0.0f32..=1.0, 0.0f32..=1.0, 0.001f32..=0.5), 1..4).prop_map(|rs| {
            EditOp::RedEye {
                regions: rs
                    .into_iter()
                    .map(|(cx, cy, radius)| RedEyeRegion { cx, cy, radius })
                    .collect(),
            }
        }),
    ]
}

proptest! {
    #![proptest_config(Config { rng_seed: RngSeed::Fixed(PROPTEST_SEED), ..Config::default() })]

    #[test]
    fn any_valid_recipe_round_trips(ops in prop::collection::vec(arb_op(), 0..8)) {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().join("p.jpg.leoht.json")).unwrap();
        let sidecar = Sidecar::new(EditRecipe { ops });
        sidecar.write(&path).unwrap();
        prop_assert_eq!(Sidecar::read(&path).unwrap().unwrap(), sidecar);
    }
}

#[test]
fn sidecar_layout_snapshot() {
    let sidecar = Sidecar::new(EditRecipe {
        ops: vec![
            EditOp::Crop {
                x: 0.25,
                y: 0.25,
                width: 0.5,
                height: 0.5,
            },
            EditOp::Rotate { quarter_turns: 1 },
            EditOp::Exposure { stops: 0.5 },
        ],
    });
    insta::assert_json_snapshot!(sidecar);
}
