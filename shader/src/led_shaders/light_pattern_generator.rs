use nannou_core::prelude::*;
use shader_shared::{Light, Uniforms, Vertex};

use crate::helpers::TWO_PI;

fn glsl_fract(value: f32) -> f32 {
    value - value.floor()
}

fn glsl_fract_vec2(v: Vec2) -> Vec2 {
    vec2(glsl_fract(v.x), glsl_fract(v.y))
}

fn palette(t: f32, a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> Vec3 {
    a + b * vec3(
        (TWO_PI * (c.x * t + d.x)).cos(),
        (TWO_PI * (c.y * t + d.y)).cos(),
        (TWO_PI * (c.z * t + d.z)).cos(),
    )
}

pub fn shader(v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let params = uniforms.params.light_pattern_generator;

    let Light::Led {
        normalised_coords, ..
    } = v.light;

    let mut uv = normalised_coords;
    let t = uniforms.time * 0.125;
    let d = 0.3 * params.offset;

    let zoom = 8.0 + params.zoom * 64.0;
    let mut g = uv * zoom;
    uv = d * (vec2(g.x.floor(), g.y.floor()) + vec2(0.5, 0.5)) / zoom;
    g = glsl_fract_vec2(g) * 2.0 - vec2(1.0, 1.0);

    let f = uv.dot(uv) - t;
    let colour = palette(
        f * 0.5 + t,
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(1.0, 1.0, 1.0),
        vec3(0.0, 0.10, 0.20),
    );

    let denom = ((glsl_fract(f) - 0.5) * 8.0).abs().max(0.0001);
    let intensity = (1.0 - g.y * g.y) * 0.2 / denom;
    let colour = colour * intensity;

    lin_srgb(colour.x, colour.y, colour.z)
}
