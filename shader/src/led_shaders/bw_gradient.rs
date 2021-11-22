use nannou_core::prelude::*;
use shader_shared::{Light, Uniforms, Vertex};

use crate::helpers::*;
use crate::signals::*;

// https://www.interactiveshaderformat.com/sketches/2329

/* PARAMS
- speed
- dc
- amp
*/

enum Direction {
    Vertical,
    Horizontal,
}

// struct Params {
//     speed: f32,
//     dc: f32,
//     amp: f32,
//     freq: f32,
//     mirror: bool,
// }

//--------- Colour Palette
fn palette(t: f32, signal: &Signal, a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> Vec3 {
    a + b * vec3(
        signal.amp(TWO_PI * (c.x * t + d.x)),
        signal.amp(TWO_PI * (c.y * t + d.y)),
        signal.amp(TWO_PI * (c.z * t + d.z)),
    )
}

pub fn shader(v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let mut params = uniforms.params.bw_gradient;

    let mut direction = Direction::Vertical;
    let signal_type = Signal::SINE;

    if uniforms.use_midi {
        direction = if uniforms.slider1 > 0.5 {
            Direction::Vertical
        } else {
            Direction::Horizontal
        };

        params.mirror = if uniforms.slider2 > 0.5 { true } else { false };
    }

    let phase = uniforms.time * params.speed;

    let mut uv = match v.light {
        Light::Wash { index } => pt2(v.position.x, v.position.z * 2.0 - 1.0),
        Light::Led {
            index,
            col_row,
            normalised_coords,
        } => normalised_coords,
    };

    // let x = map_range(p.x, -0.13, 0.13, -1.0, 1.0);
    // let y = map_range(p.y, 0.3, 1.0, -1.0, 1.0);
    // let mut uv = vec2(x,y);

    if params.mirror {
        match direction {
            Direction::Vertical => uv.y = uv.y.abs(),
            Direction::Horizontal => uv.x = uv.x.abs(),
        }
    }

    // animate
    let t = uniforms.time * params.speed;
    match direction {
        Direction::Vertical => uv.y += t,
        Direction::Horizontal => uv.x += t,
    }

    let idx = 0.10;

    let d = match direction {
        Direction::Vertical => uv.y,
        Direction::Horizontal => uv.x * 0.15,
    };

    let col = palette(
        d,
        &signal_type,
        vec3(params.dc, params.dc, params.dc),
        vec3(params.amp, params.amp, params.amp),
        vec3(idx + params.freq, idx + params.freq, idx + params.freq),
        vec3(idx * phase, idx * phase, idx * phase),
    );

    lin_srgb(col.x, col.y, col.z)
}
