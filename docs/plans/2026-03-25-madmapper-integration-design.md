# MadMapper Integration Design

## Problem

DMX universe assignments and LED fixture positions are hardcoded. The lighting
designer (Jobe) defines the physical layout in MadMapper, then communicates
universes/channels manually. Every change requires a code update. MadMapper
should be the single source of truth.

## Goals

1. Parse MadMapper `.mad` project files in pure Rust.
2. Derive fixture layout, pixel counts, and DMX addressing from the parsed data.
3. Existing manual UI configuration continues to work when no `.mad` file is loaded.
4. User can load/remove a `.mad` file via native file dialog. Path persists across launches.

## Non-Goals

- Live OSCQuery integration with running MadMapper instance.
- Editing MadMapper projects from within the app.
- Supporting MadMapper 6 `.madworkspace` format (only v5 `.mad`).

---

## 1. MadMapper Binary Format

### Overview

`.mad` files use a proprietary binary format:

- **Magic:** `0x0BADBABE` (4 bytes)
- **Encoding:** Keys and string values are UTF-16 Big-Endian
- **Structure:** Nested typed key-value pairs

### Value encoding (after each key's UTF-16BE bytes)

All values follow the same envelope: `type_tag(4 bytes) + pad(1 byte) + payload`.

| Type tag (BE u32) | Payload | Description |
|---|---|---|
| `0x00000001` | 1 byte (0 or 1) | Boolean |
| `0x00000002` | 4-byte BE u32 | Integer |
| `0x00000003` | 2-byte BE i16 | Short integer |
| `0x00000006` | 8-byte BE f64 | Float (IEEE 754 double) |
| `0x0000000a` | 4-byte BE u32 length + UTF-16BE bytes | String |
| `0x0000001a` | 8-byte BE f64 X + 8-byte BE f64 Y | 2D point |

### Fields we extract per fixture

| Key (UTF-16BE) | Type | Meaning |
|---|---|---|
| `artnetUniverse` | int | Starting Art-Net universe for this fixture |
| `startChannel` | int | Starting DMX channel within the universe |
| `pixelMapping` | string | Space-separated DMX channel offsets (e.g. `"1 4 7 10 ..."`) |
| `positionUv` | 2D point | XY position in MadMapper's UV coordinate space |
| `name` (nearest `Fixture-*`) | string | Fixture instance name (e.g. `"Fixture-Line-2"`) |
| `product` | string | Fixture product/type (e.g. `"400 Wide"`) |
| `width` | int | Pixel width of the fixture |

### Fixture block structure

Fixtures appear as repeating blocks (~4900 bytes apart). Within each block the
key order is stable:

```
... → startChannel → ... → artnetUniverse → visualId → uvFlip →
fixtureline → surfaceId → sliders → scale → rotation → positionUv →
opacity → [fixture name string] → ... → pixelMapping → ...
```

### Parsing strategy

We don't need a full recursive parser. Instead:

1. Scan for all `artnetUniverse` key occurrences (UTF-16BE byte pattern).
2. For each occurrence, read the int value (type `0x02` at offset, value at +5..+9).
3. From the same fixture block, find `startChannel`, `pixelMapping`, `positionUv`,
   `product` by scanning forward/backward within a bounded window (~5000 bytes).
4. Extract fixture name by searching for the `Fixture-Line-\d+` pattern in the
   surrounding UTF-16BE text.
5. Derive pixel count from the number of space-separated entries in `pixelMapping`.
6. Derive channels-per-pixel from the step between first two `pixelMapping` entries
   (3 for RGB, 4 for RGBW, etc.).

This approach is robust against format changes in unrelated parts of the file
and avoids needing to understand the full recursive container format.

---

## 2. Data Structures

### `mad_mapper.rs` — new module

