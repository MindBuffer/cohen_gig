# MIDI Mapping Implementation Plan

Reference: [Design Doc](./2026-03-25-generic-midi-mapping-design.md)

---

## Phase 1: MIDI Module (no existing code changes)

### Task 1 — `midi/mapping.rs`: Types, serialization, load/save
**Create** `cohen_gig/src/midi/mapping.rs`

- `MidiTarget` enum with all 36 variants (see design doc)
  - Derive `Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize`
  - Add `MidiTarget::all() -> Vec<MidiTarget>` returning all 36 targets in display order
  - Add `MidiTarget::label(&self) -> &'static str` for GUI display names
  - Add `MidiTarget::category(&self) -> &'static str` returning "Blending/Mix", "Audio", "Colour", "Shader Left", "Shader Right"
- `MidiMappingEntry { port_name: String, cc: u8, target: MidiTarget }`
  - Derive `Clone, Debug, Serialize, Deserialize`
- `MidiMappingPreset { name: String, entries: Vec<MidiMappingEntry> }`
  - Derive `Clone, Debug, Serialize, Deserialize`
  - `fn lookup_table(&self) -> HashMap<(String, u8), MidiTarget>` — builds the O(1) runtime lookup
  - `fn save(&self, dir: &Path) -> io::Result<()>` — saves as `dir/{name}.json`
  - `fn load(path: &Path) -> io::Result<Self>` — loads from JSON file
  - `fn list_presets(dir: &Path) -> Vec<String>` — lists preset names from `assets/midi_mappings/`
- `MidiMapping` — runtime state wrapping a preset + its computed lookup table
  - `preset: MidiMappingPreset`
  - `lookup: HashMap<(String, u8), MidiTarget>`
  - `fn set_preset(&mut self, preset: MidiMappingPreset)` — rebuilds lookup
  - `fn assign(&mut self, port_name: String, cc: u8, target: MidiTarget)` — add/replace entry, rebuild lookup
  - `fn unassign(&mut self, target: MidiTarget)` — remove entry for target
  - `fn target_for(&self, port_name: &str, cc: u8) -> Option<MidiTarget>`

**Verify**: `cargo check` passes.

### Task 2 — `midi/mod.rs`: MidiManager with auto-connect and hot-plug
**Create** `cohen_gig/src/midi/mod.rs`

- `pub mod mapping;`
- `pub mod learn;`
- `MidiMessage { port_name: String, cc: u8, value: u8 }`
- `MidiManager` struct:
  - `midi_input: midir::MidiInput` (kept for re-scanning)
  - `connections: Vec<(String, midir::MidiInputConnection<()>)>` — active connections by port name
  - `tx: mpsc::Sender<MidiMessage>`
  - `rx: mpsc::Receiver<MidiMessage>`
  - `last_scan: Instant`
  - `connected_port_names: HashSet<String>` — for detecting changes
- `MidiManager::new() -> Self` — creates channel, does initial port scan + connect all
- `MidiManager::poll(&mut self)` — re-scans ports every ~2s, connects new ones, drops stale
- `MidiManager::drain(&self) -> impl Iterator<Item = MidiMessage>` — `self.rx.try_iter()`
- CC parsing in the connection callback: check `status & 0xF0 == 0xB0`, extract cc and value bytes
- Print connected/disconnected port names to stdout for debugging

**Verify**: `cargo check` passes.

### Task 3 — `midi/learn.rs`: MIDI learn state machine
**Create** `cohen_gig/src/midi/learn.rs`

- `LearnState` enum: `Idle`, `Listening(MidiTarget)`, `Learned(Instant)` (instant for brief flash)
- `LearnState::start(target: MidiTarget) -> Self`
- `LearnState::cancel() -> Self`
- `LearnState::receive(msg: &MidiMessage) -> Option<(String, u8)>` — returns port+cc to assign
- `LearnState::update(&mut self)` — transitions Learned → Idle after ~0.5s
- `LearnState::is_listening_for(&self, target: MidiTarget) -> bool` — for GUI highlight

**Verify**: `cargo check` passes.

---

## Phase 2: Decouple from Korg

### Task 4 — Remove `korg_nano_kontrol_2` dependency, re-define Button types
**Modify** `shader_shared/Cargo.toml`: remove `korg_nano_kontrol_2`
**Modify** `shader_shared/src/lib.rs`:
- Remove `use korg_nano_kontrol_2::{...};`
- Define the korg types locally (only what `Button` and `ButtonState` need):
  - `ButtonRow` (Top, Bottom)
  - `Strip` (A..H)
  - `State` (On, Off)
  - `TrackButton` (Previous, Next)
  - `MarkerButton` (Set, Left, Right)
  - `Transport` (Rew, Ff, Stop, Play, Rec)
  - Derive necessary traits: Copy, Clone, Debug, PartialEq, Eq, Hash
