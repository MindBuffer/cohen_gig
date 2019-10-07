use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
    pub spot_dmx_addrs: [u8; crate::SPOT_COUNT]
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
}
