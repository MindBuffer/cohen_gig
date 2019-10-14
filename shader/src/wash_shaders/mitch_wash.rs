use nannou::prelude::*;
use shader_shared::{Button, Uniforms, Vertex, Light};

use crate::helpers::*;

struct Params {
    speed: f32,
    pulse_speed: f32,
}

pub fn shader(v: Vertex , uniforms: &Uniforms) -> LinSrgb {
    let speed = uniforms.params.mitch_wash.speed;
    let pulse_speed = uniforms.params.mitch_wash.pulse_speed;

    let p = v.position;
    let t = uniforms.time * speed;
    let b = (p.z + t).sin() * 0.5 + 0.5;
    let r = (p.x + t * 2.0 * p.x.signum()).cos() * 0.5 + 0.5;
    let g = (p.y + t).cos() * 0.5 + 0.5;
    let mut col = vec3(0.0,0.0,0.0);//vec3(b*r*0.5, g*b, b);

    // Add a burst of light emanating from the led wall down the venue on cycle press.
    if let Some(state) = uniforms.buttons.get(&Button::Cycle) {
        let s = state.secs * (0.1 + pulse_speed);
        let env = (1.0 - s).max(0.0).powf(2.0);
        let m = p.magnitude();
        let dist = (m - s * 4.0).abs();
        let l = (1.0 - dist * 2.0).max(0.0);
        let glow = l * env;
        col += vec3(1.0, 1.0, 1.0) * glow;
    }

    lin_srgb(col.x, col.y, col.z)
}
