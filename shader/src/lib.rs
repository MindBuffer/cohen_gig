//! The shader function hotloaded at runtime by the cohen_gig crate.

use nannou::prelude::*;
use shader_shared::{BlendMode, Light, MixingInfo, Shader, Uniforms, Vertex};

mod signals;
mod helpers;
mod blend_modes;
mod shaders;

mod solid_hsv_colour;
mod solid_rgb_colour;
mod colour_palettes;

mod led_shaders;

mod wash_shaders;

#[no_mangle]
fn shader(v: Vertex, uniforms: &Uniforms, mix: &MixingInfo) -> LinSrgb {
    let left = render_shader(v, &uniforms, mix.left);
    let right = render_shader(v, &uniforms, mix.right);
    let colour = render_shader(v, &uniforms, mix.colourise);

    let xfl = lin_srgb(mix.xfade_left,mix.xfade_left,mix.xfade_left);
    let xfr = lin_srgb(mix.xfade_right,mix.xfade_right,mix.xfade_right);

    let mut col = match mix.blend_mode {
        BlendMode::Add => blend_modes::add(left*xfl, right*xfr) * colour,
        BlendMode::Subtract => blend_modes::subtract(left*xfl, right*xfr) * colour,
        BlendMode::Multiply => blend_modes::multiply(left, right) * colour,
        BlendMode::Average => blend_modes::average(left*xfl, right*xfr) * colour,
        BlendMode::Difference => blend_modes::difference(left*xfl, right*xfr) * colour,
        BlendMode::Negation => blend_modes::negation(left*xfl, right*xfr) * colour,
        BlendMode::Exclusion => blend_modes::exclusion(left*xfl, right*xfr) * colour,
    };

    if let Light::Wash { .. } = v.light {
        col = crate::helpers::lerp_lin_srgb(v.last_color, col, uniforms.wash_lerp_amt);
    }

    col
}

fn render_shader(v: Vertex, uniforms: &Uniforms, shader: Shader) -> LinSrgb {
    let p = v.position;
    match shader {
        Shader::SolidHsvColour => solid_hsv_colour::shader(v, uniforms),
        Shader::SolidRgbColour => solid_rgb_colour::shader(v, uniforms),
        Shader::ColourPalettes => colour_palettes::shader(v, uniforms),
        Shader::AcidGradient => led_shaders::acid_gradient::shader(v, uniforms),
        Shader::BlinkyCircles => led_shaders::blinky_circles::shader(v, uniforms),
        Shader::BwGradient => led_shaders::bw_gradient::shader(v, uniforms),
        Shader::ColourGrid => led_shaders::colour_grid::shader(v,uniforms),
        Shader::EscherTilings => led_shaders::escher_tilings::shader(v, uniforms),
        Shader::GilmoreAcid => led_shaders::gilmore_acid::shader(v, uniforms),
        Shader::JustRelax => led_shaders::just_relax::shader(v, uniforms),
        Shader::LifeLedWall => led_shaders::life_led_wall::shader(v, uniforms),
        Shader::LineGradient => led_shaders::line_gradient::shader(v, uniforms),
        Shader::Metafall => led_shaders::metafall::shader(v, uniforms),
        Shader::ParticleZoom => led_shaders::particle_zoom::shader(v, uniforms),
        Shader::RadialLines => led_shaders::radial_lines::shader(v, uniforms),
        Shader::SatisSpiraling => led_shaders::satis_spiraling::shader(v, uniforms),
        Shader::SpiralIntersect => led_shaders::spiral_intersect::shader(v, uniforms),
        Shader::SquareTunnel => led_shaders::square_tunnel::shader(v, uniforms),
        Shader::ThePulse => led_shaders::the_pulse::shader(v, uniforms),
        Shader::TunnelProjection => led_shaders::tunnel_projection::shader(v, uniforms),
        Shader::VertColourGradient => led_shaders::vert_colour_gradient::shader(v, uniforms),
        _ => lin_srgb(0.0, 0.0, 0.0)
    }
}
