//! Parser for MadMapper v5 `.mad` binary project files.
//!
//! Extracts fixture layout, pixel counts, and DMX addressing by scanning for
//! known UTF-16BE keys rather than fully parsing the recursive container format.

use std::collections::BTreeSet;
use std::path::Path;

const MAGIC: u32 = 0x0BAD_BABE;
const TYPE_INT: u32 = 0x0000_0002;
const TYPE_STRING: u32 = 0x0000_000A;
const TYPE_POINT2D: u32 = 0x0000_001A;

/// Search window (bytes) backward from an `artnetUniverse` key to find the
/// remaining fixture fields. Fixture blocks are ~4900 bytes apart.
const FIXTURE_SEARCH_WINDOW: usize = 5000;

#[derive(Clone, Debug)]
pub struct MadProject {
    pub fixtures: Vec<Fixture>,
}

#[derive(Clone, Debug)]
pub struct Fixture {
    pub name: String,
    pub product: String,
    pub universe: u16,
    pub start_channel: u16,
    pub pixel_count: usize,
    pub channels_per_pixel: u8,
    pub position: [f64; 2],
}

impl MadProject {
    pub fn total_pixels(&self) -> usize {
        self.fixtures.iter().map(|f| f.pixel_count).sum()
    }

    pub fn universe_count(&self) -> usize {
        let universes: BTreeSet<u16> = self.fixtures.iter().map(|f| f.universe).collect();
        universes.len()
    }

    pub fn universe_range(&self) -> (u16, u16) {
        let min = self.fixtures.iter().map(|f| f.universe).min().unwrap_or(0);
        let max = self.fixtures.iter().map(|f| f.universe).max().unwrap_or(0);
        (min, max)
    }

    /// Fixtures sorted by position Y descending (top row first).
    pub fn fixtures_by_row(&self) -> Vec<&Fixture> {
        let mut sorted: Vec<_> = self.fixtures.iter().collect();
        sorted.sort_by(|a, b| {
            b.position[1]
                .partial_cmp(&a.position[1])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }
}

pub fn parse<P: AsRef<Path>>(path: P) -> Result<MadProject, String> {
    let data = std::fs::read(path.as_ref())
        .map_err(|e| format!("Failed to read MadMapper file: {}", e))?;
    parse_bytes(&data)
}

pub fn parse_bytes(data: &[u8]) -> Result<MadProject, String> {
    if data.len() < 4 {
        return Err("File too small to be a MadMapper project".into());
    }
    let magic = read_u32(data, 0);
    if magic != MAGIC {
        return Err(format!(
            "Not a MadMapper file (expected magic 0x{:08X}, got 0x{:08X})",
            MAGIC, magic
        ));
    }

    let universe_key = encode_utf16be("artnetUniverse");
    let start_channel_key = encode_utf16be("startChannel");
    let pixel_mapping_key = encode_utf16be("pixelMapping");
    let position_uv_key = encode_utf16be("positionUv");
    let product_key = encode_utf16be("product");
    let fixture_name_key = encode_utf16be("Fixture-");

    let universe_offsets = find_all(data, &universe_key);
    if universe_offsets.is_empty() {
        return Err("No fixtures found in MadMapper project".into());
    }

    let start_channel_offsets = find_all(data, &start_channel_key);
    let pixel_mapping_offsets = find_all(data, &pixel_mapping_key);
    let position_uv_offsets = find_all(data, &position_uv_key);
    let product_offsets = find_all(data, &product_key);
    let fixture_name_offsets = find_all(data, &fixture_name_key);

    let mut fixtures = Vec::with_capacity(universe_offsets.len());

    for &uni_offset in &universe_offsets {
        let universe = read_int_value(data, uni_offset + universe_key.len())
            .ok_or_else(|| format!("Truncated artnetUniverse at offset {:#x}", uni_offset))?
            as u16;

        let start_channel =
            find_nearest(&start_channel_offsets, uni_offset, FIXTURE_SEARCH_WINDOW)
                .and_then(|off| read_int_value(data, off + start_channel_key.len()))
                .unwrap_or(1) as u16;

        let (pixel_count, channels_per_pixel) =
            find_nearest(&pixel_mapping_offsets, uni_offset, FIXTURE_SEARCH_WINDOW)
                .and_then(|off| {
                    let s = read_string_value(data, off + pixel_mapping_key.len())?;
                    parse_pixel_mapping(&s)
                })
                .unwrap_or((0, 3));

        let position =
            find_nearest(&position_uv_offsets, uni_offset, FIXTURE_SEARCH_WINDOW)
                .and_then(|off| read_point2d_value(data, off + position_uv_key.len()))
                .unwrap_or([0.0, 0.0]);

        let product = find_nearest(&product_offsets, uni_offset, FIXTURE_SEARCH_WINDOW)
            .and_then(|off| read_string_value(data, off + product_key.len()))
            .unwrap_or_default();

        let name = find_nearest(&fixture_name_offsets, uni_offset, FIXTURE_SEARCH_WINDOW)
            .and_then(|off| read_fixture_name(data, off))
            .unwrap_or_default();

        if pixel_count == 0 {
            eprintln!(
                "Skipping fixture at offset {:#x}: no pixelMapping found",
                uni_offset
            );
            continue;
        }

        fixtures.push(Fixture {
            name,
            product,
            universe,
            start_channel,
            pixel_count,
            channels_per_pixel,
            position,
        });
    }

    if fixtures.is_empty() {
        return Err("No valid fixtures found in MadMapper project".into());
    }

    Ok(MadProject { fixtures })
}

// ---------------------------------------------------------------------------
// Binary helpers
// ---------------------------------------------------------------------------

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

fn read_f64(data: &[u8], offset: usize) -> f64 {
    f64::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ])
}

