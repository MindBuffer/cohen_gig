use nannou::prelude::*;
use shader_shared::Uniforms;
use nannou::math::Matrix2;

use crate::signals::*;
use crate::helpers::*;

// https://www.shadertoy.com/view/wllGzB

struct Params {
    speed: f32,
    scale: f32,
    red: f32,
    green: f32,
    blue: f32,
}


pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let mut params = Params {
        speed: 0.47,
        scale: 1.0,
        red: 1.0,
        green: 1.0,
        blue: 1.0,
    };

/*
    params.scale = 1.0 + uniforms.slider4 * 15.0;
    params.red = uniforms.slider5;
    params.green = uniforms.slider6;
    params.blue = uniforms.slider7;
*/

    let t = uniforms.time * params.speed;
    
    let x = map_range(p.x, -0.135, 0.135, 0.0, 1.0);
    let y = map_range(p.y, 0.25, 1.05, 0.0, 1.0);
    let uv = vec2(x,y);
    
    let r = uniforms.resolution;
    let mut q = vec2(0.0,0.0);
    let mut d = vec2(0.0,0.0);
    let mut v = vec2(params.scale,params.scale) * uv;
    v.y += t;
    let mut pt = 0.0;

    //lin_srgb(v.x, v.y, 1.0)
    for k in 0..9 {
        q = vec2(fmod(k as f32, 3.0)-1.0, k as f32 / 3.0 - 1.0);
        let c = ceil(v-q);
        d = fract(vec2(10000.0,10000.0) * sin(multiply_mat2_with_vec2(Matrix2::new(r.x,r.y,r.x,r.y), c))) - vec2(0.5,0.5);
        q = fract(v) -vec2(0.5,0.5) + q + d;
        pt += smoothstep(1.3 * uv.y, 0.0, length(vec3(q.x,q.y,0.0)));
    }
    let c = pt;// - 0.5;
   // dbg!(c);
    lin_srgb(c, c, c) * lin_srgb(params.red, params.green, params.blue)
}
