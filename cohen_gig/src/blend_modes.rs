use nannou::prelude::*;

pub fn add(base: LinSrgb, blend: LinSrgb) -> LinSrgb {
    lin_srgb((base.red+blend.red).min(1.0), (base.green+blend.green).min(1.0), (base.blue+blend.blue).min(1.0))
}

pub fn subtract(base: LinSrgb, blend: LinSrgb) -> LinSrgb {
    lin_srgb((base.red+blend.red-1.0).max(0.0) , (base.green+blend.green-1.0).max(0.0), (base.blue+blend.blue-1.0).max(0.0))
}

pub fn multiply(base: LinSrgb, blend: LinSrgb) -> LinSrgb {
    lin_srgb(base.red*blend.red, base.green*blend.green, base.blue*blend.blue)
}

pub fn average(base: LinSrgb, blend: LinSrgb) -> LinSrgb {
    lin_srgb((base.red+blend.red)/2.0, (base.green+blend.green)/2.0, (base.blue+blend.blue)/2.0)
}

pub fn difference(base: LinSrgb, blend: LinSrgb) -> LinSrgb {
    lin_srgb((base.red-blend.red).abs(), (base.green-blend.green).abs(), (base.blue-blend.blue).abs())
}

pub fn negation(base: LinSrgb, blend: LinSrgb) -> LinSrgb {
    lin_srgb(1.0-(1.0-base.red-blend.red).abs(), 1.0-(1.0-base.green-blend.green).abs(), 1.0-(1.0-base.blue-blend.blue).abs())
}

pub fn exclusion(base: LinSrgb, blend: LinSrgb) -> LinSrgb {
    lin_srgb(base.red+blend.red-2.0*base.red*blend.red, base.green+blend.green-2.0*base.green*blend.green, base.blue+blend.blue-2.0*base.blue*blend.blue)
}
