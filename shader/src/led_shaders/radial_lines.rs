use nannou::prelude::*;
use shader_shared::{Uniforms, Vertex, Light};

use crate::helpers::*;

// https://www.shadertoy.com/view/Mt2BWR

// struct Params {
//     speed: f32,
//     zoom_amount: f32,
// }

pub fn shader(v: Vertex , uniforms: &Uniforms) -> LinSrgb {
    let mut params = uniforms.params.radial_lines;

    
    let t = uniforms.time * params.speed;
    
    let p = match v.light {
        Light::Wash{index} => pt2(v.position.x,v.position.z * 2.0 - 1.0),
        Light::Led{index,col_row,normalised_coords} => normalised_coords,
    };

    let x = map_range(p.x, -1.0, 1.0, 0.0, 1.0);
    let y = map_range(p.y, -1.2, 1.0, 0.0, 1.0);
    let mut uv = vec2(x,y);
    uv.x -= 0.5;
    if uniforms.use_midi {
        uv.y -= map_range(uniforms.slider2,0.0,1.0,-1.0,2.0);
        params.zoom_amount = uniforms.slider1;
    } 
    uv *= uniforms.resolution.x / uniforms.resolution.y;
    
    let n = 1.2.powf((0.95 + t.sin() * 0.5) * (20.0+params.zoom_amount*10.0));
    let pp = PI / n;
    let a = fmod(atan(uv.y, uv.x) + pp, pp+pp) - pp;

    let c = smoothstep(3.0 / uniforms.resolution.y, 0.0, length(vec3(uv.x,uv.y,0.0) * a.sin().abs()));
    lin_srgb(c, c, c)
}
