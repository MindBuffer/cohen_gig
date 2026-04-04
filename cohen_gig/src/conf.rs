use nannou::io::{load_from_json, save_to_json};
use serde::{Deserialize, Serialize};
use shader_shared::{
    AcidGradient, BarTest, BlendMode, BlinkyCircles, BwGradient, ColourGrid, ColourPalettes,
    EscherTilings, GilmoreAcid, GradientBars, HoopLoop, ImitationRiley, JustRelax, LifeLedWall,
    LightPatternGenerator, LineGradient, Metafall, MitchWash, ParticleZoom, RadialKeta,
    RadialLines, RowTest, SatisSpiraling, Shader, ShaderParams, ShapeEnvelopes, SolidHsvColour,
    SolidRgbColour, SpiralIntersect, SquareTunnel, ThePulse, ToneMapping, TunnelProjection,
    TwoDTiles, VertColourGradient,
};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::net::{AddrParseError, Ipv4Addr};
use std::path::{Path, PathBuf};

/// Global runtime configuration.
///
/// Loaded from `assets/config.json` on launch. Saved only when the user
/// explicitly presses the "Save Config" button. This includes the preset
/// ordering/selection index, but not the full preset payloads.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Whether or not DMX is enabled.
    #[serde(default)]
    pub dmx_on: bool,
    /// Whether or not the LED previs window is visible.
    #[serde(default = "default::preview_window_on")]
    pub preview_window_on: bool,
    /// The preferred audio input device name to restore on startup when available.
    #[serde(default = "default::audio_input_device")]
    pub audio_input_device: String,
    /// The starting universe from which LED data is sent.
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
    /// Optional path to a MadMapper .mad project file.
    /// When Some, the layout and DMX addressing are derived from this file
    /// instead of the manual `led_layout` and `led_start_universe` fields.
    #[serde(default)]
    pub madmapper_project_path: Option<String>,
    #[serde(default)]
    pub preset_lerp_secs: f32,
    #[serde(default = "default::master_speed")]
    pub master_speed: f32,
    /// Order and current selection of the per-file shader presets.
    #[serde(default)]
    pub shader_preset_index: ShaderPresetIndex,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ShaderPresetIndex {
    /// Stable preset file ID for the selected preset.
    #[serde(default)]
    pub selected_preset_id: String,
    /// Stable preset file IDs in UI order.
    #[serde(default)]
    pub order: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LedLayout {
    #[serde(default = "default::led_layout::leds_per_metre")]
    pub leds_per_metre: usize,
    #[serde(default = "default::led_layout::metres_per_row")]
    pub metres_per_row: usize,
    #[serde(default = "default::led_layout::row_count")]
    pub row_count: usize,
}

/// Runtime preset state used by the app and GUI.
///
/// This stays in-memory as a flat list for UI simplicity, but serialises to
/// one JSON file per preset on disk.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Presets {
    #[serde(default = "default::presets::selected_preset_name")]
    pub selected_preset_name: String,
    #[serde(default = "default::presets::selected_preset_idx")]
    pub selected_preset_idx: usize,
    #[serde(default = "default::presets::list")]
    pub list: Vec<Preset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Preset {
    /// Stable file identifier used for per-preset JSON filenames.
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default::preset::shader_left")]
    pub shader_left: Shader,
    #[serde(default = "default::preset::shader_right")]
    pub shader_right: Shader,
    #[serde(default = "default::preset::colourise")]
    pub colourise: Shader,
    #[serde(default = "default::preset::tone_mapping")]
    pub tone_mapping: ToneMapping,
    #[serde(default = "default::preset::tone_mapping_amount")]
    pub tone_mapping_amount: f32,
    #[serde(default = "default::preset::left_right_mix")]
    pub left_right_mix: f32,
    #[serde(default = "default::preset::blend_mode")]
    pub blend_mode: BlendMode,
    /// Per-slot shader parameters. Each slot has independent params so the same
    /// shader type can be used in multiple slots without cross-contamination.
    #[serde(default)]
    pub shader_params_left: ShaderParams,
    #[serde(default)]
    pub shader_params_colourise: ShaderParams,
    #[serde(default)]
    pub shader_params_right: ShaderParams,
    #[serde(default)]
    pub shader_mod_amounts_left: Vec<f32>,
    #[serde(default)]
    pub shader_mod_amounts_colourise: Vec<f32>,
    #[serde(default)]
    pub shader_mod_amounts_right: Vec<f32>,
    // Legacy fields for backwards compatibility with old config.json.
    #[serde(default, alias = "shader_params", skip_serializing)]
    legacy_shader_params: Option<ShaderParams>,
    #[serde(default, alias = "shader_mod_amounts", skip_serializing)]
    legacy_shader_mod_amounts: Option<Vec<f32>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct StoredPreset {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default = "default::preset::shader_left")]
    shader_left: Shader,
    #[serde(default = "default::preset::shader_right")]
    shader_right: Shader,
    #[serde(default = "default::preset::colourise")]
    colourise: Shader,
    #[serde(default = "default::preset::tone_mapping")]
    tone_mapping: ToneMapping,
    #[serde(default = "default::preset::tone_mapping_amount")]
    tone_mapping_amount: f32,
    #[serde(default = "default::preset::left_right_mix")]
    left_right_mix: f32,
    #[serde(default = "default::preset::blend_mode")]
    blend_mode: BlendMode,
    #[serde(default)]
    shader_params_left: SparseShaderParams,
    #[serde(default)]
    shader_params_colourise: SparseShaderParams,
    #[serde(default)]
    shader_params_right: SparseShaderParams,
    #[serde(default)]
    shader_mod_amounts_left: Vec<f32>,
    #[serde(default)]
    shader_mod_amounts_colourise: Vec<f32>,
    #[serde(default)]
    shader_mod_amounts_right: Vec<f32>,
}