/// Read an int value (type 0x02) starting at the byte right after the key.
/// Layout: type_tag(4) + pad(1) + value(4).
fn read_int_value(data: &[u8], offset: usize) -> Option<u32> {
    if offset + 9 > data.len() {
        return None;
    }
    let type_tag = read_u32(data, offset);
    if type_tag != TYPE_INT {
        return None;
    }
    Some(read_u32(data, offset + 5))
}

/// Read a string value (type 0x0A) starting at the byte right after the key.
/// Layout: type_tag(4) + pad(1) + length(4) + UTF-16BE bytes.
fn read_string_value(data: &[u8], offset: usize) -> Option<String> {
    if offset + 9 > data.len() {
        return None;
    }
    let type_tag = read_u32(data, offset);
    if type_tag != TYPE_STRING {
        return None;
    }
    let byte_len = read_u32(data, offset + 5) as usize;
    let str_start = offset + 9;
    let str_end = str_start + byte_len;
    if str_end > data.len() {
        return None;
    }
    Some(decode_utf16be(&data[str_start..str_end]))
}

/// Read a 2D point value (type 0x1A) starting at the byte right after the key.
/// Layout: type_tag(4) + pad(1) + x(8) + y(8).
fn read_point2d_value(data: &[u8], offset: usize) -> Option<[f64; 2]> {
    if offset + 21 > data.len() {
        return None;
    }
    let type_tag = read_u32(data, offset);
    if type_tag != TYPE_POINT2D {
        return None;
    }
    let x = read_f64(data, offset + 5);
    let y = read_f64(data, offset + 13);
    Some([x, y])
}

/// Read a fixture name starting at a "Fixture-" occurrence in raw bytes.
/// Scans forward in UTF-16BE until a null char or non-printable char.
fn read_fixture_name(data: &[u8], offset: usize) -> Option<String> {
    let mut end = offset;
    while end + 1 < data.len() {
        let hi = data[end];
        let lo = data[end + 1];
        let ch = u16::from_be_bytes([hi, lo]);
        let is_name_char = ch == b'-' as u16
            || (ch >= b'0' as u16 && ch <= b'9' as u16)
            || (ch >= b'A' as u16 && ch <= b'Z' as u16)
            || (ch >= b'a' as u16 && ch <= b'z' as u16);
        if ch == 0 || !is_name_char {
            break;
        }
        end += 2;
    }
    if end == offset {
        return None;
    }
    Some(decode_utf16be(&data[offset..end]))
}

// ---------------------------------------------------------------------------
// UTF-16BE helpers
// ---------------------------------------------------------------------------

fn encode_utf16be(s: &str) -> Vec<u8> {
    s.encode_utf16()
        .flat_map(|ch| ch.to_be_bytes())
        .collect()
}

fn decode_utf16be(data: &[u8]) -> String {
    let chars: Vec<u16> = data
        .chunks_exact(2)
        .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
        .collect();
    String::from_utf16_lossy(&chars)
}

// ---------------------------------------------------------------------------
// Scanning helpers
// ---------------------------------------------------------------------------

fn find_all(data: &[u8], needle: &[u8]) -> Vec<usize> {
    let mut results = Vec::new();
    let mut start = 0;
    while start + needle.len() <= data.len() {
        if let Some(pos) = memchr_find(data, needle, start) {
            results.push(pos);
            start = pos + 1;
        } else {
            break;
        }
    }
    results
}

