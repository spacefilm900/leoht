//! Thumbnail source choice and the orientation golden table (FR-LIB-07).

use camino::Utf8PathBuf;
use leoht_core::Orientation;
use leoht_testkit::fixtures::{self, ExifSpec};

use super::*;

const R: [u8; 3] = [255, 0, 0];
const G: [u8; 3] = [0, 255, 0];
const B: [u8; 3] = [0, 0, 255];
const W: [u8; 3] = [255, 255, 255];

/// Stored image: red top-left, green top-right, blue bottom-left, white
/// bottom-right. For each EXIF orientation, the colours an upright display
/// shows at (top-left, top-right, bottom-left, bottom-right). Worked out from
/// the EXIF 2.32 definitions, independently of the `image` crate:
/// 2 mirror, 3 rotate 180, 4 flip, 5 transpose, 6 rotate 90 CW,
/// 7 transverse, 8 rotate 90 CCW.
const UPRIGHT_CORNERS: [(u16, [[u8; 3]; 4]); 8] = [
    (1, [R, G, B, W]),
    (2, [G, R, W, B]),
    (3, [W, B, G, R]),
    (4, [B, W, R, G]),
    (5, [R, B, G, W]),
    (6, [B, R, W, G]),
    (7, [W, G, B, R]),
    (8, [G, W, R, B]),
];

fn temp_file(dir: &tempfile::TempDir, name: &str, bytes: &[u8]) -> Utf8PathBuf {
    let p = Utf8PathBuf::from_path_buf(dir.path().join(name)).unwrap();
    std::fs::write(&p, bytes).unwrap();
    p
}

fn close(px: image::Rgb<u8>, want: [u8; 3]) -> bool {
    px.0.iter().zip(want).all(|(&a, b)| a.abs_diff(b) < 60)
}

fn corners(img: &RgbImage) -> [image::Rgb<u8>; 4] {
    let (left, right) = (img.width() / 4, img.width() * 3 / 4);
    let (top, bottom) = (img.height() / 4, img.height() * 3 / 4);
    [
        *img.get_pixel(left, top),
        *img.get_pixel(right, top),
        *img.get_pixel(left, bottom),
        *img.get_pixel(right, bottom),
    ]
}

fn oriented(orientation: u16) -> ExifSpec {
    ExifSpec {
        orientation: Some(orientation),
        ..ExifSpec::default()
    }
}

fn request<'a>(source: &'a Utf8Path, preview: Option<&'a [u8]>, may_decode: bool) -> Request<'a> {
    Request {
        source,
        preview,
        orientation: Orientation::UPRIGHT,
        max_edge: 32,
        may_decode,
    }
}

#[test]
fn all_eight_orientations_are_upright_for_jpeg_and_png() {
    let dir = tempfile::tempdir().unwrap();
    let img = fixtures::quadrants(48, 32);
    for (value, want) in UPRIGHT_CORNERS {
        let spec = oriented(value);
        for (ext, bytes) in [
            ("jpg", fixtures::jpeg(&img, Some(&spec), None).unwrap()),
            ("png", fixtures::png(&img, Some(&spec), None).unwrap()),
        ] {
            let path = temp_file(&dir, &format!("o{value}.{ext}"), &bytes);
            let (thumb, origin) = render(&request(&path, None, true)).unwrap();
            assert_eq!(origin, ThumbOrigin::Decoded);
            let swapped = value >= 5;
            let dims = if swapped { (21, 32) } else { (32, 21) };
            assert_eq!(
                (thumb.width(), thumb.height()),
                dims,
                "{ext} orientation {value}"
            );
            for (got, want) in corners(&thumb).iter().zip(want) {
                assert!(
                    close(*got, want),
                    "{ext} orientation {value}: {got:?} != {want:?}"
                );
            }
        }
    }
}

#[test]
fn a_big_enough_preview_is_used_and_oriented_from_the_photo() {
    let dir = tempfile::tempdir().unwrap();
    let preview = fixtures::jpeg(&fixtures::quadrants(64, 48), None, None).unwrap();
    let path = temp_file(&dir, "p.jpg", b"not decoded");
    let mut req = request(&path, Some(&preview), true);
    req.orientation = Orientation::from_exif(6).unwrap();
    let (thumb, origin) = render(&req).unwrap();
    assert_eq!(origin, ThumbOrigin::EmbeddedPreview);
    assert_eq!((thumb.width(), thumb.height()), (24, 32));
    assert!(close(corners(&thumb)[0], B), "rotated 90 CW: blue top-left");
}

#[test]
fn a_small_preview_falls_back_to_decoding_the_photo() {
    let dir = tempfile::tempdir().unwrap();
    let tiny = fixtures::jpeg(&fixtures::quadrants(8, 6), None, None).unwrap();
    let photo = fixtures::jpeg(&fixtures::quadrants(96, 64), None, None).unwrap();
    let path = temp_file(&dir, "big.jpg", &photo);
    let (thumb, origin) = render(&request(&path, Some(&tiny), true)).unwrap();
    assert_eq!(origin, ThumbOrigin::Decoded);
    assert_eq!(thumb.width(), 32);
}

#[test]
fn raw_style_requests_use_any_preview_and_never_decode() {
    let dir = tempfile::tempdir().unwrap();
    let tiny = fixtures::jpeg(&fixtures::quadrants(8, 6), None, None).unwrap();
    let path = temp_file(&dir, "a.raf", b"raw sensor data");
    let (thumb, origin) = render(&request(&path, Some(&tiny), false)).unwrap();
    assert_eq!(origin, ThumbOrigin::EmbeddedPreview);
    assert_eq!(thumb.width(), 8, "never upscaled");
    assert!(matches!(
        render(&request(&path, None, false)),
        Err(ThumbError::NoSource { .. })
    ));
    let garbage = [0xFF, 0xD8, 1, 2, 3];
    assert!(matches!(
        render(&request(&path, Some(&garbage), false)),
        Err(ThumbError::NoSource { .. })
    ));
}

#[test]
fn big_sources_are_pre_shrunk_to_the_same_size() {
    let img = DynamicImage::ImageRgb8(fixtures::quadrants(1000, 500));
    let thumb = resize_to_fit(&img, 100);
    assert_eq!((thumb.width(), thumb.height()), (100, 50));
    assert!(close(corners(&thumb)[3], W));
}

#[test]
fn atomic_write_leaves_only_the_thumbnail() {
    let dir = tempfile::tempdir().unwrap();
    let dest = Utf8PathBuf::from_path_buf(dir.path().join("a/b/t.jpg")).unwrap();
    write_atomically(&fixtures::quadrants(10, 10), &dest).unwrap();
    let names: Vec<_> = std::fs::read_dir(dest.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names, vec!["t.jpg"]);
    assert_eq!(image::open(&dest).unwrap().width(), 10);
}
