use nannou::prelude::*;
use shader_shared::Uniforms;

use crate::signals::*;
use crate::helpers::*;

// https://www.interactiveshaderformat.com/sketches/4822

struct Params {
    speed: f32,
    scale: f32,
    colour_iter: f32,
    line_amp: f32,
    diag_amp: f32,
    boarder_amp: f32,
}

fn hue(colour: Vector3, shift: f32) -> Vector3 {
    let k_rgb_to_yprime: Vector3 = vec3(0.299, 0.587, 0.114);
    let k_rgb_to_i: Vector3 = vec3(0.596, -0.275, -0.321);
    let k_rgb_to_q: Vector3 = vec3(0.212, -0.523, 0.311);

    let k_yiq_to_r: Vector3 = vec3(1.0, 0.956, 0.621);
    let k_yiq_to_g: Vector3 = vec3(1.0, -0.272, -0.647);
    let k_yiq_to_b: Vector3 = vec3(1.0, -1.107, 1.704);

    // Convert to YIQ
    let y_prime = colour.dot(k_rgb_to_yprime);
    let mut i = colour.dot(k_rgb_to_i);
    let mut q = colour.dot(k_rgb_to_q);

    // Calculate the hue and chroma
    let mut hue = atan(q, i);
    let chroma = (i * i + q * q).sqrt();

    // Make the user's adjustments
    hue += shift;

    // Convert back to YIQ
    q = chroma * hue.sin();
    i = chroma * hue.cos();

    // Convert back to RGB
    let y_iq = vec3(y_prime, i, q);
    let mut c = vec3(0.0,0.0,0.0);
    c.x = y_iq.dot(k_yiq_to_r);
    c.y = y_iq.dot(k_yiq_to_g);
    c.z = y_iq.dot(k_yiq_to_b);
    c
    //colour.x = yIQ.dot(kYIQToR);
    //colour.y = yIQ.dot(kYIQToG);
    //colour.z = yIQ.dot(kYIQToB);
    //colour
}
pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let params = Params {
        speed: 2.15,
        scale: 0.83,
        colour_iter: 0.015,
        line_amp: 1.0,
        diag_amp: 0.0,
        boarder_amp: 9.0,
    };

    let t = uniforms.time * params.speed;
    
    let x = map_range(p.x, -0.13, 0.13, 0.0, 1.0);
    let y = map_range(p.y, 0.25, 1.0, 0.0, 1.0);
    let mut uv = vec2(x,y) * uniforms.resolution;

    let i = 4.0 + params.scale * 35.0;
    uv = uv / vec2(uniforms.resolution.x, uniforms.resolution.x) * vec2(i,i);
    let mut d = uv.y;
    let a = atan(uv.y, uv.x) + (d*0.4+t*0.3).sin() * (1.0 / uniforms.resolution.y * 2.0) + t * 0.2;
    d = d.powf(1.5);
    let j = (t*0.4).powf(PI);
    let mut f;
    f = (((fmod(uv.x,1.0) - 0.5).abs() - 0.45 ) * params.boarder_amp) * params.line_amp;    
    f = f.max( ((fmod(uv.y,0.5)-0.25).abs() - 0.2) * params.boarder_amp) * params.line_amp;

    f = mix(f, f.max( ((fmod(uv.y+uv.x*1.5,1.0)-0.5).abs() - 0.4) * params.boarder_amp), params.diag_amp);
    f = mix(f, f.max( ((fmod(uv.y+uv.x*-1.5,1.0)-0.5).abs() - 0.4) * params.boarder_amp), params.diag_amp);

    let mut c = vec3(0.0,0.0,0.0);
    c.x = f;
    c.y = (f + t.sin()).cos() * 0.5 + 0.5;
    c.z = f.abs();

    let s = t + d * (params.colour_iter * 0.5);
    c = hue(c,s);

    lin_srgb(c.x, c.y, c.z)
}
