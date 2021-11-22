//! The layout of the structure and lighting according to the given floorplans.

use nannou::prelude::*;

/// A rough polyline that describes the venue walls.
pub const WALL_METRES: &[[f32; 2]] = &[
    [3.0, -12.0],
    [3.0, -12.0],
    [3.0, 0.0],
    [11.0, 0.0],
    [11.0, 10.0],
    [10.0, 12.0],
    [-12.0, 12.0],
    [-12.0, -12.0],
];

/// Total number of spot lights.
pub const SPOT_LIGHT_COUNT: usize = 2;
/// Total number of wash lights.
pub const WASH_COUNT: usize = 28;
/// The number of uniquely coloured LEDs per metre.
pub const LEDS_PER_METRE: usize = 144;
/// The number of metres of LEDs in each row.
pub const METRES_PER_LED_ROW: usize = 5;
/// The number of LEDs per row.
pub const LEDS_PER_ROW: usize = LEDS_PER_METRE * METRES_PER_LED_ROW;
/// The gap between individual LEDs in metres.
pub const LED_GAP_METRES: f32 = 1.0 / LEDS_PER_METRE as f32;
/// Total number of LED rows.
pub const LED_ROW_COUNT: usize = 8;
/// Total number of LEDs.
pub const LED_COUNT: usize = LEDS_PER_ROW * LED_ROW_COUNT;
/// Height gap between each LED row.
pub const LED_ROW_GAP_METRES: f32 = 0.45;
/// Distance from the ground at which the bottom LED is situated.
pub const BOTTOM_LED_ROW_FROM_GROUND_METRES: f32 = 1.3;
/// The distance from the ground of the top LED row.
pub const TOP_LED_ROW_FROM_GROUND: f32 =
    BOTTOM_LED_ROW_FROM_GROUND_METRES + LED_ROW_GAP_METRES * (LED_ROW_COUNT - 1) as f32;
/// The shader origin position in metres.
pub const SHADER_ORIGIN_METRES: [f32; 2] = [-4.5, 12.0];

/// The height of the LED row from the ground. Row `0` is at the bottom.
fn led_row_index_to_height_metres(idx: usize) -> f32 {
    BOTTOM_LED_ROW_FROM_GROUND_METRES + LED_ROW_GAP_METRES * idx as f32
}

/// The x location of all of the LEDs in a row.
fn led_row_xs_metres() -> impl Iterator<Item = f32> {
    let x_start = SHADER_ORIGIN_METRES[0] + METRES_PER_LED_ROW as f32 * -0.5;
    (0..LEDS_PER_ROW).map(move |ix| x_start + ix as f32 * LED_GAP_METRES)
}

/// The row index, x and height of every LED in all of the rows.
///
/// Starts from the left-most LED of the top row.
pub fn led_positions_metres() -> impl Iterator<Item = (usize, f32, f32)> {
    (0..LED_ROW_COUNT).rev().flat_map(|row_ix| {
        let h = led_row_index_to_height_metres(row_ix);
        led_row_xs_metres().map(move |x| (row_ix, x, h))
    })
}

/// The rect that bounds the venue in metres.
fn venue_bounding_rect_metres() -> geom::Rect {
    let mut r = geom::Rect::from_wh(vec2(0.0, 0.0));
    for &p in WALL_METRES {
        r = r.stretch_to_point(p);
    }
    r
}

/// Converts the given topdown metres coords to the coordinate system ready for the shader.
pub fn topdown_metres_to_shader_coords(topdown_point_m: Point2, height_m: f32) -> Point3 {
    // Translate based on the shader origin.
    let topdown_point_m = topdown_point_m - Vec2::from_slice(&SHADER_ORIGIN_METRES);
    // Use the bounding rect to normalise the coords using venue width.
    let r = venue_bounding_rect_metres();
    let scale = 1.0 / r.w();
    let topdown_point_s = topdown_point_m * scale;
    // Use the inverse of the y as the z axis for shader coords.
    let [x, y] = topdown_point_s.to_array();
    let z = -y;
    // Scale the height in metres by the top of the LED rows.
    let y = height_m / TOP_LED_ROW_FROM_GROUND;
    let point = pt3(x, y, z);
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
        8 => pt2(-10.0, 3.0),
        9 => pt2(-6.0, 3.0),
        10 => pt2(-3.0, 3.0),
        11 => pt2(0.5, 3.0),
        12 => pt2(-10.0, -3.0),
        13 => pt2(-6.0, -3.0),
        14 => pt2(-3.0, -3.0),
        15 => pt2(0.5, -3.0),
        16 => pt2(-10.0, -8.5),
        17 => pt2(-6.0, -8.5),
        18 => pt2(-3.0, -8.5),
        19 => pt2(0.5, -8.5),
        20 => pt2(-6.0, -9.0),
        21 => pt2(-3.0, -9.0),
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
        8 => pt2(-11.0, 3.0),
        9 => pt2(-6.0, 3.0),
        10 => pt2(-3.0, 3.0),
        11 => pt2(1.0, 3.0),
        12 => pt2(-11.0, -2.0),
        13 => pt2(-6.0, -3.0),
        14 => pt2(-3.0, -3.0),
        15 => pt2(2.0, -3.0),
        16 => pt2(-11.0, -8.5),
        17 => pt2(-12.0, -5.0),
        18 => pt2(3.0, -5.0),
        19 => pt2(2.0, -8.0),
        20 => pt2(-6.0, -11.0),
        21 => pt2(-3.0, -11.0),
        22 => pt2(3.5, 12.0),
        23 => pt2(9.0, 10.0),
        24 => pt2(4.0, 5.5),
        25 => pt2(11.0, 3.0),
        26 => pt2(5.0, 0.0),
        27 => pt2(9.0, 0.0),
        _ => panic!("idx `{}` exceeded wash count `{}`", idx, WASH_COUNT),
    }
}

/// The rough height of the target at which each wash is pointed.
pub fn wash_index_to_target_height_metres(idx: usize) -> f32 {
    let floor_h = 0.0;
    let wall_h = 3.0;
    match idx {
        0 => wall_h,
        1 => floor_h,
        2 => floor_h,
        3 => floor_h,
        4 => wall_h,
        5 => floor_h,
        6 => floor_h,
        7 => wall_h,
        8 => wall_h,
        9 | 10 => floor_h,
        11 => wall_h,
        12 => wall_h,
        13 | 14 => floor_h,
        15 => wall_h,
        16..=19 => wall_h,
        20 | 21 => floor_h,
        22 | 23 => wall_h,
        24 => floor_h,
        25..=27 => wall_h,
        _ => panic!("idx `{}` exceeded wash count `{}`", idx, WASH_COUNT),
    }
}
