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
    pub just_relax: JustRelax,
    #[serde(default)]
    pub life_led_wall: LifeLedWall,
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
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AcidGradient {
    #[serde(default = "default::acid_gradient::speed")]
    pub speed: f32,
    #[serde(default = "default::acid_gradient::zoom")]
    pub zoom: f32,
    #[serde(default = "default::acid_gradient::offset")]
    pub offset: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlinkyCircles {
    #[serde(default = "default::blinky_circles::speed")]
    pub speed: f32,
    #[serde(default = "default::blinky_circles::zoom")]
    pub zoom: f32,
    #[serde(default = "default::blinky_circles::offset")]
    pub offset: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BwGradient {
    #[serde(default = "default::bw_gradient::speed")]
    pub speed: f32,
    #[serde(default = "default::bw_gradient::dc")]
    pub dc: f32,
    #[serde(default = "default::bw_gradient::amp")]
    pub amp: f32,
    #[serde(default = "default::bw_gradient::freq")]
    pub freq: f32,
    #[serde(default = "default::bw_gradient::mirror")]
    pub mirror: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColourGrid {
    #[serde(default = "default::colour_grid::speed")]
    pub speed: f32,
    #[serde(default = "default::colour_grid::zoom_amount")]
    pub zoom_amount: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EscherTilings {
    #[serde(default = "default::escher_tilings::speed")]
    pub speed: f32,
    #[serde(default = "default::escher_tilings::scale")]
    pub scale: f32,
    #[serde(default = "default::escher_tilings::shape_iter")]
    pub shape_iter: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GilmoreAcid {
    #[serde(default = "default::gilmore_acid::speed")]
    pub speed: f32,
    #[serde(default = "default::gilmore_acid::displace")]
    pub displace: f32,
    #[serde(default = "default::gilmore_acid::colour_offset")]
    pub colour_offset: f32,
    #[serde(default = "default::gilmore_acid::grid_size")]
    pub grid_size: f32,
    #[serde(default = "default::gilmore_acid::wave")]
    pub wave: f32,
    #[serde(default = "default::gilmore_acid::zoom_amount")]
    pub zoom_amount: f32,
    #[serde(default = "default::gilmore_acid::rotation_amount")]
    pub rotation_amount: f32,
    #[serde(default = "default::gilmore_acid::brightness")]
    pub brightness: f32,
    #[serde(default = "default::gilmore_acid::saturation")]
    pub saturation: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JustRelax {
    #[serde(default = "default::just_relax::speed")]
    pub speed: f32,
    #[serde(default = "default::just_relax::shape_offset")]
    pub shape_offset: f32,
    #[serde(default = "default::just_relax::iter")]
    pub iter: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LifeLedWall {
    #[serde(default = "default::life_led_wall::speed")]
    pub speed: f32,
    #[serde(default = "default::life_led_wall::size")]
    pub size: f32,
    #[serde(default = "default::life_led_wall::red")]
    pub red: f32,
    #[serde(default = "default::life_led_wall::green")]
    pub green: f32,
    #[serde(default = "default::life_led_wall::blue")]
    pub blue: f32,
    #[serde(default = "default::life_led_wall::saturation")]
    pub saturation: f32,
    #[serde(default = "default::life_led_wall::colour_offset")]
    pub colour_offset: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LineGradient {
    #[serde(default = "default::line_gradient::speed")]
    pub speed: f32,
    #[serde(default = "default::line_gradient::num_stripes")]
    pub num_stripes: f32,
    #[serde(default = "default::line_gradient::stripe_width")]
    pub stripe_width: f32,
    #[serde(default = "default::line_gradient::angle")]
    pub angle: f32,
    #[serde(default = "default::line_gradient::smooth_width")]
    pub smooth_width: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Metafall {
    #[serde(default = "default::metafall::speed")]
    pub speed: f32,
    #[serde(default = "default::metafall::scale")]
    pub scale: f32,
    #[serde(default = "default::metafall::red")]
    pub red: f32,
    #[serde(default = "default::metafall::green")]
    pub green: f32,
    #[serde(default = "default::metafall::blue")]
    pub blue: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParticleZoom {
    #[serde(default = "default::particle_zoom::speed")]
    pub speed: f32,
    #[serde(default = "default::particle_zoom::density")]
    pub density: f32,
    #[serde(default = "default::particle_zoom::shape")]
    pub shape: f32,
    #[serde(default = "default::particle_zoom::tau")]
    pub tau: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RadialLines {
    #[serde(default = "default::radial_lines::speed")]
    pub speed: f32,
    #[serde(default = "default::radial_lines::zoom_amount")]
    pub zoom_amount: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SatisSpiraling {
    #[serde(default = "default::satis_spiraling::speed")]
    pub speed: f32,
    #[serde(default = "default::satis_spiraling::loops")]
    pub loops: f32,
    #[serde(default = "default::satis_spiraling::mirror")]
    pub mirror: bool,
    #[serde(default = "default::satis_spiraling::rotate")]
    pub rotate: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpiralIntersect {
    #[serde(default = "default::spiral_intersect::speed")]
    pub speed: f32,
    #[serde(default = "default::spiral_intersect::g1")]
    pub g1: f32,
    #[serde(default = "default::spiral_intersect::g2")]
    pub g2: f32,
    #[serde(default = "default::spiral_intersect::rot1")]
    pub rot1: f32,
    #[serde(default = "default::spiral_intersect::rot2")]
    pub rot2: f32,
    #[serde(default = "default::spiral_intersect::colours")]
    pub colours: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SquareTunnel {
    #[serde(default = "default::square_tunnel::speed")]
    pub speed: f32,
    #[serde(default = "default::square_tunnel::rotation_speed")]
    pub rotation_speed: f32,
    #[serde(default = "default::square_tunnel::rotation_offset")]
    pub rotation_offset: f32,
    #[serde(default = "default::square_tunnel::zoom")]
    pub zoom: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThePulse {
    #[serde(default = "default::the_pulse::speed")]
    pub speed: f32,
    #[serde(default = "default::the_pulse::scale")]
    pub scale: f32,
    #[serde(default = "default::the_pulse::colour_iter")]
    pub colour_iter: f32,
    #[serde(default = "default::the_pulse::thickness")]
    pub thickness: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TunnelProjection {
    #[serde(default = "default::tunnel_projection::speed")]
    pub speed: f32,
    #[serde(default = "default::tunnel_projection::res")]
    pub res: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VertColourGradient {
    #[serde(default = "default::vert_colour_gradient::speed")]
    pub speed: f32,
    #[serde(default = "default::vert_colour_gradient::scale")]
    pub scale: f32,
    #[serde(default = "default::vert_colour_gradient::colour_iter")]
    pub colour_iter: f32,
    #[serde(default = "default::vert_colour_gradient::line_amp")]
    pub line_amp: f32,
    #[serde(default = "default::vert_colour_gradient::diag_amp")]
    pub diag_amp: f32,
    #[serde(default = "default::vert_colour_gradient::boarder_amp")]
    pub boarder_amp: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SolidHsvColour {
    #[serde(default = "default::solid_hsv_colour::hue")]
    pub hue: f32,
    #[serde(default = "default::solid_hsv_colour::saturation")]
    pub saturation: f32,
    #[serde(default = "default::solid_hsv_colour::value")]
    pub value: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SolidRgbColour {
    #[serde(default = "default::solid_rgb_colour::red")]
    pub red: f32,
    #[serde(default = "default::solid_rgb_colour::green")]
    pub green: f32,
    #[serde(default = "default::solid_rgb_colour::blue")]
    pub blue: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColourPalettes {
    #[serde(default = "default::colour_palettes::speed")]
    pub speed: f32,
    #[serde(default = "default::colour_palettes::interval")]
    pub interval: f32,
    #[serde(default = "default::colour_palettes::selected")]
    pub selected: usize,
}

impl Default for AcidGradient {
    fn default() -> Self {
        AcidGradient {
            speed: default::acid_gradient::speed(),
            zoom: default::acid_gradient::zoom(),
            offset: default::acid_gradient::offset(),
        }
    }
}

impl Default for BlinkyCircles {
    fn default() -> Self {
        BlinkyCircles {
            speed: default::blinky_circles::speed(),
            zoom: default::blinky_circles::zoom(),
            offset: default::blinky_circles::offset(),
        }
    }
}

impl Default for BwGradient {
    fn default() -> Self {
        BwGradient {
            speed: default::bw_gradient::speed(),
            dc: default::bw_gradient::dc(),
            amp: default::bw_gradient::amp(),
            freq: default::bw_gradient::freq(),
            mirror: default::bw_gradient::mirror(),
        }
    }
}

impl Default for ColourGrid {
    fn default() -> Self {
        ColourGrid {
            speed: default::colour_grid::speed(),
            zoom_amount: default::colour_grid::zoom_amount(),
        }
    }
}

impl Default for EscherTilings {
    fn default() -> Self {
        EscherTilings {
            speed: default::escher_tilings::speed(),
            scale: default::escher_tilings::scale(),
            shape_iter: default::escher_tilings::shape_iter(),
        }
    }
}

impl Default for GilmoreAcid {
    fn default() -> Self {
        GilmoreAcid {
            speed: default::gilmore_acid::speed(),
            displace: default::gilmore_acid::displace(),
            colour_offset: default::gilmore_acid::colour_offset(),
            grid_size: default::gilmore_acid::grid_size(),
            wave: default::gilmore_acid::wave(),
            zoom_amount: default::gilmore_acid::zoom_amount(),
            rotation_amount: default::gilmore_acid::rotation_amount(),
            brightness: default::gilmore_acid::brightness(),
            saturation: default::gilmore_acid::saturation(),
        }
    }
}

impl Default for JustRelax {
    fn default() -> Self {
        JustRelax {
            speed: default::just_relax::speed(),
            shape_offset: default::just_relax::shape_offset(),
            iter: default::just_relax::iter(),
        }
    }
}

impl Default for LifeLedWall {
    fn default() -> Self {
        LifeLedWall {
            speed: default::life_led_wall::speed(),
            size: default::life_led_wall::size(),
            red: default::life_led_wall::red(),
            green: default::life_led_wall::green(),
            blue: default::life_led_wall::blue(),
            saturation: default::life_led_wall::saturation(),
            colour_offset: default::life_led_wall::colour_offset(),
        }
    }
}

impl Default for LineGradient {
    fn default() -> Self {
        LineGradient {
            speed: default::line_gradient::speed(),
            num_stripes: default::line_gradient::num_stripes(),
            stripe_width: default::line_gradient::stripe_width(),
            angle: default::line_gradient::angle(),
            smooth_width: default::line_gradient::smooth_width(),
        }
    }
}

impl Default for Metafall {
    fn default() -> Self {
        Metafall {
            speed: default::metafall::speed(),
            scale: default::metafall::scale(),
            red: default::metafall::red(),
            green: default::metafall::green(),
            blue: default::metafall::blue(),
        }
    }
}

impl Default for ParticleZoom {
    fn default() -> Self {
        ParticleZoom {
            speed: default::particle_zoom::speed(),
            density: default::particle_zoom::density(),
            shape: default::particle_zoom::shape(),
            tau: default::particle_zoom::tau(),
        }
    }
}

impl Default for RadialLines {
    fn default() -> Self {
        RadialLines {
            speed: default::radial_lines::speed(),
            zoom_amount: default::radial_lines::zoom_amount(),
        }
    }
}

impl Default for SatisSpiraling {
    fn default() -> Self {
        SatisSpiraling {
            speed: default::satis_spiraling::speed(),
            loops: default::satis_spiraling::loops(),
            mirror: default::satis_spiraling::mirror(),
            rotate: default::satis_spiraling::rotate(),
        }
    }
}

impl Default for SpiralIntersect {
    fn default() -> Self {
        SpiralIntersect {
            speed: default::spiral_intersect::speed(),
            g1: default::spiral_intersect::g1(),
            g2: default::spiral_intersect::g2(),
            rot1: default::spiral_intersect::rot1(),
            rot2: default::spiral_intersect::rot2(),
            colours: default::spiral_intersect::colours(),
        }
    }
}

impl Default for SquareTunnel {
    fn default() -> Self {
        SquareTunnel {
            speed: default::square_tunnel::speed(),
            rotation_speed: default::square_tunnel::rotation_speed(),
            rotation_offset: default::square_tunnel::rotation_offset(),
            zoom: default::square_tunnel::zoom(),
        }
    }
}

impl Default for ThePulse {
    fn default() -> Self {
        ThePulse {
            speed: default::the_pulse::speed(),
            scale: default::the_pulse::scale(),
            colour_iter: default::the_pulse::colour_iter(),
            thickness: default::the_pulse::thickness(),
        }
    }
}

impl Default for TunnelProjection {
    fn default() -> Self {
        TunnelProjection {
            speed: default::tunnel_projection::speed(),
            res: default::tunnel_projection::res(),
        }
    }
}

impl Default for VertColourGradient {
    fn default() -> Self {
        VertColourGradient {
            speed: default::vert_colour_gradient::speed(),
            scale: default::vert_colour_gradient::scale(),
            colour_iter: default::vert_colour_gradient::colour_iter(),
            line_amp: default::vert_colour_gradient::line_amp(),
            diag_amp: default::vert_colour_gradient::diag_amp(),
            boarder_amp: default::vert_colour_gradient::boarder_amp(),
        }
    }
}

impl Default for SolidHsvColour {
    fn default() -> Self {
        SolidHsvColour {
            hue: default::solid_hsv_colour::hue(),
            saturation: default::solid_hsv_colour::saturation(),
            value: default::solid_hsv_colour::value(),
        }
    }
}

impl Default for SolidRgbColour {
    fn default() -> Self {
        SolidRgbColour {
            red: default::solid_rgb_colour::red(),
            green: default::solid_rgb_colour::green(),
            blue: default::solid_rgb_colour::blue(),
        }
    }
}

impl Default for ColourPalettes {
    fn default() -> Self {
        ColourPalettes {
            speed: default::colour_palettes::speed(),
            interval: default::colour_palettes::interval(),
            selected: default::colour_palettes::selected(),
        }
    }
}


pub mod default {
    pub mod acid_gradient {
        pub fn speed() -> f32 {
            0.5125
        }
        pub fn zoom() -> f32 {
            0.0
        }
        pub fn offset() -> f32 {
            0.75
        }
    }

    pub mod blinky_circles {
        pub fn speed() -> f32 {
            0.5125
        }
        pub fn zoom() -> f32 {
            0.05
        }
        pub fn offset() -> f32 {
            0.25
        }
    }

    pub mod bw_gradient {
        pub fn speed() -> f32 {
            0.5125
        }
        pub fn dc() -> f32 {
            0.05
        }
        pub fn amp() -> f32 {
            0.5
        }
        pub fn freq() -> f32 {
            0.5
        }
        pub fn mirror() -> bool {
            false
        }  
    }

    pub mod colour_grid {
        pub fn speed() -> f32 {
            0.5
        }
        pub fn zoom_amount() -> f32 {
            0.1
        }
    }

    pub mod escher_tilings {
        pub fn speed() -> f32 {
            0.2
        }
        pub fn scale() -> f32 {
            0.2
        }
        pub fn shape_iter() -> f32 {
            0.2
        }
    }

    pub mod gilmore_acid {
        pub fn speed() -> f32 {
            0.025
        }
        pub fn displace() -> f32 {
            0.01
        }
        pub fn colour_offset() -> f32 {
            0.85
        }
        pub fn grid_size() -> f32 {
            0.345
        }
        pub fn wave() -> f32 {
            0.088
        }
        pub fn zoom_amount() -> f32 {
            0.0
        }
        pub fn rotation_amount() -> f32 {
            0.0
        }
        pub fn brightness() -> f32 {
            1.0
        }
        pub fn saturation() -> f32 {
            0.15
        }
    }

    pub mod just_relax {
        pub fn speed() -> f32 {
            0.6
        }
        pub fn shape_offset() -> f32 {
            0.728
        }
        pub fn iter() -> f32 {
            1.0
        }
    }

    pub mod life_led_wall {
        pub fn speed() -> f32 {
            0.25
        }
        pub fn size() -> f32 {
            0.73
        }
        pub fn red() -> f32 {
            0.5
        }
        pub fn green() -> f32 {
            0.2
        }
        pub fn blue() -> f32 {
            0.1
        }
        pub fn saturation() -> f32 {
            1.0
        }
        pub fn colour_offset() -> f32 {
            0.01
        }
    }

    pub mod line_gradient {
        pub fn speed() -> f32 {
            0.03
        }
        pub fn num_stripes() -> f32 {
            1.0
        }
        pub fn stripe_width() -> f32 {
            0.9
        }
        pub fn angle() -> f32 {
            0.5
        }
        pub fn smooth_width() -> f32 {
            0.155
        }
    }

    pub mod metafall {
        pub fn speed() -> f32 {
            0.47
        }
        pub fn scale() -> f32 {
            0.0
        }
        pub fn red() -> f32 {
            1.0
        }
        pub fn green() -> f32 {
            1.0
        }
        pub fn blue() -> f32 {
            1.0
        }
    }

    pub mod particle_zoom {
        pub fn speed() -> f32 {
            0.01
        }
        pub fn density() -> f32 {
            0.01
        }
        pub fn shape() -> f32 {
            0.35
        }
        pub fn tau() -> f32 {
            1.0
        }
    }

    pub mod radial_lines {
        pub fn speed() -> f32 {
            0.05
        }
        pub fn zoom_amount() -> f32 {
            0.8
        }
    }

    pub mod satis_spiraling {
        pub fn speed() -> f32 {
            0.5
        }
        pub fn loops() -> f32 {
            0.8
        }
        pub fn mirror() -> bool {
            true
        }
        pub fn rotate() -> bool {
            true
        }
    }

    pub mod spiral_intersect {
        pub fn speed() -> f32 {
            0.02
        }
        pub fn g1() -> f32 {
            0.4
        }
        pub fn g2() -> f32 {
            0.6
        }
        pub fn rot1() -> f32 {
            1.0
        }
        pub fn rot2() -> f32 {
            0.5
        }
        pub fn colours() -> f32 {
            1.0
        }
    }

    pub mod square_tunnel {
        pub fn speed() -> f32 {
            0.6
        }
        pub fn rotation_speed() -> f32 {
            0.025
        }
        pub fn rotation_offset() -> f32 {
            0.0
        }
        pub fn zoom() -> f32 {
            0.8
        }
    }

    pub mod the_pulse {
        pub fn speed() -> f32 {
            0.08
        }
        pub fn scale() -> f32 {
            0.1
        }
        pub fn colour_iter() -> f32 {
            0.25
        }
        pub fn thickness() -> f32 {
            0.0
        }
    }

    pub mod tunnel_projection {
        pub fn speed() -> f32 {
            0.5
        }
        pub fn res() -> f32 {
            0.5
        }
    }

    pub mod vert_colour_gradient {
        pub fn speed() -> f32 {
            0.5
        }
        pub fn scale() -> f32 {
            0.83
        }
        pub fn colour_iter() -> f32 {
            0.015
        }
        pub fn line_amp() -> f32 {
            0.0
        }
        pub fn diag_amp() -> f32 {
            0.0
        }
        pub fn boarder_amp() -> f32 {
            0.65
        }
    }

    pub mod solid_hsv_colour {
        pub fn hue() -> f32 {
            1.0
        }
        pub fn saturation() -> f32 {
            0.0
        }
        pub fn value() -> f32 {
            1.0
        }
    }

    pub mod solid_rgb_colour {
        pub fn red() -> f32 {
            1.0
        }
        pub fn green() -> f32 {
            1.0
        }
        pub fn blue() -> f32 {
            1.0
        }
    }

    pub mod colour_palettes {
        pub fn speed() -> f32 {
            0.1
        }
        pub fn interval() -> f32 {
            0.05
        }
        pub fn selected() -> usize {
            0
        }
    }
}