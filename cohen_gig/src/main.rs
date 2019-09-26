use nannou::prelude::*;
use nannou::Ui;
use shader_shared::Uniforms;

mod arch;
mod gui;
mod layout;
mod shader;
mod strip;
mod wash;

use crate::shader::{Shader, ShaderFnPtr, ShaderReceiver};

const WINDOW_PAD: i32 = 20;
const GUI_WINDOW_X: i32 = WINDOW_PAD;
const GUI_WINDOW_Y: i32 = WINDOW_PAD;
const VIS_WINDOW_X: i32 = GUI_WINDOW_X + gui::WINDOW_WIDTH as i32 + WINDOW_PAD;
const VIS_WINDOW_Y: i32 = GUI_WINDOW_Y;
const VIS_WINDOW_W: u32 = 1920 / 2 - WINDOW_PAD as u32 * 3 - gui::WINDOW_WIDTH;
const VIS_WINDOW_H: u32 = 480;
const TOPDOWN_WINDOW_X: i32 = VIS_WINDOW_X;
const TOPDOWN_WINDOW_Y: i32 = VIS_WINDOW_Y + VIS_WINDOW_H as i32 + WINDOW_PAD;
const TOPDOWN_WINDOW_W: u32 = VIS_WINDOW_W;
const TOPDOWN_WINDOW_H: u32 = 480;

pub const FAR_Z: f32 = 0.0;
pub const CLOSE_Z: f32 = 1.0;
pub const LEFT_X: f32 = -1.0;
pub const RIGHT_X: f32 = 1.0;
pub const FLOOR_Y: f32 = -1.0;
pub const ROOF_Y: f32 = 1.0;

pub const LED_PPM: f32 = 60.0;

struct Model {
    _gui_window: window::Id,
    vis_window: window::Id,
    topdown_window: window::Id,
    dmx_source: Option<sacn::DmxSource>,
    ui: Ui,
    ids: gui::Ids,
    shader_rx: ShaderReceiver,
    shader: Option<Shader>,
}

