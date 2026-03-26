# Split Config and Presets into Separate Files

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Decouple global config from shader presets so each can be saved independently via dedicated buttons, and neither auto-saves on exit.

**Architecture:** Split the current monolithic `Config` struct into two: `GlobalConfig` (settings like DMX, MadMapper path, audio device, LED layout) and `Presets` (the preset list and selection state). Each gets its own JSON file (`config.json`, `presets.json`) and its own explicit save button. On launch both are loaded; on exit neither is auto-saved.

**Tech Stack:** Rust, serde/serde_json, nannou_conrod (GUI), nannou::io (save_to_json/load_from_json)

---

### Task 1: Split the `Config` struct in `conf.rs`

**Files:**
- Modify: `cohen_gig/src/conf.rs`

**Step 1: Rename `Config` to `GlobalConfig` and remove the `presets` field**

The new `GlobalConfig` keeps all non-preset fields. The existing `Presets` struct stays as-is but becomes a top-level saved entity.

```rust
/// Global runtime configuration (non-preset settings).
///
/// Loaded from `assets/config.json` on launch. Saved only when the user
/// explicitly presses the "Save Config" button.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub dmx_on: bool,
    #[serde(default = "default::preview_window_on")]
    pub preview_window_on: bool,
    #[serde(default = "default::audio_input_device")]
    pub audio_input_device: String,
    #[serde(default = "default::led_start_universe")]
    pub led_start_universe: u16,
    #[serde(default)]
    pub fade_to_black: FadeToBlack,
    #[serde(default = "default::sacn_interface_ip")]
    pub sacn_interface_ip: String,
    #[serde(default)]
    pub led_output_fps: LedOutputFps,
    #[serde(default)]
    pub led_layout: LedLayout,
    #[serde(default)]
    pub madmapper_project_path: Option<String>,
    #[serde(default)]
    pub preset_lerp_secs: f32,
}
```

**Step 2: Add separate path functions**

```rust
pub fn config_path(assets: &Path) -> PathBuf {
    assets.join("config.json")
}

pub fn presets_path(assets: &Path) -> PathBuf {
    assets.join("presets.json")
}
```

Remove the old `pub fn path()`.

**Step 3: Update `Default` impl for `GlobalConfig`**

Same as old `Config::default()` but without the `presets` field.

**Step 4: Add backwards-compatible loading**

For migration from the old combined `config.json`, add a `LegacyConfig` struct that can deserialize both old and new formats:

```rust
/// Used only for one-time migration from the old combined config.json.
#[derive(Deserialize)]
struct LegacyConfig {
    #[serde(flatten)]
    global: GlobalConfig,
    #[serde(default)]
    presets: Option<Presets>,
}
```

Add a public loading function:

```rust
/// Load global config and presets. Handles migration from the old single-file format.
pub fn load(assets: &Path) -> (GlobalConfig, Presets) {
    let config_path = config_path(assets);
    let presets_path = presets_path(assets);

    // If presets.json already exists, load both independently.
    if presets_path.exists() {
        let global: GlobalConfig = load_from_json(&config_path)
            .ok()
            .unwrap_or_default();
        let presets: Presets = load_from_json(&presets_path)
            .ok()
            .unwrap_or_default();
        return (global, presets);
    }

    // Otherwise try to migrate from the old combined config.json.
    if let Ok(legacy) = load_from_json::<_, LegacyConfig>(&config_path) {
        let presets = legacy.presets.unwrap_or_default();
        return (legacy.global, presets);
    }

    (GlobalConfig::default(), Presets::default())
}
```

**Step 5: Run `cargo check`**

This will fail with many errors (expected — other files still reference `Config`). That's fine, we fix them in the next tasks.

---

### Task 2: Update `Model` and `main.rs` to use split types

**Files:**
- Modify: `cohen_gig/src/main.rs`

**Step 1: Replace `config: Config` with `global_config: GlobalConfig` and `presets: Presets` in `Model`**

```rust
struct Model {
    // ... other fields unchanged ...
    global_config: GlobalConfig,
    presets: conf::Presets,
    // ... rest unchanged ...
}
```

**Step 2: Update `model()` function**

Replace the config loading block:

```rust
let (mut global_config, mut presets) = conf::load(&assets);
global_config.led_layout.normalise();
for preset in &mut presets.list {
    preset.migrate_legacy();
    gui::normalise_preset_shader_mod_amounts(preset);
}
```

Update all `Model` construction to use the new field names.