- Keep `Button`, `ButtonState`, `Uniforms` as-is for now (buttons still work, just local types)

**Modify** `cohen_gig/Cargo.toml`: remove `korg_nano_kontrol_2` dependency
**Modify** `cohen_gig/src/main.rs`:
- Remove `use korg_nano_kontrol_2 as korg;`
- Update `ButtonState` to use `shader_shared::State` instead of `korg::State`
- Update `update_korg_button` to use local types

**Verify**: `cargo check` passes.

### Task 5 — Replace Controller + hardcoded MIDI with MidiManager
**Modify** `cohen_gig/src/main.rs`:
- Add `mod midi;`
- Remove `Controller` struct entirely
- Remove `midi_cv_phase_amp: f32` from `Model`
- Remove `mod_amp1`-`mod_amp4` from `AudioInput` (modify `audio_input.rs`)
- Remove `target_slider_values`, `target_pot_values` from `Model`
- Add `MidiTargetState` struct:
  ```rust
  struct MidiTargetState {
      target: f32,    // 0-1 normalized from last CC
      smoothed: f32,  // current smoothed value
      active: bool,   // true = MIDI is driving this param, false = GUI overrode it
  }
  ```
  - `active` set to `true` when a CC arrives
  - `active` set to `false` when the user moves the corresponding GUI slider
  - Smoothing only applied to `active` targets
  - When the next CC arrives for an inactive target, `active` becomes `true` again and smoothing resumes from the current actual param value
- Add to `Model`:
  - `midi_manager: midi::MidiManager`
  - `midi_mapping: midi::mapping::MidiMapping`
  - `midi_learn: midi::learn::LearnState`
  - `midi_values: HashMap<midi::mapping::MidiTarget, MidiTargetState>` — per-target state
  - `colour_channels: [f32; 3]` — replaces pot6/7/8 (fed into Uniforms)
- In `model()` fn:
  - Create `MidiManager::new()` instead of hardcoded Korg connection
  - Load MIDI mapping preset from `assets/midi_mappings/` (or default empty)
  - Initialize midi_targets and midi_smoothed as empty HashMaps
  - Remove old controller, target_slider_values, target_pot_values init