/// Sparse on-disk storage for shader params.
///
/// Only the active shader for each slot is populated when saving, which avoids
/// persisting the full parameter blob for every shader in every preset.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct SparseShaderParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    acid_gradient: Option<AcidGradient>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    blinky_circles: Option<BlinkyCircles>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bw_gradient: Option<BwGradient>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    colour_grid: Option<ColourGrid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    escher_tilings: Option<EscherTilings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    gilmore_acid: Option<GilmoreAcid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    gradient_bars: Option<GradientBars>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    just_relax: Option<JustRelax>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    life_led_wall: Option<LifeLedWall>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    light_pattern_generator: Option<LightPatternGenerator>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    line_gradient: Option<LineGradient>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    metafall: Option<Metafall>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    particle_zoom: Option<ParticleZoom>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    radial_lines: Option<RadialLines>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    satis_spiraling: Option<SatisSpiraling>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    spiral_intersect: Option<SpiralIntersect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    square_tunnel: Option<SquareTunnel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    the_pulse: Option<ThePulse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tunnel_projection: Option<TunnelProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    vert_colour_gradient: Option<VertColourGradient>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    solid_hsv_colour: Option<SolidHsvColour>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    solid_rgb_colour: Option<SolidRgbColour>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    colour_palettes: Option<ColourPalettes>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mitch_wash: Option<MitchWash>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    shape_envelopes: Option<ShapeEnvelopes>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    row_test: Option<RowTest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bar_test: Option<BarTest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    two_d_tiles: Option<TwoDTiles>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    radial_keta: Option<RadialKeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    imitation_riley: Option<ImitationRiley>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hoop_loop: Option<HoopLoop>,
}

/// Fade to black parameters for each kind of fixture.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FadeToBlack {
    #[serde(default = "default::fade_to_black::led")]
    pub led: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedOutputFps {
    Free,
    Fps90,
    Fps80,
    Fps70,
    Fps60,
    Fps50,
    Fps40,
}

pub fn config_path(assets: &Path) -> PathBuf {
    assets.join("config.json")
}

pub fn presets_dir(assets: &Path) -> PathBuf {
    assets.join("presets")
}

fn legacy_presets_path(assets: &Path) -> PathBuf {
    assets.join("presets.json")
}

impl Presets {
    /// Produces a reference to the selected preset.
    pub fn selected(&self) -> &Preset {
        &self.list[self.selected_preset_idx]
    }

    /// Mutable access to the selected preset.
    pub fn selected_mut(&mut self) -> &mut Preset {
        &mut self.list[self.selected_preset_idx]
    }

