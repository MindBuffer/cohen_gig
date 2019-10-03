use nannou::prelude::*;
use shader_shared::Uniforms;
use nannou::math::Matrix2;

use crate::signals::*;
use crate::helpers::*;

// https://www.interactiveshaderformat.com/sketches/1701

enum Direction {
    Vertical,
    Horizontal,
}

struct Params {
    speed: f32,
    scale: f32,
    shape_iter: f32,
    direction: Direction,
}


pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let params = Params {
        speed: 0.2,
        scale: uniforms.slider1 * 30.0, //6.0,
        shape_iter: 0.2,
        direction: Direction::Horizontal,
    };

    let t = uniforms.time * params.speed;
    
    let x = map_range(p.x, -0.13, 0.13, 0.0, 1.0);
    let y = map_range(p.y, 0.3, 1.0, 0.0, 1.0);
    let mut uv = vec2(x,y) * uniforms.resolution;
    uv *= params.scale / uniforms.resolution.y;
    
    let mut col = vec3(0.0,0.0,0.0);
    
    let f = vec2(uv.x.floor(), uv.y.floor());
    let mut u = vec2(2.0,2.0) * vec2(uv.x.fract(), uv.y.fract()) - vec2(1.0,1.0);
    let mut y = 0.0;

    let d = match params.direction {
        Direction::Horizontal => uv.x,
        Direction::Vertical => uv.y,
    }; 

    for _ in 0..4 {
        u = multiply_mat2_with_vec2(Matrix2::new(0.0,-1.0,1.0,0.0), u);
        y = 2.0 * (t + d * (params.shape_iter * 0.05)).fract() - 1.0;
        let o = smoothstep(0.55, 0.45, length(vec3(u.x, u.y, 0.0) - vec3(0.5, 1.5 * y, 0.0)));
        col += vec3(o,o,o);
    }

    lin_srgb(col.x, col.y, col.z)
}
