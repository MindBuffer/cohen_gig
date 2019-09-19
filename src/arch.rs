use crate::strip;
use nannou::prelude::*;
pub const COUNT: usize = 5;
pub const L: f32 = -1.0;
pub const T: f32 = 0.3;
pub const R: f32 = 1.0;
pub const B: f32 = -0.7;
pub const W: f32 = R - L;
pub const H: f32 = T - B;
pub const W_METRES: f32 = 7.0;
pub const METRES_PER_UNIT: f32 = W_METRES / W;
pub const BL: Point2 = Point2 { x: L, y: B };
pub const TL: Point2 = Point2 { x: L, y: T };
pub const TR: Point2 = Point2 { x: R, y: T };
pub const BR: Point2 = Point2 { x: R, y: B };
pub const PTS: [Point2; 4] = [BL, TL, TR, BR];
pub const Z_GAP: f32 = W * 0.5 * (4.0 / 7.0);

/// A path around the arch subdivided into pixels per metre.
pub fn path_points(ppm: f32) -> impl Iterator<Item = Point2> {
    PTS.windows(2).flat_map(move |w| strip::points(w[0], w[1], ppm))
}
