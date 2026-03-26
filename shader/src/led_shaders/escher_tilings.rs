use nannou_core::prelude::*;
use shader_shared::{Light, Uniforms, Vertex};

use crate::helpers::*;

// https://www.interactiveshaderformat.com/sketches/1701

// struct Params {
//     speed: f32,
//     scale: f32,
//     shape_iter: f32,
// }

pub fn shader(v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let params = uniforms.params.escher_tilings;

    let t = uniforms.time * params.speed;

    let Light::Led {
        normalised_coords, ..
    } = v.light;
    let uv = normalised_coords;

    let x = map_range(uv.x, -1.0, 1.0, 0.0, 1.0);
    let y = map_range(uv.y, -1.0, 1.0, 0.0, 1.0);
    let mut uv = vec2(x, y) * uniforms.resolution;
    uv *= (params.scale * 30.0) / uniforms.resolution.y;

    let mut col = vec3(0.0, 0.0, 0.0);

    let mut u = vec2(2.0, 2.0) * vec2(uv.x.fract(), uv.y.fract()) - vec2(1.0, 1.0);
    let d = uv.x;

    for _ in 0..4 {
        u = multiply_mat2_with_vec2(
            Mat2::from_cols(Vec2::new(0.0, -1.0), Vec2::new(1.0, 0.0)),
            u,
        );
        let y = 2.0 * (t + d * (params.shape_iter * 0.05)).fract() - 1.0;
        let o = smoothstep(
            0.55,
            0.45,
            length(vec3(u.x, u.y, 0.0) - vec3(0.5, 1.5 * y, 0.0)),
        );
        col += vec3(o, o, o);
    }

    lin_srgb(col.x, col.y, col.z)
}
