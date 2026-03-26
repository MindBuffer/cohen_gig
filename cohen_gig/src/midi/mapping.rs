use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::Path;

pub const MAX_SHADER_PARAMS: u8 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MidiTarget {
    // Blending/Mix
    LeftRightMix,
    SmoothingSpeed,
    FadeToBlack,
    // Audio
    AudioGain,
    AudioThreshold,
    AudioAttack,
    AudioHold,
    AudioRelease,
    // Colour
    ColourChannel1,
    ColourChannel2,
    ColourChannel3,
    ColourPalette,
    // Shader params (index 0–5)
    ShaderLeftParam(u8),
    ShaderRightParam(u8),
    // Shader mod amounts (index 0–5)
    ShaderLeftMod(u8),
    ShaderRightMod(u8),
}

impl MidiTarget {
    pub fn all() -> Vec<MidiTarget> {
        let mut targets = vec![
            // Blending/Mix
            MidiTarget::LeftRightMix,
            MidiTarget::SmoothingSpeed,
            MidiTarget::FadeToBlack,
            // Audio
            MidiTarget::AudioGain,
            MidiTarget::AudioThreshold,
            MidiTarget::AudioAttack,
            MidiTarget::AudioHold,
            MidiTarget::AudioRelease,
            // Colour
            MidiTarget::ColourChannel1,
            MidiTarget::ColourChannel2,
            MidiTarget::ColourChannel3,
            MidiTarget::ColourPalette,
        ];
        // Shader Left: params then mods.
        for i in 0..MAX_SHADER_PARAMS {
            targets.push(MidiTarget::ShaderLeftParam(i));
        }
        for i in 0..MAX_SHADER_PARAMS {
            targets.push(MidiTarget::ShaderLeftMod(i));
        }
        // Shader Right: params then mods.
        for i in 0..MAX_SHADER_PARAMS {
            targets.push(MidiTarget::ShaderRightParam(i));
        }
        for i in 0..MAX_SHADER_PARAMS {
            targets.push(MidiTarget::ShaderRightMod(i));
        }
        targets
    }

    pub fn label(&self) -> &'static str {
        match self {
            MidiTarget::LeftRightMix => "Left/Right Mix",
            MidiTarget::SmoothingSpeed => "Smoothing Speed",
            MidiTarget::FadeToBlack => "Fade to Black",
            MidiTarget::AudioGain => "Audio Gain",
            MidiTarget::AudioThreshold => "Audio Threshold",
            MidiTarget::AudioAttack => "Audio Attack",
            MidiTarget::AudioHold => "Audio Hold",
            MidiTarget::AudioRelease => "Audio Release",
            MidiTarget::ColourChannel1 => "Colour Ch 1 (R/H)",
            MidiTarget::ColourChannel2 => "Colour Ch 2 (G/S)",
            MidiTarget::ColourChannel3 => "Colour Ch 3 (B/V)",
            MidiTarget::ColourPalette => "Colour Palette",
            MidiTarget::ShaderLeftParam(n) | MidiTarget::ShaderRightParam(n) => match n {
                0 => "param 1",
                1 => "param 2",
                2 => "param 3",
                3 => "param 4",
                4 => "param 5",
                5 => "param 6",
                _ => "param ?",
            },
            MidiTarget::ShaderLeftMod(n) | MidiTarget::ShaderRightMod(n) => match n {
                0 => "mod 1",
                1 => "mod 2",
                2 => "mod 3",
                3 => "mod 4",
                4 => "mod 5",
                5 => "mod 6",
                _ => "mod ?",
            },
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            MidiTarget::LeftRightMix | MidiTarget::SmoothingSpeed | MidiTarget::FadeToBlack => {
                "Blending/Mix"
            }
            MidiTarget::AudioGain
            | MidiTarget::AudioThreshold
            | MidiTarget::AudioAttack
            | MidiTarget::AudioHold
            | MidiTarget::AudioRelease => "Audio",
            MidiTarget::ColourChannel1
            | MidiTarget::ColourChannel2
            | MidiTarget::ColourChannel3
            | MidiTarget::ColourPalette => "Colour",
            MidiTarget::ShaderLeftParam(_) | MidiTarget::ShaderLeftMod(_) => "Shader Left",
            MidiTarget::ShaderRightParam(_) | MidiTarget::ShaderRightMod(_) => "Shader Right",
        }
    }

    pub fn categories() -> &'static [&'static str] {
        &[
            "Blending/Mix",
            "Audio",
            "Colour",
            "Shader Left",
            "Shader Right",
        ]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MidiMappingEntry {
    pub port_name: String,
    pub cc: u8,
    pub target: MidiTarget,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MidiMappingPreset {
    pub name: String,
    pub entries: Vec<MidiMappingEntry>,
}

impl MidiMappingPreset {
    pub fn new(name: String) -> Self {
        Self {
            name,
            entries: Vec::new(),
        }
    }

    pub fn lookup_table(&self) -> HashMap<(String, u8), MidiTarget> {
        self.entries
            .iter()
            .map(|e| ((e.port_name.clone(), e.cc), e.target))
            .collect()
    }

    pub fn save(&self, dir: &Path) -> io::Result<()> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(format!("{}.json", self.name));
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    pub fn list_presets(dir: &Path) -> Vec<String> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        entries
            .filter_map(|e| {
                let e = e.ok()?;
                let path = e.path();
                if path.extension()?.to_str()? == "json" {
                    path.file_stem()?.to_str().map(String::from)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for MidiMappingPreset {
    fn default() -> Self {
        Self::new("Default".to_string())
    }
}

pub struct MidiMapping {
    pub preset: MidiMappingPreset,
    lookup: HashMap<(String, u8), MidiTarget>,
}

impl MidiMapping {
    pub fn new(preset: MidiMappingPreset) -> Self {
        let lookup = preset.lookup_table();
        Self { preset, lookup }
    }

    pub fn set_preset(&mut self, preset: MidiMappingPreset) {
        self.lookup = preset.lookup_table();
        self.preset = preset;
    }

    pub fn assign(&mut self, port_name: String, cc: u8, target: MidiTarget) {
        // Remove any existing entry for this target.
        self.preset.entries.retain(|e| e.target != target);
        // Remove any existing entry for this (port, cc) combo.
        self.preset
            .entries
            .retain(|e| !(e.port_name == port_name && e.cc == cc));
        self.preset.entries.push(MidiMappingEntry {
            port_name,
            cc,
            target,
        });
        self.lookup = self.preset.lookup_table();
    }

    #[allow(dead_code)]
    pub fn unassign(&mut self, target: MidiTarget) {
        self.preset.entries.retain(|e| e.target != target);
        self.lookup = self.preset.lookup_table();
    }

    pub fn target_for(&self, port_name: &str, cc: u8) -> Option<MidiTarget> {
        self.lookup.get(&(port_name.to_string(), cc)).copied()
    }

    pub fn entry_for(&self, target: MidiTarget) -> Option<&MidiMappingEntry> {
        self.preset.entries.iter().find(|e| e.target == target)
    }
}

impl Default for MidiMapping {
    fn default() -> Self {
        Self::new(MidiMappingPreset::default())
    }
}
