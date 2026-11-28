//! Probing one file: check its content, read its metadata, record damage.
//!
//! Probing never fails as a whole. Whatever goes wrong with one file becomes a
//! [`Damage`] record on that photo (FR-LIB-10) and the scan carries on.

mod detect;
mod exif_map;
mod exif_read;
mod header;
mod icc;

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

use camino::Utf8Path;
use leoht_core::{
    CaptureTime, ColorInfo, Damage, DamageKind, DateSource, Photo, PhotoFormat, PhotoMeta,
    utc_timestamp,
};

pub use detect::matches_format;

use crate::FoundFile;

/// Images declaring more pixels than this are recorded as [`DamageKind::TooLarge`]:
/// a safety limit against hostile headers, well above any camera sold today.
pub const MAX_PIXELS: u64 = 400_000_000;

/// Version of the probing rules. Bump it whenever probing would produce
/// different results for the same file, so incremental rescans re-probe
/// everything instead of trusting stale catalog rows.
pub const PROBE_VERSION: u32 = 1;

/// Probes `found` and returns the complete [`Photo`] (metadata and any damage).
/// The capture time always ends up set: when the file has no usable EXIF date
/// it falls back to the modification time, marked [`DateSource::FileModified`].
#[must_use]
pub fn probe_file(found: &FoundFile) -> Photo {
    let mut photo = Photo::new(
        found.rel.clone(),
        found.format,
        found.size_bytes,
        found.modified_unix_secs,
    );
    photo.modified_nanos = found.modified_nanos;
    if let Err(damage) = probe_into(&found.abs, found.format, found.size_bytes, &mut photo.meta) {
        tracing::debug!(path = %found.abs, kind = damage.kind.name(), "damaged file");
        photo.damage = Some(damage);
    }
    if photo.meta.capture_time.is_none() {
        photo.meta.capture_time = Some(CaptureTime {
            local: utc_timestamp(found.modified_unix_secs),
            offset: None,
            source: DateSource::FileModified,
        });
    }
    photo
}

fn probe_into(
    path: &Utf8Path,
    format: PhotoFormat,
    size: u64,
    meta: &mut PhotoMeta,
) -> Result<(), Damage> {
    if size == 0 {
        return Err(Damage::new(DamageKind::Empty, "the file is empty"));
    }
    let unreadable = |e: std::io::Error| Damage::new(DamageKind::Unreadable, e.to_string());
    let mut file = File::open(path).map_err(unreadable)?;
    let mut head = Vec::with_capacity(detect::HEAD_LEN);
    (&mut file)
        .take(detect::HEAD_LEN as u64)
        .read_to_end(&mut head)
        .map_err(unreadable)?;
    if !matches_format(&head, format) {
        return Err(Damage::new(
            DamageKind::ContentMismatch,
            format!("the content is not a valid {} file", format.name()),
        ));
    }
    if let Some(exif) = exif_read::read_exif(&mut file, &head) {
        exif_map::apply_exif(&exif, meta);
    }
    let header = match format {
        PhotoFormat::Jpeg => Some(header::jpeg_header(&mut file)),
        PhotoFormat::Png => {
            file.seek(SeekFrom::Start(0)).map_err(unreadable)?;
            Some(header::png_header(BufReader::new(&mut file)))
        }
        // Detection and EXIF only until a decoder exists (ADR-0004, SQ-1).
        PhotoFormat::Heic | PhotoFormat::Raw | PhotoFormat::Tiff | PhotoFormat::Webp => None,
    };
    if let Some(header) = header {
        let info = header.map_err(|msg| Damage::new(DamageKind::UnreadableHeader, msg))?;
        if let Some((w, h)) = info.size {
            (meta.width, meta.height) = (Some(w), Some(h));
        }
        // An embedded profile is authoritative; a bare sRGB declaration is next.
        if let Some(icc) = info.icc {
            meta.color = ColorInfo::Icc {
                description: icc::description(&icc),
            };
        } else if info.srgb {
            meta.color = ColorInfo::Srgb;
        }
    }
    check_size(meta)
}

fn check_size(meta: &PhotoMeta) -> Result<(), Damage> {
    if let (Some(w), Some(h)) = (meta.width, meta.height) {
        let pixels = u64::from(w) * u64::from(h);
        if pixels > MAX_PIXELS {
            return Err(Damage::new(
                DamageKind::TooLarge,
                format!("{w} x {h} pixels is over the {MAX_PIXELS}-pixel safety limit"),
            ));
        }
    }
    Ok(())
}
