//! The layout of the structure and lighting according to the given floorplans.

use crate::conf::LedLayout;
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

/// Height gap between each LED row.
pub const LED_ROW_GAP_METRES: f32 = 0.45;
/// Distance from the ground at which the bottom LED is situated.
pub const BOTTOM_LED_ROW_FROM_GROUND_METRES: f32 = 1.3;
/// The shader origin position in metres.
pub const SHADER_ORIGIN_METRES: [f32; 2] = [-4.5, 12.0];

/// The height of the LED row from the ground. Row `0` is at the bottom.
fn led_row_index_to_height_metres(idx: usize) -> f32 {
    BOTTOM_LED_ROW_FROM_GROUND_METRES + LED_ROW_GAP_METRES * idx as f32
}

/// The x location of all of the LEDs in a row.
pub fn top_led_row_from_ground(led_layout: &LedLayout) -> f32 {
    BOTTOM_LED_ROW_FROM_GROUND_METRES + LED_ROW_GAP_METRES * (led_layout.row_count - 1) as f32
}

fn led_row_xs_metres(led_layout: &LedLayout) -> impl Iterator<Item = f32> + '_ {
    let x_start = SHADER_ORIGIN_METRES[0] + led_layout.metres_per_row as f32 * -0.5;
    let led_gap_metres = 1.0 / led_layout.leds_per_metre as f32;
    (0..led_layout.leds_per_row()).map(move |ix| x_start + ix as f32 * led_gap_metres)
}

/// The row index, x and height of every LED in all of the rows.
///
/// Starts from the left-most LED of the top row.
pub fn led_positions_metres(
    led_layout: &LedLayout,
) -> impl Iterator<Item = (usize, f32, f32)> + '_ {
    (0..led_layout.row_count).rev().flat_map(move |row_ix| {
        let h = led_row_index_to_height_metres(row_ix);
        led_row_xs_metres(led_layout).map(move |x| (row_ix, x, h))
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
pub fn topdown_metres_to_shader_coords(
    topdown_point_m: Point2,
    height_m: f32,
    led_layout: &LedLayout,
) -> Point3 {
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
    let y = height_m / top_led_row_from_ground(led_layout);
    pt3(x, y, z)
}

pub fn shader_resolution(led_layout: &LedLayout) -> Vec2 {
    let pixels_per_metre = led_layout.leds_per_metre as f32;
    vec2(
        led_layout.metres_per_row as f32 * pixels_per_metre,
        top_led_row_from_ground(led_layout) * pixels_per_metre,
    )
}
