//! Items shared between the hotloaded shader file and the `cohen_gig` executable. This is
//! important in order to ensure types are laid out the same way between the dynamic library and
//! the exe.

use devault::Devault;
use nannou_core::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_half() -> f32 {
    0.5
}

// Button-related types (previously from korg_nano_kontrol_2).

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum TrackButton {
    Left,
    Right,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum MarkerButton {
    Set,
    Left,
    Right,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Transport {
    Rewind,
    Fastforward,
    Stop,
    Play,
    Record,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ButtonRow {
    Solo,
    Mute,
    Record,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum State {
    On,
    Off,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Strip {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

/// Attributes unique to each vertex.
#[derive(Copy, Clone)]
pub struct Vertex {
    /// Positioned normalised across the entire venue space.
    pub position: Point3,
    /// Information specific to the light fixture type.
    pub light: Light,
    /// The last colour produced by the shader for this vertex.
    pub last_color: LinSrgb,
}

#[derive(Copy, Clone)]
pub enum Light {
    /// Single LED light info.
    Led {
        /// The index of the LED within all LEDs.
        index: usize,
        /// The column and row indices respectively.
        col_row: [usize; 2],
        /// The coordinates of the light normalised to the bounds of the LED strips.
        ///
        /// - Left edge is -1.0
        /// - Right edge is 1.0
        /// - Bottom edge is -1.0
        /// - Top edge is 1.0
        normalised_coords: Point2,
    },
}

/// Data that is uniform across all shader calls for a single frame.
#[repr(C)]
#[derive(Clone)]
pub struct Uniforms {
    pub time: f32,
    pub resolution: Vec2,
    pub pot6: f32,
    pub pot7: f32,
    pub pot8: f32,
    pub params: ShaderParams,
    pub mix: MixingInfo,
    /// Only contains buttons that have been pressed at least once.
    pub buttons: HashMap<Button, ButtonState>,
}

/// Describes one of the buttons on the korg.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Button {
    Row(ButtonRow, Strip),
    Track(TrackButton),
    Cycle,
    Marker(MarkerButton),
    Transport(Transport),
}

/// The state of a button that has been interacted with.
#[derive(Clone)]
pub struct ButtonState {
    /// Seconds since the button was pressed.
    pub secs: f32,
    /// The current state of the button (on or off).
    pub state: State,
}

#[derive(Clone)]
pub struct MixingInfo {
    pub left: Shader,
    pub right: Shader,
    pub colourise: Shader,
    pub blend_mode: BlendMode,
    /// x fade left amount
    pub xfade_left: f32,
    /// x fade right amount
    pub xfade_right: f32,
    /// Per-slot shader params so each slot is independent.
    pub params_left: ShaderParams,
    pub params_right: ShaderParams,
    pub params_colourise: ShaderParams,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ShaderParams {
    #[serde(default)]
    pub acid_gradient: AcidGradient,
    #[serde(default)]
    pub blinky_circles: BlinkyCircles,
    #[serde(default)]
    pub bw_gradient: BwGradient,
    #[serde(default)]
    pub colour_grid: ColourGrid,
    #[serde(default)]
    pub escher_tilings: EscherTilings,
    #[serde(default)]
    pub gilmore_acid: GilmoreAcid,
    #[serde(default)]
    pub gradient_bars: GradientBars,
    #[serde(default)]
    pub just_relax: JustRelax,
    #[serde(default)]
    pub life_led_wall: LifeLedWall,
    #[serde(default)]
    pub light_pattern_generator: LightPatternGenerator,
    #[serde(default)]
    pub line_gradient: LineGradient,
    #[serde(default)]
    pub metafall: Metafall,
    #[serde(default)]
    pub particle_zoom: ParticleZoom,
    #[serde(default)]
    pub radial_lines: RadialLines,
    #[serde(default)]
    pub satis_spiraling: SatisSpiraling,
    #[serde(default)]
    pub spiral_intersect: SpiralIntersect,
    #[serde(default)]
    pub square_tunnel: SquareTunnel,
    #[serde(default)]
    pub the_pulse: ThePulse,
    #[serde(default)]
    pub tunnel_projection: TunnelProjection,
    #[serde(default)]
    pub vert_colour_gradient: VertColourGradient,
    #[serde(default)]
    pub solid_hsv_colour: SolidHsvColour,
    #[serde(default)]
    pub solid_rgb_colour: SolidRgbColour,
    #[serde(default)]
    pub colour_palettes: ColourPalettes,
    #[serde(default)]
    pub mitch_wash: MitchWash,
    #[serde(default)]
    pub shape_envelopes: ShapeEnvelopes,
    #[serde(default)]
    pub row_test: RowTest,
    #[serde(default)]
    pub bar_test: BarTest,
}

/// Refers to the selected blend mode type for a preset.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlendMode {
    Add,
    Subtract,
    Multiply,
    Average,
    Difference,
    Negation,
    Exclusion,
}

/// For selecting between each of the available shaders at runtime.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Shader {
    SolidHsvColour,
    SolidRgbColour,
    ColourPalettes,
    AcidGradient,
    BlinkyCircles,
    BwGradient,
    ColourGrid,
    EscherTilings,
    GilmoreAcid,
    GradientBars,
    JustRelax,
    LifeLedWall,
    LightPatternGenerator,
    LineGradient,
    Metafall,
    ParticleZoom,
    RadialLines,
    SatisSpiraling,
    SpiralIntersect,
    SquareTunnel,
    ThePulse,
    TunnelProjection,
    VertColourGradient,
    MitchWash,
    ShapeEnvelopes,
    RowTest,
    BarTest,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct AcidGradient {
    #[devault("0.5125")]
    pub speed: f32,
    #[devault("0.0")]
    pub zoom: f32,
    #[devault("0.75")]
    pub offset: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct BlinkyCircles {
    #[devault("0.5125")]
    pub speed: f32,
    #[devault("0.05")]
    pub zoom: f32,
    #[devault("0.25")]
    pub offset: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct BwGradient {
    #[devault("0.5125")]
    pub speed: f32,
    #[devault("0.05")]
    pub dc: f32,
    #[devault("0.5")]
    pub amp: f32,
    #[devault("0.5")]
    pub freq: f32,
    #[devault("false")]
    pub mirror: bool,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct ColourGrid {
    #[devault("0.5")]
    pub speed: f32,
    #[devault("0.1")]
    pub zoom_amount: f32,
    #[serde(default = "default_half")]
    #[devault("0.5")]
    pub colour_amount: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct EscherTilings {
    #[devault("0.2")]
    pub speed: f32,
    #[devault("0.2")]
    pub scale: f32,
    #[devault("0.2")]
    pub shape_iter: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct GilmoreAcid {
    #[devault("0.025")]
    pub speed: f32,
    #[devault("0.01")]
    pub displace: f32,
    #[devault("0.85")]
    pub colour_offset: f32,
    #[devault("0.345")]
    pub grid_size: f32,
    #[devault("0.088")]
    pub wave: f32,
    #[devault("0.0")]
    pub zoom_amount: f32,
    #[devault("0.0")]
    pub rotation_amount: f32,
    #[devault("1.0")]
    pub brightness: f32,
    #[devault("0.15")]
    pub saturation: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct GradientBars {
    #[devault("0")]
    pub easing_type: usize,
    #[devault("0.2")]
    pub gradient_pow: f32,
    #[devault("0.15")]
    pub balance: f32,
    #[devault("0.1")]
    pub speed: f32,
    #[devault("0.2")]
    pub invert_speed: f32,
    #[devault("4.0")]
    pub offset: f32,
    #[devault("false")]
    pub use_odd_dirs: bool,
    #[devault("2.0")]
    pub phase_iter: f32,
    #[devault("15.0")]
    pub num_columns: f32,
    #[devault("2.0")]
    pub x_iter: f32,
    #[devault("false")]
    pub use_columns: bool,
}

pub const GRADIENT_BARS_EASING_LABELS: &[&str] = &[
    "Ease In Sine",
    "Ease Out Sine",
    "Ease InOut Sine",
    "Ease In Quad",
    "Ease Out Quad",
    "Ease InOut Quad",
    "Ease In Cubic",
    "Ease Out Cubic",
    "Ease InOut Cubic",
    "Ease In Quart",
    "Ease Out Quart",
    "Ease InOut Quart",
    "Ease In Quint",
    "Ease Out Quint",
    "Ease InOut Quint",
    "Ease In Expo",
    "Ease Out Expo",
    "Ease InOut Expo",
    "Ease In Circ",
    "Ease Out Circ",
    "Ease InOut Circ",
    "Ease In Back",
    "Ease Out Back",
    "Ease InOut Back",
    "Ease In Elastic",
    "Ease Out Elastic",
    "Ease InOut Elastic",
    "Ease In Bounce",
    "Ease Out Bounce",
    "Ease InOut Bounce",
];

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct JustRelax {
    #[devault("0.6")]
    pub speed: f32,
    #[devault("0.728")]
    pub shape_offset: f32,
    #[devault("1.0")]
    pub iter: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct LifeLedWall {
    #[devault("0.25")]
    pub speed: f32,
    #[devault("0.73")]
    pub size: f32,
    #[devault("0.5")]
    pub red: f32,
    #[devault("0.2")]
    pub green: f32,
    #[devault("0.1")]
    pub blue: f32,
    #[devault("1.0")]
    pub saturation: f32,
    #[devault("0.01")]
    pub colour_offset: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct LightPatternGenerator {
    #[devault("0.5")]
    pub zoom: f32,
    #[devault("0.2")]
    pub offset: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct LineGradient {
    #[devault("0.03")]
    pub speed: f32,
    #[devault("1.0")]
    pub num_stripes: f32,
    #[devault("0.9")]
    pub stripe_width: f32,
    #[devault("0.5")]
    pub angle: f32,
    #[devault("0.155")]
    pub smooth_width: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct Metafall {
    #[devault("0.47")]
    pub speed: f32,
    #[devault("0.0")]
    pub scale: f32,
    #[devault("1.0")]
    pub red: f32,
    #[devault("1.0")]
    pub green: f32,
    #[devault("1.0")]
    pub blue: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct ParticleZoom {
    #[devault("0.01")]
    pub speed: f32,
    #[devault("0.01")]
    pub density: f32,
    #[devault("0.35")]
    pub shape: f32,
    #[devault("1.0")]
    pub tau: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct RadialLines {
    #[devault("0.05")]
    pub speed: f32,
    #[devault("0.8")]
    pub zoom_amount: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct SatisSpiraling {
    #[devault("0.5")]
    pub speed: f32,
    #[devault("0.8")]
    pub loops: f32,
    #[devault("true")]
    pub mirror: bool,
    #[devault("true")]
    pub rotate: bool,
    #[serde(default)]
    #[devault("0.0")]
    pub colour_offset: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct SpiralIntersect {
    #[devault("0.02")]
    pub speed: f32,
    #[devault("0.4")]
    pub g1: f32,
    #[devault("0.6")]
    pub g2: f32,
    #[devault("1.0")]
    pub rot1: f32,
    #[devault("0.5")]
    pub rot2: f32,
    #[devault("1.0")]
    pub colours: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct SquareTunnel {
    #[devault("0.6")]
    pub speed: f32,
    #[devault("0.025")]
    pub rotation_speed: f32,
    #[devault("0.0")]
    pub rotation_offset: f32,
    #[devault("0.8")]
    pub zoom: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct ThePulse {
    #[devault("0.08")]
    pub speed: f32,
    #[devault("0.1")]
    pub scale: f32,
    #[devault("0.25")]
    pub colour_iter: f32,
    #[devault("0.0")]
    pub thickness: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct TunnelProjection {
    #[devault("0.5")]
    pub speed: f32,
    #[devault("0.5")]
    pub res: f32,
    #[serde(default = "default_half")]
    #[devault("0.5")]
    pub y_offset: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct VertColourGradient {
    #[devault("0.5")]
    pub speed: f32,
    #[devault("0.83")]
    pub scale: f32,
    #[devault("0.015")]
    pub colour_iter: f32,
    #[devault("0.0")]
    pub line_amp: f32,
    #[devault("0.0")]
    pub diag_amp: f32,
    #[devault("0.65")]
    pub boarder_amp: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct MitchWash {
    #[devault("1.0")]
    pub speed: f32,
    #[devault("1.0")]
    pub pulse_speed: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct ShapeEnvelopes {
    #[devault("1.0")]
    pub speed: f32,
    #[devault("1.0")]
    pub pulse_speed: f32,
    #[devault("0.0")]
    pub line_thickness: f32,
    #[devault("0.0")]
    pub shape_thickness: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct SolidHsvColour {
    #[devault("1.0")]
    pub hue: f32,
    #[devault("0.0")]
    pub saturation: f32,
    #[devault("1.0")]
    pub value: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct SolidRgbColour {
    #[devault("1.0")]
    pub red: f32,
    #[devault("1.0")]
    pub green: f32,
    #[devault("1.0")]
    pub blue: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct ColourPalettes {
    #[devault("0.1")]
    pub speed: f32,
    #[devault("0.05")]
    pub interval: f32,
    #[devault("0")]
    pub selected: usize,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct RowTest {
    #[devault("0.0")]
    pub row: f32,
}

#[derive(Copy, Clone, Debug, Devault, PartialEq, Serialize, Deserialize)]
pub struct BarTest {
    #[devault("0.0")]
    pub row: f32,
    #[devault("0.0")]
    pub bar: f32,
}

pub const ALL_BLEND_MODES: &[BlendMode] = &[
    BlendMode::Add,
    BlendMode::Subtract,
    BlendMode::Multiply,
    BlendMode::Average,
    BlendMode::Difference,
    BlendMode::Negation,
    BlendMode::Exclusion,
];

pub const ALL_SHADERS: &[Shader] = &[
    Shader::SolidHsvColour,
    Shader::SolidRgbColour,
    Shader::ColourPalettes,
    Shader::AcidGradient,
    Shader::BlinkyCircles,
    Shader::BwGradient,
    Shader::ColourGrid,
    Shader::EscherTilings,
    Shader::GilmoreAcid,
    Shader::GradientBars,
    Shader::JustRelax,
    Shader::LifeLedWall,
    Shader::LightPatternGenerator,
    Shader::LineGradient,
    Shader::Metafall,
    Shader::ParticleZoom,
    Shader::RadialLines,
    Shader::SatisSpiraling,
    Shader::SpiralIntersect,
    Shader::SquareTunnel,
    Shader::ThePulse,
    Shader::TunnelProjection,
    Shader::VertColourGradient,
    Shader::MitchWash,
    Shader::ShapeEnvelopes,
    Shader::RowTest,
    Shader::BarTest,
];

pub const SOLID_COLOUR_SHADERS: &[Shader] = &[
    Shader::SolidHsvColour,
    Shader::SolidRgbColour,
    Shader::ColourPalettes,
];

impl BlendMode {
    /// The name of the variant in the form of a string for GUI presentation.
    pub fn name(&self) -> &str {
        match *self {
            BlendMode::Add => "Add",
            BlendMode::Subtract => "Subtract",
            BlendMode::Multiply => "Multiply",
            BlendMode::Average => "Average",
            BlendMode::Difference => "Difference",
            BlendMode::Negation => "Negation",
            BlendMode::Exclusion => "Exclusion",
        }
    }

    pub fn to_index(&self) -> usize {
        match *self {
            BlendMode::Add => 0,
            BlendMode::Subtract => 1,
            BlendMode::Multiply => 2,
            BlendMode::Average => 3,
            BlendMode::Difference => 4,
            BlendMode::Negation => 5,
            BlendMode::Exclusion => 6,
        }
    }

    pub fn from_index(index: usize) -> Option<Self> {
        let mode = match index {
            0 => BlendMode::Add,
            1 => BlendMode::Subtract,
            2 => BlendMode::Multiply,
            3 => BlendMode::Average,
            4 => BlendMode::Difference,
            5 => BlendMode::Negation,
            6 => BlendMode::Exclusion,
            _ => return None,
        };
        Some(mode)
    }
}

impl Shader {
    /// The name of the variant in the form of a string for GUI presentation.
    pub fn name(&self) -> &str {
        match *self {
            Shader::SolidHsvColour => "SolidHsvColour",
            Shader::SolidRgbColour => "SolidRgbColour",
            Shader::ColourPalettes => "ColourPalettes",
            Shader::AcidGradient => "AcidGradient",
            Shader::BlinkyCircles => "BlinkyCircles",
            Shader::BwGradient => "BwGradient",
            Shader::ColourGrid => "ColourGrid",
            Shader::EscherTilings => "EscherTilings",
            Shader::GilmoreAcid => "GilmoreAcid",
            Shader::GradientBars => "GradientBars",
            Shader::JustRelax => "JustRelax",
            Shader::LifeLedWall => "LifeLedWall",
            Shader::LightPatternGenerator => "LightPatternGenerator",
            Shader::LineGradient => "LineGradient",
            Shader::Metafall => "Metafall",
            Shader::ParticleZoom => "ParticleZoom",
            Shader::RadialLines => "RadialLines",
            Shader::SatisSpiraling => "SatisSpiraling",
            Shader::SpiralIntersect => "SpiralIntersect",
            Shader::SquareTunnel => "SquareTunnel",
            Shader::ThePulse => "ThePulse",
            Shader::TunnelProjection => "TunnelProjection",
            Shader::VertColourGradient => "VertColourGradient",
            Shader::MitchWash => "MitchWash",
            Shader::ShapeEnvelopes => "ShapeEnvelopes",
            Shader::RowTest => "RowTest",
            Shader::BarTest => "BarTest",
        }
    }

    pub fn to_index(&self) -> usize {
        match *self {
            Shader::SolidHsvColour => 0,
            Shader::SolidRgbColour => 1,
            Shader::ColourPalettes => 2,
            Shader::AcidGradient => 3,
            Shader::BlinkyCircles => 4,
            Shader::BwGradient => 5,
            Shader::ColourGrid => 6,
            Shader::EscherTilings => 7,
            Shader::GilmoreAcid => 8,
            Shader::GradientBars => 9,
            Shader::JustRelax => 10,
            Shader::LifeLedWall => 11,
            Shader::LightPatternGenerator => 12,
            Shader::LineGradient => 13,
            Shader::Metafall => 14,
            Shader::ParticleZoom => 15,
            Shader::RadialLines => 16,
            Shader::SatisSpiraling => 17,
            Shader::SpiralIntersect => 18,
            Shader::SquareTunnel => 19,
            Shader::ThePulse => 20,
            Shader::TunnelProjection => 21,
            Shader::VertColourGradient => 22,
            Shader::MitchWash => 23,
            Shader::ShapeEnvelopes => 24,
            Shader::RowTest => 25,
            Shader::BarTest => 26,
        }
    }

    pub fn from_index(index: usize) -> Option<Self> {
        let shader = match index {
            0 => Shader::SolidHsvColour,
            1 => Shader::SolidRgbColour,
            2 => Shader::ColourPalettes,
            3 => Shader::AcidGradient,
            4 => Shader::BlinkyCircles,
            5 => Shader::BwGradient,
            6 => Shader::ColourGrid,
            7 => Shader::EscherTilings,
            8 => Shader::GilmoreAcid,
            9 => Shader::GradientBars,
            10 => Shader::JustRelax,
            11 => Shader::LifeLedWall,
            12 => Shader::LightPatternGenerator,
            13 => Shader::LineGradient,
            14 => Shader::Metafall,
            15 => Shader::ParticleZoom,
            16 => Shader::RadialLines,
            17 => Shader::SatisSpiraling,
            18 => Shader::SpiralIntersect,
            19 => Shader::SquareTunnel,
            20 => Shader::ThePulse,
            21 => Shader::TunnelProjection,
            22 => Shader::VertColourGradient,
            23 => Shader::MitchWash,
            24 => Shader::ShapeEnvelopes,
            25 => Shader::RowTest,
            26 => Shader::BarTest,
            _ => return None,
        };
        Some(shader)
    }
}
