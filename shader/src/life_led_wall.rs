use nannou::prelude::*;
use shader_shared::Uniforms;

use crate::signals::*;
use crate::helpers::*;

// https://www.interactiveshaderformat.com/sketches/369

////////////////////////////////////////////////////////////
// ALifeLEDWall  by mojovideotech
//
// based on:
// glslsandbox.com/\e#25692.0
//
// Creative Commons Attribution-NonCommercial-ShareAlike 3.0
////////////////////////////////////////////////////////////

struct Params {
    speed: f32,
    size: f32,
    red: f32,
    green: f32,
    blue: f32,
    saturation: f32,
}

pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let params = Params {
        speed: 0.25,
        size: (uniforms.time*0.04).sin().abs() * 91.0,
        red: 0.5,
        green: 0.2,
        blue: 0.1, 
        saturation: 1.0,
    };
    
    let x = map_range(p.x, -0.13, 0.13, 0.0, 1.0);
    let y = map_range(p.y, 0.25, 1.05, 0.0, 1.0);
    let uv = vec2(x,y) * params.size;

    let t = uniforms.time;
    let s = params.speed;
    let mut brightness = vec3(
        (rand(vec2(uv.x.floor(), uv.y.floor())) + t / 1.1 * s).fract(),
        (rand(vec2(uv.x.floor(), uv.y.floor())) + t / 1.2 * s).fract(),
        (rand(vec2(uv.x.floor(), uv.y.floor())) + t / 1.3 * s).fract());
    let d = 0.5 - vec2(uv.x.fract(),uv.y.fract()).distance(vec2(0.45,0.45));
    brightness *= vec3(d,d,d);
    let r = (brightness.x*params.red) * params.saturation;
    let g = (brightness.y*params.green) * params.saturation;
    let b = (brightness.z*params.blue) * params.saturation; 
    lin_srgb(r, g, b)

}
