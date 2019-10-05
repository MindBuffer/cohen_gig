use nannou::prelude::*;
use shader_shared::Uniforms;

use crate::helpers::*;

// https://www.shadertoy.com/view/Mt2BWR

// struct Params {
//     speed: f32,
//     zoom_amount: f32,
// }

pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let mut params = uniforms.params.radial_lines;

    params.zoom_amount = 20.0+uniforms.slider1*10.0;
    let t = uniforms.time * params.speed;
    
    let x = map_range(p.x, -0.18, 0.13, 0.0, 1.0);
    let y = map_range(p.y, 0.3, 1.0, 0.0, 1.0);
    let mut uv = vec2(x,y);
    uv -= vec2(0.58, map_range(uniforms.slider2,0.0,1.0,-1.0,2.0));
    uv *= uniforms.resolution.x / uniforms.resolution.y;
    
    let n = 1.2.powf((0.95 + t.sin() * 0.5) * (20.0+params.zoom_amount*10.0));
    let pp = PI / n;
    let a = fmod(atan(uv.y, uv.x) + pp, pp+pp) - pp;

    let c = smoothstep(3.0 / uniforms.resolution.y, 0.0, length(vec3(uv.x,uv.y,0.0) * a.sin().abs()));
    lin_srgb(c, c, c)
}
