use korg_nano_kontrol_2::{ButtonRow, Strip};
use nannou_core::prelude::*;
use shader_shared::{Button, Light, Uniforms, Vertex};

use crate::helpers::*;

struct Params {
    speed: f32,
    pulse_speed: f32,
    line_thickness: f32,
    shape_thckness: f32,
}
//---------------------------------------------------------
// draw endless line through point A and B with radius r
//---------------------------------------------------------
fn line(P: Vec2, A: Vec2, B: Vec2, r: f32) -> f32 {
    let g = B - A;
    let d = vec2(g.y, -g.x).normalize().dot(P - A).abs();
    smoothstep(r, 0.5 * r, d)
}
//---------------------------------------------------------
// draw rectangle frame with rounded edges
//---------------------------------------------------------
fn roundedFrame(uv: Vec2, pos: Vec2, size: Vec2, radius: f32, thickness: f32) -> f32 {
    let d = length(vec3(
        abs(uv.x - pos.x).max(size.x) - size.x,
        abs(uv.y - pos.y).max(size.y) - size.y,
        0.0,
    )) - radius;
    smoothstep(0.55, 0.45, abs(d / thickness) * 5.0)
}
//---------------------------------------------------------
// draw ring at pos
//---------------------------------------------------------
fn haloRing(uv: Vec2, pos: Vec2, radius: f32, thick: f32) -> f32 {
    clamp(
        -(abs(length(vec3(uv.x - pos.x, uv.y - pos.y, 0.0)) - radius) * 100.0 / thick) + 0.9,
        0.0,
        1.0,
    )
}

