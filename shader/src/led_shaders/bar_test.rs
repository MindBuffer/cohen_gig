use nannou_core::prelude::*;
use shader_shared::{Light, Uniforms, Vertex};

use crate::helpers::*;

pub fn shader(v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let mut params = uniforms.params.row_test;

    let (p, col, row) = match v.light {
        Light::Wash { index } => (pt2(v.position.x, v.position.z * 2.0 - 1.0) ,0,0),
        Light::Led {
            index,
            col_row,
            normalised_coords,
        } => {
            (normalised_coords, col_row[1], col_row[0])
        },
    };

    let x = (p.x * 3.0) as i32;

    let col = map_range(x, -3, 3, 0.0, 1.0);

    // if row == params.row as usize {
    //     lin_srgb(1.0, 1.0, col)
    // } else {
    //     lin_srgb(0.0, 0.0, 0.0)
    // }

    lin_srgb(1.0-col, col, 1.0-col)
}
