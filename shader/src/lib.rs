//! The shader function hotloaded at runtime by the cohen_gig crate.

use nannou::prelude::*;
use shader_shared::{Uniforms, Vertex};

mod signals;
mod helpers;
mod shaders;

mod solid_hsv_colour;
mod solid_rgb_colour;

mod led_shaders;

mod wash_shaders;

#[no_mangle]
fn shader(v: Vertex, uniforms: &Uniforms, shader_name: &String) -> LinSrgb {
    let p = v.position;
    match shader_name.as_ref() {
        "SolidHsvColour" => solid_hsv_colour::shader(p, uniforms),
        "SolidRgbColour" => solid_rgb_colour::shader(p, uniforms),

        "AcidGradient" => led_shaders::acid_gradient::shader(p, uniforms),
        "BlinkyCircles" => led_shaders::blinky_circles::shader(p, uniforms),
        "BwGradient" => led_shaders::bw_gradient::shader(p, uniforms),
        "ColourGrid" => led_shaders::colour_grid::shader(p,uniforms),
        "EscherTilings" => led_shaders::escher_tilings::shader(p, uniforms),
        "GilmoreAcid" => led_shaders::gilmore_acid::shader(p, uniforms),
        "JustRelax" => led_shaders::just_relax::shader(p, uniforms),
        "LifeLedWall" => led_shaders::life_led_wall::shader(p, uniforms),
        "LineGradient" => led_shaders::line_gradient::shader(p, uniforms),
        "Metafall" => led_shaders::metafall::shader(p, uniforms),
        "ParticleZoom" => led_shaders::particle_zoom::shader(p, uniforms),
        "RadialLines" => led_shaders::radial_lines::shader(p, uniforms),
        "SatisSpiraling" => led_shaders::satis_spiraling::shader(p, uniforms),
        "SpiralIntersect" => led_shaders::spiral_intersect::shader(p, uniforms),
        "SquareTunnel" => led_shaders::square_tunnel::shader(p, uniforms),
        "ThePulse" => led_shaders::the_pulse::shader(p, uniforms),
        "TunnelProjection" => led_shaders::tunnel_projection::shader(p, uniforms),
        "VertColourGradient" => led_shaders::vert_colour_gradient::shader(p, uniforms),

        "MitchWash" => wash_shaders::mitch_wash::shader(p, uniforms),
        "ColourPalettes" => wash_shaders::colour_palettes::shader(p, uniforms),
        _ => lin_srgb(0.0, 0.0, 0.0)
    }


}
