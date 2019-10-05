use nannou::prelude::*;
use shader_shared::Uniforms;

use crate::helpers::*;

// Created by inigo quilez - iq/2015
// License Creative Commons Attribution-NonCommercial-ShareAlike 3.0 Unported License.

// A simple way to create color variation in a cheap way (yes, trigonometrics ARE cheap
// in the GPU, don't try to be smart and use a triangle wave instead).

// See http://iquilezles.org/www/articles/palettes/palettes.htm for more information

struct Params {
    speed: f32,
    interval: f32,
}

//iq colour palette
fn palette(t: f32, a: Vector3, b: Vector3, c: Vector3, d: Vector3) -> Vector3 {
    a + b * vec3(
        (TWO_PI * (c.x * t + d.x)).cos(),
        (TWO_PI * (c.y * t + d.y)).cos(),
        (TWO_PI * (c.z * t + d.z)).cos())
}

pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let mut params = Params {
        speed: 01.25,
        interval: 0.05,
    };

    params.interval = 0.05 + uniforms.slider5;

    let t = uniforms.time * params.speed;
    let mut uv = vec3(p.x,p.y,p.z);

    let selected = map_range(uniforms.slider6,0.0,1.0,0.0,9.0) as usize;
    
    // animate
    uv.z += t;

    let interval = vec3(params.interval,params.interval,params.interval);

    let colz = get_palette(uv.z, selected, interval);

    uv.x += (t * p.x.signum()).cos() * 0.5 + 0.5;
    let colx = get_palette(uv.x, selected, interval);

    let col = colz * colx;
    lin_srgb(col.x, col.y, col.z)
}

fn get_palette(axis: f32, selected: usize, interval: Vector3) -> Vector3 {
    match selected {
        0 => palette(axis,vec3(0.55,0.4,0.3),vec3(0.50,0.51,0.35)+vec3(0.1,0.1,0.1),vec3(0.8,0.75,0.8)*interval,vec3(0.075,0.33,0.67)+vec3(0.21,0.21,0.21)),
        1 => palette(axis,vec3(0.55,0.55,0.55),vec3(0.8,0.8,0.8),vec3(0.29,0.29,0.29)*interval,vec3(0.00,0.05,0.15) + vec3(0.54,0.54,0.54)),
        2 => palette(axis,vec3(0.5,0.5,0.5),vec3(0.55,0.55,0.55),vec3(0.45,0.45,0.45)*interval,vec3(0.00,0.10,0.20) + vec3(0.47,0.47,0.47)),
        3 => palette(axis,vec3(0.5,0.5,0.5),vec3(0.5,0.5,0.5),vec3(0.9,0.9,0.9)*interval,vec3(0.3,0.20,0.20) + vec3(0.31,0.31,0.31)),
        4 => palette(axis,vec3(0.5,0.5,0.5),vec3(0.5,0.5,0.5),vec3(0.9,0.9,0.9)*interval,vec3(0.0,0.10,0.20) + vec3(0.47,0.47,0.47)),
        5 => palette(axis,vec3(0.5,0.5,0.5),vec3(0.5,0.5,0.5),vec3(1.0,1.0,0.5)*interval,vec3(0.8,0.90,0.30)),
        6 => palette(axis,vec3(0.5,0.5,0.5),vec3(0.5,0.5,0.5),vec3(1.0,0.7,0.4)*interval,vec3(0.0,0.15,0.20)),
        7 => palette(axis,vec3(0.5,0.5,0.5),vec3(0.5,0.5,0.5),vec3(2.0,1.0,0.0)*interval,vec3(0.5,0.20,0.25)),
        8 => palette(axis,vec3(0.5,0.5,0.5),vec3(0.5,0.5,0.5),vec3(1.0,1.0,1.0)*interval,vec3(0.0,0.33,0.67)),
        9 => palette(axis,vec3(0.8,0.5,0.4),vec3(0.2,0.4,0.2),vec3(2.0,1.0,1.0)*interval,vec3(0.0,0.25,0.25)),
        _ => vec3(0.0,0.0,0.0),
    }
}
