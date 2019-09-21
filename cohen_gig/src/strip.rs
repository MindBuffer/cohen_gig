use crate::arch;
use nannou::prelude::*;

/// Count the number of points in a strip from a to b.
pub fn count_points(a: Point2, b: Point2, ppm: f32) -> usize {
    let dist = a.distance(b);
    let dist_m = dist * arch::METRES_PER_UNIT;
    (ppm * dist_m) as usize
}

/// Convert the given line into a strip of pixel positions based on ppm.
pub fn points(a: Point2, b: Point2, ppm: f32) -> impl Iterator<Item = Point2> {
    let n_px = count_points(a, b, ppm);
    (0..n_px).map(move |i| {
        let f = i as f32 / n_px as f32;
        a.lerp(b, f)
    })
}
