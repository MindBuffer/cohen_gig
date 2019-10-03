use nannou::prelude::*;
use shader_shared::Uniforms;

use crate::helpers::*;

// https://www.shadertoy.com/view/Msc3WN

struct Params {
    speed: f32,
    res: f32,
}

pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let params = Params {
        speed: 0.5,
        res: 20.0,
    };

    let t = uniforms.time * params.speed;
    let x = map_range(p.x, -0.13, 0.13, 0.0, 1.0);
    let y = map_range(p.y, 0.3, 1.0, 0.0, 1.0);
    let uv = vec3(x,y, 1.0);
    let o = vec3(0.5,0.5,0.5);

    let w = uv - o; 
    let l = length(w);

    let g = (0.3 * t).sin();
    let d = vec3(g,g,g) * (o / vec3(l, l, l)) + vec3(t  + (g * l),t + (g * 0.1),t);
    let f = (params.res * atan(w.y, w.x) + l * 0.1 + 3.0 * t).sin();
    lin_srgb(d.x.sin() * f, d.y.sin() * f, d.z.sin() * f)
}
