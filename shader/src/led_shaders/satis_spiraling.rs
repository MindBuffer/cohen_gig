use nannou::prelude::*;
use shader_shared::Uniforms;

use crate::helpers::*;

// https://www.interactiveshaderformat.com/sketches/1657

////////////////////////////////////////////////////////////
// SatisSpiraling  by mojovideotech
//
// based on :
// Overly satisfying  by nimitz
// shadertoy.com\/view\/Mts3zM
//
// Creative Commons Attribution-NonCommercial-ShareAlike 3.0
////////////////////////////////////////////////////////////

// struct Params {
//     speed: f32,
//     loops: f32,
//     mirror: bool,
//     rotate: bool,
// }

pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let mut params = uniforms.params.satis_spiraling;

    //params.speed = map_range((uniforms.time*0.01).sin() * (uniforms.time*0.001 * 10.0).cos(), -1.0, 1.0, 0.05, 0.2);
    if uniforms.use_midi {
        params.loops = uniforms.slider3;
    }
    
    let t = uniforms.time * -params.speed;
    let aspect = uniforms.resolution.x/uniforms.resolution.y;
    let w = 50.0/(uniforms.resolution.x*aspect+uniforms.resolution.y).sqrt();

    let x = map_range(p.x, -0.18, 0.13, -1.0, 1.0);
    let y = map_range(p.y, 0.3, 1.0, -1.0, 1.0);
    let mut uv = vec2(x,y);
    
    uv.x *= aspect;
    
    uv *= vec2(0.5,0.5);

    uv.x -= 0.115;
    
    if params.rotate { 
        uv = multiply_mat2_with_vec2(rotate_2d(t*0.25), uv);
    }
    
    let loops = 1.0 + (params.loops*150.0);
    let lp = length(vec3(uv.x,uv.y,0.0));
    let id = (lp*loops+0.5).floor() / loops;
    if params.mirror {
        uv.y = uv.y.abs();
        uv.x = uv.x.abs();
    }
    let plr = vec2(lp, atan(uv.y, uv.x));
    let mut rz = 1.0-(((plr.x*PI*loops).sin()).abs()*1.25 / w.powf(0.25)).powf(2.5);
    let enp = plr.y + (t+id*5.5).sin() * 1.52 - 1.5;
    rz *= smoothstep(0.0, 0.05, enp);
    rz *= smoothstep(0.0, 0.022*w/plr.x, enp) * step(id, 1.0);
    if params.mirror {
        rz *= smoothstep(-0.01, 0.02*w/plr.x, PI-plr.y);
    }
    let colour_offset = uniforms.slider4 * PI;
    let palette = vec3(0.0,1.4,2.0) + vec3(colour_offset,colour_offset,colour_offset);

    let mut col = vec3(((palette.x+id*5.0+t).sin()*0.5+0.5)*rz,
                ((palette.y+id*5.0+t).sin()*0.5+0.5)*rz,
                ((palette.z+id*5.0+t).sin()*0.5+0.5)*rz);
    col *= smoothstep(0.8, 1.15, rz) * 0.7 + 0.8;
    
    //lin_srgb(lp, lp, 1.0)
    lin_srgb(col.x, col.y, col.z)
}
