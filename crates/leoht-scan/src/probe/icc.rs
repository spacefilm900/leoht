//! Reading the description of an embedded ICC colour profile.
//!
//! Layout from the public ICC specification (ICC.1): a 128-byte header, a tag
//! count, then 12-byte tag entries (signature, offset, size). The `desc` tag is
//! either `textDescriptionType` (ICC v2: ASCII) or `multiLocalizedUnicodeType`
//! (`mluc`, ICC v4: UTF-16BE records). Only the first record is used.

/// Longest description we keep; real ones are a few dozen characters.
const MAX_DESCRIPTION: usize = 256;

/// The profile description (for example `"Display P3"`), if it can be read.
/// Never fails: an odd or hostile profile just yields `None`.
#[must_use]
pub fn description(profile: &[u8]) -> Option<String> {
    let count = be_u32(profile, 128)? as usize;
    let tag = (0..count.min(1024))
        .map(|i| 132 + i * 12)
        .find(|&at| profile.get(at..at + 4) == Some(b"desc"))?;
    let offset = be_u32(profile, tag + 4)? as usize;
    let size = be_u32(profile, tag + 8)? as usize;
    let data = profile.get(offset..offset.checked_add(size)?)?;
    let text = match data.get(..4)? {
        b"desc" => ascii_desc(data)?,
        b"mluc" => mluc_desc(data)?,
        _ => return None,
    };
    let text = text.trim_matches(char::from(0)).trim();
    (!text.is_empty()).then(|| text.chars().take(MAX_DESCRIPTION).collect())
}

fn be_u32(b: &[u8], at: usize) -> Option<u32> {
    let s = b.get(at..at + 4)?;
    Some(u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
}

/// `textDescriptionType`: signature, reserved, ASCII count (incl. NUL), ASCII.
fn ascii_desc(data: &[u8]) -> Option<String> {
    let len = be_u32(data, 8)? as usize;
    let bytes = data.get(12..12 + len.min(data.len().saturating_sub(12)))?;
    Some(bytes.iter().map(|&b| char::from(b)).collect())
}

/// `multiLocalizedUnicodeType`: signature, reserved, record count, record size,
/// then records of language, country, byte length and offset (from the tag start).
fn mluc_desc(data: &[u8]) -> Option<String> {
    if be_u32(data, 8)? == 0 {
        return None;
    }
    let len = be_u32(data, 20)? as usize;
    let offset = be_u32(data, 24)? as usize;
    let utf16 = data.get(offset..offset.checked_add(len)?)?;
    let units: Vec<u16> = utf16
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_be_bytes(*c))
        .collect();
    Some(String::from_utf16_lossy(&units))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal profile with one `desc` tag holding `tag_data`.
    fn profile_with_desc(tag_data: &[u8]) -> Vec<u8> {
        let mut p = vec![0u8; 128];
        p.extend_from_slice(&1u32.to_be_bytes());
        p.extend_from_slice(b"desc");
        p.extend_from_slice(&144u32.to_be_bytes());
        p.extend_from_slice(&u32::try_from(tag_data.len()).unwrap().to_be_bytes());
        p.extend_from_slice(tag_data);
        p
    }

    #[test]
    fn reads_v2_ascii_description() {
        let mut tag = b"desc\0\0\0\0".to_vec();
        tag.extend_from_slice(&5u32.to_be_bytes());
        tag.extend_from_slice(b"sRGB\0");
        assert_eq!(
            description(&profile_with_desc(&tag)).as_deref(),
            Some("sRGB")
        );
    }

    #[test]
    fn reads_v4_mluc_description() {
        let text: Vec<u8> = "Display P3"
            .encode_utf16()
            .flat_map(u16::to_be_bytes)
            .collect();
        let mut tag = b"mluc\0\0\0\0".to_vec();
        tag.extend_from_slice(&1u32.to_be_bytes()); // one record
        tag.extend_from_slice(&12u32.to_be_bytes()); // record size
        tag.extend_from_slice(b"enUS");
        tag.extend_from_slice(&u32::try_from(text.len()).unwrap().to_be_bytes());
        tag.extend_from_slice(&28u32.to_be_bytes()); // string offset from tag start
        tag.extend_from_slice(&text);
        assert_eq!(
            description(&profile_with_desc(&tag)).as_deref(),
            Some("Display P3")
        );
    }

    #[test]
    fn garbage_yields_none_without_panicking() {
        assert_eq!(description(&[]), None);
        assert_eq!(description(&[0xFF; 200]), None);
        let mut huge = profile_with_desc(b"desc");
        huge[136..140].copy_from_slice(&u32::MAX.to_be_bytes()); // offset past the end
        assert_eq!(description(&huge), None);
    }
}
