use nannou::prelude::*;
use shader_shared::Uniforms;

use crate::signals::*;
use crate::helpers::*;

pub fn test(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let t = uniforms.time;
    let b = (p.z + t).sin() * 0.5 + 0.5;
    let r = (p.x * 3.0 + t * 2.0 * p.x.signum()).cos() * 0.5 + 0.5;
    let g = (p.y * 5.0 + t).cos() * 0.5 + 0.5;
    let mut col = vec3(b*r*0.5, g*b, b);
    
    let x = map_range(p.x, -0.13, 0.13, 0.0, 1.0);
    let y = map_range(p.y, 0.3, 1.0, 0.0, 1.0);
    if p.z > 0.521739 && p.z < 0.5217392 && p.x < 0.3 {
        col.x = 0.0;
        col.y = 1.0-fmod((x + p.x).cos() * 0.5 + 0.5 + y + t * (0.05+(0.5+(t * 0.02).sin() * 0.5) * 0.005), 1.0);;
        col.z = fmod((x + p.y).sin() + t * 0.4, col.y.sin());
    }
    lin_srgb(col.x, col.y, col.z)
}

// https://www.shadertoy.com/view/Msc3WN
pub fn tunnel_projection(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let speed = 0.3;
    let res = 20.0;
    let t = uniforms.time * speed;
    let x = map_range(p.x, -0.13, 0.13, 0.0, 1.0);
    let y = map_range(p.y, 0.3, 1.0, 0.0, 1.0);
    let uv = vec3(x,y, 1.0);
    let o = vec3(0.5,0.5,0.5);

    let w = uv - o; 
    let l = length(w);

    let g = (0.3 * t).sin();
    let d = vec3(g,g,g) * (o / vec3(l, l, l)) + vec3(t  + (g * l),t + (g * 0.1),t);
    let f = (res * atan(w.y, w.x) + l * 0.1 + 3.0 * t).sin();
    lin_srgb(d.x.sin() * f, d.y.sin() * f, d.z.sin() * f)
}
