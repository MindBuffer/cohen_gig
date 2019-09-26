//! The layout of the structure and lighting according to the given floorplans.

use nannou::prelude::*;

/// A rough polyline that describes the venue walls.
pub const WALL_METRES: &[Point2] = &[
    Point2 { x: 3.0, y: -12.0 },
    Point2 { x: 3.0, y: 0.0 },
    Point2 { x: 11.0, y: 0.0 },
    Point2 { x: 11.0, y: 10.0 },
    Point2 { x: 10.0, y: 12.0 },
    Point2 { x: -12.0, y: 12.0 },
    Point2 { x: -12.0, y: -12.0 },
];

/// Total number of wash lights.
pub const WASH_COUNT: usize = 28;

/// The shader origin position in metres.
pub const SHADER_ORIGIN_METRES: Point2 = Point2 { x: -4.5, y: 12.0 };

/// The rect that bounds the venue in metres.
fn venue_bounding_rect_metres() -> geom::Rect {
    let mut r = geom::Rect::from_wh(vec2(0.0, 0.0));
    for &p in WALL_METRES {
        r = r.stretch_to_point(p);
    }
    r
}

/// Converts the given topdown metres coords to the coordinate system ready for the shader.
pub fn topdown_metres_to_shader_coords(topdown_point_m: Point2) -> Point3 {
    // Translate based on the shader origin.
    let topdown_point_m = topdown_point_m - SHADER_ORIGIN_METRES;
    // Use the bounding rect to normalise the coords using venue width.
    let r = venue_bounding_rect_metres();
    let scale = 1.0 / r.w();
    let topdown_point_s = topdown_point_m * scale;
    // Use the inverse of the y as the z axis for shader coords.
    let Point2 { x, y } = topdown_point_s;
    let point = pt3(x, 0.0, -y);
    point
}

/// The topdown position in metres at which the wash fixture at the given index is fixed.
pub fn wash_index_to_topdown_source_position_metres(idx: usize) -> Point2 {
    match idx {
        0 => pt2(-6.0, 11.0),
        1 => pt2(-5.0, 11.0),
        2 => pt2(-4.0, 11.0),
        3 => pt2(-3.0, 11.0),
        4 => pt2(-6.0, 8.0),
        5 => pt2(-5.0, 8.0),
        6 => pt2(-4.0, 8.0),
        7 => pt2(-3.0, 8.0),
        8 => pt2(-6.0, 3.0),
        9 => pt2(-3.0, 3.0),
        10 => pt2(-10.0, -3.0),
        11 => pt2(-6.0, -3.0),
        12 => pt2(-3.0, -3.0),
        13 => pt2(0.5, -3.0),
        14 => pt2(-10.0, -8.5),
        15 => pt2(-6.0, -8.5),
        16 => pt2(-3.0, -8.5),
        17 => pt2(0.5, -8.5),
        18 => pt2(-6.0, -9.0),
        19 => pt2(-3.0, -9.0),
        20 => pt2(4.0, 9.5),
        21 => pt2(8.0, 9.5),
        22 => pt2(4.0, 9.0),
        23 => pt2(8.0, 9.0),
        24 => pt2(4.0, 5.5),
        25 => pt2(8.0, 5.5),
        26 => pt2(4.0, 5.0),
        27 => pt2(8.0, 5.0),
        _ => panic!("idx `{}` exceeded wash count `{}`", idx, WASH_COUNT),
    }
}

/// The topdown position in metres to which the wash fixture at the given index is pointing.
pub fn wash_index_to_topdown_target_position_metres(idx: usize) -> Point2 {
    match idx {
        0 => pt2(-11.0, 9.0),
        1 => pt2(-5.0, 9.0),
        2 => pt2(-4.0, 9.0),
        3 => pt2(0.0, 10.0),
        4 => pt2(-11.0, 6.0),
        5 => pt2(-5.0, 10.0),
        6 => pt2(-4.0, 10.0),
        7 => pt2(0.0, 4.0),
        8 => pt2(-6.0, 3.0),
        9 => pt2(-3.0, 3.0),
        10 => pt2(-11.0, -2.0),
        11 => pt2(-6.0, -3.0),
        12 => pt2(-3.0, -3.0),
        13 => pt2(2.0, -3.0),
        14 => pt2(-11.0, -8.5),
        15 => pt2(-12.0, -5.0),
        16 => pt2(3.0, -5.0),
        17 => pt2(2.0, -8.0),
        18 => pt2(-6.0, -11.0),
        19 => pt2(-3.0, -11.0),
        20 => pt2(3.0, 12.0),
        21 => pt2(7.0, 12.0),
        22 => pt2(3.0, 9.0),
        23 => pt2(11.0, 9.0),
        24 => pt2(3.0, 5.5),
        25 => pt2(11.0, 3.0),
        26 => pt2(5.0, 0.0),
        27 => pt2(9.0, 0.0),
        _ => panic!("idx `{}` exceeded wash count `{}`", idx, WASH_COUNT),
    }
}
