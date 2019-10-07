use nannou::prelude::*;
use shader_shared::Uniforms;

// struct Params {
//     red: f32,
//     green: f32,
//     blue: f32,
// }


pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let mut params = uniforms.params.solid_rgb_colour;

    if uniforms.use_midi {
        params.red = uniforms.pot6;
        params.green = uniforms.pot7;
        params.blue = uniforms.pot8;
    }
        
    lin_srgb(params.red, params.green, params.blue)
}
