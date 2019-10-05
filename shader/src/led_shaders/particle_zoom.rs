use nannou::prelude::*;
use shader_shared::Uniforms;

use crate::helpers::*;

// https://www.interactiveshaderformat.com/sketches/569

// struct Params {
//     speed: f32,
//     density: f32,
//     shape: f32,
//     tau: f32,
// }

fn cell(coord: Vector2, p: &shader_shared::ParticleZoom) -> f32 {
    let c = vec2(coord.x.fract(), coord.y.fract()) * vec2(0.5,2.0) - vec2(0.1,0.5);
    (1.0 - length(vec3(c.x*2.0-1.0,c.y*2.0-1.0,0.0)) * step(rand(vec2(coord.x.floor(),coord.y.floor())), p.density)) * 5.0
}

pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let params = uniforms.params.particle_zoom;

    let y_offset = (uniforms.time*0.1).sin();
    let t = uniforms.time * params.speed;
    
    let x = map_range(p.x, -0.13, 0.13, -1.0, 1.0);
    let y = map_range(p.y, 0.3, 1.0, -1.0, 1.0);
    let mut uv = vec2(x,y) / uniforms.resolution - vec2(0.0,y_offset * 0.005); 
    uv *= uniforms.resolution.x / uniforms.resolution.y;

    let a = (atan(uv.x,uv.y) / (params.tau*10.0)).fract();
    let d = length(vec3(uv.x,uv.y,0.0));
    let mut coord = vec2(d.powf(params.shape), a) * vec2(256.0,256.0);
    let delta = vec2(-t * 256.0, 1.0);
    let mut c = 0.0;

    for _ in 0..1 {
        coord += delta;
        c = c.max(cell(coord, &params));
    }

    c = (c*d)*15.0;

    lin_srgb(c,c,c)
}
