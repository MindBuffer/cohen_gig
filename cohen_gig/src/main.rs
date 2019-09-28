//use nannou_osc as osc;
use nannou::prelude::*;
use nannou::Ui;
use shader_shared::Uniforms;

mod gui;
mod layout;
mod shader;

use crate::shader::{Shader, ShaderFnPtr, ShaderReceiver};

const WINDOW_PAD: i32 = 20;
const GUI_WINDOW_X: i32 = WINDOW_PAD;
const GUI_WINDOW_Y: i32 = WINDOW_PAD;
const LED_STRIP_WINDOW_X: i32 = GUI_WINDOW_X + gui::WINDOW_WIDTH as i32 + WINDOW_PAD;
const LED_STRIP_WINDOW_Y: i32 = GUI_WINDOW_Y;
const LED_STRIP_WINDOW_W: u32 = 1920 / 2 - WINDOW_PAD as u32 * 3 - gui::WINDOW_WIDTH;
const LED_STRIP_WINDOW_H: u32 = 480;
const TOPDOWN_WINDOW_X: i32 = LED_STRIP_WINDOW_X;
const TOPDOWN_WINDOW_Y: i32 = LED_STRIP_WINDOW_Y + LED_STRIP_WINDOW_H as i32 + WINDOW_PAD;
const TOPDOWN_WINDOW_W: u32 = LED_STRIP_WINDOW_W;
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
    led_strip_window: window::Id,
    topdown_window: window::Id,
    dmx: Dmx,
    osc: Osc,
    ui: Ui,
    ids: gui::Ids,
    shader_rx: ShaderReceiver,
    shader: Option<Shader>,
    state: State,
    wash_colors: Box<[LinSrgb; layout::WASH_COUNT]>,
    led_colors: Box<[LinSrgb; layout::LED_COUNT]>,
}

pub struct State {
    osc_on: bool,
    dmx_on: bool,
    osc_addr_textbox_string: String,
}

struct Dmx {
    source: Option<sacn::DmxSource>,
    buffer: Vec<u8>,
}