```rust
/// A parsed MadMapper project.
#[derive(Clone, Debug)]
pub struct MadProject {
    pub fixtures: Vec<Fixture>,
}

/// A single LED fixture parsed from the .mad file.
#[derive(Clone, Debug)]
pub struct Fixture {
    /// Fixture instance name (e.g. "Fixture-Line-2").
    pub name: String,
    /// Fixture product/type (e.g. "400 Wide").
    pub product: String,
    /// Art-Net universe this fixture starts on.
    pub universe: u16,
    /// DMX start channel within the first universe.
    pub start_channel: u16,
    /// Number of pixels in this fixture.
    pub pixel_count: usize,
    /// DMX channels per pixel (3 for RGB, 4 for RGBW).
    pub channels_per_pixel: u8,
    /// Position in MadMapper's UV coordinate space.
    pub position: [f64; 2],
}
```

### Derived layout info

```rust
impl MadProject {
    /// Total pixel count across all fixtures.
    pub fn total_pixels(&self) -> usize { ... }

    /// Total number of distinct universes used.
    pub fn universe_count(&self) -> usize { ... }

    /// Universe range as (min, max) inclusive.
    pub fn universe_range(&self) -> (u16, u16) { ... }

    /// Fixtures sorted by position Y descending (top row first),
    /// matching the existing row iteration order.
    pub fn fixtures_by_row(&self) -> Vec<&Fixture> { ... }
}
```

---

## 3. Config Changes

### `conf.rs`

Add one field to `Config`:

```rust
pub struct Config {
    // ... existing fields ...

    /// Optional path to a MadMapper .mad project file.
    /// When Some, the layout and DMX addressing are derived from this file
    /// instead of the manual `led_layout` and `led_start_universe` fields.
    #[serde(default)]
    pub madmapper_project_path: Option<String>,
}
```

This is `Option<String>` (not `PathBuf`) for simple JSON serialization. The
manual `led_layout` and `led_start_universe` fields remain in config and
continue to be saved — they're just ignored while a MadMapper project is active.

---

## 4. Layout Source Abstraction

### `layout.rs` changes

Add a trait or enum that both layout sources produce:

```rust
/// Everything the LED worker needs to know about the physical layout.
pub struct ResolvedLayout {
    /// One entry per LED in output order. Position in shader coordinate space.
    pub shader_inputs: Vec<CachedLedShaderInput>,
    /// How to pack LED RGB data into sACN universes.
    pub dmx_map: DmxMap,
}

/// DMX addressing strategy.
pub enum DmxMap {
    /// Sequential packing starting at a given universe (current behavior).
    Sequential { start_universe: u16 },
    /// Per-fixture packing with explicit universe assignments.
    PerFixture(Vec<FixtureDmxEntry>),
}

pub struct FixtureDmxEntry {
    /// Index of the first LED in the flat output array that belongs to this fixture.
    pub led_offset: usize,
    /// Number of LEDs in this fixture.
    pub led_count: usize,
    /// Art-Net universe this fixture starts on.
    pub start_universe: u16,
    /// DMX start channel (typically 1).
    pub start_channel: u16,
}
```

### Building `ResolvedLayout` from MadMapper data

```rust
pub fn resolve_from_mad_project(project: &MadProject) -> ResolvedLayout {
    // 1. Sort fixtures by Y descending (top row first) to match
    //    existing iteration order (top-left → bottom-right).
    //
    // 2. For each fixture, generate `pixel_count` LED positions:
    //    - Y coordinate: derived from fixture's UV Y position,
    //      normalized to shader height range [0, 1].
    //    - X coordinate: spread pixels evenly across the fixture width,
    //      centered on the fixture's UV X position,
    //      normalized to shader X range [-1, 1].
    //    - Z coordinate: 0 (all fixtures coplanar).
    //
    // 3. Build the DmxMap::PerFixture entries, tracking the cumulative
    //    led_offset as fixtures are appended.
    //
    // 4. Build CachedLedShaderInput for each LED with position + Light::Led.
}
```

