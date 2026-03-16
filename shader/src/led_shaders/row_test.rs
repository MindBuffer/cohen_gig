use nannou_core::prelude::*;
use shader_shared::{Light, Uniforms, Vertex};

pub fn shader(v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let params = uniforms.params.row_test;

    let Light::Led { col_row, .. } = v.light;
    let row = col_row[1];

    if row == params.row as usize {
        lin_srgb(1.0, 1.0, 1.0)
    } else {
        lin_srgb(0.0, 0.0, 0.0)
    }

}
