use nannou::prelude::*;
use shader_shared::Uniforms;

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

// struct Params {
//     speed: f32,
//     size: f32,
//     red: f32,
//     green: f32,
//     blue: f32,
//     saturation: f32,
//     colour_offset: f32,
// }

pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let mut params = uniforms.params.life_led_wall;

    if uniforms.use_midi {
        params.size = uniforms.slider3;
        params.colour_offset = uniforms.slider4 * 0.01;
    }
        
    let x = map_range(p.x, -0.13, 0.13, 0.0, 1.0);
    let y = map_range(p.y, 0.25, 1.05, 0.0, 1.0);
    let uv = vec2(x,y) * (params.size*80.0);

    let t = uniforms.time;
    let s = params.speed;
    let mut brightness = vec3(
        (rand(vec2(uv.x.floor(), uv.y.floor())) + t / (1.0 + params.colour_offset) * s).fract(),
        (rand(vec2(uv.x.floor(), uv.y.floor())) + t / (1.0 + (params.colour_offset*2.0)) * s).fract(),
        (rand(vec2(uv.x.floor(), uv.y.floor())) + t / (1.0 + (params.colour_offset*3.0)) * s).fract());
    let d = 0.5 - vec2(uv.x.fract(),uv.y.fract()).distance(vec2(0.5,0.5));

    brightness *= vec3(d,d,d);
    let r = (brightness.x*params.red) * (params.saturation*10.0);
    let g = (brightness.y*params.green) * (params.saturation*10.0);
    let b = (brightness.z*params.blue) * (params.saturation*10.0); 
    lin_srgb(r, g, b)

}
