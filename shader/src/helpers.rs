use nannou::prelude::*;
use nannou::math::Matrix2;

pub const TWO_PI: f32 = 6.2831853072;
pub const HALF_PI: f32 = 1.5707963267948966;
pub const TAU: f32 = (2.0*PI);

///---------------- CONVERSION HELPERS
pub fn length(p: Vector3) -> f32 {
    p.dot(p).sqrt()
}

pub fn atan(x: f32, y: f32) -> f32 {
    (x / y).atan()
}

pub fn smoothstep(edge0: f32, edge1: f32, p: f32) -> f32 {
    let t = clamp((p - edge0) / (edge1 - edge0), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn step(edge: f32, p: f32) -> f32 {
    if p < edge {
        0.0
    } else {
        1.0
    }
}

pub fn mix(x: f32, y: f32, p: f32) -> f32 {
    x * (1.0 - p) + y * p
}

pub fn fract(p: Vector2) -> Vector2 {
    vec2(p.x.fract(), p.y.fract())
}

pub fn ceil(p: Vector2) -> Vector2 {
    vec2(p.x.ceil(), p.y.ceil())
}

pub fn sin(p: Vector2) -> Vector2 {
    vec2(p.x.sin(), p.y.sin())
}


///--------------- HANDY FUNCTIONS
pub fn rotate_2d(angle: f32) -> Matrix2<f32> {
    Matrix2::new(angle.cos(), -angle.sin(), angle.sin(), angle.cos())
}

pub fn multiply_mat2_with_vec2(mat: Matrix2<f32>, vec: Vector2) -> Vector2 {
    vec2( (mat.x.x * vec.x) +  (mat.y.x * vec.y),
          (mat.x.y * vec.x) +  (mat.y.y * vec.y))
}

pub fn coord_to_hex(coord: Vector2, scale: f32, angle: f32) -> Vector3 {
    let m = rotate_2d(angle);
    let c = multiply_mat2_with_vec2(m, coord);
    let q = (1.0 / 3.0 * 3.0.sqrt() * c.x - 1.0 / 3.0 * c.y) * scale;
    let r = 2.0 / 3.0 * c.y * scale;
    vec3(q, r, -q - r) 
}

pub fn hex_to_cell(hex: Vector3, m: f32) -> Vector3 {
    let x = (hex.x / m).fract() * 2.0 - 1.0;
    let y = (hex.y / m).fract() * 2.0 - 1.0;
    let z = (hex.z / m).fract() * 2.0 - 1.0;
    vec3(x,y,z)
}

pub fn abs_max(v: Vector3) -> f32 {
    v.x.abs().max(v.y.abs()).max(v.z.abs())
}

pub fn nsin(value: f32) -> f32 {
    (value * TWO_PI).sin() * 0.5 + 0.5
}

pub fn hex_to_float(hex: Vector3, amt: f32) -> f32 {
    mix(abs_max(hex), 1.0 - length(hex) / 3.0.sqrt(), amt)
}


pub fn rand (uv: Vector2) -> f32{
    (uv.dot(vec2(12.9898,78.233)).sin()*43758.5453123).fract()
}