use nannou::prelude::*;
use shader_shared::Uniforms;

struct Params {
    red: f32,
    green: f32,
    blue: f32,
}


pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let mut params = Params {
        red: 1.0,
        green: 0.0,
        blue: 0.0,
    };

    params.red = uniforms.pot6;
    params.green = uniforms.pot7;
    params.blue = uniforms.pot8;
    
    lin_srgb(params.red, params.green, params.blue)
}
