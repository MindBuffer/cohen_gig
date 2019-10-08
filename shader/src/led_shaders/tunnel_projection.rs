use nannou::prelude::*;
use shader_shared::{Uniforms, Vertex, Light};

use crate::helpers::*;

// https://www.shadertoy.com/view/Msc3WN

// struct Params {
//     speed: f32,
//     res: f32,
// }

pub fn shader(v: Vertex , uniforms: &Uniforms) -> LinSrgb {
    let mut params = uniforms.params.tunnel_projection;

    if uniforms.use_midi {
        params.res = uniforms.slider3;
    }
    
    let t = uniforms.time * params.speed;

    let p = match v.light {
        Light::Wash{index} => pt2(v.position.x,v.position.z * 2.0 - 1.0),
        Light::Led{index,col_row,normalised_coords} => normalised_coords,
    };

    let x = map_range(p.x, -1.0, 1.0, 0.0, 1.0);
    let y = map_range(p.y, -1.0, 1.0, 0.0, 1.0);
    let uv = vec3(x,y, 1.0);
    let o = vec3(0.5,uniforms.slider4,0.5);

    let w = uv - o; 
    let l = length(w);

    let g = (0.3 * t).sin();
    let d = vec3(g,g,g) * (o / vec3(l, l, l)) + vec3(t  + (g * l),t + (g * 0.1),t);
    let f = ((params.res*40.0) * atan(w.y, w.x) + l * 0.1 + 3.0 * t).sin();
    lin_srgb(d.x.sin() * f, d.y.sin() * f, d.z.sin() * f)
}