### Building `ResolvedLayout` from manual `LedLayout` (existing path)

```rust
pub fn resolve_from_manual(led_layout: &LedLayout, start_universe: u16) -> ResolvedLayout {
    // Wraps the existing `rebuild_led_shader_inputs` + DmxMap::Sequential.
}
```

---

## 5. DMX Payload Building

### `build_led_sacn_payloads` changes

Replace the single function with a dispatcher:

```rust
fn build_sacn_payloads(
    dmx_map: &DmxMap,
    led_outputs: &[LinSrgb],
) -> Vec<(u16, Vec<u8>)> {
    match dmx_map {
        DmxMap::Sequential { start_universe } => {
            // Existing logic: linear pack into sequential universes.
            build_sequential_payloads(*start_universe, led_outputs)
        }
        DmxMap::PerFixture(entries) => {
            // For each fixture entry, slice led_outputs[offset..offset+count],
            // pack into universes starting at entry.start_universe.
            // Collect all (universe, payload) pairs, merging any that share
            // a universe number (shouldn't happen with proper MadMapper config).
            build_per_fixture_payloads(entries, led_outputs)
        }
    }
}
```

`build_sequential_payloads` is exactly the existing `build_led_sacn_payloads`
renamed. `build_per_fixture_payloads` calls the same packing logic per-fixture,
using each entry's `start_universe`.

---

## 6. LED Worker Changes

### `LedWorkerConfig`

Add the resolved layout:

```rust
struct LedWorkerConfig {
    // ... existing fields ...
    // Replace:
    //   led_start_universe: u16,
    //   led_layout: conf::LedLayout,
    // With:
    resolved_layout: ResolvedLayout,
}
```

Actually — to minimize churn, keep `led_layout` and `led_start_universe` for
the manual path and add the `DmxMap` alongside. The `ResolvedLayout` gets built
in the main thread and passed to the worker via the existing config snapshot.

### Change detection

Currently `sync_led_worker_buffers` rebuilds when `led_layout` changes. Add a
check: if the `DmxMap` variant changed (Sequential ↔ PerFixture), or if the
fixture data changed, also rebuild.

---

## 7. GUI Changes

### Output sidebar (`set_output_sidebar_widgets`)

Add a "MadMapper Project" section above the existing "Universes" / "LED Layout"
controls:

```
┌─────────────────────────────┐
│ MadMapper Project           │
│ ┌─────────────────────────┐ │
│ │ [Load .mad File]        │ │  ← native file dialog (rfd crate)
│ └─────────────────────────┘ │
│                             │  (when no file loaded, shows manual controls below)
│                             │
│ ── OR when file loaded: ──  │
│                             │
│ SJ02-JOSH_COHEN-MM5-01.mad │  ← filename display
│ 16 fixtures, 6400 pixels    │
│ 46 universes (U0-U43)       │
│ ┌─────────────────────────┐ │
│ │ [Remove]                │ │  ← clears path, returns to manual mode
│ └─────────────────────────┘ │
│                             │
│ ─── Manual controls ──────  │  (hidden when .mad loaded)
│ Universes                   │
│ [Start Universe: 1       ]  │
│ LED Layout                  │
│ [LEDs / Metre:  100      ]  │
│ [Row Length (m): 6        ]  │
│ [Rows:          7         ]  │
│ 4,200 LEDs across 25 uni... │
└─────────────────────────────┘
```

### New dependency

```toml
rfd = "0.15"   # native file dialog — used only for the Load button
```

Use `rfd::FileDialog::new().add_filter("MadMapper", &["mad"]).pick_file()` on
a background thread to avoid blocking the UI. Since nannou_conrod is
immediate-mode, trigger the dialog on button click and poll for the result on
subsequent frames.

### Widget IDs to add

```rust
madmapper_section_text,
madmapper_load_button,
madmapper_filename_text,
madmapper_stats_text,
madmapper_remove_button,
```

