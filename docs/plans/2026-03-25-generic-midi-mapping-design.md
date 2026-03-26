# Generic MIDI Mapping System

## Problem

The software is hardcoded to the Korg NanoKontrol2 (8 sliders, 8 knobs). This is insufficient for upcoming tour use where multiple lighting engineers need to use their own controllers with custom parameter assignments.

## Design Decisions

- **MIDI Learn + Manual Table** for assigning CCs to params
- **MIDI mappings are fully separate from shader presets** — set up once at soundcheck, persist across preset changes
- **Generic shader slots** — map to "Shader Left Param 1" not "GilmoreAcid.speed", so mappings work across shader switches
- **6 shader params max** per side (left/right)
- **CC + Port name** for multi-device support (not MIDI channel)
- **Auto-connect all MIDI ports** with hot-plug detection (re-scan every ~2s)
- **Stored in `assets/midi_mappings/`** as individual JSON files per preset

## Mappable Parameters (36 total)

**Blending/Mix (3):** LeftRightMix, SmoothingSpeed, FadeToBlack

**Audio (5):** AudioGain, AudioThreshold, AudioAttack, AudioHold, AudioRelease

**Colour (4):** ColourChannel1, ColourChannel2, ColourChannel3, ColourPalette

**Shader Params (12):** ShaderLeftParam(0–5), ShaderRightParam(0–5)

**Shader Mod Amounts (12):** ShaderLeftMod(0–5), ShaderRightMod(0–5)

## Removed

- `korg_nano_kontrol_2` crate dependency
- `Controller` struct
- `mod_amp1`–`mod_amp4` on `AudioInput`
- `midi_cv_phase_amp` on `Model`
- All hardcoded Korg strip/knob match arms

## Architecture

### Module Structure

```
src/midi/
  mod.rs          — MidiManager (port discovery, hot-plug, connections)
  mapping.rs      — MidiMapping, MidiTarget enum, preset load/save
  learn.rs        — MIDI learn state machine
```

### Data Flow

```
MIDI device → midir → raw bytes → parse CC →
  lookup (port_name, cc_number) in mapping table →
    MidiTarget variant → write to target_values array →
      existing smoothing/interpolation → param applied
```

### MidiTarget Enum

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MidiTarget {
    LeftRightMix,
    SmoothingSpeed,
    FadeToBlack,
    AudioGain,
    AudioThreshold,
    AudioAttack,
    AudioHold,
    AudioRelease,
    ColourChannel1,
    ColourChannel2,
    ColourChannel3,
    ColourPalette,
    ShaderLeftParam(u8),   // 0–5
    ShaderRightParam(u8),  // 0–5
    ShaderLeftMod(u8),     // 0–5
    ShaderRightMod(u8),    // 0–5
}
```

### Mapping Entry & Preset

```rust
pub struct MidiMappingEntry {
    pub port_name: String,
    pub cc: u8,
    pub target: MidiTarget,
}

pub struct MidiMappingPreset {
    pub name: String,
    pub entries: Vec<MidiMappingEntry>,
}
```

Runtime lookup via `HashMap<(String, u8), MidiTarget>` for O(1) routing.

CC values 0–127 normalized to 0.0–1.0, then target-specific range mapping applied (e.g. smoothing speed → 0.0008–0.08).

### MIDI Port Manager

```rust
pub struct MidiManager {
    connected_ports: HashMap<String, midir::MidiInputConnection<()>>,
    midi_rx: mpsc::Receiver<MidiMessage>,
    midi_tx: mpsc::Sender<MidiMessage>,
    last_scan: Instant,
}

pub struct MidiMessage {
    pub port_name: String,
    pub cc: u8,
    pub value: u8,
}
```

- Auto-connect all ports on init
- Re-scan every ~2s for hot-plug/unplug
- Each connection callback parses CC (`status & 0xF0 == 0xB0`) and sends through channel
- Main loop drains `midi_rx` and routes through mapping table

### MIDI Learn

```rust
pub enum LearnState {
    Idle,
    Listening(MidiTarget),
    Learned,
}
```

- Click Learn → Listening → wiggle knob → capture (port, cc) → assign → Idle
- Reassigns if CC already mapped elsewhere (one CC = one param)
- Cancel by clicking Learn again or Escape

### MIDI Tab UI

New `LeftPanelTab::Midi` variant.

**Top:** Preset management — dropdown, name field, save/load/delete/new buttons.

**Bottom:** Scrollable mapping table — columns: Param Name | Port | CC | Learn button. Rows grouped by category with headers. Shader param rows show current shader's param name in parentheses (e.g. "Shader Left Param 1 (speed)").
