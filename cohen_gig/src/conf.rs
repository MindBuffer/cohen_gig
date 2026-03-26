use nannou::io::{load_from_json, save_to_json};
use serde::{Deserialize, Serialize};
use shader_shared::{BlendMode, Shader, ShaderParams};
use std::net::{AddrParseError, Ipv4Addr};
use std::path::{Path, PathBuf};

/// Global runtime configuration (non-preset settings).
///
/// Loaded from `assets/config.json` on launch. Saved only when the user
/// explicitly presses the "Save Config" button.
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
    #[serde(default)]
    pub name: String,
    #[serde(default = "default::preset::shader_left")]
    pub shader_left: Shader,
    #[serde(default = "default::preset::shader_right")]
    pub shader_right: Shader,
    #[serde(default = "default::preset::colourise")]
    pub colourise: Shader,
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

pub fn presets_path(assets: &Path) -> PathBuf {
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
        }
    }
}

/// Used only for one-time migration from the old combined config.json.
#[derive(Deserialize)]
struct LegacyConfig {
    #[serde(flatten)]
    global: GlobalConfig,
    #[serde(default)]
    presets: Option<Presets>,
}

/// Load global config and presets. Handles migration from the old single-file format.
pub fn load(assets: &Path) -> (GlobalConfig, Presets) {
    let config_path = config_path(assets);
    let presets_path = presets_path(assets);

    if presets_path.exists() {
        let global: GlobalConfig = load_from_json(&config_path).ok().unwrap_or_default();
        let presets: Presets = load_from_json(&presets_path).ok().unwrap_or_default();
        return (global, presets);
    }

    if let Ok(legacy) = load_from_json::<_, LegacyConfig>(&config_path) {
        let global = legacy.global;
        let presets = legacy.presets.unwrap_or_default();
        // Persist the split files so the legacy path is only hit once.
        let _ = save_to_json(&config_path, &global);
        let _ = save_to_json(&presets_path, &presets);
        return (global, presets);
    }

    (GlobalConfig::default(), Presets::default())
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
        Presets {
            selected_preset_name: default::presets::selected_preset_name(),
            selected_preset_idx: default::presets::selected_preset_idx(),
            list: default::presets::list(),
        }
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
            name: default::presets::selected_preset_name(),
            shader_left: default::preset::shader_left(),
            shader_right: default::preset::shader_right(),
            left_right_mix: default::preset::left_right_mix(),
            colourise: default::preset::colourise(),
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
        use shader_shared::{BlendMode, Shader};
        pub fn shader_left() -> Shader {
            Shader::SatisSpiraling
        }
        pub fn shader_right() -> Shader {
            Shader::SolidHsvColour
        }
        pub fn colourise() -> Shader {
            Shader::SolidHsvColour
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
