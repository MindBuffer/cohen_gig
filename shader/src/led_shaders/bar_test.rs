use nannou_core::prelude::*;
use shader_shared::{Light, Uniforms, Vertex};

pub fn shader(v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let params = uniforms.params.row_test;

    let Light::Led { normalised_coords, .. } = v.light;
    let p = normalised_coords;

    let x = (p.x * 3.0) as i32;

    let col = map_range(x, -3, 3, 0.0, 1.0);

    // if row == params.row as usize {
    //     lin_srgb(1.0, 1.0, col)
    // } else {
    //     lin_srgb(0.0, 0.0, 0.0)
    // }

    lin_srgb(1.0-col, col, 1.0-col)
}