    /// Moves a preset to a new position while keeping the current selection on the same preset.
    pub fn move_preset(&mut self, from: usize, to: usize) -> bool {
        if self.list.is_empty() || from >= self.list.len() || to >= self.list.len() || from == to {
            return false;
        }

        let preset = self.list.remove(from);
        self.list.insert(to, preset);

        self.selected_preset_idx = match self.selected_preset_idx {
            idx if idx == from => to,
            idx if from < idx && idx <= to => idx - 1,
            idx if to <= idx && idx < from => idx + 1,
            idx => idx,
        };
        self.selected_preset_name = self.selected().name.clone();

        true
    }

    pub fn index(&self) -> ShaderPresetIndex {
        let selected_preset_id = self
            .list
            .get(self.selected_preset_idx)
            .map(|preset| preset.id.clone())
            .or_else(|| self.list.first().map(|preset| preset.id.clone()))
            .unwrap_or_default();

        ShaderPresetIndex {
            selected_preset_id,
            order: self.list.iter().map(|preset| preset.id.clone()).collect(),
        }
    }

    pub fn sync_global_config(&self, global_config: &mut GlobalConfig) {
        global_config.shader_preset_index = self.index();
    }

    pub fn next_preset_id(&self, name: &str) -> String {
        let used_ids: HashSet<_> = self.list.iter().map(|preset| preset.id.clone()).collect();
        next_available_preset_id(name, &used_ids)
    }

    fn ensure_valid(&mut self) {
        if self.list.is_empty() {
            self.list = default::presets::list();
        }

        let mut used_ids = HashSet::new();
        for preset in &mut self.list {
            if preset.id.is_empty() || !used_ids.insert(preset.id.clone()) {
                let id = next_available_preset_id(&preset.name, &used_ids);
                used_ids.insert(id.clone());
                preset.id = id;
            }
        }

        if self.selected_preset_idx >= self.list.len() {
            self.selected_preset_idx = 0;
        }
        self.selected_preset_name = self.selected().name.clone();
    }

    fn from_index(index: &ShaderPresetIndex, mut by_id: BTreeMap<String, Preset>) -> Self {
        let mut list = Vec::with_capacity(by_id.len().max(1));

        for id in &index.order {
            if let Some(preset) = by_id.remove(id) {
                list.push(preset);
            }
        }

        list.extend(by_id.into_values());

        let mut presets = if list.is_empty() {
            Presets::default()
        } else {
            let selected_preset_idx = list
                .iter()
                .position(|preset| preset.id == index.selected_preset_id)
                .unwrap_or(0);
            let selected_preset_name = list[selected_preset_idx].name.clone();
            Presets {
                selected_preset_name,
                selected_preset_idx,
                list,
            }
        };

        presets.ensure_valid();
        presets
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfig {
            dmx_on: Default::default(),
            preview_window_on: default::preview_window_on(),
            audio_input_device: default::audio_input_device(),
            led_start_universe: default::led_start_universe(),
            fade_to_black: Default::default(),
            sacn_interface_ip: default::sacn_interface_ip(),
            led_output_fps: Default::default(),
            led_layout: Default::default(),
            madmapper_project_path: None,
            preset_lerp_secs: Default::default(),
            master_speed: default::master_speed(),
            shader_preset_index: Default::default(),
        }
    }
}

/// Used only for migration from older config formats.
#[derive(Deserialize)]
struct LegacyConfig {
    #[serde(flatten)]
    global: GlobalConfig,
    #[serde(default)]
    presets: Option<Presets>,
}

/// Load global config and presets. Handles migration from:
/// - old combined config.json
/// - old monolithic presets.json
/// - new per-file presets directory
pub fn load(assets: &Path) -> (GlobalConfig, Presets) {
    let config_path = config_path(assets);
    let preset_dir = presets_dir(assets);
    let legacy_presets_path = legacy_presets_path(assets);

    let legacy_config = load_from_json::<_, LegacyConfig>(&config_path).ok();
    let mut global = legacy_config
        .as_ref()
        .map(|legacy| legacy.global.clone())
        .unwrap_or_default();

    let mut migrated = false;

    let mut presets = {
        let presets_from_dir = load_preset_files(&preset_dir);
        if !presets_from_dir.is_empty() {
            Presets::from_index(&global.shader_preset_index, presets_from_dir)
        } else if let Ok(legacy_presets) = load_from_json::<_, Presets>(&legacy_presets_path) {
            migrated = true;
            let mut presets = legacy_presets;
            presets.ensure_valid();
            presets
        } else if let Some(mut legacy) = legacy_config {
            let mut presets = legacy.presets.take().unwrap_or_default();
            presets.ensure_valid();
            global = legacy.global;
            migrated = true;
            presets
        } else {
            Presets::default()
        }
    };

    presets.ensure_valid();
    presets.sync_global_config(&mut global);

    if migrated {
        let _ = try_save_presets(assets, &mut global, &presets);
    }

    (global, presets)
}