// type Universe = u16;
// type Address = u16;
//
// struct LedStrip {
//     start: (Universe, Address),
//     end: (Universe, Address),
// }

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let gui_window = app
        .new_window()
        .with_title("COHEN GIG - GUI")
        .with_dimensions(gui::WINDOW_WIDTH, gui::WINDOW_HEIGHT)
        .view(gui_view)
        .build()
        .expect("failed to build GUI window");

    let vis_window = app
        .new_window()
        .with_title("COHEN GIG - PREVIS")
        .with_dimensions(VIS_WINDOW_W, VIS_WINDOW_H)
        .view(vis_view)
        .build()
        .unwrap();

    let topdown_window = app
        .new_window()
        .with_title("COHEN GIG - TOPDOWN")
        .with_dimensions(TOPDOWN_WINDOW_W, TOPDOWN_WINDOW_H)
        .view(topdown_view)
        .build()
        .unwrap();

    let mut ui = app
        .new_ui()
        .window(gui_window)
        .build()
        .expect("failed to build `Ui` for GUI window");
    let ids = gui::Ids::new(ui.widget_id_generator());

    app.window(gui_window)
        .expect("GUI window closed unexpectedly")
        .set_position(GUI_WINDOW_X, GUI_WINDOW_Y);

    {
        let w = app.window(vis_window)
            .expect("visualisation window closed unexpectedly");
        w.set_position(VIS_WINDOW_X, VIS_WINDOW_Y);
        let w = app.window(topdown_window)
            .expect("visualisation window closed unexpectedly");
        w.set_position(TOPDOWN_WINDOW_X, TOPDOWN_WINDOW_Y);
    }

    let dmx_source = None;
    let shader = None;
    let shader_rx = shader::spawn_watch();

    Model {
        _gui_window: gui_window,
        vis_window,
        topdown_window,
        dmx_source,
        ui,
        ids,
        shader_rx,
        shader,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let ui = model.ui.set_widgets();
    gui::update(ui, &model.ids, update.since_start, model.shader_rx.activity());

    // Check for an update to the shader.
    if let Some(shader) = model.shader_rx.update() {
        model.shader = Some(shader);
    }

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

        // Retrieve the shader or fall back to black if its not ready.
        let maybe_shader = model.shader.as_ref().map(|s| s.get_fn());
        let black_shader: ShaderFnPtr = shader::black;
        let shader: &ShaderFnPtr = match maybe_shader {
            Some(ref symbol) => symbol,
            None => &black_shader,
        };

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

fn gui_view(app: &App, model: &Model, frame: &Frame) {
    model
        .ui
        .draw_to_frame(app, frame)
        .expect("failed to draw `Ui` to `Frame`");
}

fn topdown_view(app: &App, model: &Model, frame: &Frame) {
    let draw = app.draw_for_window(model.topdown_window).unwrap();
    draw.background().color(BLACK);

    let w = app.window(model.topdown_window).unwrap().rect();

    // Functions ready for metres <-> point translations.
    let metres_to_points_scale = 15.0;

    let m_to_p = |m| m * metres_to_points_scale;
    let p_to_m = |p| p / metres_to_points_scale;
    let pm_to_pp = |pm: Point2| pt2(m_to_p(pm.x), m_to_p(pm.y));
    let pp_to_pm = |pp: Point2| pt2(p_to_m(pp.x), p_to_m(pp.y));

    // Topdown metres <-> shader coords.
    let pm_to_ps = |pm: Point2, h: f32| layout::topdown_metres_to_shader_coords(pm, h);

    // Retrieve the shader or fall back to black if its not ready.
    let maybe_shader = model.shader.as_ref().map(|s| s.get_fn());
    let black_shader: ShaderFnPtr = shader::black;
    let shader: &ShaderFnPtr = match maybe_shader {
        Some(ref symbol) => symbol,
        None => &black_shader,
    };

    // Draw the walls.
    let ps = layout::WALL_METRES.iter().cloned().map(pm_to_pp);
    draw.path().fill().points(ps).rgb(0.1, 0.1, 0.1);

    // Shade the wash lights based on their target location.
    let uniforms = Uniforms {
        time: app.time,
    };
    let mut wash_colors = [lin_srgb(0.0, 0.0, 0.0); layout::WASH_COUNT];
    for wash_ix in 0..layout::WASH_COUNT {
        let trg_m = layout::wash_index_to_topdown_target_position_metres(wash_ix);
        let trg_h = layout::wash_index_to_target_height_metres(wash_ix);
        let trg_s = pm_to_ps(trg_m, trg_h);
        wash_colors[wash_ix] = shader(trg_s, &uniforms);
    }

    // Draw the wash target ellipses.
    for wash_ix in 0..layout::WASH_COUNT {
        let trg_m = layout::wash_index_to_topdown_target_position_metres(wash_ix);
        let trg_p = pm_to_pp(trg_m);
        let r_m = 3.0;
        let r = m_to_p(r_m);
        let color = wash_colors[wash_ix];
        let alpha = 0.2;
        let c = nannou::color::Alpha { color, alpha };
        draw.ellipse().xy(trg_p).radius(r).color(c);
    }

    // Draw the wash source indices.
    for wash_ix in 0..layout::WASH_COUNT {
        let src_m = layout::wash_index_to_topdown_source_position_metres(wash_ix);
        let src_p = pm_to_pp(src_m);
        let trg_m = layout::wash_index_to_topdown_target_position_metres(wash_ix);
        let trg_p = pm_to_pp(trg_m);
        let color = wash_colors[wash_ix];
        draw.line().color(color).points(src_p, trg_p);
        draw.text(&format!("{}", wash_ix))
            .font_size(16)
            .xy(src_p);
    }

    // Draw blackness outside the walls as an adhoc crop.
    let crop_p = layout::WALL_METRES[0] - pt2(0.0, 20.0);
    let crop_bl = crop_p - pt2(20.0, 0.0);
    let crop_tl = crop_bl + pt2(0.0, 50.0);
    let crop_tr = crop_tl + pt2(50.0, 0.0);
    let crop_br = crop_tr - pt2(0.0, 50.0);
    let crop = [crop_p, crop_bl, crop_tl, crop_tr, crop_br, crop_p];
    let blackness_points = layout::WALL_METRES.iter().cloned()
        .chain(Some(layout::WALL_METRES[0]))
        .chain(crop.iter().cloned())
        .map(pm_to_pp);
    draw.polygon().points(blackness_points).color(BLACK);

    // Draw the mouse position in shader coords.
    if app.window_id() == model.topdown_window && app.keys.down.contains(&Key::LShift) {
        let mouse_p = app.mouse.position();
        let mouse_m = pp_to_pm(mouse_p);
        let mouse_s = pm_to_ps(mouse_m, 0.0);
        let coords_text = format!("{:.2}x, {:.2}z", mouse_s.x, mouse_s.z);
        draw.text(&coords_text)
            .x(mouse_p.x)
            .y(mouse_p.y + 16.0)
            .font_size(16);
    }

    draw_hotload_feedback(app, model, &draw, w);

    draw.to_frame(app, &frame).unwrap();
}

fn vis_view(app: &App, model: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw_for_window(model.vis_window).unwrap();

    // Clear the background to blue.
    draw.background().color(BLACK);

    let uniforms = Uniforms {
        time: app.time,
    };

    let w = app.window(model.vis_window).unwrap().rect();
    let vis_z_scale = 0.5;
    //let vis_y_offset = w.h() * -0.2;
    let front_arch_scale = w.right().min(w.top()) * 4.0 / 7.0;
    let perspective_scale = 0.66;
    let total_dist = (arch::COUNT - 1) as f32 * arch::Z_GAP * front_arch_scale;

    // Retrieve the shader or fall back to black if its not ready.
    let maybe_shader = model.shader.as_ref().map(|s| s.get_fn());
    let black_shader: ShaderFnPtr = shader::black;
    let shader: &ShaderFnPtr = match maybe_shader {
        Some(ref symbol) => symbol,
        None => &black_shader,
    };

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

    draw_hotload_feedback(app, model, &draw, w);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

// Draw hotloading status in top-left corner. Flash screen on build completion.
fn draw_hotload_feedback(app: &App, model: &Model, draw: &app::Draw, w: geom::Rect) {
    // If we only recently loaded a new shader, flash the screen a little.
    let secs_since_load = model.shader_rx.last_timestamp().elapsed().secs();
    if secs_since_load < 1.0 {
        let flash_alpha = (1.0 - secs_since_load).powi(8);
        let flash_color = match model.shader_rx.last_incoming() {
            shader::LastIncoming::Succeeded => GREEN,
            shader::LastIncoming::Failed(_) => RED,
        };
        let color = nannou::color::Alpha { color: flash_color, alpha: flash_alpha };
        draw.rect()
            .wh(w.wh())
            .color(color);
    }

    // If we are building or there was some error compiling recently, display it.
    match model.shader_rx.activity() {
        shader::Activity::Incoming => {
            let s = "Compiling";
            let r = w.pad(20.0);
            let color = YELLOW;
            let alpha = (app.time * 2.0 * PI).sin() * 0.35 + 0.5;
            let color = nannou::color::Alpha { color, alpha };
            draw.text(&s)
                .font_size(16)
                .wh(r.wh())
                .color(color)
                .left_justify()
                .align_text_top();
        }
        shader::Activity::LastIncoming(last) => {
            if let shader::LastIncoming::Failed(ref err) = last {
                let s = format!("{}", err);
                let r = w.pad(20.0);
                draw.text(&s)
                    .font_size(16)
                    .wh(r.wh())
                    .color(RED)
                    .left_justify()
                    .align_text_top();
            }
        }
    }
}
