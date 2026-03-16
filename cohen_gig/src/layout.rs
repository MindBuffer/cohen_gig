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

/// The number of uniquely coloured LEDs per metre.
pub const LEDS_PER_METRE: usize = 100; //144;
/// The number of metres of LEDs in each row.
pub const METRES_PER_LED_ROW: usize = 6;
/// The number of LEDs per row.
pub const LEDS_PER_ROW: usize = LEDS_PER_METRE * METRES_PER_LED_ROW;
/// The gap between individual LEDs in metres.
pub const LED_GAP_METRES: f32 = 1.0 / LEDS_PER_METRE as f32;
/// Total number of LED rows.
pub const LED_ROW_COUNT: usize = 7;
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