impl LedLayout {
    pub fn normalise(&mut self) {
        self.leds_per_metre = self.leds_per_metre.max(1);
        self.metres_per_row = self.metres_per_row.max(1);
        self.row_count = self.row_count.max(1);
    }

    pub fn leds_per_row(&self) -> usize {
        self.leds_per_metre * self.metres_per_row
    }

    pub fn led_count(&self) -> usize {
        self.leds_per_row() * self.row_count
    }
}

impl Default for FadeToBlack {
    fn default() -> Self {
        FadeToBlack {
            led: default::fade_to_black::led(),
        }
    }
}

impl Default for LedOutputFps {
    fn default() -> Self {
        Self::Free
    }
}

impl LedOutputFps {
    pub const ALL: [Self; 7] = [
        Self::Free,
        Self::Fps90,
        Self::Fps80,
        Self::Fps70,
        Self::Fps60,
        Self::Fps50,
        Self::Fps40,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Free => "Free",
            Self::Fps90 => "90 FPS",
            Self::Fps80 => "80 FPS",
            Self::Fps70 => "70 FPS",
            Self::Fps60 => "60 FPS",
            Self::Fps50 => "50 FPS",
            Self::Fps40 => "40 FPS",
        }
    }

    pub fn fps_limit(self) -> Option<f32> {
        match self {
            Self::Free => None,
            Self::Fps90 => Some(90.0),
            Self::Fps80 => Some(80.0),
            Self::Fps70 => Some(70.0),
            Self::Fps60 => Some(60.0),
            Self::Fps50 => Some(50.0),
            Self::Fps40 => Some(40.0),
        }
    }

    pub fn to_index(self) -> usize {
        Self::ALL
            .iter()
            .position(|mode| *mode == self)
            .expect("LedOutputFps variant missing from ALL")
    }

    pub fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }
}

impl Default for LedLayout {
    fn default() -> Self {
        let mut led_layout = LedLayout {
            leds_per_metre: default::led_layout::leds_per_metre(),
            metres_per_row: default::led_layout::metres_per_row(),
            row_count: default::led_layout::row_count(),
        };
        led_layout.normalise();
        led_layout
    }
}

impl Default for Presets {
    fn default() -> Self {
        let mut presets = Presets {
            selected_preset_name: default::presets::selected_preset_name(),
            selected_preset_idx: default::presets::selected_preset_idx(),
            list: default::presets::list(),
        };
        presets.ensure_valid();
        presets
    }
}

impl Preset {
    /// Migrate legacy single-params to per-slot params if present.
    pub fn migrate_legacy(&mut self) {
        if let Some(params) = self.legacy_shader_params.take() {
            // Only migrate if per-slot params are all defaults (i.e. not already set).
            if self.shader_params_left == ShaderParams::default()
                && self.shader_params_colourise == ShaderParams::default()
                && self.shader_params_right == ShaderParams::default()
            {
                self.shader_params_left = params;
                self.shader_params_colourise = params;
                self.shader_params_right = params;
            }
        }
        if let Some(mod_amounts) = self.legacy_shader_mod_amounts.take() {
            if !mod_amounts.is_empty()
                && self.shader_mod_amounts_left.is_empty()
                && self.shader_mod_amounts_colourise.is_empty()
                && self.shader_mod_amounts_right.is_empty()
            {
                // Legacy mod amounts were interleaved: left, colourise, right.
                // We need to split them by counting each slot's f32 params.
                // For simplicity, just put them all in left — normalise will fix sizes.
                self.shader_mod_amounts_left = mod_amounts;
            }
        }
    }
}

