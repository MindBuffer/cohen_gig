use nannou_core::prelude::*;
use shader_shared::{Light, Uniforms, Vertex};

use crate::helpers::smoothstep;

pub fn shader(v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let params = uniforms.params.radial_keta;

    let Light::Led {
        normalised_coords, ..
    } = v.light;

    let mut uv = normalised_coords * 0.5;
    uv.x *= uniforms.resolution.x / uniforms.resolution.y;

    let radius = uv.length();
    let angle = uv.x.atan2(uv.y);
    let t = (uniforms.time * params.speed).sin() * params.iter;

    let d = radius * (t * angle).cos();
    let s = smoothstep(0.0, 0.05, d);

    let d2 = radius * (t * angle).cos();
    let s2 = smoothstep(0.0, 0.05, d2);

    let value = s.min(s2);
    lin_srgb(value, value, value)
}