pub fn shader(v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let speed = uniforms.params.shape_envelopes.speed;
    let pulse_speed = uniforms.params.shape_envelopes.pulse_speed;
    let mut line_thickness = uniforms.params.shape_envelopes.line_thickness;
    let mut shape_thickness = uniforms.params.shape_envelopes.shape_thickness;

    let t = uniforms.time * speed;

    if uniforms.use_midi {
        line_thickness = uniforms.slider1;
        shape_thickness = uniforms.slider2;
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

    let mut col = vec3(0.0, 0.0, 0.0);
    let circle_amp = 2.0;
    let square_amp = 1.0;

    //--- HALO RING ---
    let ring_color = vec3(1.0, 1.0, 1.5) * vec3(circle_amp, circle_amp, circle_amp);
    let ring_thickness = 2.0 + (shape_thickness * 40.0);
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Solo, Strip::A))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let env = s.max(0.0).powf(2.0);
        let intensity = haloRing(uv, vec2(0.0, 0.0), env * 2.0, ring_thickness);
        col += vec3(
            mix(col.x, ring_color.x, intensity),
            mix(col.y, ring_color.y, intensity),
            mix(col.z, ring_color.z, intensity),
        );
    }
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Mute, Strip::A))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let env = s.max(0.0).powf(2.0);
        let intensity = haloRing(uv, vec2(0.0, 0.0), env * 2.0, ring_thickness);
        col += vec3(
            mix(col.x, ring_color.x, intensity),
            mix(col.y, ring_color.y, intensity),
            mix(col.z, ring_color.z, intensity),
        );
    }
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Record, Strip::A))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let env = s.max(0.0).powf(2.0);
        let intensity = haloRing(uv, vec2(0.0, 0.0), env * 2.0, ring_thickness);
        col += vec3(
            mix(col.x, ring_color.x, intensity),
            mix(col.y, ring_color.y, intensity),
            mix(col.z, ring_color.z, intensity),
        );
    }

    //--- rounded frame ---
    let frame_color = vec3(1.0, 1.0, 1.5) * vec3(square_amp, square_amp, square_amp);
    let square_thickness = 0.07 + (shape_thickness * 1.7);
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Solo, Strip::B))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let env = s.max(0.0).powf(2.0);
        let size = vec2(env * 2.0, env * 2.0);
        let intensity = roundedFrame(uv, vec2(0.0, 0.0), size, 0.2, square_thickness);
        col += vec3(
            mix(col.x, frame_color.x, intensity),
            mix(col.y, frame_color.y, intensity),
            mix(col.z, frame_color.z, intensity),
        );
    }
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Mute, Strip::B))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let env = s.max(0.0).powf(2.0);
        let size = vec2(env * 2.0, env * 2.0);
        let intensity = roundedFrame(uv, vec2(0.0, 0.0), size, 0.2, square_thickness);
        col += vec3(
            mix(col.x, frame_color.x, intensity),
            mix(col.y, frame_color.y, intensity),
            mix(col.z, frame_color.z, intensity),
        );
    }
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Record, Strip::B))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let env = s.max(0.0).powf(2.0);
        let size = vec2(env * 2.0, env * 2.0);
        let intensity = roundedFrame(uv, vec2(0.0, 0.0), size, 0.2, square_thickness);
        col += vec3(
            mix(col.x, frame_color.x, intensity),
            mix(col.y, frame_color.y, intensity),
            mix(col.z, frame_color.z, intensity),
        );
    }

    // 45 degree square
    let mut uv2 = uv;
    uv2 = multiply_mat2_with_vec2(rotate_2d(t * 0.25), uv2);

    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Solo, Strip::C))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let env = s.max(0.0).powf(2.0);
        let size = vec2(env * 2.0, env * 2.0);
        let intensity = roundedFrame(uv2, vec2(0.0, 0.0), size, 0.2, square_thickness);
        col += vec3(
            mix(col.x, frame_color.x, intensity),
            mix(col.y, frame_color.y, intensity),
            mix(col.z, frame_color.z, intensity),
        );
    }
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Mute, Strip::C))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let env = s.max(0.0).powf(2.0);
        let size = vec2(env * 2.0, env * 2.0);
        let intensity = roundedFrame(uv2, vec2(0.0, 0.0), size, 0.2, square_thickness);
        col += vec3(
            mix(col.x, frame_color.x, intensity),
            mix(col.y, frame_color.y, intensity),
            mix(col.z, frame_color.z, intensity),
        );
    }
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Record, Strip::C))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let env = s.max(0.0).powf(2.0);
        let size = vec2(env * 2.0, env * 2.0);
        let intensity = roundedFrame(uv2, vec2(0.0, 0.0), size, 0.2, square_thickness);
        col += vec3(
            mix(col.x, frame_color.x, intensity),
            mix(col.y, frame_color.y, intensity),
            mix(col.z, frame_color.z, intensity),
        );
    }

    //--- horizontal line ---
    let line_color = vec3(1.0, 1.0, 1.7);
    let line_weight = 0.016 + line_thickness;
    // Line Left
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Solo, Strip::D))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let env = s.max(0.0).powf(1.5) - map_range(line_thickness, 0.0, 1.0, 0.8, 1.3);
        let intensity = line(uv, vec2(env * 2.0, -1.0), vec2(env * 2.0, 1.0), line_weight);
        col += vec3(
            mix(col.x, line_color.x, intensity),
            mix(col.y, line_color.y, intensity),
            mix(col.z, line_color.z, intensity),
        );
    }
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Mute, Strip::D))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let env = s.max(0.0).powf(1.5) - map_range(line_thickness, 0.0, 1.0, 0.8, 1.3);
        let intensity = line(uv, vec2(env * 2.0, -1.0), vec2(env * 2.0, 1.0), line_weight);
        col += vec3(
            mix(col.x, line_color.x, intensity),
            mix(col.y, line_color.y, intensity),
            mix(col.z, line_color.z, intensity),
        );
    }
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Record, Strip::D))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let env = s.max(0.0).powf(1.5) - map_range(line_thickness, 0.0, 1.0, 0.8, 1.3);
        let intensity = line(uv, vec2(env * 2.0, -1.0), vec2(env * 2.0, 1.0), line_weight);
        col += vec3(
            mix(col.x, line_color.x, intensity),
            mix(col.y, line_color.y, intensity),
            mix(col.z, line_color.z, intensity),
        );
    }

    // Line Right
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Solo, Strip::E))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let mut env = s.max(0.0).powf(1.5) - map_range(line_thickness, 0.0, 1.0, 0.8, 1.3);
        env *= -1.0;
        let intensity = line(uv, vec2(env * 2.0, -1.0), vec2(env * 2.0, 1.0), line_weight);
        col += vec3(
            mix(col.x, line_color.x, intensity),
            mix(col.y, line_color.y, intensity),
            mix(col.z, line_color.z, intensity),
        );
    }
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Mute, Strip::E))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let mut env = s.max(0.0).powf(1.5) - map_range(line_thickness, 0.0, 1.0, 0.8, 1.3);
        env *= -1.0;
        let intensity = line(uv, vec2(env * 2.0, -1.0), vec2(env * 2.0, 1.0), line_weight);
        col += vec3(
            mix(col.x, line_color.x, intensity),
            mix(col.y, line_color.y, intensity),
            mix(col.z, line_color.z, intensity),
        );
    }
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Record, Strip::E))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let mut env = s.max(0.0).powf(1.5) - map_range(line_thickness, 0.0, 1.0, 0.8, 1.3);
        env *= -1.0;
        let intensity = line(uv, vec2(env * 2.0, -1.0), vec2(env * 2.0, 1.0), line_weight);
        col += vec3(
            mix(col.x, line_color.x, intensity),
            mix(col.y, line_color.y, intensity),
            mix(col.z, line_color.z, intensity),
        );
    }

    // Line Top
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Solo, Strip::F))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let mut env = s.max(0.0).powf(1.5) - map_range(line_thickness, 0.0, 1.0, 1.0, 1.9);
        env *= -1.0;
        let intensity = line(uv, vec2(-1.0, env * 1.0), vec2(1.0, env * 1.0), line_weight);
        col += vec3(
            mix(col.x, line_color.x, intensity),
            mix(col.y, line_color.y, intensity),
            mix(col.z, line_color.z, intensity),
        );
    }
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Mute, Strip::F))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let mut env = s.max(0.0).powf(1.5) - map_range(line_thickness, 0.0, 1.0, 1.0, 1.9);
        env *= -1.0;
        let intensity = line(uv, vec2(-1.0, env * 1.0), vec2(1.0, env * 1.0), line_weight);
        col += vec3(
            mix(col.x, line_color.x, intensity),
            mix(col.y, line_color.y, intensity),
            mix(col.z, line_color.z, intensity),
        );
    }
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Record, Strip::F))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let mut env = s.max(0.0).powf(1.5) - map_range(line_thickness, 0.0, 1.0, 1.0, 1.9);
        env *= -1.0;
        let intensity = line(uv, vec2(-1.0, env * 1.0), vec2(1.0, env * 1.0), line_weight);
        col += vec3(
            mix(col.x, line_color.x, intensity),
            mix(col.y, line_color.y, intensity),
            mix(col.z, line_color.z, intensity),
        );
    }

    // Line Bottom
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Solo, Strip::G))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let env = s.max(0.0).powf(1.5) - map_range(line_thickness, 0.0, 1.0, 1.0, 1.9);
        let intensity = line(uv, vec2(-1.0, env * 1.0), vec2(1.0, env * 1.0), line_weight);
        col += vec3(
            mix(col.x, line_color.x, intensity),
            mix(col.y, line_color.y, intensity),
            mix(col.z, line_color.z, intensity),
        );
    }
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Mute, Strip::G))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let env = s.max(0.0).powf(1.5) - map_range(line_thickness, 0.0, 1.0, 1.0, 1.9);
        let intensity = line(uv, vec2(-1.0, env * 1.0), vec2(1.0, env * 1.0), line_weight);
        col += vec3(
            mix(col.x, line_color.x, intensity),
            mix(col.y, line_color.y, intensity),
            mix(col.z, line_color.z, intensity),
        );
    }
    if let Some(state) = uniforms
        .buttons
        .get(&Button::Row(ButtonRow::Record, Strip::G))
    {
        let s = state.secs * (0.1 + pulse_speed);
        let env = s.max(0.0).powf(1.5) - map_range(line_thickness, 0.0, 1.0, 1.0, 1.9);
        let intensity = line(uv, vec2(-1.0, env * 1.0), vec2(1.0, env * 1.0), line_weight);
        col += vec3(
            mix(col.x, line_color.x, intensity),
            mix(col.y, line_color.y, intensity),
            mix(col.z, line_color.z, intensity),
        );
    }

    lin_srgb(col.x, col.y, col.z)
}
