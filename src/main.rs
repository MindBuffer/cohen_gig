use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    _window: window::Id,
    dmx_source: Option<sacn::DmxSource>,
}

struct Uniforms {
    time: f32,
}

type Universe = u16;
type Address = u16;

struct LedStrip {
    start: (Universe, Address),
    end: (Universe, Address),
}

pub const FAR_Z: f32 = 0.0;
pub const CLOSE_Z: f32 = 1.0;
pub const LEFT_X: f32 = -1.0;
pub const RIGHT_X: f32 = 1.0;
pub const FLOOR_Y: f32 = -1.0;
pub const ROOF_Y: f32 = 1.0;

pub const LED_PPM: f32 = 60.0;

mod strip {
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
}

mod arch {
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
}

mod wash {
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
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .with_dimensions(1024, 720)
        .view(view)
        .build()
        .unwrap();
    let dmx_source = None;
    Model {
        _window,
        dmx_source,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // Ensure we are connected to a DMX source.
    if model.dmx_source.is_none() {
        let source = sacn::DmxSource::new("Cohen Pre-vis")
            .expect("failed to connect to DMX source");
        model.dmx_source = Some(source);
    }

    // If we have a DMX source ready, send data over it!
    if let Some(ref dmx) = model.dmx_source {
        let uniforms = Uniforms { time: app.time };

        // For each arch, emit the DMX
        let total_dist = (arch::COUNT - 1) as f32 * arch::Z_GAP;
        let universe = 1;
        let mut data = vec![];
        for i in (0..arch::COUNT).rev() {
            let zn = total_dist - i as f32 * arch::Z_GAP;
            // For each area.
            for area in wash::AREAS {
                let lin_srgb = shader(area.pn.extend(zn), &uniforms);
                let lin_bytes: LinSrgb<u8> = lin_srgb.into_format();
                let color_data = [lin_bytes.red, lin_bytes.green, lin_bytes.blue, 0];
                //let color_data = [0u8, 0, 0, 255];
                data.extend(color_data.iter().cloned());
            }
        }

        dmx.send(universe, &data[..])
            .expect("failed to send DMX data");
    }
}

fn view(app: &App, _model: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();

    // Clear the background to blue.
    draw.background().color(BLACK);

    let uniforms = Uniforms {
        time: app.time,
    };

    let w = app.main_window().rect();
    let vis_z_scale = 0.5;
    let vis_y_offset = w.h() * -0.2;
    let front_arch_scale = w.right().min(w.top()) * 4.0 / 7.0;
    let perspective_scale = 0.66;
    let total_dist = (arch::COUNT - 1) as f32 * arch::Z_GAP * front_arch_scale;

    for i in (0..arch::COUNT).rev() {
        let dist_scale = perspective_scale.powi(i as i32);
        let z = total_dist - i as f32 * arch::Z_GAP * front_arch_scale;
        let zn = z / total_dist;

        let tp = |pn| pn * front_arch_scale * dist_scale;

        // LED strips.
        let pts = arch::path_points(LED_PPM).map(|pn| {
            let p = tp(pn);
            let lin_srgb = shader(pn.extend(zn), &uniforms);
            (p, lin_srgb)
        });
        let weight = 4.0 * dist_scale;
        draw.path()
            .stroke()
            .weight(weight)
            .colored_points(pts)
            //.y(vis_y_offset)
            .z(z * vis_z_scale);

        // Draw an ellipse over the wash area.
        fn draw_wash_area(
            draw: &app::Draw,
            area: &wash::Area,
            z: f32,
            col: LinSrgb,
            translate_vector: &dyn Fn(Point2) -> Point2,
        ) {
            let p = translate_vector(area.pn);
            let v = translate_vector(area.vn);
            draw.ellipse()
                .color(wash::apply_fade(col, wash::fade_scalar(area.vn.x, area.vn.y)))
                .xy(p)
                //.xy(p + pt2(0.0, vis_y_offset))
                .wh(v)
                .z(z)
                .rotate(area.rad);
        }

        for area in wash::AREAS {
            let color = shader(area.pn.extend(zn), &uniforms);
            draw_wash_area(&draw, &area, z * vis_z_scale, color, &tp);
        }
    }

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn shader(p: Vector3, uniforms: &Uniforms) -> LinSrgb {
    let t = uniforms.time;
    let b = (p.z + t).sin() * 0.5 + 0.5;
    let r = (p.x + t * 2.0 * p.x.signum()).cos() * 0.5 + 0.5;
    let g = (p.y + t).cos() * 0.5 + 0.5;
    let col = vec3(b*r*0.5, g*b, b);
    lin_srgb(col.x, col.y, col.z)
}
