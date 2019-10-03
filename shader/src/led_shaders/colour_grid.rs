use nannou::prelude::*;
use shader_shared::Uniforms;

use crate::helpers::*;

// https://www.interactiveshaderformat.com/sketches/529

// IQ_ColoredGridThingy by mojovideotech
// source : www.shadertoy.com/view/4dBSRK
// created by IQ : www.iquilezles.org/
// interactive mods by DoctorMojo : www.mojovideotech.com/

///////////////////////////////////

// Created by inigo quilez - iq/2014
// License Creative Commons Attribution-NonCommercial-ShareAlike 3.0 Unported License.

struct Params {
    speed: f32,
    zoom_amount: f32,
}

pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let params = Params {
        speed: 2.5,
        zoom_amount: uniforms.slider1 * 100.0,
    };

    let t = uniforms.time * params.speed;
    
    let x = map_range(p.x, -0.18, 0.13, 0.0, 1.0);
    let y = map_range(p.y, 0.3, 1.0, 0.0, 1.0);
    let mut uv = vec2(x,y) * uniforms.resolution;
    uv *= params.zoom_amount / uniforms.resolution.y;
    let px = uv;
    let id = 0.5 + 0.5 * (t + (vec2((px.x+0.5).floor(), (px.y+0.5).floor()).dot(vec2(113.1, 17.81)).sin())*43758.545).cos();
    
    let co = vec3(0.5 + 0.5 * (t + 3.5 * id + 0.0).cos(),
                0.5 + 0.5 * (t + 3.5 * id + 1.57).cos(),
                0.5 + 0.5 * (t + 3.5 * id + 3.14).cos()); 

    let pa = vec2(id*(0.5+0.5*(TWO_PI*px.x)).cos(),
                id*(0.5+0.5*(TWO_PI*px.y)).cos());

    let c = vec3(co.x*pa.x*pa.y, co.y*pa.x*pa.y, co.z*pa.x*pa.y);

    lin_srgb(c.x, c.y, c.z)
}
