use nannou::prelude::*;
use shader_shared::Uniforms;

use crate::signals::*;
use crate::helpers::*;

// https://www.interactiveshaderformat.com/sketches/1706

struct Params {
    speed: f32,
    shape_offset: f32,
}


pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let params = Params {
        speed: 2.15,
        shape_offset: 0.728,
    };

    let t = uniforms.time * params.speed;
    
    let x = map_range(p.x, -0.13, 0.13, -0.5, 0.5);
    let y = map_range(p.y, 0.3, 1.0, -0.5, 0.5);
    let uv = vec2(x,y);
    let mut uv2 = uv * vec2(0.6,0.6);
    uv2.x *= uniforms.resolution.x / uniforms.resolution.y;
    let mut co = vec3(0.0,0.0,0.0);
    let l = length(vec3(uv2.x, uv2.y, 0.0));
    let f = 0.1;
    let a = 3.0;
    let pp = vec2(0.0,0.0);
    let c = 0.5 + (t * 0.1 + (params.shape_offset * 8.0)).sin() / PI + 0.55;
    for i in 0..6 {
        uv2 = vec2(uv2.x.abs(), uv2.y.abs()) / uv2.dot(uv2) - vec2(c,c);
    }
    let b = (1.0 - fmod(atan(uv2.x, uv2.y) * 6.0 + t, 2.0)).abs();
    let l = length(vec3(uv2.x, uv2.y, 0.0)) - b;
    let tex = vec3(0.0,0.0,0.0);
    let co1 = (0.0.max(1.0-l)).powf(1.5);
    let co2 = (0.0.max(0.5-length(vec3(uv2.x, uv2.y, 0.0))/0.5)).powf(2.0);
    co = vec3(co1,co1,co1) * tex + vec3(co2,co2,co2) * vec3(1.0, 0.8, 0.5);
    let co3 = (0.0.max(0.5-length(vec3(uv.x, uv.y, 0.0))/0.5)).powf(3.0);
    co += vec3(co3,co3,co3) * vec3(0.7,0.5,0.3);
    co *= 0.0.max(0.7-length(vec3(uv.x, uv.y, 0.0))) / 0.7;

    lin_srgb(co.y, co.y, co.y)
}
