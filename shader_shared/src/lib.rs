//! Items shared between the hotloaded shader file and the `cohen_gig` executable. This is
//! important in order to ensure types are laid out the same way between the dynamic library and
//! the exe.

use nannou::prelude::*;
use serde::{Deserialize, Serialize};

/// Attributes unique to each vertex.
#[derive(Copy, Clone)]
pub struct Vertex {
    /// Positioned normalised across the entire venue space.
    pub position: Point3,
    /// Information specific to the light fixture type.
    pub light: Light,
    /// The last colour produced by the shader for this vertex.
    pub last_color: LinSrgb,
    /// Amount of interpolation 
    pub lerp_amt: f32,
}

#[derive(Clone)]
pub struct MixingInfo {
    pub left_name: String,
    pub right_name: String,
    pub colour_name: String,
    pub blend_mode: String,
    /// x fade left amount
    pub xfade_left: f32,
    /// x fade right amount
    pub xfade_right: f32,
}

#[derive(Copy, Clone)]
pub enum Light {
    /// Wash light info.
    Wash {
        /// The index of the light within the layout.
        index: usize,
    },
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
pub struct Uniforms {
    pub time: f32,
    pub resolution: Vector2,
    pub use_midi: bool,
    pub slider1: f32,
    pub slider2: f32,
    pub slider3: f32,
    pub slider4: f32,
    pub slider5: f32,
    pub slider6: f32,
    pub pot6: f32,
    pub pot7: f32,
    pub pot8: f32,
    pub params: ShaderParams,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShaderParams {
    pub acid_gradient: AcidGradient,
    pub blinky_circles: BlinkyCircles,
    pub bw_gradient: BwGradient,
    pub colour_grid: ColourGrid,
    pub escher_tilings: EscherTilings,
    pub gilmore_acid: GilmoreAcid,
    pub just_relax: JustRelax,
    pub life_led_wall: LifeLedWall,
    pub line_gradient: LineGradient,
    pub metafall: Metafall,
    pub particle_zoom: ParticleZoom,
    pub radial_lines: RadialLines,
    pub satis_spiraling: SatisSpiraling,
    pub spiral_intersect: SpiralIntersect,
    pub square_tunnel: SquareTunnel,
    pub the_pulse: ThePulse,
    pub tunnel_projection: TunnelProjection,
    pub vert_colour_gradient: VertColourGradient,
    pub solid_hsv_colour: SolidHsvColour,
    pub solid_rgb_colour: SolidRgbColour,
    pub colour_palettes: ColourPalettes,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AcidGradient {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub zoom: f32,
    #[serde(default)]
    pub offset: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlinkyCircles {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub zoom: f32,
    #[serde(default)]
    pub offset: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BwGradient {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub dc: f32,
    #[serde(default)]
    pub amp: f32,
    #[serde(default)]
    pub freq: f32,
    #[serde(default)]
    pub mirror: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColourGrid {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub zoom_amount: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EscherTilings {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub scale: f32,
    #[serde(default)]
    pub shape_iter: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GilmoreAcid {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub displace: f32,
    #[serde(default)]
    pub colour_offset: f32,
    #[serde(default)]
    pub grid_size: f32,
    #[serde(default)]
    pub wave: f32,
    #[serde(default)]
    pub zoom_amount: f32,
    #[serde(default)]
    pub rotation_amount: f32,
    #[serde(default)]
    pub brightness: f32,
    #[serde(default)]
    pub saturation: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JustRelax {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub shape_offset: f32,
    #[serde(default)]
    pub iter: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LifeLedWall {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub size: f32,
    #[serde(default)]
    pub red: f32,
    #[serde(default)]
    pub green: f32,
    #[serde(default)]
    pub blue: f32,
    #[serde(default)]
    pub saturation: f32,
    #[serde(default)]
    pub colour_offset: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LineGradient {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub num_stripes: f32,
    #[serde(default)]
    pub stripe_width: f32,
    #[serde(default)]
    pub angle: f32,
    #[serde(default)]
    pub smooth_width: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Metafall {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub scale: f32,
    #[serde(default)]
    pub red: f32,
    #[serde(default)]
    pub green: f32,
    #[serde(default)]
    pub blue: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParticleZoom {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub density: f32,
    #[serde(default)]
    pub shape: f32,
    #[serde(default)]
    pub tau: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RadialLines {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub zoom_amount: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SatisSpiraling {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub loops: f32,
    #[serde(default)]
    pub mirror: bool,
    #[serde(default)]
    pub rotate: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpiralIntersect {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub g1: f32,
    #[serde(default)]
    pub g2: f32,
    #[serde(default)]
    pub rot1: f32,
    #[serde(default)]
    pub rot2: f32,
    #[serde(default)]
    pub colours: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SquareTunnel {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub rotation_speed: f32,
    #[serde(default)]
    pub rotation_offset: f32,
    #[serde(default)]
    pub zoom: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThePulse {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub scale: f32,
    #[serde(default)]
    pub colour_iter: f32,
    #[serde(default)]
    pub thickness: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TunnelProjection {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub res: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VertColourGradient {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub scale: f32,
    #[serde(default)]
    pub colour_iter: f32,
    #[serde(default)]
    pub line_amp: f32,
    #[serde(default)]
    pub diag_amp: f32,
    #[serde(default)]
    pub boarder_amp: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SolidHsvColour {
    #[serde(default)]
    pub hue: f32,
    #[serde(default)]
    pub saturation: f32,
    #[serde(default)]
    pub value: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SolidRgbColour {
    #[serde(default)]
    pub red: f32,
    #[serde(default)]
    pub green: f32,
    #[serde(default)]
    pub blue: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColourPalettes {
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub interval: f32,
    #[serde(default)]
    pub selected: usize,
}