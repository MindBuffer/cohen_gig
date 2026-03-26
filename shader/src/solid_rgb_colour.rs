use nannou_core::prelude::*;
use shader_shared::{Uniforms, Vertex};

// struct Params {
//     red: f32,
//     green: f32,
//     blue: f32,
// }

pub fn shader(_v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    lin_srgb(uniforms.pot6, uniforms.pot7, uniforms.pot8)
}
