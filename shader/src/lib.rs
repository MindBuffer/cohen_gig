//! The shader function hotloaded at runtime by the cohen_gig crate.

use nannou::prelude::*;
use shader_shared::Uniforms;

mod signals;
mod helpers;
mod shaders;

mod bw_gradient;
mod line_gradient;
mod square_tunnel;
mod escher_tilings;
mod blinky_circles;
mod just_relax;
mod acid_gradient;
mod the_pulse;
mod gilmore_acid;
mod life_led_wall;
mod particle_zoom;

#[no_mangle]
fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let t = uniforms.time;
    let b = (p.z + t).sin() * 0.5 + 0.5;
    let r = (p.x + t * 2.0 * p.x.signum()).cos() * 0.5 + 0.5;
    let g = (p.y + t).cos() * 0.5 + 0.5;
    let col = vec3(b*r*0.5, g*b, b);
    //lin_srgb(col.x, col.y, col.z)* line_gradient::shader(p, uniforms)

    acid_gradient::shader(p, uniforms) * escher_tilings::shader(p, uniforms)
    //blinky_circles::shader(p, uniforms)
    //bw_gradient::shader(p, uniforms)
    //escher_tilings::shader(p, uniforms)
    //gilmore_acid::shader(p, uniforms)
    //just_relax::shader(p, uniforms)
    //line_gradient::shader(p, uniforms)
    //square_tunnel::shader(p, uniforms)
    //the_pulse::shader(p, uniforms)
    //life_led_wall::shader(p, uniforms)
    //particle_zoom::shader(p, uniforms)
    //shaders::tunnel_projection(p, uniforms)
    
}
