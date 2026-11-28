//! Mapping between [`Photo`] values and `photos` table rows.

use leoht_core::{
    CaptureTime, ColorInfo, ContentHash, Damage, DamageKind, DateSource, Exposure, Orientation,
    Photo, PhotoFormat, PhotoMeta, Ratio, RelPath,
};
use rusqlite::{Row, ToSql, types::Null};

use crate::CatalogError;

/// Every column, in the order used by [`COLUMNS`] and [`photo_from_row`].
pub const COLUMNS: &str = "path, folder, format, size_bytes, modified_unix_secs, modified_nanos, \
    content_hash, capture_local, capture_offset, capture_source, camera_make, camera_model, \
    lens_model, exposure_time, f_number, iso, focal_length_mm, width, height, orientation, \
    color_kind, icc_description, damage_kind, damage_detail";

/// Number of columns in [`COLUMNS`].
pub const COLUMN_COUNT: usize = 24;

/// Column values for `photo`, in [`COLUMNS`] order.
pub fn photo_params(photo: &Photo) -> Vec<Box<dyn ToSql + '_>> {
    let m = &photo.meta;
    let time = m.capture_time.as_ref();
    let (color_kind, icc) = match &m.color {
        ColorInfo::Untagged => ("untagged", None),
        ColorInfo::Srgb => ("srgb", None),
        ColorInfo::Icc { description } => ("icc", description.as_deref()),
    };
    let ratio = |r: Option<Ratio>| r.map(|r| r.to_string());
    vec![
        Box::new(photo.path.as_str()),
        Box::new(photo.folder().map_or_else(String::new, |f| f.to_string())),
        Box::new(photo.format.name()),
        // SQLite integers are i64; photo files are nowhere near 8 EiB.
        Box::new(i64::try_from(photo.size_bytes).unwrap_or(i64::MAX)),
        Box::new(photo.modified_unix_secs),
        Box::new(photo.modified_nanos),
        Box::new(photo.hash.as_ref().map(ContentHash::as_str)),
        Box::new(time.map(|t| t.local.as_str())),
        Box::new(time.and_then(|t| t.offset.as_deref())),
        Box::new(time.map(|t| source_name(t.source))),
        Box::new(m.camera_make.as_deref()),
        Box::new(m.camera_model.as_deref()),
        Box::new(m.lens_model.as_deref()),
        Box::new(ratio(m.exposure.exposure_time)),
        Box::new(ratio(m.exposure.f_number)),
        Box::new(m.exposure.iso),
        Box::new(ratio(m.exposure.focal_length_mm)),
        Box::new(m.width),
        Box::new(m.height),
        Box::new(m.orientation.value()),
        Box::new(color_kind),
        Box::new(icc),
        match &photo.damage {
            Some(d) => Box::new(d.kind.name()),
            None => Box::new(Null),
        },
        Box::new(photo.damage.as_ref().map(|d| d.detail.as_str())),
    ]
}

/// Rebuilds a [`Photo`] from a row selected with [`COLUMNS`].
pub fn photo_from_row(row: &Row<'_>) -> Result<Photo, CatalogError> {
    let path: String = row.get(0)?;
    let format: String = row.get(2)?;
    let size: i64 = row.get(3)?;
    let hash: Option<String> = row.get(6)?;
    let mut photo = Photo::new(
        RelPath::new(path)?,
        PhotoFormat::from_name(&format).ok_or(CatalogError::UnknownFormat(format))?,
        u64::try_from(size).unwrap_or(0),
        row.get(4)?,
    );
    photo.modified_nanos = row.get(5)?;
    photo.hash = hash.map(ContentHash::try_from).transpose()?;
    photo.meta = meta_from_row(row)?;
    let damage_kind: Option<String> = row.get(22)?;
    if let Some(kind) = damage_kind {
        let kind = DamageKind::from_name(&kind).ok_or_else(|| bad("damage kind", &kind))?;
        let detail: Option<String> = row.get(23)?;
        photo.damage = Some(Damage::new(kind, detail.unwrap_or_default()));
    }
    Ok(photo)
}

fn meta_from_row(row: &Row<'_>) -> Result<PhotoMeta, CatalogError> {
    let ratio = |i: usize| -> Result<Option<Ratio>, CatalogError> {
        let text: Option<String> = row.get(i)?;
        text.map(|t| Ratio::parse(&t).ok_or_else(|| bad("rational", &t)))
            .transpose()
    };
    let local: Option<String> = row.get(7)?;
    let source: Option<String> = row.get(9)?;
    let capture_time = match (local, source) {
        (Some(local), Some(source)) => Some(CaptureTime {
            local,
            offset: row.get(8)?,
            source: source_from_name(&source).ok_or_else(|| bad("date source", &source))?,
        }),
        _ => None,
    };
    let orientation: u32 = row.get(19)?;
    let color_kind: String = row.get(20)?;
    let color = match color_kind.as_str() {
        "untagged" => ColorInfo::Untagged,
        "srgb" => ColorInfo::Srgb,
        "icc" => ColorInfo::Icc {
            description: row.get(21)?,
        },
        other => return Err(bad("colour kind", other)),
    };
    Ok(PhotoMeta {
        capture_time,
        camera_make: row.get(10)?,
        camera_model: row.get(11)?,
        lens_model: row.get(12)?,
        exposure: Exposure {
            exposure_time: ratio(13)?,
            f_number: ratio(14)?,
            iso: row.get(15)?,
            focal_length_mm: ratio(16)?,
        },
        width: row.get(17)?,
        height: row.get(18)?,
        orientation: Orientation::from_exif(orientation)
            .ok_or_else(|| bad("orientation", &orientation.to_string()))?,
        color,
    })
}

fn bad(what: &str, value: &str) -> CatalogError {
    CatalogError::BadValue(format!("{what} {value:?}"))
}

fn source_name(source: DateSource) -> &'static str {
    match source {
        DateSource::ExifOriginal => "exif_original",
        DateSource::ExifDigitized => "exif_digitized",
        DateSource::FileModified => "file_modified",
    }
}

fn source_from_name(name: &str) -> Option<DateSource> {
    [
        DateSource::ExifOriginal,
        DateSource::ExifDigitized,
        DateSource::FileModified,
    ]
    .into_iter()
    .find(|s| source_name(*s) == name)
}
