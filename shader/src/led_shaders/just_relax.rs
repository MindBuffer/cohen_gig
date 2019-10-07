use nannou::prelude::*;
use shader_shared::Uniforms;

use crate::helpers::*;

// https://www.interactiveshaderformat.com/sketches/1706

// struct Params {
//     speed: f32,
//     shape_offset: f32,
//     iter: f32,
// }

pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let mut params = uniforms.params.just_relax;

    if uniforms.use_midi {
        params.shape_offset = uniforms.slider1;
        params.iter = 0.5 + uniforms.slider2;
    }

    let t = uniforms.time * (params.speed*4.0);
    
    let x = map_range(p.x, -0.13, 0.13, -0.5, 0.5);
    let y = map_range(p.y, 0.3, 1.0, -0.5, 0.5);
    let uv = vec2(x,y);
    let mut uv2 = uv * vec2(params.iter * 4.0,params.iter * 4.0);
    uv2.x *= uniforms.resolution.x / uniforms.resolution.y;
    let c = 0.5 + (t * 0.1 + (params.shape_offset * 8.0)).sin() / PI + 0.55;
    for _ in 0..6 {
        uv2 = vec2(uv2.x.abs(), uv2.y.abs()) / uv2.dot(uv2) - vec2(c,c);
    }
    let b = (1.0 - fmod(atan(uv2.x, uv2.y) * params.iter + t, 2.0)).abs();
    let l = length(vec3(uv2.x, uv2.y, 0.0)) - b;
    let tex = vec3(0.0,0.0,0.0);
    let co1 = (0.0.max(1.0-l)).powf(0.15);
    let co2 = (0.0.max(0.5-length(vec3(uv2.x, uv2.y, 0.0))/0.5)).powf(0.20);
    let mut co = vec3(co1,co1,co1) * tex + vec3(co2,co2,co2) * vec3(1.0, 0.8, 0.5);
    let co3 = (0.0.max(0.5-length(vec3(uv.x, uv.y, 0.0))/0.5)).powf(3.0);
    co += vec3(co3,co3,co3) * vec3(0.7,0.5,0.3);
    co *= 0.0.max(0.7-length(vec3(uv.x, uv.y, 0.0))) / 0.7;


    lin_srgb(co.y, co.y, co.y)
}
