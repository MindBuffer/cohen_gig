use nannou::prelude::*;

/// All statically known wash light areas.
pub const AREAS: &[Area] = &[
    roof::L_AREA,
    roof::R_AREA,
    floor::L_AREA,
    floor::R_AREA,
    wall::R_AREA,
    wall::L_AREA,
];

pub struct Area {
    pub pn: Point2,
    pub vn: Vector2,
    pub rad: f32,
}

pub fn fade_scalar(w: f32, h: f32) -> f32 {
    use crate::arch::*;
    ((R - L) - w.max(h)) * 0.4
}

pub fn apply_fade(c: LinSrgb, f: f32) -> LinSrgb {
    lin_srgb(c.red * f, c.green * f, c.blue * f)
}

pub mod floor {
    use nannou::prelude::*;
    use crate::arch;
    use crate::wash::Area;
    pub const L: f32 = arch::L + arch::W * 0.3;
    pub const R: f32 = arch::L + arch::W * 0.7;
    pub const Y: f32 = arch::B;
    pub const W: f32 = arch::W * 0.3;
    pub const H: f32 = W * 0.3;

    pub const L_AREA: Area = Area {
        pn: Point2 { x: L, y: Y },
        vn: Vector2 { x: W, y: H },
        rad: 0.0,
    };
    pub const R_AREA: Area = Area {
        pn: Point2 { x: R, y: Y },
        vn: Vector2 { x: W, y: H },
        rad: 0.0,
    };
}

pub mod wall {
    use nannou::prelude::*;
    use crate::arch;
    use crate::wash::Area;
    pub const H: f32 = arch::W * 0.5;
    pub const W: f32 = H * 0.3;
    pub const L: f32 = arch::L - arch::W * 0.4;
    pub const R: f32 = arch::R + arch::W * 0.4;
    pub const Y: f32 = arch::B + arch::H * 0.5;

    pub const L_AREA: Area = Area {
        pn: Point2 { x: L, y: Y },
        vn: Vector2 { x: W, y: H },
        rad: 0.0,
    };
    pub const R_AREA: Area = Area {
        pn: Point2 { x: R, y: Y },
        vn: Vector2 { x: W, y: H },
        rad: 0.0,
    };
}

pub mod roof {
    use nannou::prelude::*;
    use crate::arch;
    use crate::wash::Area;
    pub const H: f32 = arch::W * 0.6;
    pub const W: f32 = H * 0.3;
    pub const L: f32 = arch::L + arch::W * 0.2;
    pub const R: f32 = arch::L + arch::W * 0.8;
    pub const Y: f32 = arch::T + arch::H * 1.5;
    pub const L_RAD: f32 = 0.8;
    pub const R_RAD: f32 = -L_RAD;

    pub const L_AREA: Area = Area {
        pn: Point2 { x: L, y: Y },
        vn: Vector2 { x: W, y: H },
        rad: L_RAD,
    };
    pub const R_AREA: Area = Area {
        pn: Point2 { x: R, y: Y },
        vn: Vector2 { x: W, y: H },
        rad: R_RAD,
    };
}
