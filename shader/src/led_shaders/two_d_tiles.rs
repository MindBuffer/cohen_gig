use nannou_core::prelude::*;
use shader_shared::{Light, Uniforms, Vertex};

use crate::helpers::{mix, smoothstep, step};

fn glsl_fract(value: f32) -> f32 {
    value - value.floor()
}

fn glsl_fract_vec2(v: Vec2) -> Vec2 {
    vec2(glsl_fract(v.x), glsl_fract(v.y))
}

pub fn shader(v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let params = uniforms.params.two_d_tiles;

    let Light::Led {
        normalised_coords, ..
    } = v.light;

    let aspect_ratio = uniforms.resolution.y / uniforms.resolution.x;
    let mut uv = vec2(
        normalised_coords.x * 0.5,
        normalised_coords.y * 0.5 * aspect_ratio,
    );
    uv += vec2(0.025, 0.025);

    let scaled_uv = params.size * uv;
    let tile = glsl_fract_vec2(scaled_uv);
    let tile_dist = tile.x.min(1.0 - tile.x).min(tile.y.min(1.0 - tile.y));
    let square = vec2(scaled_uv.x.floor(), scaled_uv.y.floor());
    let square_dist = square.length();

    let mut edge = (uniforms.time * params.speed - square_dist * params.offset).sin();
    edge = (edge * edge).fract();

    let mut value = mix(tile_dist, 1.0 - tile_dist, step(params.step_thresh, edge));
    edge = (1.0 - edge).abs().powf(params.power) * 0.5;

    value = smoothstep(edge - 0.05, edge, 0.95 * value);
    lin_srgb(value, value, value)
}
