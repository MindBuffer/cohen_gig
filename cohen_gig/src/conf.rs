use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use shader_shared::{BlendMode, Shader, ShaderParams};

/// Runtime configuration parameters.
///
/// These are loaded from `assets/config.json` when the program starts and then saved when the
/// program closes.
///
/// If no `assets/config.json` exists, a default one will be created.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Whether or not OSC is enabled.
    #[serde(default)]
    pub osc_on: bool,
    /// Whether or not DMX is enabled.
    #[serde(default)]
    pub dmx_on: bool,
    /// Whether or not MIDI is enabled.
    #[serde(default)]
    pub midi_on: bool,
    /// A map from the layout index of each wash to their starting DMX address.
    #[serde(default = "default::wash_dmx_addrs")]
    pub wash_dmx_addrs: Box<[u8; crate::layout::WASH_COUNT]>,
    /// A map from the index of each spotlight to their starting DMX address.
    #[serde(default = "default::spot_dmx_addrs")]
    pub spot_dmx_addrs: [u8; crate::SPOT_COUNT],
    /// The universe on which wash/spot data is sent.
    #[serde(default = "default::wash_spot_universe")]
    pub wash_spot_universe: u16,
    /// The starting universe from which LED data is sent.
    #[serde(default = "default::led_start_universe")]
    pub led_start_universe: u16,
    #[serde(default)]
    pub fade_to_black: FadeToBlack,
    #[serde(default = "default::osc_addr_textbox_string")]
    pub osc_addr_textbox_string: String,
    #[serde(default)]
    pub presets: Presets,
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
    #[serde(default = "default::preset::wash_lerp_amt")]
    pub wash_lerp_amt: f32,
    #[serde(default = "default::preset::blend_mode")]
    pub blend_mode: BlendMode,
    #[serde(default)]
    pub shader_params: ShaderParams,
}

/// Fade to black parameters for each kind of fixture.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FadeToBlack {
    #[serde(default = "default::fade_to_black::led")]
    pub led: f32,
    #[serde(default = "default::fade_to_black::wash")]
    pub wash: f32,
    #[serde(default = "default::fade_to_black::spot1")]
    pub spot1: f32,
    #[serde(default = "default::fade_to_black::spot2")]
    pub spot2: f32,
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
            osc_on: Default::default(),
            dmx_on: Default::default(),
            midi_on: Default::default(),
            wash_dmx_addrs: default::wash_dmx_addrs(),
            spot_dmx_addrs: default::spot_dmx_addrs(),
            wash_spot_universe: default::wash_spot_universe(),
            led_start_universe: default::led_start_universe(),
            fade_to_black: Default::default(),
            osc_addr_textbox_string: default::osc_addr_textbox_string(),
            presets: Default::default(),
        }
    }
}

impl Default for FadeToBlack {
    fn default() -> Self {
        FadeToBlack {
            led: default::fade_to_black::led(),
            wash: default::fade_to_black::wash(),
            spot1: default::fade_to_black::spot1(),
            spot2: default::fade_to_black::spot2(),
        }
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
            wash_lerp_amt: default::preset::wash_lerp_amt(),
            colourise: default::preset::colourise(),
            blend_mode: default::preset::blend_mode(),
            shader_params: shader_shared::ShaderParams::default(),
        }
    }
}

pub mod default {
    /// The default starting dmx address for each wash light.
    pub fn wash_dmx_addrs() -> Box<[u8; crate::layout::WASH_COUNT]> {
        let mut wash_dmx_addrs = Box::new([0; crate::layout::WASH_COUNT]);
        for (i, w) in wash_dmx_addrs.iter_mut().enumerate() {
            *w = i as u8 * crate::DMX_ADDRS_PER_WASH;
        }
        wash_dmx_addrs
    }

    /// The default starting dmx address for each spot light.
    pub fn spot_dmx_addrs() -> [u8; crate::SPOT_COUNT] {
        let mut spot_dmx_addrs = [0; crate::SPOT_COUNT];
        let start_addr = crate::layout::WASH_COUNT as u8 * crate::DMX_ADDRS_PER_WASH;
        for (i, s) in spot_dmx_addrs.iter_mut().enumerate() {
            *s = start_addr + i as u8 * crate::DMX_ADDRS_PER_SPOT;
        }
        spot_dmx_addrs
    }

    /// The default universe to which wash and spot data is sent.
    pub fn wash_spot_universe() -> u16 {
        1
    }

    /// The default universe to which LED data is sent.
    pub fn led_start_universe() -> u16 {
        wash_spot_universe() + 1
    }

    pub fn osc_addr_textbox_string() -> String {
        "127.0.0.1:8000".to_string()
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
        pub fn wash_lerp_amt() -> f32 {
            0.5
        }
        pub fn blend_mode() -> BlendMode {
            BlendMode::Add
        }
    }

    pub mod fade_to_black {
        pub fn led() -> f32 {
            1.0
        }
        pub fn wash() -> f32 {
            1.0
        }
        pub fn spot1() -> f32 {
            1.0
        }
        pub fn spot2() -> f32 {
            1.0
        }
    }
}