impl Default for Preset {
    fn default() -> Self {
        Preset {
            id: String::new(),
            name: default::presets::selected_preset_name(),
            shader_left: default::preset::shader_left(),
            shader_right: default::preset::shader_right(),
            left_right_mix: default::preset::left_right_mix(),
            colourise: default::preset::colourise(),
            tone_mapping: default::preset::tone_mapping(),
            tone_mapping_amount: default::preset::tone_mapping_amount(),
            blend_mode: default::preset::blend_mode(),
            shader_params_left: ShaderParams::default(),
            shader_params_colourise: ShaderParams::default(),
            shader_params_right: ShaderParams::default(),
            shader_mod_amounts_left: Vec::new(),
            shader_mod_amounts_colourise: Vec::new(),
            shader_mod_amounts_right: Vec::new(),
            legacy_shader_params: None,
            legacy_shader_mod_amounts: None,
        }
    }
}

impl StoredPreset {
    fn from_runtime(preset: &Preset) -> Self {
        StoredPreset {
            id: preset.id.clone(),
            name: preset.name.clone(),
            shader_left: preset.shader_left,
            shader_right: preset.shader_right,
            colourise: preset.colourise,
            tone_mapping: preset.tone_mapping,
            tone_mapping_amount: preset.tone_mapping_amount,
            left_right_mix: preset.left_right_mix,
            blend_mode: preset.blend_mode,
            shader_params_left: SparseShaderParams::from_runtime(
                preset.shader_left,
                &preset.shader_params_left,
            ),
            shader_params_colourise: SparseShaderParams::from_runtime(
                preset.colourise,
                &preset.shader_params_colourise,
            ),
            shader_params_right: SparseShaderParams::from_runtime(
                preset.shader_right,
                &preset.shader_params_right,
            ),
            shader_mod_amounts_left: preset.shader_mod_amounts_left.clone(),
            shader_mod_amounts_colourise: preset.shader_mod_amounts_colourise.clone(),
            shader_mod_amounts_right: preset.shader_mod_amounts_right.clone(),
        }
    }

    fn into_runtime(self, fallback_id: String) -> Preset {
        let id = if self.id.trim().is_empty() {
            fallback_id
        } else {
            self.id
        };

        Preset {
            id,
            name: if self.name.trim().is_empty() {
                default::presets::selected_preset_name()
            } else {
                self.name
            },
            shader_left: self.shader_left,
            shader_right: self.shader_right,
            colourise: self.colourise,
            tone_mapping: self.tone_mapping,
            tone_mapping_amount: self.tone_mapping_amount,
            left_right_mix: self.left_right_mix,
            blend_mode: self.blend_mode,
            shader_params_left: self.shader_params_left.into_runtime(self.shader_left),
            shader_params_colourise: self.shader_params_colourise.into_runtime(self.colourise),
            shader_params_right: self.shader_params_right.into_runtime(self.shader_right),
            shader_mod_amounts_left: self.shader_mod_amounts_left,
            shader_mod_amounts_colourise: self.shader_mod_amounts_colourise,
            shader_mod_amounts_right: self.shader_mod_amounts_right,
            legacy_shader_params: None,
            legacy_shader_mod_amounts: None,
        }
    }
}

