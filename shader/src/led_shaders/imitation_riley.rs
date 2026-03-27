use nannou_core::prelude::*;
use shader_shared::{Light, Uniforms, Vertex};

use crate::helpers::mix;

fn glsl_mod(value: f32, modulus: f32) -> f32 {
    value - modulus * (value / modulus).floor()
}

pub fn shader(v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let params = uniforms.params.imitation_riley;

    let Light::Led {
        normalised_coords, ..
    } = v.light;

    let mut uv = normalised_coords * 0.5;
    uv.x *= uniforms.resolution.x / uniforms.resolution.y;

    if uv.x > params.x_mirror {
        uv.x = params.x_mirror - (uv.x - params.x_mirror);
    }
    uv.x *= (uv.x + params.offset).tan().abs() + 1.0;
    uv.x += uniforms.time * params.speed;

    let uv_steps = uv * params.steps;
    let uv_mod = vec2(
        glsl_mod(uv_steps.x, 2.0).floor(),
        glsl_mod(uv_steps.y, 2.0).floor(),
    );
    let value = uv_mod.x * uv_mod.y + (1.0 - uv_mod.x) * (1.0 - uv_mod.y);
    let black_level = params.black_level.clamp(0.0, 1.0);
    let white_level = params.white_level.clamp(black_level, 1.0);
    let value = mix(black_level, white_level, value);

    lin_srgb(value, value, value)
}
