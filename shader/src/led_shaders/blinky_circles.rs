use nannou::prelude::*;
use shader_shared::Uniforms;

use crate::helpers::*;

// https://www.interactiveshaderformat.com/sketches/1617

/* PARAMS
- speed
- zoom
- offset
*/

//iq colour palette
fn palette(t: f32, a: Vector3, b: Vector3, c: Vector3, d: Vector3) -> Vector3 {
    a + b * vec3(
        (TWO_PI * (c.x * t + d.x)).cos(),
        (TWO_PI * (c.y * t + d.y)).cos(),
        (TWO_PI * (c.z * t + d.z)).cos())
}
pub fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let mut params = uniforms.params.blinky_circles;

    let t = uniforms.time * params.speed;

    if uniforms.use_midi {
        params.zoom = uniforms.slider3;
        params.offset = uniforms.slider4;
    }
    let d = 0.3 * (params.offset* 10.0);

    let x = map_range(p.x, -0.13, 0.13, 0.0, 1.0);
    let y = map_range(p.y, 0.3, 1.0, 0.0,1.0);
    let mut uv = vec2(x,y);// / uniforms.resolution;
    uv = uv * vec2(2.0,2.0) - vec2(1.0,1.0);
    uv.x *= uniforms.resolution.x / uniforms.resolution.y;

    let z = 8.0 + (params.zoom * 64.0);
    let mut g = uv * vec2(z,z);
    uv = vec2(d,d) * (vec2(g.x.floor(), g.y.floor()) + vec2(0.5,0.5)) / vec2(z,z);
    g = vec2(g.x.fract(), g.y.fract()) * vec2(2.0,2.0) - vec2(1.0,1.0);
    
    let f = uv.dot(uv) - t;

    let c = palette( f * 0.5 + t,
            vec3(0.5,0.5,0.5),
            vec3(0.5,0.5,0.5),
            vec3(1.0,1.0,1.0),
            vec3(0.0,0.10,0.2));

    let x1 = (1.5 + t*0.2).sin() * 8.0;
    let x2 = (2.5 + t*0.1).cos() * 4.0;

    let r1 = length(vec3(uv.x * -x1,uv.y * x1,0.0)).powf(10.5);
    let r2 = length(vec3(uv.x * -x2 ,uv.y * -x2, 0.0)).powf(10.5);
    let r3= r1*r2;

    let mut e = (1.0 - g.dot(g)) * 0.2 / ((f.fract() - 0.5) * 8.0).abs();
    e = map_range(e, -0.02, 0.02, 0.0, 1.0);
    let e = 1.0-((1.0) * 0.2 / ((f.fract() - 0.5) * 8.0).abs()).sqrt() * r3;//.powf(0.75);
    //dbg!(g);
    
    //lin_srgb(r1*r2,r1*r2, r1*r2)
    lin_srgb(c.x * e, c.y * e, c.z * e)
    
}
