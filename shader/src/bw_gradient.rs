use nannou::prelude::*;
use shader_shared::Uniforms;

use crate::signals::*;
use crate::helpers::*;

// https://www.interactiveshaderformat.com/sketches/2329

enum Direction {
    Vertical,
    Horizontal,
}

struct Params {
    speed: f32,
    dc: f32,
    amp: f32,
    freq: f32,
    num_bands: f32,
    direction: Direction,
    mirror: bool,
    signal_type: Signal,
}

//--------- Colour Palette
fn palette(t: f32, signal: &Signal, a: Vector3, b: Vector3, c: Vector3, d: Vector3) -> Vector3 {
    a + b * vec3(
        signal.amp(TWO_PI * (c.x * t + d.x)),
        signal.amp(TWO_PI * (c.y * t + d.y)),
        signal.amp(TWO_PI * (c.z * t + d.z)))
}

pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let params = Params {
        speed: 0.03,
        dc: 0.05,
        amp: 0.5,
        freq: 0.5,
        num_bands: 1.0,
        direction: Direction::Horizontal,
        mirror: true,
        signal_type: Signal::SINE_IN_OUT,
    };

    let phase = uniforms.time * params.speed;
    
    let x = map_range(p.x, -0.13, 0.13, -1.0, 1.0);
    let y = map_range(p.y, 0.3, 1.0, -1.0, 1.0);
    let mut uv = vec2(x,y);
    
    

    if params.mirror {
        match params.direction {
            Direction::Horizontal => uv.y = uv.y.abs(),
            Direction::Vertical => uv.x = uv.x.abs(),
        }
    }


    // animate
    let t = uniforms.time * params.speed;
    match params.direction {
        Direction::Horizontal => uv.y += t,
        Direction::Vertical => uv.x += t,
    }

    let idx = 0.10;

    let d = match params.direction {
        Direction::Horizontal => uv.y,
        Direction::Vertical => uv.x,
    };

    let col = palette(d, 
        &params.signal_type,
        vec3(params.dc,params.dc,params.dc),
        vec3(params.amp, params.amp, params.amp),
        vec3(idx + params.freq, idx + params.freq, idx + params.freq),
        vec3(idx * phase, idx * phase, idx * phase));

    lin_srgb(col.x, col.y, col.z)
    
}
