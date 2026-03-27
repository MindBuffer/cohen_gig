use nannou_core::prelude::*;
use shader_shared::ToneMapping;

pub fn apply(x: Vec3, tone_mapping: ToneMapping) -> Vec3 {
    match tone_mapping {
        ToneMapping::None => x,
        ToneMapping::Aces => aces(x),
        ToneMapping::Hable => hable(x),
        ToneMapping::Unreal => unreal(x),
        ToneMapping::Tanh => tanh_curve(x),
    }
}

fn aces(x: Vec3) -> Vec3 {
    const A: f32 = 2.51;
    const B: f32 = 0.03;
    const C: f32 = 2.43;
    const D: f32 = 0.59;
    const E: f32 = 0.14;

    let numerator = x * (x * A + Vec3::splat(B));
    let denominator = x * (x * C + Vec3::splat(D)) + Vec3::splat(E);
    numerator / denominator
}

fn hable(x: Vec3) -> Vec3 {
    let x = x * 16.0;
    const A: f32 = 0.15;
    const B: f32 = 0.50;
    const C: f32 = 0.10;
    const D: f32 = 0.20;
    const E: f32 = 0.02;
    const F: f32 = 0.30;

    ((x * (x * A + Vec3::splat(C * B)) + Vec3::splat(D * E))
        / (x * (x * A + Vec3::splat(B)) + Vec3::splat(D * F)))
        - Vec3::splat(E / F)
}

fn unreal(x: Vec3) -> Vec3 {
    x / (x + Vec3::splat(0.155)) * 1.019
}

fn tanh_curve(x: Vec3) -> Vec3 {
    vec3(
        component_tanh(x.x),
        component_tanh(x.y),
        component_tanh(x.z),
    )
}

fn component_tanh(x: f32) -> f32 {
    let x = x.clamp(-40.0, 40.0);
    let pos = x.exp();
    let neg = (-x).exp();
    (pos - neg) / (pos + neg)
}
