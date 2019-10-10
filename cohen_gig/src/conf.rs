use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use shader_shared::ShaderParams;

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
    
    #[serde(default = "default::osc_addr_textbox_string")]
    pub osc_addr_textbox_string: String,
    #[serde(default = "default::shader_names")]
    pub shader_names: Vec<String>,
    #[serde(default = "default::solid_colour_names")]
    pub solid_colour_names: Vec<String>,
    #[serde(default = "default::blend_mode_names")]
    pub blend_mode_names: Vec<String>,

    //-----------------PRESETS
    #[serde(default)]
    pub presets: Presets,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Presets {
    #[serde(default = "default::presets::selected_preset_name")]
    pub selected_preset_name: String,
    #[serde(default = "default::presets::selected_preset_idx")]
    pub selected_preset_idx: Option<usize>,
    #[serde(default = "default::presets::presets")]
    pub presets: Vec<Preset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Preset {
    //-----------------LAYER SELECT
    #[serde(default)]
    pub state: State,
    #[serde(default)]
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct State {
    #[serde(default = "default::state::led_shader_idx_left")]
    pub led_shader_idx_left: Option<usize>,
    #[serde(default = "default::state::led_shader_idx_right")]
    pub led_shader_idx_right: Option<usize>,
    #[serde(default = "default::state::led_left_right_mix")]
    pub led_left_right_mix: f32,
    #[serde(default = "default::state::led_fade_to_black")]
    pub led_fade_to_black: f32,
    #[serde(default = "default::state::wash_fade_to_black")]
    pub wash_fade_to_black: f32,
    #[serde(default = "default::state::spot_light1_fade_to_black")]
    pub spot_light1_fade_to_black: f32,
    #[serde(default = "default::state::spot_light2_fade_to_black")]
    pub spot_light2_fade_to_black: f32,
    #[serde(default = "default::state::lerp_amt")]
    pub lerp_amt: f32,
    #[serde(default = "default::state::solid_colour_idx")]
    pub solid_colour_idx: Option<usize>,
    #[serde(default = "default::state::blend_mode_idx")]
    pub blend_mode_idx: Option<usize>,
    #[serde(default)]  
    pub shader_params: ShaderParams,
}



/// The path to the configuration file.
pub fn path(assets: &Path) -> PathBuf {
    assets.join("config.json")
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
            osc_addr_textbox_string: default::osc_addr_textbox_string(),
            shader_names: default::shader_names(),
            solid_colour_names: default::solid_colour_names(),
            blend_mode_names: default::blend_mode_names(),
            presets: Default::default(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            led_shader_idx_left: default::state::led_shader_idx_left(),
            led_shader_idx_right: default::state::led_shader_idx_right(),
            led_left_right_mix: default::state::led_left_right_mix(),
            led_fade_to_black: default::state::led_fade_to_black(),
            wash_fade_to_black: default::state::wash_fade_to_black(),
            spot_light1_fade_to_black: default::state::spot_light1_fade_to_black(),
            spot_light2_fade_to_black: default::state::spot_light2_fade_to_black(),
            lerp_amt: default::state::lerp_amt(),
            solid_colour_idx: default::state::solid_colour_idx(),
            blend_mode_idx: default::state::blend_mode_idx(),
            shader_params: shader_shared::ShaderParams::default(),
        }
    }
}

impl Default for Presets {
    fn default() -> Self {
        Presets {
            selected_preset_name: default::presets::selected_preset_name(),
            selected_preset_idx: default::presets::selected_preset_idx(),
            presets: default::presets::presets(),
        }
    }
}

impl Default for Preset {
    fn default() -> Self {
        Preset {
            state: Default::default(),
            name: "init".to_string(),
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
        let addr = "127.0.0.1:8000".parse().unwrap();
        format!("{}", addr)
    }

    pub fn shader_names() -> Vec<String> {
        let mut shader_names = Vec::new();
        shader_names.push("BwGradient".to_string());
        shader_names.push("EscherTilings".to_string());
        shader_names.push("JustRelax".to_string());
        shader_names.push("LineGradient".to_string());
        shader_names.push("Metafall".to_string());
        shader_names.push("ParticleZoom".to_string());
        shader_names.push("RadialLines".to_string());
        shader_names.push("SquareTunnel".to_string());

        shader_names.push("AcidGradient".to_string());
        shader_names.push("BlinkyCircles".to_string());
        shader_names.push("ColourGrid".to_string());
        shader_names.push("GilmoreAcid".to_string());
        shader_names.push("LifeLedWall".to_string());
        shader_names.push("SatisSpiraling".to_string());
        shader_names.push("SpiralIntersect".to_string());
        shader_names.push("ThePulse".to_string());
        shader_names.push("TunnelProjection".to_string());
        shader_names.push("VertColourGradient".to_string());

        shader_names.push("SolidHsvColour".to_string());
        shader_names.push("SolidRgbColour".to_string());
        shader_names.push("ColourPalettes".to_string());
        shader_names
    }

    pub fn solid_colour_names() -> Vec<String> {
        let mut solid_colour_names = Vec::new();
        solid_colour_names.push("SolidHsvColour".to_string());
        solid_colour_names.push("SolidRgbColour".to_string());
        solid_colour_names.push("ColourPalettes".to_string());
        solid_colour_names
    }

    pub fn blend_mode_names() -> Vec<String> {
        let mut blend_mode_names = Vec::new();
        blend_mode_names.push("Add".to_string());
        blend_mode_names.push("Subtract".to_string());
        blend_mode_names.push("Multiply".to_string());
        blend_mode_names.push("Average".to_string());
        blend_mode_names.push("Difference".to_string());
        blend_mode_names.push("Negation".to_string());
        blend_mode_names.push("Exclusion".to_string());
        blend_mode_names
    }

    pub mod presets {
        pub fn selected_preset_name() -> String {
            "Empty".to_string()
        }
        pub fn selected_preset_idx() -> Option<usize> {
            Some(0)
        }
        pub fn presets() -> Vec<crate::conf::Preset> {
            vec![crate::conf::Preset::default()]
        }
    }

    pub mod state {
        pub fn led_shader_idx_left() -> Option<usize> {
            Some(15)
        }
        pub fn led_shader_idx_right() -> Option<usize> {
            Some(0)
        }
        pub fn led_left_right_mix() -> f32 {
            0.0
        }
        pub fn led_fade_to_black() -> f32 { 
            1.0
        }
        pub fn wash_fade_to_black() -> f32 { 
            1.0
        }
        pub fn spot_light1_fade_to_black() -> f32 { 
            1.0
        }
        pub fn spot_light2_fade_to_black() -> f32 { 
            1.0
        }
        pub fn lerp_amt() -> f32 {
            0.5
        }
        pub fn solid_colour_idx() -> Option<usize> {
            Some(0)
        }
        pub fn blend_mode_idx() -> Option<usize> { 
            Some(0)
        }
    }
}

