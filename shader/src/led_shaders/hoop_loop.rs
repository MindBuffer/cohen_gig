use nannou_core::prelude::*;
use shader_shared::{HoopLoop, Light, Uniforms, Vertex};

use crate::helpers::{smoothstep, TWO_PI};

const NUM_CIRCLES: usize = 50;

fn glsl_fract(value: f32) -> f32 {
    value - value.floor()
}

fn draw_circle(
    p: Vec2,
    center: Vec2,
    radius: f32,
    edge_width: f32,
    color: Vec3,
    params: &HoopLoop,
) -> Vec3 {
    let dist = (p - center).length();
    let look = match params.function {
        0 => (dist - params.size).abs(),
        1 => glsl_fract(dist - params.size),
        _ => (dist - params.size).abs(),
    };

    color * (1.0 - params.line_effect - smoothstep(radius, radius + edge_width, look))
}

fn invert_color(color: Vec3) -> Vec3 {
    color * -1.0 + vec3(1.0, 1.0, 1.0)
}

fn rot(uv: Vec2, angle: f32) -> Vec2 {
    vec2(
        uv.x * angle.cos() - uv.y * angle.sin(),
        uv.y * angle.cos() + uv.x * angle.sin(),
    )
}

pub fn shader(v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let params = uniforms.params.hoop_loop;

    let Light::Led {
        normalised_coords, ..
    } = v.light;

    let mut uv = vec2(
        map_range(normalised_coords.x, -1.0, 1.0, 0.0, 1.0),
        map_range(normalised_coords.y, -1.0, 1.0, 0.0, 1.0),
    );
    uv -= vec2(params.pos_x, params.pos_y);
    uv.x *= uniforms.resolution.x / uniforms.resolution.y;
    uv *= params.zoom;
    uv = rot(uv, params.rotate * PI);

    let mut color = vec3(0.0, 0.0, 0.0);
    let angle_increment = TWO_PI / NUM_CIRCLES as f32;
    let mut loop_uv = uv;

    for i in 0..NUM_CIRCLES {
        let t = angle_increment * i as f32;
        let r = (params.r_sin * t + uniforms.time * params.animate).sin();
        let p = vec2(r * (t * params.x_cos).cos(), r * (t * params.y_sin).sin());

        loop_uv = rot(loop_uv, params.pattern_offset * PI);

        if params.line_effect >= 0.2 {
            color = invert_color(color);
        }

        color += draw_circle(
            loop_uv,
            p,
            params.thickness,
            params.blur,
            vec3(1.0, 1.0, 1.0),
            &params,
        );
    }

    if params.invert {
        color = invert_color(color);
    }

    lin_srgb(color.x, color.y, color.z)
}
