use nannou_core::prelude::*;
use shader_shared::{Light, Uniforms, Vertex};

use crate::helpers::*;

// https://www.interactiveshaderformat.com/sketches/1678

// struct Params {
//     speed: f32,
//     scale: f32,
//     colour_iter: f32,
//     thickness: f32,
// }

fn hsv(h: f32, s: f32, v: f32) -> Vec3 {
    let f = vec3(h + 3.0, h + 2.0, h + 1.0) / vec3(3.0, 3.0, 3.0);
    let f1 =
        vec3(f.x.fract(), f.y.fract(), f.z.fract()) * vec3(6.0, 6.0, 6.0) - vec3(3.0, 3.0, 3.0);
    let f2 = vec3(f1.x.abs(), f1.y.abs(), f1.z.abs()) - vec3(1.0, 1.0, 1.0);
    let f3 = vec3(
        clamp(f2.x, 0.0, 1.0),
        clamp(f2.y, 0.0, 1.0),
        clamp(f2.z, 0.0, 1.0),
    );
    vec3(
        mix(1.0, f3.x, s) * v,
        mix(1.0, f3.y, s) * v,
        mix(1.0, f3.z, s) * v,
    )
}

fn circle(uv: Vec2, r: f32, thickness: f32) -> f32 {
    smoothstep(
        0.1 + (thickness * 0.6),
        0.0,
        (length(vec3(uv.x, uv.y, 0.0)) - r).abs(),
    )
}

pub fn shader(v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let mut params = uniforms.params.the_pulse;

    if uniforms.use_midi {
        params.scale = uniforms.slider3 * 0.8;
        params.colour_iter = uniforms.slider4;
    }

    let mut uv = match v.light {
        Light::Wash { index } => pt2(v.position.x, v.position.z * 2.0 - 1.0),
        Light::Led {
            index,
            col_row,
            normalised_coords,
        } => normalised_coords,
    };

    uv.x *= uniforms.resolution.x / uniforms.resolution.y;
    uv *= vec2(2.0 + (params.scale * 50.0), 2.0 + (params.scale * 50.0));
    let r = smoothstep(
        -0.7,
        0.7,
        (uniforms.time * (0.1 + params.speed * 3.5)
            - length(vec3(uv.x, uv.y, 0.0)) * (0.01 + params.colour_iter * 0.7))
            .sin(),
    ) + 1.0;
    let r3 = 3.0.sqrt();
    let rep = vec2(4.0, r3 * 4.0);
    let p1 = vec2(
        fmod(uv.x, rep.x) - rep.x * 0.5,
        fmod(uv.y, rep.y) - rep.y * 0.5,
    );
    let p2 = vec2(
        fmod(uv.x + 2.0, rep.x) - rep.x * 0.5,
        fmod(uv.y, rep.y) - rep.y * 0.5,
    );
    let p3 = vec2(
        fmod(uv.x + 1.0, rep.x) - rep.x * 0.5,
        fmod(uv.y + r3, rep.y) - rep.y * 0.5,
    );
    let p4 = vec2(
        fmod(uv.x + 3.0, rep.x) - rep.x * 0.5,
        fmod(uv.y + r3, rep.y) - rep.y * 0.5,
    );
    let p5 = vec2(
        fmod(uv.x + 0.0, rep.x) - rep.x * 0.5,
        fmod(uv.y + r3 * 2.0, rep.y) - rep.y * 0.5,
    );
    let p6 = vec2(
        fmod(uv.x + 2.0, rep.x) - rep.x * 0.5,
        fmod(uv.y + r3 * 2.0, rep.y) - rep.y * 0.5,
    );
    let p7 = vec2(
        fmod(uv.x + 1.0, rep.x) - rep.x * 0.5,
        fmod(uv.y + r3 * 2.0, rep.y) - rep.y * 0.5,
    );
    let p8 = vec2(
        fmod(uv.x + 3.0, rep.x) - rep.x * 0.5,
        fmod(uv.y + r3 * 2.0, rep.y) - rep.y * 0.5,
    );

    let mut c = 0.0;
    c += circle(p1, r, params.thickness);
    c += circle(p2, r, params.thickness);
    c += circle(p3, r, params.thickness);
    c += circle(p4, r, params.thickness);
    c += circle(p5, r, params.thickness);
    c += circle(p6, r, params.thickness);
    c += circle(p7, r, params.thickness);
    c += circle(p8, r, params.thickness);

    let col = hsv(r + 0.7, 1.0, c);
    lin_srgb(col.x, col.y, col.z)
}
