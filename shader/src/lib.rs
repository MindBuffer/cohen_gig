//! The shader function hotloaded at runtime by the cohen_gig crate.

use nannou::prelude::*;
use shader_shared::{Light, Uniforms, Vertex, MixingInfo};

mod signals;
mod helpers;
mod blend_modes;
mod shaders;

mod solid_hsv_colour;
mod solid_rgb_colour;

mod led_shaders;

mod wash_shaders;

#[no_mangle]
fn shader(v: Vertex, uniforms: &Uniforms, mix: &MixingInfo) -> LinSrgb {
    let left = render_shader(v, &uniforms, &mix.left_name);
    let right = render_shader(v, &uniforms, &mix.right_name);
    let colour = render_shader(v, &uniforms, &mix.colour_name);

    let xfl = lin_srgb(mix.xfade_left,mix.xfade_left,mix.xfade_left);
    let xfr = lin_srgb(mix.xfade_right,mix.xfade_right,mix.xfade_right);

    let mut col = match mix.blend_mode.as_str() {
        "Add" => blend_modes::add(left*xfl, right*xfr) * colour,
        "Subtract" => blend_modes::subtract(left*xfl, right*xfr) * colour,
        "Multiply" => blend_modes::multiply(left, right) * colour,
        "Average" => blend_modes::average(left*xfl, right*xfr) * colour,
        "Difference" => blend_modes::difference(left*xfl, right*xfr) * colour,
        "Negation" => blend_modes::negation(left*xfl, right*xfr) * colour,
        "Exclusion" => blend_modes::exclusion(left*xfl, right*xfr) * colour,
        _ => colour,
    };

    if let Light::Wash { .. } = v.light {
        col = crate::helpers::lerp_lin_srgb(v.last_color, col, v.lerp_amt);
    }

    col
}

fn render_shader(v: Vertex, uniforms: &Uniforms, shader_name: &String) -> LinSrgb {
    let p = v.position;
    match shader_name.as_ref() {
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
    }
}