impl SparseShaderParams {
    fn from_runtime(shader: Shader, params: &ShaderParams) -> Self {
        let mut sparse = SparseShaderParams::default();
        match shader {
            Shader::SolidHsvColour => sparse.solid_hsv_colour = Some(params.solid_hsv_colour),
            Shader::SolidRgbColour => sparse.solid_rgb_colour = Some(params.solid_rgb_colour),
            Shader::ColourPalettes => sparse.colour_palettes = Some(params.colour_palettes),
            Shader::AcidGradient => sparse.acid_gradient = Some(params.acid_gradient),
            Shader::BlinkyCircles => sparse.blinky_circles = Some(params.blinky_circles),
            Shader::BwGradient => sparse.bw_gradient = Some(params.bw_gradient),
            Shader::ColourGrid => sparse.colour_grid = Some(params.colour_grid),
            Shader::EscherTilings => sparse.escher_tilings = Some(params.escher_tilings),
            Shader::GilmoreAcid => sparse.gilmore_acid = Some(params.gilmore_acid),
            Shader::GradientBars => sparse.gradient_bars = Some(params.gradient_bars),
            Shader::JustRelax => sparse.just_relax = Some(params.just_relax),
            Shader::LifeLedWall => sparse.life_led_wall = Some(params.life_led_wall),
            Shader::LightPatternGenerator => {
                sparse.light_pattern_generator = Some(params.light_pattern_generator);
            }
            Shader::LineGradient => sparse.line_gradient = Some(params.line_gradient),
            Shader::Metafall => sparse.metafall = Some(params.metafall),
            Shader::ParticleZoom => sparse.particle_zoom = Some(params.particle_zoom),
            Shader::RadialLines => sparse.radial_lines = Some(params.radial_lines),
            Shader::SatisSpiraling => sparse.satis_spiraling = Some(params.satis_spiraling),
            Shader::SpiralIntersect => sparse.spiral_intersect = Some(params.spiral_intersect),
            Shader::SquareTunnel => sparse.square_tunnel = Some(params.square_tunnel),
            Shader::ThePulse => sparse.the_pulse = Some(params.the_pulse),
            Shader::TunnelProjection => sparse.tunnel_projection = Some(params.tunnel_projection),
            Shader::VertColourGradient => {
                sparse.vert_colour_gradient = Some(params.vert_colour_gradient);
            }
            Shader::MitchWash => sparse.mitch_wash = Some(params.mitch_wash),
            Shader::ShapeEnvelopes => sparse.shape_envelopes = Some(params.shape_envelopes),
            Shader::RowTest => sparse.row_test = Some(params.row_test),
            Shader::BarTest => sparse.bar_test = Some(params.bar_test),
            Shader::TwoDTiles => sparse.two_d_tiles = Some(params.two_d_tiles),
            Shader::RadialKeta => sparse.radial_keta = Some(params.radial_keta),
            Shader::ImitationRiley => sparse.imitation_riley = Some(params.imitation_riley),
            Shader::HoopLoop => sparse.hoop_loop = Some(params.hoop_loop),
        }
        sparse
    }

    fn into_runtime(self, shader: Shader) -> ShaderParams {
        let mut params = ShaderParams::default();
        match shader {
            Shader::SolidHsvColour => {
                params.solid_hsv_colour = self.solid_hsv_colour.unwrap_or_default()
            }
            Shader::SolidRgbColour => {
                params.solid_rgb_colour = self.solid_rgb_colour.unwrap_or_default()
            }
            Shader::ColourPalettes => {
                params.colour_palettes = self.colour_palettes.unwrap_or_default()
            }
            Shader::AcidGradient => params.acid_gradient = self.acid_gradient.unwrap_or_default(),
            Shader::BlinkyCircles => {
                params.blinky_circles = self.blinky_circles.unwrap_or_default()
            }
            Shader::BwGradient => params.bw_gradient = self.bw_gradient.unwrap_or_default(),
            Shader::ColourGrid => params.colour_grid = self.colour_grid.unwrap_or_default(),
            Shader::EscherTilings => {
                params.escher_tilings = self.escher_tilings.unwrap_or_default()
            }
            Shader::GilmoreAcid => params.gilmore_acid = self.gilmore_acid.unwrap_or_default(),
            Shader::GradientBars => params.gradient_bars = self.gradient_bars.unwrap_or_default(),
            Shader::JustRelax => params.just_relax = self.just_relax.unwrap_or_default(),
            Shader::LifeLedWall => params.life_led_wall = self.life_led_wall.unwrap_or_default(),
            Shader::LightPatternGenerator => {
                params.light_pattern_generator = self.light_pattern_generator.unwrap_or_default()
            }
            Shader::LineGradient => params.line_gradient = self.line_gradient.unwrap_or_default(),
            Shader::Metafall => params.metafall = self.metafall.unwrap_or_default(),
            Shader::ParticleZoom => params.particle_zoom = self.particle_zoom.unwrap_or_default(),
            Shader::RadialLines => params.radial_lines = self.radial_lines.unwrap_or_default(),
            Shader::SatisSpiraling => {
                params.satis_spiraling = self.satis_spiraling.unwrap_or_default()
            }
            Shader::SpiralIntersect => {
                params.spiral_intersect = self.spiral_intersect.unwrap_or_default()
            }
            Shader::SquareTunnel => params.square_tunnel = self.square_tunnel.unwrap_or_default(),
            Shader::ThePulse => params.the_pulse = self.the_pulse.unwrap_or_default(),
            Shader::TunnelProjection => {
                params.tunnel_projection = self.tunnel_projection.unwrap_or_default()
            }
            Shader::VertColourGradient => {
                params.vert_colour_gradient = self.vert_colour_gradient.unwrap_or_default()
            }
            Shader::MitchWash => params.mitch_wash = self.mitch_wash.unwrap_or_default(),
            Shader::ShapeEnvelopes => {
                params.shape_envelopes = self.shape_envelopes.unwrap_or_default()
            }
            Shader::RowTest => params.row_test = self.row_test.unwrap_or_default(),
            Shader::BarTest => params.bar_test = self.bar_test.unwrap_or_default(),
            Shader::TwoDTiles => params.two_d_tiles = self.two_d_tiles.unwrap_or_default(),
            Shader::RadialKeta => params.radial_keta = self.radial_keta.unwrap_or_default(),
            Shader::ImitationRiley => {
                params.imitation_riley = self.imitation_riley.unwrap_or_default()
            }
            Shader::HoopLoop => params.hoop_loop = self.hoop_loop.unwrap_or_default(),
        }
        params
    }
}

