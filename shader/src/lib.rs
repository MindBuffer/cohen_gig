//! The shader function hotloaded at runtime by the cohen_gig crate.

use nannou_core::prelude::*;
use shader_shared::{BlendMode, Shader, Uniforms, Vertex};

mod blend_modes;
pub mod helpers;
pub mod shaders;
pub mod signals;

mod colour_palettes;
mod solid_hsv_colour;
mod solid_rgb_colour;

mod led_shaders;

mod wash_shaders;

#[no_mangle]
fn shader(v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let mix = &uniforms.mix;

    // Retrieve the shader functions for left, right and colourising.
    let left_shader = get_shader(mix.left);
    let right_shader = get_shader(mix.right);
    let colourise = get_shader(mix.colourise);

    // Run each sub-shader with its own params so there's no collision
    // when the same shader type is used in multiple slots.
    let left = left_shader(v, &with_params(uniforms, mix.params_left));
    let right = right_shader(v, &with_params(uniforms, mix.params_right));
    let colour = colourise(v, &with_params(uniforms, mix.params_colourise));

    // Mix the left and right shaders.
    let xfl = lin_srgb(mix.xfade_left, mix.xfade_left, mix.xfade_left);
    let xfr = lin_srgb(mix.xfade_right, mix.xfade_right, mix.xfade_right);

    // Apply the blend mode.
    let mut col = match mix.blend_mode {
        BlendMode::Add => blend_modes::add(left * xfl, right * xfr),
        BlendMode::Subtract => blend_modes::subtract(left * xfl, right * xfr),
        BlendMode::Multiply => blend_modes::multiply(left, right),
        BlendMode::Average => blend_modes::average(left * xfl, right * xfr),
        BlendMode::Difference => blend_modes::difference(left * xfl, right * xfr),
        BlendMode::Negation => blend_modes::negation(left * xfl, right * xfr),
        BlendMode::Exclusion => blend_modes::exclusion(left * xfl, right * xfr),
    };

    // Colourise.
    col *= colour;

    col
}

fn with_params(uniforms: &Uniforms, params: shader_shared::ShaderParams) -> Uniforms {
    Uniforms {
        params,
        ..uniforms.clone()
    }
}

fn get_shader(shader: Shader) -> fn(Vertex, &Uniforms) -> LinSrgb {
    match shader {
        Shader::SolidHsvColour => solid_hsv_colour::shader,
        Shader::SolidRgbColour => solid_rgb_colour::shader,
        Shader::ColourPalettes => colour_palettes::shader,
        Shader::AcidGradient => led_shaders::acid_gradient::shader,
        Shader::BlinkyCircles => led_shaders::blinky_circles::shader,
        Shader::BwGradient => led_shaders::bw_gradient::shader,
        Shader::ColourGrid => led_shaders::colour_grid::shader,
        Shader::EscherTilings => led_shaders::escher_tilings::shader,
        Shader::GilmoreAcid => led_shaders::gilmore_acid::shader,
        Shader::GradientBars => led_shaders::gradient_bars::shader,
        Shader::JustRelax => led_shaders::just_relax::shader,
        Shader::LifeLedWall => led_shaders::life_led_wall::shader,
        Shader::LineGradient => led_shaders::line_gradient::shader,
        Shader::Metafall => led_shaders::metafall::shader,
        Shader::ParticleZoom => led_shaders::particle_zoom::shader,
        Shader::RadialLines => led_shaders::radial_lines::shader,
        Shader::SatisSpiraling => led_shaders::satis_spiraling::shader,
        Shader::SpiralIntersect => led_shaders::spiral_intersect::shader,
        Shader::SquareTunnel => led_shaders::square_tunnel::shader,
        Shader::ThePulse => led_shaders::the_pulse::shader,
        Shader::TunnelProjection => led_shaders::tunnel_projection::shader,
        Shader::VertColourGradient => led_shaders::vert_colour_gradient::shader,
        Shader::RowTest => led_shaders::row_test::shader,
        Shader::BarTest => led_shaders::bar_test::shader,
        Shader::MitchWash => wash_shaders::mitch_wash::shader,
        Shader::ShapeEnvelopes => wash_shaders::shape_envelopes::shader,
    }
}
