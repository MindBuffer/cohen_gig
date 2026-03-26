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
    let params = uniforms.params.acid_gradient;

    let t = uniforms.time * params.speed;
    let d = 0.3 * (1.0 + params.offset * 10.0);

    let Light::Led {
        normalised_coords, ..
    } = v.light;
    let mut uv = normalised_coords;

    uv.x *= uniforms.resolution.x / uniforms.resolution.y;

    let z = 8.0 + (params.zoom * 64.0);
    let grid = uv * vec2(z, z);
    uv = vec2(d, d) * (vec2(grid.x.floor(), grid.y.floor()) + vec2(0.5, 0.5)) / vec2(z, z);

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