**Verify**: `cargo check` passes (will have many compiler errors from removed fields — that's expected, fixed in next tasks).

### Task 6 — MIDI routing: CC → mapping → smoothing → apply (with GUI override)
**Modify** `cohen_gig/src/main.rs` `update()` fn:
- Replace the entire `for event in model.midi_rx.try_iter()` block with:
  ```rust
  model.midi_manager.poll(); // hot-plug scan
  for msg in model.midi_manager.drain() {
      // Check learn mode first
      if let LearnState::Listening(target) = &model.midi_learn {
          model.midi_mapping.assign(msg.port_name.clone(), msg.cc, *target);
          model.midi_learn = LearnState::Learned(Instant::now());
          continue;
      }
      // Normal routing
      if let Some(target) = model.midi_mapping.target_for(&msg.port_name, msg.cc) {
          let normalized = msg.value as f32 / 127.0;
          let state = model.midi_values.entry(target).or_insert(MidiTargetState {
              target: normalized,
              smoothed: normalized,
              active: false,
          });
          state.target = normalized;
          state.active = true; // MIDI takes control
      }
  }
  model.midi_learn.update();
  ```
- Replace the slider/pot smoothing block with:
  ```rust
  for (_target, state) in &mut model.midi_values {
      if state.active {
          state.smoothed = state.smoothed * (1.0 - model.smoothing_speed)
              + state.target * model.smoothing_speed;
      }
  }
  ```
- **GUI override mechanism**: when any GUI slider is moved for a param that has a MIDI mapping, set `active = false` on the corresponding `MidiTargetState`. This stops MIDI from overriding the GUI value. When the MIDI fader is physically moved again (new CC arrives), `active` becomes `true` and smoothing resumes from the current actual value.
  - Shader param sliders: the GUI knows the param index → map to `ShaderLeftParam(n)` / `ShaderRightParam(n)`
  - Mod knobs: map to `ShaderLeftMod(n)` / `ShaderRightMod(n)`
  - Named params (fade to black, audio, etc.): direct MidiTarget lookup
- Add `apply_midi_values(model)` function that reads `midi_values` (only active ones) and writes to appropriate locations:
  - `SmoothingSpeed` → `model.smoothing_speed` (map 0-1 to 0.0008-0.08)
  - `FadeToBlack` → `model.config.fade_to_black.led`
  - `LeftRightMix` → `model.config.presets.selected_mut().left_right_mix` (map 0-1 to -1.0-1.0)
  - `AudioGain` → `model.audio_input.gain_db` (map 0-1 to 0-MAX_INPUT_GAIN_DB)
  - `AudioThreshold` → `model.audio_input.threshold`
  - `AudioAttack` → `model.audio_input.attack`
  - `AudioHold` → `model.audio_input.hold`
  - `AudioRelease` → `model.audio_input.release`
  - `ColourChannel1/2/3` → `model.colour_channels[0/1/2]`
  - `ColourPalette` → `model.config.presets.selected_mut().shader_params.colour_palettes.interval` (or selected, TBD)
  - `ShaderLeftParam(n)` → write to preset's ShaderParams for left shader, param index n
  - `ShaderRightParam(n)` → write to preset's ShaderParams for right shader, param index n
  - `ShaderLeftMod(n)` → write to preset's `shader_mod_amounts` at the correct offset for left shader
  - `ShaderRightMod(n)` → write to preset's `shader_mod_amounts` at the correct offset for right shader
  - For shader param application, reuse the existing `gui::shader_params()` fn + `Params` trait to get `param_mut(n)` and write the mapped value
- Handle button events: for now, forward raw MIDI note-on/off to the button system (keep basic button support)

**Verify**: `cargo check` passes.

### Task 7 — Update LedWorkerInputState and Uniforms construction
**Modify** `cohen_gig/src/main.rs`:
- Remove `Controller` from `LedWorkerInputState`
- Remove `audio_mod_amps: [f32; 4]` from `LedWorkerInputState`
- Remove `midi_cv_phase_amp: f32` from `LedWorkerInputState`
- Add `colour_channels: [f32; 3]` to `LedWorkerInputState`
- Update `build_led_worker_input_state()` to use new fields
- In `led_worker_update()` (the shader compute fn):
  - Remove the `bw_param1-4` computation (slider + piano_mod)
  - Remove the `audio_mod_amps` application (this is now handled by `shader_mod_amounts` + `apply_shader_modulation`)
  - Set `pot6/pot7/pot8` from `state.colour_channels`
  - Set `slider1-6` to 0.0 (deprecated, will be removed from Uniforms in Phase 3)
  - Remove `midi_cv_phase_amp` from time computation (just use `time` directly)
  - The shader modulation via `apply_shader_modulation` already handles mod amounts — this stays as-is

**Verify**: `cargo check` passes, app runs, MIDI routing works.

---

## Phase 3: Update Shaders

### Task 8 — Remove slider/pot overrides from shader source files
**Modify** all files under `shader/src/` that reference `uniforms.slider*` or `uniforms.pot*`:
- Remove all `if uniforms.use_midi { params.x = uniforms.sliderN; }` blocks
- Shaders should only read from `uniforms.params.*` (their ShaderParams)
- The colour shaders (`solid_hsv_colour.rs`, `solid_rgb_colour.rs`) keep reading `uniforms.pot6/7/8` for colour channels
- `colour_palettes.rs`: remove `uniforms.slider5` override, read from params only
- `wash_shaders/shape_envelopes.rs`: remove slider1/2 override
- `wash_shaders/mitch_wash.rs`: keep button usage (no slider refs)

Files to modify (based on grep results):
- `shader/src/led_shaders/gilmore_acid.rs` — remove slider3/4 overrides
- `shader/src/led_shaders/radial_lines.rs` — remove slider1/2 overrides
- `shader/src/led_shaders/bw_gradient.rs` — remove slider1/2 overrides
- `shader/src/led_shaders/satis_spiraling.rs` — remove slider3/4 overrides
- `shader/src/led_shaders/spiral_intersect.rs` — remove slider3/4 overrides
- `shader/src/led_shaders/line_gradient.rs` — remove slider1/2 overrides
- `shader/src/led_shaders/escher_tilings.rs` — remove slider1/2 overrides
- `shader/src/led_shaders/colour_grid.rs` — remove slider3/4 overrides
- `shader/src/led_shaders/acid_gradient.rs` — remove slider3/4 overrides
- `shader/src/led_shaders/life_led_wall.rs` — remove slider3/4 overrides
- `shader/src/led_shaders/square_tunnel.rs` — remove slider1/2 overrides
- `shader/src/led_shaders/blinky_circles.rs` — remove slider3/4 overrides
- `shader/src/led_shaders/the_pulse.rs` — remove slider3/4 overrides
- `shader/src/led_shaders/tunnel_projection.rs` — remove slider3/4 overrides
- `shader/src/led_shaders/vert_colour_gradient.rs` — remove slider3/4 overrides
- `shader/src/led_shaders/just_relax.rs` — remove slider1/2 overrides
- `shader/src/led_shaders/metafall.rs` — remove slider1 override
- `shader/src/wash_shaders/shape_envelopes.rs` — remove slider1/2 overrides
- `shader/src/colour_palettes.rs` — remove slider5 override

**Verify**: shader compiles, app runs with no visual regressions (shader params now driven by ShaderParams + MIDI mapping).

### Task 9 — Clean up Uniforms (remove deprecated slider fields)
**Modify** `shader_shared/src/lib.rs`:
- Remove `slider1`-`slider6` from `Uniforms`
- Remove `use_midi` from `Uniforms` (no longer needed — MIDI gating done at routing level)
- Keep `pot6`, `pot7`, `pot8` (still used by colour shaders)

**Modify** `cohen_gig/src/main.rs`:
- Remove slider1-6, use_midi from Uniforms construction
- Remove `midi_on` from `LedWorkerConfig` if no longer needed in worker

**Modify** all shader files:
- Remove any remaining `uniforms.use_midi` checks
- Remove any remaining slider references

**Modify** `cohen_gig/src/conf.rs`:
- Remove `midi_on` from `Config` if no longer needed (or keep as "enable MIDI input" toggle)

**Verify**: `cargo check` across all crates, app runs.

---

## Phase 4: GUI

### Task 10 — Add MIDI tab and widget IDs
**Modify** `cohen_gig/src/gui.rs`:
- Add `Midi` to `LeftPanelTab` enum
- Add `midi_tab_button` to widget_ids
- Add MIDI tab widget IDs to `widget_ids!` macro:
  - `midi_preset_ddl`, `midi_preset_name_text_box`
  - `midi_preset_save_button`, `midi_preset_delete_button`, `midi_preset_new_button`
  - `midi_mapping_rows[]` (dynamic array for table rows)
  - `midi_mapping_learn_buttons[]` (dynamic array)
  - `midi_mapping_port_texts[]`, `midi_mapping_cc_texts[]` (dynamic arrays)
  - `midi_mapping_category_headers[]` (dynamic array)
- Add MIDI tab button in the tab bar rendering (alongside Live/Output)

**Modify** `cohen_gig/src/gui.rs` `UpdateContext`:
- Add `midi_mapping: &'a mut midi::mapping::MidiMapping`
- Add `midi_learn: &'a mut midi::learn::LearnState`

**Verify**: tab button appears, clicking it doesn't crash.

### Task 11 — MIDI tab preset management UI
**Modify** `cohen_gig/src/gui.rs`:
- Add preset management section at top of MIDI tab:
  - Dropdown listing available mapping presets from `assets/midi_mappings/`
  - Text box for preset name
  - Save button → saves current mapping to disk
  - New button → creates empty preset
  - Delete button → removes preset file
- Wire up preset switching → loads preset, rebuilds lookup table

**Verify**: can create, save, load, and delete MIDI mapping presets.

### Task 12 — MIDI tab mapping table UI
**Modify** `cohen_gig/src/gui.rs`:
- Add mapping table in MIDI tab body:
  - For each category in MidiTarget::categories():
    - Render category header text
    - For each target in category:
      - Row: param label | port name (or "—") | CC number (or "—") | Learn button
      - Shader param rows show `"Shader Left Param 1 (speed)"` using current shader's `Params` trait
  - Learn button click → set `midi_learn = LearnState::Listening(target)`
  - Highlight row when learn is active for that target
  - Cancel learn on second click or Escape key
- Ensure dynamic widget ID arrays are resized properly

**Verify**: full MIDI mapping workflow — click Learn, wiggle knob, see assignment appear, save preset, reload app, mapping persists.

---

## Phase 5: Cleanup

### Task 13 — Final cleanup and dead code removal
- Remove any remaining references to `Controller`, `mod_amp1-4`, `midi_cv_phase_amp`
- Remove `_midi_inputs` field from Model (replaced by MidiManager)
- Remove `korg_nano_kontrol_2` from workspace `Cargo.lock`
- Ensure `config.midi_on` toggle in GUI still works (controls whether CC messages are routed)
- Test: app starts clean, MIDI learn works, presets save/load, shader params respond to MIDI, smoothing works
- `cargo clippy` clean

---

## File Change Summary

**New files:**
- `cohen_gig/src/midi/mod.rs`
- `cohen_gig/src/midi/mapping.rs`
- `cohen_gig/src/midi/learn.rs`

**Modified files:**
- `cohen_gig/Cargo.toml` — remove korg dep
- `shader_shared/Cargo.toml` — remove korg dep
- `shader_shared/src/lib.rs` — local Button types, remove sliders from Uniforms
- `cohen_gig/src/main.rs` — major: replace Controller + MIDI handling
- `cohen_gig/src/audio_input.rs` — remove mod_amp1-4
- `cohen_gig/src/gui.rs` — add MIDI tab, update context
- `cohen_gig/src/conf.rs` — possibly remove midi_on or keep as routing toggle
- ~19 shader source files — remove slider override blocks

**New directories:**
- `assets/midi_mappings/` — created at runtime on first preset save
