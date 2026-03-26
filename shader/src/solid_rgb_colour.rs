use nannou_core::prelude::*;
use shader_shared::{Uniforms, Vertex};

// struct Params {
//     red: f32,
//     green: f32,
//     blue: f32,
// }

pub fn shader(_v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let p = uniforms.params.solid_rgb_colour;
    lin_srgb(p.red, p.green, p.blue)
}
