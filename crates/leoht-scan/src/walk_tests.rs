//! Unit tests for the walk (kept beside it to keep `walk.rs` short).

use std::time::Duration;

use super::*;

fn touch(root: &Utf8Path, rel: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, b"x").unwrap();
}

fn root(dir: &tempfile::TempDir) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap()
}

fn listed(root: &Utf8Path, config: &LibraryConfig) -> Vec<String> {
    let mut found = Vec::new();
    walk_library(root, config, |f| found.push(f.rel.to_string())).unwrap();
    found
}

#[test]
fn split_time_handles_both_sides_of_the_epoch() {
    assert_eq!(split_time(UNIX_EPOCH), (0, 0));
    assert_eq!(
        split_time(UNIX_EPOCH + Duration::from_millis(5_250)),
        (5, 250_000_000)
    );
    assert_eq!(split_time(UNIX_EPOCH - Duration::from_secs(1)), (-1, 0));
    assert_eq!(
        split_time(UNIX_EPOCH - Duration::from_millis(500)),
        (-1, 500_000_000)
    );
}

#[test]
fn finds_images_recursively_in_name_order_and_ignores_other_files() {
    let dir = tempfile::tempdir().unwrap();
    let r = root(&dir);
    for f in [
        "b.jpg",
        "2020/b.PNG",
        "2020/trip/c.heic",
        "2020/notes.txt",
        "a.jpg.leoht.json",
        "a.jpg",
    ] {
        touch(&r, f);
    }
    assert_eq!(
        listed(&r, &LibraryConfig::default()),
        ["2020/b.PNG", "2020/trip/c.heic", "a.jpg", "b.jpg"]
    );
}

#[test]
fn skips_hidden_files_and_folders() {
    let dir = tempfile::tempdir().unwrap();
    let r = root(&dir);
    for f in [".leoht/thumb.jpg", "._a.jpg", ".hidden.jpg", "keep/a.jpg"] {
        touch(&r, f);
    }
    assert_eq!(listed(&r, &LibraryConfig::default()), ["keep/a.jpg"]);
}

#[test]
fn ignore_patterns_and_format_switches_apply() {
    let dir = tempfile::tempdir().unwrap();
    let r = root(&dir);
    for f in [
        "keep/a.jpg",
        "Backups/b.jpg",
        "deep/x/Backups/c.jpg",
        "raw/d.cr2",
    ] {
        touch(&r, f);
    }
    let config = LibraryConfig {
        ignore: vec!["Backups/".into()],
        formats: [("raw".to_owned(), false)].into(),
        ..LibraryConfig::default()
    };
    assert_eq!(listed(&r, &config), ["keep/a.jpg"]);
}

#[test]
fn bad_ignore_pattern_is_a_config_error() {
    let dir = tempfile::tempdir().unwrap();
    let config = LibraryConfig {
        ignore: vec!["a{b".into()],
        ..LibraryConfig::default()
    };
    assert!(matches!(
        walk_library(&root(&dir), &config, |_| {}),
        Err(ScanError::Config { .. })
    ));
}

#[test]
fn records_format_size_and_time() {
    let dir = tempfile::tempdir().unwrap();
    let r = root(&dir);
    touch(&r, "x.CR3");
    let mut found = Vec::new();
    walk_library(&r, &LibraryConfig::default(), |f| found.push(f)).unwrap();
    assert_eq!(found[0].format, PhotoFormat::Raw);
    assert_eq!(found[0].size_bytes, 1);
    assert!(found[0].modified_unix_secs > 1_600_000_000);
}

#[cfg(unix)]
#[test]
fn symlinks_are_recorded_not_followed() {
    let dir = tempfile::tempdir().unwrap();
    let r = root(&dir);
    touch(&r, "real/a.jpg");
    std::os::unix::fs::symlink(r.join("real"), r.join("loop")).unwrap();
    std::os::unix::fs::symlink(r.join("real/a.jpg"), r.join("link.jpg")).unwrap();
    let mut found = Vec::new();
    let skipped = walk_library(&r, &LibraryConfig::default(), |f| {
        found.push(f.rel.to_string());
    })
    .unwrap();
    assert_eq!(found, ["real/a.jpg"]);
    assert_eq!(skipped.len(), 2);
    assert!(skipped.iter().all(|s| s.reason == SkipReason::Symlink));
}

#[test]
fn missing_root_is_an_error() {
    assert!(matches!(
        walk_library(
            Utf8Path::new("/definitely/not/here"),
            &LibraryConfig::default(),
            |_| {}
        ),
        Err(ScanError::NotADirectory(_))
    ));
}
