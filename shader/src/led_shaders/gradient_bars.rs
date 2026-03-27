use nannou_core::prelude::*;
use shader_shared::{Light, Uniforms, Vertex};

use crate::helpers::mix;
use crate::signals::{ease_lfo, EasingType};

const EASING_TYPES: [EasingType; 30] = [
    EasingType::SineIn,
    EasingType::SineOut,
    EasingType::SineInOut,
    EasingType::QuadIn,
    EasingType::QuadOut,
    EasingType::QuadInOut,
    EasingType::CubicIn,
    EasingType::CubicOut,
    EasingType::CubicInOut,
    EasingType::QuartIn,
    EasingType::QuartOut,
    EasingType::QuartInOut,
    EasingType::QuintIn,
    EasingType::QuintOut,
    EasingType::QuintInOut,
    EasingType::ExpoIn,
    EasingType::ExpoOut,
    EasingType::ExpoInOut,
    EasingType::CircIn,
    EasingType::CircOut,
    EasingType::CircInOut,
    EasingType::BackIn,
    EasingType::BackOut,
    EasingType::BackInOut,
    EasingType::ElasticIn,
    EasingType::ElasticOut,
    EasingType::ElasticInOut,
    EasingType::BounceIn,
    EasingType::BounceOut,
    EasingType::BounceInOut,
];

fn glsl_fract(value: f32) -> f32 {
    value - value.floor()
}

fn easing_type(index: usize) -> EasingType {
    EASING_TYPES
        .get(index)
        .copied()
        .unwrap_or(EasingType::SineIn)
}

fn adjust_balance(value: f32, balance: f32) -> f32 {
    let gamma = mix(-2.0, 2.0, balance).exp();
    value.powf(gamma)
}

pub fn shader(v: Vertex, uniforms: &Uniforms) -> LinSrgb {
    let params = uniforms.params.gradient_bars;

    let Light::Led {
        normalised_coords, ..
    } = v.light;

    let uv = vec2(
        map_range(normalised_coords.x, -1.0, 1.0, 0.0, 1.0),
        map_range(normalised_coords.y, -1.0, 1.0, 0.0, 1.0),
    );

    let primary = if params.use_columns { uv.x } else { uv.y };
    let mut secondary = if params.use_columns { uv.y } else { uv.x };

    let mirrored_primary = (primary - 0.5).abs() * params.x_iter;
    let stripe_index = (mirrored_primary * params.num_columns).ceil();

    let phase_offset = stripe_index * (1.0 / (params.num_columns * params.offset.max(0.001)));
    let phase = glsl_fract(uniforms.time * params.speed + phase_offset);
    let lfo = ease_lfo(easing_type(params.easing_type), phase) * params.phase_iter
        - params.phase_iter / 2.0;

    let is_even_stripe = stripe_index.rem_euclid(2.0) < 1.0;
    if params.use_odd_dirs && !is_even_stripe {
        secondary = 1.0 - secondary;
    }

    let mut gradient = secondary.powf(params.gradient_pow);
    let invert_mix = 0.5 + (uniforms.time * params.invert_speed).sin() * 0.5;
    gradient = mix(gradient, 1.0 - gradient, invert_mix);

    let animated_coord = glsl_fract(lfo + gradient);
    let col = 0.5 + 0.5 * (animated_coord * std::f32::consts::TAU).sin();
    let value = 1.0 - adjust_balance(col, params.balance);

    lin_srgb(value, value, value)
}