---

## 8. Main Thread Integration

### Startup

```rust
// In model():
let mad_project = config.madmapper_project_path.as_ref().and_then(|path| {
    match mad_mapper::parse(path) {
        Ok(project) => Some(project),
        Err(e) => {
            eprintln!("Failed to parse MadMapper project: {}", e);
            None
        }
    }
});
```

### Model additions

```rust
struct Model {
    // ... existing fields ...
    mad_project: Option<mad_mapper::MadProject>,
    pending_file_dialog: Option</* receiver for rfd result */>,
}
```

### Layout resolution (called when config changes)

```rust
fn resolve_layout(config: &Config, mad_project: &Option<MadProject>) -> ResolvedLayout {
    if let Some(project) = mad_project {
        layout::resolve_from_mad_project(project)
    } else {
        layout::resolve_from_manual(&config.led_layout, config.led_start_universe)
    }
}
```

---

## 9. Error Handling

- **Bad magic bytes:** Return `Err("Not a MadMapper file (bad magic)")`.
- **No fixtures found:** Return `Err("No fixtures found in MadMapper project")`.
- **Truncated data:** Return `Err` with offset info. Don't panic.
- **Missing fields:** Skip fixtures that lack `artnetUniverse` or `pixelMapping`.
  Log a warning, include the partial fixture count in the error.
- **File not found on startup:** Clear `madmapper_project_path` in config,
  log warning, fall back to manual mode.

All errors are `String`-based (`Result<MadProject, String>`) — no need for a
custom error enum given the narrow scope.

---

## 10. File Structure

```
cohen_gig/src/
├── mad_mapper.rs        ← NEW: .mad binary parser + MadProject types
├── layout.rs            ← MODIFIED: add ResolvedLayout, resolve_from_*
├── conf.rs              ← MODIFIED: add madmapper_project_path
├── gui.rs               ← MODIFIED: add MadMapper section to output sidebar
├── main.rs              ← MODIFIED: wire up MadProject, layout resolution, payload building
└── ...
```

### New dependency in `Cargo.toml`

```toml
rfd = "0.15"
```

---

## 11. Testing

### `mad_mapper.rs` unit tests

- `parse_magic_bytes` — rejects files without `0x0BADBABE`.
- `parse_known_project` — parse the checked-in test file
  `assets/map_mapper_projects/SJ02-JOSH_COHEN-MM5-01.mad`, assert 16 fixtures,
  400 pixels each, expected universe assignments.
- `pixel_count_from_mapping` — parse `"1 4 7 10"` → 4 pixels, step 3 (RGB).
- `position_extraction` — verify UV coordinates match known values from
  reverse-engineering (e.g. Fixture-Line-2 at UV ≈ (-2.0, 0.816)).

### `main.rs` payload tests

- Existing `build_led_sacn_payloads` tests remain (renamed to `build_sequential_payloads`).
- New: `build_per_fixture_payloads` test — 2 fixtures with different universes,
  verify each fixture's pixels land in the correct universe range.

### Integration test

- Load the test `.mad` file, resolve layout, verify `led_shader_inputs` count
  matches total pixel count (6400), verify `DmxMap::PerFixture` entries have
  correct universe assignments.

---

## 12. Implementation Order

1. **`mad_mapper.rs`** — parser + types + unit tests against the real `.mad` file.
2. **`conf.rs`** — add `madmapper_project_path` field.
3. **`layout.rs`** — add `ResolvedLayout`, `DmxMap`, `resolve_from_manual`,
   `resolve_from_mad_project`.
4. **`main.rs`** — wire up `ResolvedLayout` in `LedWorkerConfig`, update
   `build_sacn_payloads`, update `sync_led_worker_buffers`.
5. **`gui.rs`** — add Load/Remove buttons, fixture summary, conditional
   visibility of manual controls.
6. **`Cargo.toml`** — add `rfd` dependency.
