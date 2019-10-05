use nannou::prelude::*;
use shader_shared::Uniforms;

use crate::helpers::*;

// https://www.interactiveshaderformat.com/sketches/4211

////////////////////////////////////////////////////////////
// Equirec_SpiralIntersect   by mojovideotech
//
// mod of 
// shadertoy.com\/4dyfW1 by iridule
//
// Creative Commons Attribution-NonCommercial-ShareAlike 3.0
////////////////////////////////////////////////////////////

// struct Params {
//     speed: f32,
//     g1: f32,
//     g2: f32,
//     rot1: f32,
//     rot2: f32,
//     colours: f32,
// }

fn spiral(u: Vector2, a: f32, r: f32, t: f32, d: f32) -> f32 {
    ((t+r*length(vec3(u.x,u.y,0.0))+a*(d*atan(u.y,u.x))).sin()).abs()
}

fn sinp(p: f32) -> f32 {
    0.5+p.sin()*0.5
}

pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let mut params = uniforms.params.spiral_intersect;

    let mut t = uniforms.time * params.speed;

    params.g1 = uniforms.slider3;
    params.g2 = uniforms.slider3;

    params.colours = uniforms.slider4;
    
    let x = map_range(p.x, -0.18, 0.13, 0.0, 1.0);
    let y = map_range(p.y, 0.25, 1.05, 0.0, 1.00);
    let uv = vec2(x,y);// / uniforms.resolution;
    let th = uv.y * PI;
    let ph = uv.x * TWO_PI;
    let mut st = vec3(th.sin() * ph.cos(), -th.cos(), th.sin() * ph.sin());

    let mut xz = vec2(st.x, st.z);
    xz = multiply_mat2_with_vec2(rotate_2d(-t / (params.rot1*8.0)), xz);
    let mut xy = vec2(xz.x, st.y);
    xy = multiply_mat2_with_vec2(rotate_2d(t / (params.rot2*8.0)), xy);
    st = vec3(xy.x, xy.y, xz.y);
    let o = vec2((t/params.rot1).cos(),(t/params.rot2).sin());
    let mut col = vec![0.0; 3];
    for i in 0..3 {
        t += 0.3 * spiral(o+vec2(st.z,st.y),params.g1*12.0,16.0+128.0*o.x-o.y,-t/100.0,1.0)
            * spiral(o-vec2(st.x,st.z),params.g2*18.0,16.0+64.0*o.x-o.y,t/100.0,-1.0);
        col[i] = ((params.colours*0.1)*t-length(vec3(st.x,st.y,0.0))*10.0*sinp(t)).sin();
    }
    
    //lin_srgb(uv.x, uv.y, 1.0)
    lin_srgb(col[0], col[1], col[2])
}