pub struct Osc {
    //tx: Option<osc::Sender>,
    addr: std::net::SocketAddr,
}

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

    let led_strip_window = app
        .new_window()
        .with_title("COHEN GIG - PREVIS")
        .with_dimensions(LED_STRIP_WINDOW_W, LED_STRIP_WINDOW_H)
        .view(led_strip_view)
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
        let w = app.window(led_strip_window)
            .expect("visualisation window closed unexpectedly");
        w.set_position(LED_STRIP_WINDOW_X, LED_STRIP_WINDOW_Y);
        let w = app.window(topdown_window)
            .expect("visualisation window closed unexpectedly");
        w.set_position(TOPDOWN_WINDOW_X, TOPDOWN_WINDOW_Y);
    }

    let dmx = Dmx {
        source: None,
        buffer: vec![],
    };

    let shader = None;
    let shader_rx = shader::spawn_watch();

    // Bind an `osc::Sender` and connect it to the target address.
    //let tx = osc::sender().expect("failed to create OSC sender");
    let addr = "127.0.0.1:34254".parse().unwrap();
    let osc = Osc { addr };

    let state = State {
        dmx_on: false,
        osc_on: false,
        osc_addr_textbox_string: format!("{}", osc.addr),
    };

    let wash_colors = Box::new([lin_srgb(0.0, 0.0, 0.0); layout::WASH_COUNT]);
    let led_colors = Box::new([lin_srgb(0.0, 0.0, 0.0); layout::LED_COUNT]);

    Model {
        _gui_window: gui_window,
        led_strip_window,
        topdown_window,
        dmx,
        osc,
        ui,
        ids,
        shader_rx,
        shader,
        state,
        wash_colors,
        led_colors,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    // Apply the GUI update.
    let ui = model.ui.set_widgets();
    gui::update(
        ui,
        &model.ids,
        &mut model.state,
        &mut model.osc,
        update.since_start,
        model.shader_rx.activity(),
    );

    // Check for an update to the shader.
    if let Some(shader) = model.shader_rx.update() {
        model.shader = Some(shader);
    }

    // Retrieve the shader or fall back to black if its not ready.
    let maybe_shader = model.shader.as_ref().map(|s| s.get_fn());
    let black_shader: ShaderFnPtr = shader::black;
    let shader: &ShaderFnPtr = match maybe_shader {
        Some(ref symbol) => symbol,
        None => &black_shader,
    };

    // Topdown metres to shader coords.
    let pm_to_ps = |pm: Point2, h: f32| layout::topdown_metres_to_shader_coords(pm, h);

    // Collect the uniforms.
    let uniforms = Uniforms {
        time: app.time,
    };

    // Apply the shader for the washes.
    for wash_ix in 0..model.wash_colors.len() {
        let trg_m = layout::wash_index_to_topdown_target_position_metres(wash_ix);
        let trg_h = layout::wash_index_to_target_height_metres(wash_ix);
        let trg_s = pm_to_ps(trg_m, trg_h);
        model.wash_colors[wash_ix] = shader(trg_s, &uniforms);
    }

    // Apply the shader for the LEDs.
    for (led_ix, (x, h)) in layout::led_positions_metres().enumerate() {
        let ps = pm_to_ps(pt2(x, layout::SHADER_ORIGIN_METRES.y), h);
        model.led_colors[led_ix] = shader(ps, &uniforms);
    }

    // Ensure we are connected to a DMX source if enabled.
    if model.state.dmx_on && model.dmx.source.is_none() {
        let source = sacn::DmxSource::new("Cohen Pre-vis")
            .expect("failed to connect to DMX source");
        model.dmx.source = Some(source);
    } else if !model.state.dmx_on && model.dmx.source.is_some() {
        model.dmx.source.take();
    }

    // // Ensure we are connected to an OSC source if enabled.
    // if model.state.osc_on && model.osc.tx.is_none() {
    //     let tx = osc::sender()
    //         .expect("failed to create OSC sender");
    //     model.osc.tx = Some(tx);
    // } else if !model.state.osc_on && model.osc.tx.is_some() {
    //     model.osc.tx.take();
    // }

    // Convert the floating point f32 representation to bytes.
    fn lin_srgb_f32_to_bytes(lin_srgb: &LinSrgb) -> [u8; 3] {
        fn convert_channel(f: f32) -> u8 {
            (f.min(1.0).max(0.0) * 255.0) as u8
        }
        let r = convert_channel(lin_srgb.red);
        let g = convert_channel(lin_srgb.green);
        let b = convert_channel(lin_srgb.blue);
        [r, g, b]
    }

    // If we have a DMX source, send data over it!
    if let Some(ref dmx_source) = model.dmx.source {
        model.dmx.buffer.clear();

        // TODO: We'll use multiple universes for LEDs.
        let universe = 1;

        // Collect wash light color data.
        for col in model.wash_colors.iter() {
            let [r, g, b] = lin_srgb_f32_to_bytes(col);
            let amber = 0;
            let col = [r, g, b, amber];
            model.dmx.buffer.extend(col.iter().cloned());
        }

        // Collect LED color data.
        for col in model.led_colors.iter() {
            let col = lin_srgb_f32_to_bytes(col);
            model.dmx.buffer.extend(col.iter().cloned());
        }

        dmx_source
            .send(universe, &model.dmx.buffer[..])
            .expect("failed to send DMX data");
    }


    // // If we have an OSC sender, send data over it!
    // if let Some(ref osc_tx) = model.osc.tx {
    //     // Send wash lights colors.
    //     let addr = "/cohen/wash/";
    //     let mut args = Vec::with_capacity(model.wash_colors.len() * 3);
    //     for col in model.wash_colors.iter() {
    //         let [r, g, b] = lin_srgb_f32_to_bytes(col);
    //         args.push(Type::Int(r as _));
    //         args.push(Type::Int(g as _));
    //         args.push(Type::Int(b as _));
    //     }
    //     model.sender.send((addr, args)).ok();

    //     // Send LED colors.
    //     let addr = "/cohen/leds/";
    //     let mut args = Vec::with_capacity(metre.len() * 3);
    //     for (metre_ix, metre) in model.led_colors.chunks(layout::LEDS_PER_METRE).enumerate() {
    //         args.clear();
    //         for col in metre {
    //             let [r, g, b] = lin_srgb_f32_to_bytes(col);
    //             args.push(Type::Int(r as _));
    //             args.push(Type::Int(g as _));
    //             args.push(Type::Int(b as _));
    //         }
    //         model.sender.send((addr, args.clone())).ok();
    //     }
    // }
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

    // Draw the walls.
    let ps = layout::WALL_METRES.iter().cloned().map(pm_to_pp);
    draw.path().fill().points(ps).rgb(0.1, 0.1, 0.1);

    // Draw the wash target ellipses.
    for wash_ix in 0..layout::WASH_COUNT {
        let trg_m = layout::wash_index_to_topdown_target_position_metres(wash_ix);
        let trg_p = pm_to_pp(trg_m);
        let r_m = 3.0;
        let r = m_to_p(r_m);
        let color = model.wash_colors[wash_ix];
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
        let color = model.wash_colors[wash_ix];
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
    let crop_points = layout::WALL_METRES.iter().cloned()
        .chain(Some(layout::WALL_METRES[0]))
        .chain(crop.iter().cloned())
        .map(pm_to_pp);
    draw.polygon().points(crop_points).color(BLACK);

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

fn led_strip_view(app: &App, model: &Model, frame: &Frame) {
    let draw = app.draw_for_window(model.led_strip_window).unwrap();
    draw.background().color(BLACK);

    let w = app.window(model.led_strip_window).unwrap().rect();

    let metres_to_points_scale = (w.h() / layout::TOP_LED_ROW_FROM_GROUND as f32)
        .min(w.w() / layout::METRES_PER_LED_ROW as f32) * 0.8;
    let m_to_p = |m| m * metres_to_points_scale;
    let p_to_m = |p| p / metres_to_points_scale;
    let x_offset_m = layout::SHADER_ORIGIN_METRES.x;
    let y_offset_m = layout::TOP_LED_ROW_FROM_GROUND as f32 * 0.5;
    let pm_to_pp = |x: f32, h: f32| pt2(m_to_p(x - x_offset_m), m_to_p(h - y_offset_m));
    let pp_to_pm = |pp: Point2| (p_to_m(pp.x) + x_offset_m, p_to_m(pp.y) + y_offset_m);
    let pm_to_ps = |x: f32, h: f32| layout::topdown_metres_to_shader_coords(pt2(x, 0.0), h);

    // Draw the LEDs one row at a time.
    let mut leds = layout::led_positions_metres().zip(model.led_colors.iter());
    for _ in 0..layout::LED_ROW_COUNT {
        let vs = leds
            .by_ref()
            .take(layout::LEDS_PER_ROW)
            .map(|((x, h), &c)| {
                let pp = pm_to_pp(x, h);
                (pp, c)
            });
        draw.polyline().weight(5.0).colored_points(vs);
    }

    // Draw the mouse position in shader coords.
    if app.window_id() == model.led_strip_window && app.keys.down.contains(&Key::LShift) {
        let mouse_p = app.mouse.position();
        let (x, h) = pp_to_pm(mouse_p);
        let mouse_s = pm_to_ps(x, h);
        let coords_text = format!("{:.2}x, {:.2}y", mouse_s.x, mouse_s.y);
        draw.text(&coords_text)
            .x(mouse_p.x)
            .y(mouse_p.y + 16.0)
            .font_size(16);
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
