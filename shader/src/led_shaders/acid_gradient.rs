use nannou_core::prelude::*;
use shader_shared::{Light, Uniforms, Vertex};

use crate::helpers::*;

// https://www.interactiveshaderformat.com/sketches/1617

/* PARAMS
- speed
- zoom
- offset
*/

//iq colour palette
fn palette(t: f32, a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> Vec3 {
    a + b * vec3(
        (TWO_PI * (c.x * t + d.x)).cos(),
        (TWO_PI * (c.y * t + d.y)).cos(),
        (TWO_PI * (c.z * t + d.z)).cos(),
    )
}
pub fn shader(v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let mut params = uniforms.params.acid_gradient;

    let t = uniforms.time * params.speed;
    if uniforms.use_midi {
        params.zoom = map_range((t * 0.5).sin(), -1.0, 1.0, 0.0, uniforms.slider3);
        params.offset = map_range((t * 0.2).sin(), -1.0, 1.0, 0.0, uniforms.slider4);
    }

    let d = 0.3 * (1.0 + params.offset * 10.0);

    let mut uv = match v.light {
        Light::Wash { index } => pt2(v.position.x, v.position.z * 2.0 - 1.0),
        Light::Led {
            index,
            col_row,
            normalised_coords,
        } => normalised_coords,
    };

    uv.x *= uniforms.resolution.x / uniforms.resolution.y;

    let z = 8.0 + (params.zoom * 64.0);
    let mut g = uv * vec2(z, z);
    uv = vec2(d, d) * (vec2(g.x.floor(), g.y.floor()) + vec2(0.5, 0.5)) / vec2(z, z);
    g = vec2(g.x.fract(), g.y.fract()) * vec2(2.0, 2.0) - vec2(1.0, 1.0);

    let f = uv.dot(uv) - t;

    let c = palette(
        f * 0.5 + t,
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(1.0, 1.0, 1.0),
        vec3(0.5, 0.10, 0.0),
    );

    //let e = (1.0 - g.dot(g)) * 0.2 / ((f.fract() - 0.5) * 8.0).abs();
    let e = 1.0 - ((1.0) * 0.2 / ((f.fract() - 0.5) * 8.0).abs()).sqrt(); //.powf(0.75);

    lin_srgb(c.x * e, c.y * e, c.z * e)
}
