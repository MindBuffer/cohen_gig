use nannou_core::prelude::*;
use shader_shared::{Uniforms, Vertex};

use crate::helpers::*;

// struct Params {
//     hue: f32,
//     saturation: f32,
//     value: f32,
// }

// Smooth HSV to RGB conversion
fn hsv2rgb_smooth(c: Vec3) -> Vec3 {
    let r = clamp((fmod(c.x * 6.0 + 0.0, 6.0) - 3.0).abs() - 1.0, 0.0, 1.0);
    let g = clamp((fmod(c.x * 6.0 + 4.0, 6.0) - 3.0).abs() - 1.0, 0.0, 1.0);
    let b = clamp((fmod(c.x * 6.0 + 2.0, 6.0) - 3.0).abs() - 1.0, 0.0, 1.0);
    let mut rgb = vec3(r, g, b);

    rgb = rgb * rgb * (vec3(3.0, 3.0, 3.0) - vec3(2.0, 2.0, 2.0) * rgb); // cubic smoothing

    vec3(
        c.z * mix(1.0, rgb.x, c.y),
        c.z * mix(1.0, rgb.y, c.y),
        c.z * mix(1.0, rgb.z, c.y),
    )
}

pub fn shader(_v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let p = uniforms.params.solid_hsv_colour;
    let rgb = hsv2rgb_smooth(vec3(p.hue, p.saturation, p.value));
    lin_srgb(rgb.x, rgb.y, rgb.z)
}