**Step 3: Replace `save_config` with `save_global_config` and `save_presets`**

```rust
fn save_global_config(assets: &Path, config: &GlobalConfig) {
    let path = conf::config_path(assets);
    save_to_json(path, config).expect("failed to save global config");
}

fn save_presets(assets: &Path, presets: &conf::Presets) {
    let path = conf::presets_path(assets);
    save_to_json(path, presets).expect("failed to save presets");
}
```

**Step 4: Remove auto-save from `exit()`**

Delete the `save_config` call in `exit()`. Keep only the LED worker shutdown logic.

**Step 5: Update all `model.config.xyz` references throughout `main.rs`**

Every `model.config.field` becomes either `model.global_config.field` or `model.presets.field` depending on which struct owns it. The preset-related fields (`presets`, `preset_lerp_secs`) need care — `preset_lerp_secs` moves to `GlobalConfig`, while `presets` is its own field.

**Step 6: Run `cargo check`**

Fix remaining references until main.rs compiles (gui.rs will still fail).

---

### Task 3: Update `gui.rs` to use split types

**Files:**
- Modify: `cohen_gig/src/gui.rs`

**Step 1: Add `save_config_button` widget ID**

In the `widget_ids!` macro, add after `dmx_button`:

```rust
save_config_button,
```

**Step 2: Update `UpdateContext` to take split references**

```rust
pub struct UpdateContext<'a> {
    pub global_config: &'a mut GlobalConfig,
    pub presets: &'a mut conf::Presets,
    // ... rest unchanged ...
}
```

**Step 3: Update `gui::update()` and all sub-functions**

Replace all `config: &mut Config` params with `global_config: &mut GlobalConfig, presets: &mut conf::Presets` (or pass just the one that's needed).

Key function signatures to update:
- `set_live_sidebar_widgets` — needs `global_config` (for `preview_window_on`)
- `set_output_sidebar_widgets` — needs `global_config` only
- `set_presets_widgets` — needs `presets` and `global_config` (for `preset_lerp_secs`)
- `set_output_monitor_widgets` — needs `global_config` (for `dmx_on`)
- The shader/blend/mix column functions — need `presets` (for selected preset params)
- Audio widgets — need `global_config.audio_input_device`

**Step 4: Add "Save Config" button in the Output tab**

At the bottom of `set_output_sidebar_widgets`, after the LED layout stats text:

```rust
for _click in button()
    .down(COLUMN_ONE_SECTION_GAP)
    .label("Save Config")
    .w_h(WIDGET_W, DEFAULT_WIDGET_H)
    .color(BUTTON_COLOR)
    .set(ids.save_config_button, ui)
{
    super::save_global_config(assets, global_config);
}
```

This requires passing `assets: &Path` into `set_output_sidebar_widgets` (it's not currently a parameter).

**Step 5: Update preset "Save" button to only save presets**

In `set_presets_widgets`, change:

```rust
// Before:
super::save_config(assets, config);

// After:
super::save_presets(assets, presets);
```

**Step 6: Run `cargo check`**

Fix all remaining compile errors. The main work is mechanical: replacing `config.` with `global_config.` or `presets.` throughout gui.rs.

---

### Task 4: Update remaining modules that reference `Config`

**Files:**
- Modify: `cohen_gig/src/audio_widgets.rs` (if it references Config)
- Modify: any other files that import `Config`

**Step 1: Search for all remaining `Config` references**

```bash
grep -rn "Config" cohen_gig/src/ --include="*.rs"
```

Update imports and usages. `audio_widgets::set_widgets` takes `&mut config.audio_input_device` — this just becomes `&mut global_config.audio_input_device` at the call site.

**Step 2: Run `cargo check` — should compile clean**

**Step 3: Run `cargo test`**

```bash
cargo test
```

Fix any test failures.

---

### Task 5: Test migration from old config format

**Step 1: Verify the app loads the existing `assets/config.json` correctly**

The old config.json has presets embedded. On first load with the new code:
- `conf::load()` should detect no `presets.json` exists
- It should parse the old `config.json` via `LegacyConfig`
- Global settings and presets should both load correctly

**Step 2: Verify saving works**

- Press "Save Config" → `config.json` is written (no presets in it)
- Press preset "Save" → `presets.json` is written (only presets)
- After both saves, relaunch → both files load independently

**Step 3: Commit**

```bash
git add -A
git commit -m "Split config and presets into separate JSON files with independent save buttons"
```
