//! The shader function hotloaded at runtime by the cohen_gig crate.

use nannou::prelude::*;
use shader_shared::{Light, Uniforms, Vertex};

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
    let mut col = match shader_name.as_ref() {
        "SolidHsvColour" => solid_hsv_colour::shader(v, uniforms),
        "SolidRgbColour" => solid_rgb_colour::shader(v, uniforms),

        "AcidGradient" => led_shaders::acid_gradient::shader(v, uniforms),
        "BlinkyCircles" => led_shaders::blinky_circles::shader(v, uniforms),
        "BwGradient" => led_shaders::bw_gradient::shader(v, uniforms),
        "ColourGrid" => led_shaders::colour_grid::shader(v,uniforms),
        "EscherTilings" => led_shaders::escher_tilings::shader(v, uniforms),
        "GilmoreAcid" => led_shaders::gilmore_acid::shader(v, uniforms),
        "JustRelax" => led_shaders::just_relax::shader(v, uniforms),
        "LifeLedWall" => led_shaders::life_led_wall::shader(v, uniforms),
        "LineGradient" => led_shaders::line_gradient::shader(v, uniforms),
        "Metafall" => led_shaders::metafall::shader(v, uniforms),
        "ParticleZoom" => led_shaders::particle_zoom::shader(v, uniforms),
        "RadialLines" => led_shaders::radial_lines::shader(v, uniforms),
        "SatisSpiraling" => led_shaders::satis_spiraling::shader(v, uniforms),
        "SpiralIntersect" => led_shaders::spiral_intersect::shader(v, uniforms),
        "SquareTunnel" => led_shaders::square_tunnel::shader(v, uniforms),
        "ThePulse" => led_shaders::the_pulse::shader(v, uniforms),
        "TunnelProjection" => led_shaders::tunnel_projection::shader(v, uniforms),
        "VertColourGradient" => led_shaders::vert_colour_gradient::shader(v, uniforms),

        // "MitchWash" => wash_shaders::mitch_wash::shader(v, uniforms),
        // "ColourPalettes" => wash_shaders::colour_palettes::shader(v, uniforms),
        _ => lin_srgb(0.0, 0.0, 0.0)
    };

    if let Light::Wash { .. } = v.light {
        // TODO: Make this a slider parameter.
        let lerp_amt = 1.0;
        col = crate::helpers::lerp_lin_srgb(v.last_color, col, lerp_amt);
    }

    col
}
