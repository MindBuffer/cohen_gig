//! The shader function hotloaded at runtime by the cohen_gig crate.

use nannou::prelude::*;
use shader_shared::Uniforms;

#[no_mangle]
fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let t = uniforms.time;
    let b = (p.z + t).sin() * 0.5 + 0.5;
    let r = (p.x + t * 2.0 * p.x.signum()).cos() * 0.5 + 0.5;
    let g = (p.y + t).cos() * 0.5 + 0.5;
    let col = vec3(b*r*0.5, g*b, b);
    lin_srgb(col.x, col.y, col.z)
}
