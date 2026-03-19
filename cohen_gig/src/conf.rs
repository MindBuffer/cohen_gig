use serde::{Deserialize, Serialize};
use shader_shared::{BlendMode, Shader, ShaderParams};
use std::net::{AddrParseError, Ipv4Addr};
use std::path::{Path, PathBuf};

/// Runtime configuration parameters.
///
/// These are loaded from `assets/config.json` when the program starts and then saved when the
/// program closes.
///
/// If no `assets/config.json` exists, a default one will be created.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Whether or not DMX is enabled.
    #[serde(default)]
    pub dmx_on: bool,
    /// Whether or not MIDI is enabled.
    #[serde(default)]
    pub midi_on: bool,
    /// The starting universe from which LED data is sent.
    #[serde(default = "default::led_start_universe")]
    pub led_start_universe: u16,
    #[serde(default)]
    pub fade_to_black: FadeToBlack,
    #[serde(default = "default::sacn_interface_ip")]
    pub sacn_interface_ip: String,
    #[serde(default)]
    pub led_layout: LedLayout,
    #[serde(default)]
    pub presets: Presets,
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
    #[serde(default)]
    pub shader_params: ShaderParams,
    #[serde(default)]
    pub shader_mod_amounts: Vec<f32>,
}

/// Fade to black parameters for each kind of fixture.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FadeToBlack {
    #[serde(default = "default::fade_to_black::led")]
    pub led: f32,
}

/// The path to the configuration file.
pub fn path(assets: &Path) -> PathBuf {
    assets.join("config.json")
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

impl Default for Config {
    fn default() -> Self {
        Config {
            dmx_on: Default::default(),
            midi_on: Default::default(),
            led_start_universe: default::led_start_universe(),
            fade_to_black: Default::default(),
            sacn_interface_ip: default::sacn_interface_ip(),
            led_layout: Default::default(),
            presets: Default::default(),
            preset_lerp_secs: Default::default(),
        }
    }
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

impl Default for Preset {
    fn default() -> Self {
        Preset {
            name: default::presets::selected_preset_name(),
            shader_left: default::preset::shader_left(),
            shader_right: default::preset::shader_right(),
            left_right_mix: default::preset::left_right_mix(),
            colourise: default::preset::colourise(),
            blend_mode: default::preset::blend_mode(),
            shader_params: shader_shared::ShaderParams::default(),
            shader_mod_amounts: Vec::new(),
        }
    }
}

pub mod default {
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
