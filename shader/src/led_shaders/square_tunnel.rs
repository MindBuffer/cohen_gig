use nannou::prelude::*;
use shader_shared::Uniforms;
use nannou::math::Matrix2;

use crate::helpers::*;

// https://www.interactiveshaderformat.com/sketches/2346

// struct Params {
//     speed: f32,
//     rotation_speed: f32,
//     rotation_offset: f32,
//     zoom: f32,
// }

pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let mut params = uniforms.params.square_tunnel;

    params.rotation_offset = uniforms.slider1;
    params.zoom = uniforms.slider2;

    let t = uniforms.time * (params.speed*2.0);

    let x = map_range(p.x, -0.13, 0.13, -1.0, 1.0);
    let y = map_range(p.y, 0.25, 1.05, -1.0, 1.0);
    let mut uv = vec2(x,y);
    uv.x *= uniforms.resolution.x / uniforms.resolution.y;

    uv *= vec2(params.zoom,params.zoom);
    
    let t = params.rotation_speed * (t - (params.rotation_offset * 33.0));
    let mut r = 1.0;
    let mut c;
    let mut s;
    let mut col = 0.0;

    for i in 0..49 {
        c = t.cos();
        s = t.sin();
        let mat = Matrix2::new(c, s, -s, c);
        uv = multiply_mat2_with_vec2(mat, uv);
        r /= c.abs() + s.abs();
        col = smoothstep(3.0 / uniforms.resolution.y, 0.0, uv.x.abs().max(uv.y.abs()) - r) - col;
    }
    //lin_srgb(uv.x, uv.y, 1.0)

    lin_srgb(col, col, col)
}
