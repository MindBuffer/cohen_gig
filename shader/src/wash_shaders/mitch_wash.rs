use nannou::prelude::*;
use shader_shared::Uniforms;

use crate::helpers::*;

struct Params {
    speed: f32,
}

pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let params = Params {
        speed: 0.5,
    };

    let t = uniforms.time * params.speed;
    let b = (p.z + t).sin() * 0.5 + 0.5;
    let r = (p.x + t * 2.0 * p.x.signum()).cos() * 0.5 + 0.5;
    let g = (p.y + t).cos() * 0.5 + 0.5;
    let col = vec3(b*r*0.5, g*b, b);
    lin_srgb(col.x, col.y, col.z)
}