fn memchr_find(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    if needle.is_empty() || start + needle.len() > haystack.len() {
        return None;
    }
    haystack[start..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|pos| start + pos)
}

/// Find the nearest offset in `offsets` within `window` bytes of `anchor` (either direction).
fn find_nearest(offsets: &[usize], anchor: usize, window: usize) -> Option<usize> {
    let min = anchor.saturating_sub(window);
    let max = anchor + window;
    offsets
        .iter()
        .copied()
        .filter(|&off| off >= min && off <= max && off != anchor)
        .min_by_key(|&off| (off as isize - anchor as isize).unsigned_abs())
}

// ---------------------------------------------------------------------------
// Pixel mapping parsing
// ---------------------------------------------------------------------------

fn parse_pixel_mapping(mapping: &str) -> Option<(usize, u8)> {
    let entries: Vec<u32> = mapping
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    let pixel_count = entries.len();
    if pixel_count < 2 {
        return if pixel_count == 1 {
            Some((1, 3))
        } else {
            None
        };
    }
    let step = entries[1].saturating_sub(entries[0]);
    let channels_per_pixel = if step > 0 && step <= 16 { step as u8 } else { 3 };
    Some((pixel_count, channels_per_pixel))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_magic_bytes_rejects_bad_file() {
        let data = vec![0x00, 0x00, 0x00, 0x00];
        let result = parse_bytes(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not a MadMapper file"));
    }

    #[test]
    fn parse_magic_bytes_rejects_too_small() {
        let data = vec![0x0B, 0xAD];
        let result = parse_bytes(&data);
        assert!(result.is_err());
    }

    #[test]
    fn pixel_count_from_mapping() {
        assert_eq!(parse_pixel_mapping("1 4 7 10"), Some((4, 3)));
        assert_eq!(parse_pixel_mapping("1 5 9 13"), Some((4, 4)));
        assert_eq!(parse_pixel_mapping("1"), Some((1, 3)));
        assert_eq!(parse_pixel_mapping(""), None);
    }

    #[test]
    fn parse_known_project() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/map_mapper_projects/SJ02-JOSH_COHEN-MM5-01.mad"
        );
        let project = parse(path).expect("Failed to parse known .mad file");

        assert_eq!(project.fixtures.len(), 16);
        assert_eq!(project.total_pixels(), 6400);

        for fixture in &project.fixtures {
            assert_eq!(fixture.pixel_count, 400);
            assert_eq!(fixture.channels_per_pixel, 3);
            assert_eq!(fixture.start_channel, 1);
            assert_eq!(fixture.product, "400 Wide");
            assert!(fixture.name.starts_with("Fixture-"));
        }

        let (min_u, max_u) = project.universe_range();
        assert_eq!(min_u, 0);
        assert_eq!(max_u, 43);
    }

    #[test]
    fn position_extraction() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/map_mapper_projects/SJ02-JOSH_COHEN-MM5-01.mad"
        );
        let project = parse(path).expect("Failed to parse known .mad file");

        // Find Fixture-Line-2 and check its UV position.
        let f2 = project
            .fixtures
            .iter()
            .find(|f| f.name == "Fixture-Line-2")
            .expect("Fixture-Line-2 not found");

        assert!((f2.position[0] - (-1.9996)).abs() < 0.01);
        assert!((f2.position[1] - 0.8164).abs() < 0.01);
    }

    #[test]
    fn fixtures_by_row_descending_y() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/map_mapper_projects/SJ02-JOSH_COHEN-MM5-01.mad"
        );
        let project = parse(path).expect("Failed to parse known .mad file");
        let by_row = project.fixtures_by_row();
        for pair in by_row.windows(2) {
            assert!(pair[0].position[1] >= pair[1].position[1]);
        }
    }

    #[test]
    fn parse_mm6_project() {
        // NOTE: The MM6 file only contains 15 fixtures (Fixture-Line-16 is
        // missing from the binary). See issue #54. Should be 16 once the
        // MadMapper project is re-exported.
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/map_mapper_projects/SJ02-JOSH_COHEN-MM6-01.mad"
        );
        let project = parse(path).expect("Failed to parse MM6 .mad file");

        assert!(
            project.fixtures.len() >= 15,
            "Expected at least 15 fixtures, got {}",
            project.fixtures.len()
        );

        for fixture in &project.fixtures {
            assert_eq!(fixture.channels_per_pixel, 3);
            assert_eq!(fixture.start_channel, 1);
            assert!(fixture.pixel_count > 0);
            assert!(fixture.name.starts_with("Fixture-"));
        }
    }
}
