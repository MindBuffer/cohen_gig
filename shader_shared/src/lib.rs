//! Items shared between the hotloaded shader file and the `cohen_gig` executable. This is
//! important in order to ensure types are laid out the same way between the dynamic library and
//! the exe.

use nannou::prelude::*;

/// Attributes unique to each vertex.
#[derive(Copy, Clone)]
pub struct Vertex {
    /// Positioned normalised across the entire venue space.
    pub position: Point3,
    /// Information specific to the light fixture type.
    pub light: Light,
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

#[derive(Copy,Clone)]
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
}

#[derive(Copy,Clone)]
pub struct AcidGradient {
    pub speed: f32,
    pub zoom: f32,
    pub offset: f32,
}

#[derive(Copy,Clone)]
pub struct BlinkyCircles {
    pub speed: f32,
    pub zoom: f32,
    pub offset: f32,
}

#[derive(Copy,Clone)]
pub struct BwGradient {
    pub speed: f32,
    pub dc: f32,
    pub amp: f32,
    pub freq: f32,
    pub mirror: bool,
}

#[derive(Copy,Clone)]
pub struct ColourGrid {
    pub speed: f32,
    pub zoom_amount: f32,
}

#[derive(Copy,Clone)]
pub struct EscherTilings {
    pub speed: f32,
    pub scale: f32,
    pub shape_iter: f32,
}

#[derive(Copy,Clone)]
pub struct GilmoreAcid {
    pub speed: f32,
    pub displace: f32,
    pub colour_offset: f32,
    pub grid_size: f32,
    pub wave: f32,
    pub zoom_amount: f32,
    pub rotation_amount: f32,
    pub brightness: f32,
    pub saturation: f32,
}

#[derive(Copy,Clone)]
pub struct JustRelax {
    pub speed: f32,
    pub shape_offset: f32,
    pub iter: f32,
}

#[derive(Copy,Clone)]
pub struct LifeLedWall {
    pub speed: f32,
    pub size: f32,
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub saturation: f32,
    pub colour_offset: f32,
}

#[derive(Copy,Clone)]
pub struct LineGradient {
    pub speed: f32,
    pub num_stripes: f32,
    pub stripe_width: f32,
    pub angle: f32,
    pub smooth_width: f32,
}

#[derive(Copy,Clone)]
pub struct Metafall {
    pub speed: f32,
    pub scale: f32,
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

#[derive(Copy,Clone)]
pub struct ParticleZoom {
    pub speed: f32,
    pub density: f32,
    pub shape: f32,
    pub tau: f32,
}

#[derive(Copy,Clone)]
pub struct RadialLines {
    pub speed: f32,
    pub zoom_amount: f32,
}

#[derive(Copy,Clone)]
pub struct SatisSpiraling {
    pub speed: f32,
    pub loops: f32,
    pub mirror: bool,
    pub rotate: bool,
}

#[derive(Copy,Clone)]
pub struct SpiralIntersect {
    pub speed: f32,
    pub g1: f32,
    pub g2: f32,
    pub rot1: f32,
    pub rot2: f32,
    pub colours: f32,
}

#[derive(Copy,Clone)]
pub struct SquareTunnel {
    pub speed: f32,
    pub rotation_speed: f32,
    pub rotation_offset: f32,
    pub zoom: f32,
}

#[derive(Copy,Clone)]
pub struct ThePulse {
    pub speed: f32,
    pub scale: f32,
    pub colour_iter: f32,
    pub thickness: f32,
}

#[derive(Copy,Clone)]
pub struct TunnelProjection {
    pub speed: f32,
    pub res: f32,
}

#[derive(Copy,Clone)]
pub struct VertColourGradient {
    pub speed: f32,
    pub scale: f32,
    pub colour_iter: f32,
    pub line_amp: f32,
    pub diag_amp: f32,
    pub boarder_amp: f32,
}

#[derive(Copy,Clone)]
pub struct SolidHsvColour {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
}

#[derive(Copy,Clone)]
pub struct SolidRgbColour {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}