pub fn save_presets(assets: &Path, global_config: &mut GlobalConfig, presets: &Presets) {
    try_save_presets(assets, global_config, presets).expect("failed to save presets");
}

fn try_save_presets(
    assets: &Path,
    global_config: &mut GlobalConfig,
    presets: &Presets,
) -> Result<(), String> {
    presets.sync_global_config(global_config);

    let config_path = config_path(assets);
    save_to_json(&config_path, global_config)
        .map_err(|err| format!("failed to save {:?}: {}", config_path, err))?;

    let preset_dir = presets_dir(assets);
    fs::create_dir_all(&preset_dir)
        .map_err(|err| format!("failed to create {:?}: {}", preset_dir, err))?;

    let keep_ids: HashSet<_> = presets
        .list
        .iter()
        .map(|preset| preset.id.clone())
        .collect();
    for preset in &presets.list {
        let path = preset_dir.join(&preset.id).with_extension("json");
        let stored = StoredPreset::from_runtime(preset);
        save_to_json(&path, &stored)
            .map_err(|err| format!("failed to save {:?}: {}", path, err))?;
    }

    cleanup_removed_preset_files(&preset_dir, &keep_ids)?;

    Ok(())
}

fn load_preset_files(preset_dir: &Path) -> BTreeMap<String, Preset> {
    let mut presets = BTreeMap::new();

    let entries = match fs::read_dir(preset_dir) {
        Ok(entries) => entries,
        Err(_) => return presets,
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() || path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let file_stem = match path.file_stem().and_then(|stem| stem.to_str()) {
            Some(stem) => stem.to_string(),
            None => continue,
        };

        let stored: StoredPreset = match load_from_json(&path) {
            Ok(stored) => stored,
            Err(err) => {
                eprintln!("Failed to load {:?}: {}", path, err);
                continue;
            }
        };

        let preset = stored.into_runtime(file_stem);
        presets.insert(preset.id.clone(), preset);
    }

    presets
}

fn cleanup_removed_preset_files(
    preset_dir: &Path,
    keep_ids: &HashSet<String>,
) -> Result<(), String> {
    let entries = match fs::read_dir(preset_dir) {
        Ok(entries) => entries,
        Err(err) => return Err(format!("failed to read {:?}: {}", preset_dir, err)),
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() || path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let Some(file_stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };

        if !keep_ids.contains(file_stem) {
            fs::remove_file(&path)
                .map_err(|err| format!("failed to remove {:?}: {}", path, err))?;
        }
    }

    Ok(())
}

fn next_available_preset_id(name: &str, used_ids: &HashSet<String>) -> String {
    let base = slugify_preset_id(name);
    if !used_ids.contains(&base) {
        return base;
    }

    let mut index = 2;
    loop {
        let candidate = format!("{base}-{index}");
        if !used_ids.contains(&candidate) {
            return candidate;
        }
        index += 1;
    }
}

fn slugify_preset_id(name: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "preset".to_string()
    } else {
        slug
    }
}

pub mod default {
    pub fn preview_window_on() -> bool {
        true
    }

    pub fn audio_input_device() -> String {
        String::new()
    }

    /// The default universe to which LED data is sent.
    pub fn led_start_universe() -> u16 {
        1
    }

    pub fn sacn_interface_ip() -> String {
        String::new()
    }

    pub fn master_speed() -> f32 {
        1.0
    }

    pub mod led_layout {
        pub fn leds_per_metre() -> usize {
            100
        }

        pub fn metres_per_row() -> usize {
            6
        }

        pub fn row_count() -> usize {
            7
        }
    }

    pub mod presets {
        pub fn selected_preset_name() -> String {
            "Empty".to_string()
        }
        pub fn selected_preset_idx() -> usize {
            0
        }
        pub fn list() -> Vec<crate::conf::Preset> {
            vec![crate::conf::Preset::default()]
        }
    }

    pub mod preset {
        use shader_shared::{BlendMode, Shader, ToneMapping};
        pub fn shader_left() -> Shader {
            Shader::SatisSpiraling
        }
        pub fn shader_right() -> Shader {
            Shader::SolidHsvColour
        }
        pub fn colourise() -> Shader {
            Shader::SolidHsvColour
        }
        pub fn tone_mapping() -> ToneMapping {
            ToneMapping::Aces
        }
        pub fn tone_mapping_amount() -> f32 {
            1.0
        }
        pub fn left_right_mix() -> f32 {
            0.0
        }
        pub fn blend_mode() -> BlendMode {
            BlendMode::Add
        }
    }

    pub mod fade_to_black {
        pub fn led() -> f32 {
            1.0
        }
    }
}

pub fn parse_sacn_interface_ip(value: &str) -> Result<Option<Ipv4Addr>, AddrParseError> {
    let value = value.trim();
    if value.is_empty() {
        Ok(None)
    } else {
        value.parse::<Ipv4Addr>().map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_valid_assigns_unique_ids_for_duplicate_names() {
        let mut presets = Presets {
            selected_preset_name: "Duplicate".to_string(),
            selected_preset_idx: 0,
            list: vec![
                Preset {
                    id: String::new(),
                    name: "Duplicate".to_string(),
                    ..Preset::default()
                },
                Preset {
                    id: String::new(),
                    name: "Duplicate".to_string(),
                    ..Preset::default()
                },
            ],
        };

        presets.ensure_valid();

        assert_eq!(presets.list[0].id, "duplicate");
        assert_eq!(presets.list[1].id, "duplicate-2");
        assert_eq!(
            presets.index().order,
            vec!["duplicate".to_string(), "duplicate-2".to_string()]
        );
    }

    #[test]
    fn stored_preset_only_serializes_active_shader_params() {
        let mut preset = Preset {
            id: "pulse-gradient".to_string(),
            name: "Pulse Gradient".to_string(),
            shader_left: Shader::ThePulse,
            shader_right: Shader::AcidGradient,
            colourise: Shader::SolidHsvColour,
            tone_mapping: ToneMapping::Unreal,
            tone_mapping_amount: 0.35,
            ..Preset::default()
        };
        preset.shader_params_left.the_pulse.speed = 0.42;
        preset.shader_params_left.acid_gradient.speed = 0.99;
        preset.shader_params_right.acid_gradient.offset = 0.33;

        let stored = StoredPreset::from_runtime(&preset);
        let value = serde_json::to_value(&stored).unwrap();

        let left = value
            .get("shader_params_left")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(left.len(), 1);
        assert!(left.contains_key("the_pulse"));

        let round_trip: StoredPreset = serde_json::from_value(value).unwrap();
        let loaded = round_trip.into_runtime("pulse-gradient".to_string());
        assert_eq!(loaded.tone_mapping, ToneMapping::Unreal);
        assert_eq!(loaded.tone_mapping_amount, 0.35);
        assert_eq!(loaded.shader_params_left.the_pulse.speed, 0.42);
        assert_eq!(
            loaded.shader_params_left.acid_gradient,
            AcidGradient::default()
        );
        assert_eq!(loaded.shader_params_right.acid_gradient.offset, 0.33);
    }
}
