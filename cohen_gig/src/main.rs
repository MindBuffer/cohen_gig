use nannou::prelude::*;
use nannou::Ui;
use nannou_osc as osc;
use korg_nano_kontrol_2 as korg;
use midir;
use std::sync::mpsc;
use shader_shared::{Light, ShaderParams, Uniforms, Vertex};

mod conf;
mod gui;
mod layout;
mod shader;
mod blend_modes;

use crate::conf::Config;
use crate::shader::{Shader, ShaderFnPtr, ShaderReceiver};

const WINDOW_PAD: i32 = 20;
const GUI_WINDOW_X: i32 = WINDOW_PAD;
const GUI_WINDOW_Y: i32 = WINDOW_PAD;
const LED_STRIP_WINDOW_X: i32 = GUI_WINDOW_X + gui::WINDOW_WIDTH as i32 + WINDOW_PAD;
const LED_STRIP_WINDOW_Y: i32 = GUI_WINDOW_Y;
const LED_STRIP_WINDOW_W: u32 = 1920 / 2 - WINDOW_PAD as u32 * 3 - gui::WINDOW_WIDTH;
//const LED_STRIP_WINDOW_W: u32 = 1920 / 2 - WINDOW_PAD as u32 * 3;
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

pub const LED_SHADER_RESOLUTION_X: f32 = 864.0;
pub const LED_SHADER_RESOLUTION_Y: f32 = 600.0;

pub const SPOT_COUNT: usize = 2;
pub const DMX_ADDRS_PER_SPOT: u8 = 1;
pub const DMX_ADDRS_PER_WASH: u8 = 4;
pub const DMX_ADDRS_PER_LED: u8 = 3;
pub const DMX_ADDRS_PER_UNIVERSE: u16 = 512;

const SPOT_AND_WASH_UNIVERSE: u16 = 1;
const LED_START_UNIVERSE: u16 = 2;

struct Model {
    _gui_window: window::Id,
    led_strip_window: window::Id,
    topdown_window: window::Id,
    dmx: Dmx,
    osc: Osc,
    midi_inputs: Vec<midir::MidiInputConnection<()>>,
    midi_rx: mpsc::Receiver<korg::Event>,
    shader_rx: ShaderReceiver,
    shader: Option<Shader>,
    config: Config,
    state: State,
    uniforms: Uniforms,
    target_slider_values: Vec<f32>,
    target_pot_values: Vec<f32>,
    smoothing_speed: f32,
    wash_colors: Box<[LinSrgb; layout::WASH_COUNT]>,
    // Starts from top left, one row at a time.
    led_colors: Box<[LinSrgb; layout::LED_COUNT]>,
    spot_lights: Box<[f32; layout::SPOT_LIGHT_COUNT]>,
    ui: Ui,
    ids: gui::Ids,
    acid_gradient_ids: gui::AcidGradientIds,
    blinky_circles_ids: gui::BlinkyCirclesIds,
    bw_gradient_ids: gui::BwGradientIds,
    colour_grid_ids: gui::ColourGridIds,
    escher_tilings_ids: gui::EscherTilingsIds,
    gilmore_acid_ids: gui::GilmoreAcidIds,
    just_relax_ids: gui::JustRelaxIds,
    life_led_wall_ids: gui::LifeLedWallIds,
    line_gradient_ids: gui::LineGradientIds,
    metafall_ids: gui::MetafallIds,
    particle_zoom_ids: gui::ParticleZoomIds,
    radial_lines_ids: gui::RadialLinesIds,
    satis_spiraling_ids: gui::SatisSpiralingIds,
    spiral_intersect_ids: gui::SpiralIntersectIds,
    square_tunnel_ids: gui::SquareTunnelIds,
    the_pulse_ids: gui::ThePulseIds,
    tunnel_projection_ids: gui::TunnelProjectionIds,
    vert_colour_gradient_ids: gui::VertColourGradientIds,
    solid_hsv_colour_ids: gui::SolidHsvColourIds,
    solid_rgb_colour_ids: gui::SolidRgbColourIds,
}

pub struct State {
    osc_addr_textbox_string: String,
    shader_names: Vec<String>,
    solid_colour_names: Vec<String>,
    led_shader_idx_left: Option<usize>,
    led_shader_idx_right: Option<usize>,
    led_left_right_mix: f32,
    led_fade_to_black: f32,
    wash_fade_to_black: f32,
    spot_light1_fade_to_black: f32,
    spot_light2_fade_to_black: f32,
    solid_colour_idx: Option<usize>,
    blend_mode_names: Vec<String>,
    blend_mode_idx: Option<usize>,
    shader_params: ShaderParams,
}

struct Dmx {
    source: Option<sacn::DmxSource>,
    buffer: Vec<u8>,
}

pub struct Osc {
    tx: Option<osc::Sender>,
    addr: std::net::SocketAddr,
}

fn main() {
    nannou::app(model).update(update).exit(exit).run();
}

fn model(app: &App) -> Model {
    let assets = app
        .assets_path()
        .expect("failed to find project `assets` directory");

    let config_path = conf::path(&assets);
    let config: Config = load_from_json(config_path)
        .ok()
        .unwrap_or_else(Config::default);

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
    let acid_gradient_ids = gui::AcidGradientIds::new(ui.widget_id_generator());
    let blinky_circles_ids = gui::BlinkyCirclesIds::new(ui.widget_id_generator());
    let bw_gradient_ids = gui::BwGradientIds::new(ui.widget_id_generator());
    let colour_grid_ids = gui::ColourGridIds::new(ui.widget_id_generator());
    let escher_tilings_ids = gui::EscherTilingsIds::new(ui.widget_id_generator());
    let gilmore_acid_ids = gui::GilmoreAcidIds::new(ui.widget_id_generator());
    let just_relax_ids = gui::JustRelaxIds::new(ui.widget_id_generator());
    let life_led_wall_ids = gui::LifeLedWallIds::new(ui.widget_id_generator());
    let line_gradient_ids = gui::LineGradientIds::new(ui.widget_id_generator());
    let metafall_ids = gui::MetafallIds::new(ui.widget_id_generator());
    let particle_zoom_ids = gui::ParticleZoomIds::new(ui.widget_id_generator());
    let radial_lines_ids = gui::RadialLinesIds::new(ui.widget_id_generator());
    let satis_spiraling_ids = gui::SatisSpiralingIds::new(ui.widget_id_generator());
    let spiral_intersect_ids = gui::SpiralIntersectIds::new(ui.widget_id_generator());
    let square_tunnel_ids = gui::SquareTunnelIds::new(ui.widget_id_generator());
    let the_pulse_ids = gui::ThePulseIds::new(ui.widget_id_generator());
    let tunnel_projection_ids = gui::TunnelProjectionIds::new(ui.widget_id_generator());
    let vert_colour_gradient_ids = gui::VertColourGradientIds::new(ui.widget_id_generator());
    let solid_hsv_colour_ids = gui::SolidHsvColourIds::new(ui.widget_id_generator());
    let solid_rgb_colour_ids = gui::SolidRgbColourIds::new(ui.widget_id_generator());

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
    let tx = None;
    let addr = "127.0.0.1:8000".parse().unwrap();
    let osc = Osc { tx, addr };

    let mut shader_names = Vec::new();
    shader_names.push("BwGradient".to_string());
    shader_names.push("EscherTilings".to_string());
    shader_names.push("JustRelax".to_string());
    shader_names.push("LineGradient".to_string());
    shader_names.push("Metafall".to_string());
    shader_names.push("ParticleZoom".to_string());
    shader_names.push("RadialLines".to_string());
    shader_names.push("SquareTunnel".to_string());

    shader_names.push("AcidGradient".to_string());
    shader_names.push("BlinkyCircles".to_string());
    shader_names.push("ColourGrid".to_string());
    shader_names.push("GilmoreAcid".to_string());
    shader_names.push("LifeLedWall".to_string());
    shader_names.push("SatisSpiraling".to_string());
    shader_names.push("SpiralIntersect".to_string());
    shader_names.push("ThePulse".to_string());
    shader_names.push("TunnelProjection".to_string());
    shader_names.push("VertColourGradient".to_string());

    shader_names.push("SolidHsvColour".to_string());
    shader_names.push("SolidRgbColour".to_string());

    let mut solid_colour_names = Vec::new();
    solid_colour_names.push("SolidHsvColour".to_string());
    solid_colour_names.push("SolidRgbColour".to_string());

    let mut blend_mode_names = Vec::new();
    blend_mode_names.push("Add".to_string());
    blend_mode_names.push("Subtract".to_string());
    blend_mode_names.push("Multiply".to_string());
    blend_mode_names.push("Average".to_string());
    blend_mode_names.push("Difference".to_string());
    blend_mode_names.push("Negation".to_string());
    blend_mode_names.push("Exclusion".to_string());

    let acid_gradient = shader_shared::AcidGradient {
        speed: 0.5125,
        zoom: 0.0,
        offset: 0.75,
    };

    let blinky_circles = shader_shared::BlinkyCircles {
        speed: 0.5125,
        zoom: 0.05,
        offset: 0.25,
    };

    let bw_gradient = shader_shared::BwGradient {
        speed: 0.03,
        dc: 0.05,
        amp: 0.5,
        freq: 0.5,
        mirror: false,
    };

    let colour_grid = shader_shared::ColourGrid {
        speed: 0.5,
        zoom_amount: 0.1,
    };

    let escher_tilings = shader_shared::EscherTilings {
        speed: 0.2,
        scale: 0.20,
        shape_iter: 0.2,
    };

    let gilmore_acid = shader_shared::GilmoreAcid {
        speed: 0.025,
        displace: 0.01,
        colour_offset: 0.85,
        grid_size: 0.345,
        wave: 0.088,
        zoom_amount: 0.0,
        rotation_amount: 0.0,
        brightness: 1.0,
        saturation: 0.15,
    };

    let just_relax = shader_shared::JustRelax {
        speed: 0.6,
        shape_offset: 0.728,
        iter: 1.0,
    };

    let life_led_wall = shader_shared::LifeLedWall {
        speed: 0.25,
        size: 0.73,
        red: 0.5,
        green: 0.2,
        blue: 0.1,
        saturation: 1.0,
        colour_offset: 0.01,
    };

    let line_gradient = shader_shared::LineGradient {
        speed: 0.03,
        num_stripes: 1.0,
        stripe_width: 0.9,
        angle: 0.5,
        smooth_width: 0.155,
    };

    let metafall = shader_shared::Metafall {
        speed: 0.47,
        scale: 0.0,
        red: 1.0,
        green: 1.0,
        blue: 1.0,
    };

    let particle_zoom = shader_shared::ParticleZoom {
        speed: 0.01,
        density: 0.01,
        shape: 0.35,
        tau: 1.0,
    };

    let radial_lines = shader_shared::RadialLines {
        speed: 0.05,
        zoom_amount: 0.8,
    };

    let satis_spiraling = shader_shared::SatisSpiraling {
        speed: 0.5,
        loops: 0.8,
        mirror: true,
        rotate: true,
    };

    let spiral_intersect = shader_shared::SpiralIntersect {
        speed: 0.02,
        g1: 0.4,
        g2: 0.6,
        rot1: 1.0,
        rot2: 0.5,
        colours: 1.0,
    };

    let square_tunnel = shader_shared::SquareTunnel {
        speed: 0.6,
        rotation_speed: 0.025,
        rotation_offset: 0.0,
        zoom: 0.8,
    };

    let the_pulse = shader_shared::ThePulse {
        speed: 0.08,
        scale: 0.1,
        colour_iter: 0.25,
        thickness: 0.0,
    };

    let tunnel_projection = shader_shared::TunnelProjection {
        speed: 0.5,
        res: 0.5,
    };

    let vert_colour_gradient = shader_shared::VertColourGradient {
        speed: 0.5,
        scale: 0.83,
        colour_iter: 0.015,
        line_amp: 0.0,
        diag_amp: 0.0,
        boarder_amp: 0.65,
    };

    let solid_hsv_colour = shader_shared::SolidHsvColour {
        hue: 1.0,
        saturation: 0.0,
        value: 1.0,
    };

    let solid_rgb_colour = shader_shared::SolidRgbColour {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };


    let shader_params = ShaderParams {
        acid_gradient,
        blinky_circles,
        bw_gradient,
        colour_grid,
        escher_tilings,
        gilmore_acid,
        just_relax,
        life_led_wall,
        line_gradient,
        metafall,
        particle_zoom,
        radial_lines,
        satis_spiraling,
        spiral_intersect,
        square_tunnel,
        the_pulse,
        tunnel_projection,
        vert_colour_gradient,
        solid_hsv_colour,
        solid_rgb_colour,
    };

    let state = State {
        osc_addr_textbox_string: format!("{}", osc.addr),
        shader_names,
        solid_colour_names,
        led_shader_idx_left: Some(15),
        led_shader_idx_right: Some(0),
        led_left_right_mix: 0.0,
        led_fade_to_black: 1.0,
        wash_fade_to_black: 1.0,
        spot_light1_fade_to_black: 1.0,
        spot_light2_fade_to_black: 1.0,
        solid_colour_idx: Some(0),
        blend_mode_names,
        blend_mode_idx: Some(0),
        shader_params,
    };

    let wash_colors = Box::new([lin_srgb(0.0, 0.0, 0.0); layout::WASH_COUNT]);
    let led_colors = Box::new([lin_srgb(0.0, 0.0, 0.0); layout::LED_COUNT]);
    let spot_lights = Box::new([1.0; layout::SPOT_LIGHT_COUNT]);

    // Setup MIDI Input
    let midi_in = midir::MidiInput::new("Korg Nano Kontrol 2").unwrap();

    // A channel for sending events to the main thread.
    let (midi_tx, midi_rx) = std::sync::mpsc::channel();

    let mut midi_inputs = Vec::new();

    // For each point used by the nano kontrol 2, check for events.
    for i in 0..midi_in.port_count() {
        let name = midi_in.port_name(i).unwrap();
        let midi_tx = midi_tx.clone();
        let midi_in = midir::MidiInput::new(&name).unwrap();
        let input = midi_in.connect(i, "nanoKONTROL2 SLIDER/KNOB", move |_stamp, msg, _| {
            if let Some(event) = korg::Event::from_midi(msg) {
                midi_tx.send(event).unwrap();
            }
        }, ()).unwrap();
        midi_inputs.push(input);
    }

    let uniforms = Uniforms {
        time: 0.0,
        resolution: vec2(LED_SHADER_RESOLUTION_X,LED_SHADER_RESOLUTION_Y),
        use_midi: true,
        slider1: 0.0, // BW param 1
        slider2: 0.0, // BW param 2
        slider3: 0.0, // Colour param 1
        slider4: 0.0, // Colour param 2
        slider5: 0.0, // Wash param 1
        slider6: 0.0, // Wash param 2
        pot6: 1.0, // Red / Hue
        pot7: 0.0, // Green / Saturation
        pot8: 1.0, // Blue / Value
        params: state.shader_params.clone(),
    };

    Model {
        _gui_window: gui_window,
        led_strip_window,
        topdown_window,
        dmx,
        osc,
        midi_inputs,
        midi_rx,
        shader_rx,
        shader,
        state,
        config,
        uniforms,
        target_slider_values: vec![0.5; 6], // First 6 Sliders
        target_pot_values: vec![1.0, 0.0, 1.0], // Last 3 Pots
        smoothing_speed: 0.05,
        wash_colors,
        led_colors,
        spot_lights,
        ui,
        ids,
        acid_gradient_ids,
        blinky_circles_ids,
        bw_gradient_ids,
        colour_grid_ids,
        escher_tilings_ids,
        gilmore_acid_ids,
        just_relax_ids,
        life_led_wall_ids,
        line_gradient_ids,
        metafall_ids,
        particle_zoom_ids,
        radial_lines_ids,
        satis_spiraling_ids,
        spiral_intersect_ids,
        square_tunnel_ids,
        the_pulse_ids,
        tunnel_projection_ids,
        vert_colour_gradient_ids,
        solid_hsv_colour_ids,
        solid_rgb_colour_ids,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    // Apply the GUI update.
    let ui = model.ui.set_widgets();
    gui::update(
        ui,
        &mut model.state,
        &mut model.config,
        &mut model.osc,
        update.since_start,
        model.shader_rx.activity(),
        &model.ids,
        &model.acid_gradient_ids,
        &model.blinky_circles_ids,
        &model.bw_gradient_ids,
        &model.colour_grid_ids,
        &model.escher_tilings_ids,
        &model.gilmore_acid_ids,
        &model.just_relax_ids,
        &model.life_led_wall_ids,
        &model.line_gradient_ids,
        &model.metafall_ids,
        &model.particle_zoom_ids,
        &model.radial_lines_ids,
        &model.satis_spiraling_ids,
        &model.spiral_intersect_ids,
        &model.square_tunnel_ids,
        &model.the_pulse_ids,
        &model.tunnel_projection_ids,
        &model.vert_colour_gradient_ids,
        &model.solid_hsv_colour_ids,
        &model.solid_rgb_colour_ids,
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

     // Update the uniforms.
    model.uniforms.time = app.time;

    for event in model.midi_rx.try_iter() {
        //println!("{:?}", &event);
        match event {
            korg::Event::VerticalSlider(strip, value) => {
                match strip {
                    korg::Strip::A => model.target_slider_values[0] = map_range(value as f32 ,0.0,127.0,0.0,1.0),
                    korg::Strip::B => model.target_slider_values[1] = map_range(value as f32 ,0.0,127.0,0.0,1.0),
                    korg::Strip::C => model.target_slider_values[2] = map_range(value as f32 ,0.0,127.0,0.0,1.0),
                    korg::Strip::D => model.target_slider_values[3] = map_range(value as f32 ,0.0,127.0,0.0,1.0),
                    korg::Strip::E => model.target_slider_values[4] = map_range(value as f32 ,0.0,127.0,0.0,1.0),
                    korg::Strip::F => model.target_slider_values[5] = map_range(value as f32 ,0.0,127.0,0.0,1.0),
                    korg::Strip::G => model.state.led_left_right_mix = map_range(value as f32 ,0.0,127.0,-1.0,1.0),
                    korg::Strip::H => model.smoothing_speed = map_range(value as f32 ,0.0,127.0,0.002,0.08),
                    _ => (),
                }
            }
            korg::Event::RotarySlider(strip, value) => {
                match strip {
                    korg::Strip::A => model.state.led_fade_to_black = map_range(value as f32 ,0.0,127.0,0.0,1.0),
                    korg::Strip::B => model.state.wash_fade_to_black = map_range(value as f32 ,0.0,127.0,0.0,1.0),
                    korg::Strip::C => model.state.spot_light1_fade_to_black = map_range(value as f32 ,0.0,127.0,0.0,1.0),
                    korg::Strip::D => model.state.spot_light2_fade_to_black = map_range(value as f32 ,0.0,127.0,0.0,1.0),
                    korg::Strip::F => model.target_pot_values[0] = map_range(value as f32 ,0.0,127.0,0.0,1.0),
                    korg::Strip::G => model.target_pot_values[1] = map_range(value as f32 ,0.0,127.0,0.0,1.0),
                    korg::Strip::H => model.target_pot_values[2] = map_range(value as f32 ,0.0,127.0,0.0,1.0),
                    _ => (),
                }
            }
            _ => (),
        }
    }

    model.uniforms.slider1 = model.uniforms.slider1 * (1.0-model.smoothing_speed) + model.target_slider_values[0] * model.smoothing_speed;
    model.uniforms.slider2 = model.uniforms.slider2 * (1.0-model.smoothing_speed) + model.target_slider_values[1] * model.smoothing_speed;
    model.uniforms.slider3 = model.uniforms.slider3 * (1.0-model.smoothing_speed) + model.target_slider_values[2] * model.smoothing_speed;
    model.uniforms.slider4 = model.uniforms.slider4 * (1.0-model.smoothing_speed) + model.target_slider_values[3] * model.smoothing_speed;
    model.uniforms.slider5 = model.uniforms.slider5 * (1.0-model.smoothing_speed) + model.target_slider_values[4] * model.smoothing_speed;
    model.uniforms.slider6 = model.uniforms.slider6 * (1.0-model.smoothing_speed) + model.target_slider_values[5] * model.smoothing_speed;

    model.uniforms.pot6 = model.uniforms.pot6 * (1.0-model.smoothing_speed) + model.target_pot_values[0] * model.smoothing_speed;
    model.uniforms.pot7 = model.uniforms.pot7 * (1.0-model.smoothing_speed) + model.target_pot_values[1] * model.smoothing_speed;
    model.uniforms.pot8 = model.uniforms.pot8 * (1.0-model.smoothing_speed) + model.target_pot_values[2] * model.smoothing_speed;

    model.uniforms.use_midi = model.config.midi_on;
    
    // Update dimming control of the 2 house spot lights
    model.spot_lights[0] = model.state.spot_light1_fade_to_black;
    model.spot_lights[1] = model.state.spot_light2_fade_to_black;

    // Update the shader params
    model.uniforms.params = model.state.shader_params;

    /*
    when t is -1, volumes[0] = 0, volumes[1] = 1
    when t = 0, volumes[0] = 0.707, volumes[1] = 0.707 (equal-power cross fade)
    when t = 1, volumes[0] = 1, volumes[1] = 0
    // Equal power xfade taken from https://dsp.stackexchange.com/questions/14754/equal-power-crossfade
    */
    let xfade_left = (0.5 * (1.0 + model.state.led_left_right_mix)).sqrt();
    let xfade_right = (0.5 * (1.0 - model.state.led_left_right_mix)).sqrt();
    let xfl = lin_srgb(xfade_left,xfade_left,xfade_left);
    let xfr = lin_srgb(xfade_right,xfade_right,xfade_right);

    let blend_mode = &model.state.blend_mode_names[model.state.blend_mode_idx.unwrap()];

    // Apply the shader for the washes.
    for wash_ix in 0..model.wash_colors.len() {
        let trg_m = layout::wash_index_to_topdown_target_position_metres(wash_ix);
        let trg_h = layout::wash_index_to_target_height_metres(wash_ix);
        let trg_s = pm_to_ps(trg_m, trg_h);
        let ftb = model.state.wash_fade_to_black;
        let light = Light::Wash { index: wash_ix };
        let vertex = Vertex { position: trg_s, light };

        let left = shader(vertex, &model.uniforms, &model.state.shader_names[model.state.led_shader_idx_left.unwrap()]);
        let right = shader(vertex, &model.uniforms, &model.state.shader_names[model.state.led_shader_idx_right.unwrap()]);
        let colour = shader(vertex, &model.uniforms, &model.state.solid_colour_names[model.state.solid_colour_idx.unwrap()]);
        
        model.wash_colors[wash_ix] = match blend_mode.as_str() {
            "Add" => blend_modes::add(left*xfl, right*xfr) * colour * lin_srgb(ftb,ftb,ftb),
            "Subtract" => blend_modes::subtract(left*xfl, right*xfr) * colour * lin_srgb(ftb,ftb,ftb),
            "Multiply" => blend_modes::multiply(left, right) * colour * lin_srgb(ftb,ftb,ftb),
            "Average" => blend_modes::average(left*xfl, right*xfr) * colour * lin_srgb(ftb,ftb,ftb),
            "Difference" => blend_modes::difference(left*xfl, right*xfr) * colour * lin_srgb(ftb,ftb,ftb),
            "Negation" => blend_modes::negation(left*xfl, right*xfr) * colour * lin_srgb(ftb,ftb,ftb),
            "Exclusion" => blend_modes::exclusion(left*xfl, right*xfr) * colour * lin_srgb(ftb,ftb,ftb),
            _ => colour,
        }
    }

    // Apply the shader for the LEDs.
    for (led_ix, (row, x, h)) in layout::led_positions_metres().enumerate() {
        let ps = pm_to_ps(pt2(x, layout::SHADER_ORIGIN_METRES.y), h);
        let index = led_ix;
        let col = led_ix % layout::LEDS_PER_ROW;
        let col_row = [col, row];
        let n_x = (col as f32 / (layout::LEDS_PER_ROW - 1) as f32) * 2.0 - 1.0;
        let n_y = (row as f32 / (layout::LED_ROW_COUNT - 1) as f32) * 2.0 - 1.0;
        let normalised_coords = vec2(n_x, n_y);
        let light = Light::Led { index, col_row, normalised_coords };
        let vertex = Vertex { position: ps, light };
        let left = shader(vertex, &model.uniforms, &model.state.shader_names[model.state.led_shader_idx_left.unwrap()]);
        let right = shader(vertex, &model.uniforms, &model.state.shader_names[model.state.led_shader_idx_right.unwrap()]);
        let colour = shader(vertex, &model.uniforms, &model.state.solid_colour_names[model.state.solid_colour_idx.unwrap()]);
        let ftb = model.state.led_fade_to_black;

        model.led_colors[led_ix] = match blend_mode.as_str() {
            "Add" => blend_modes::add(left*xfl, right*xfr) * colour * lin_srgb(ftb,ftb,ftb),
            "Subtract" => blend_modes::subtract(left*xfl, right*xfr) * colour * lin_srgb(ftb,ftb,ftb),
            "Multiply" => blend_modes::multiply(left, right) * colour * lin_srgb(ftb,ftb,ftb),
            "Average" => blend_modes::average(left*xfl, right*xfr) * colour * lin_srgb(ftb,ftb,ftb),
            "Difference" => blend_modes::difference(left*xfl, right*xfr) * colour * lin_srgb(ftb,ftb,ftb),
            "Negation" => blend_modes::negation(left*xfl, right*xfr) * colour * lin_srgb(ftb,ftb,ftb),
            "Exclusion" => blend_modes::exclusion(left*xfl, right*xfr) * colour * lin_srgb(ftb,ftb,ftb),
            _ => colour,
        }
    }

    // Ensure we are connected to a DMX source if enabled.
    if model.config.dmx_on && model.dmx.source.is_none() {
        let source = sacn::DmxSource::new("Cohen Pre-vis")
            .expect("failed to connect to DMX source");
        model.dmx.source = Some(source);
    } else if !model.config.dmx_on && model.dmx.source.is_some() {
        model.dmx.source.take();
    }

    // Ensure we are connected to an OSC source if enabled.
    if model.config.osc_on && model.osc.tx.is_none() {
        let tx = osc::sender()
            .expect("failed to create OSC sender");
        model.osc.tx = Some(tx);
    } else if !model.config.osc_on && model.osc.tx.is_some() {
        model.osc.tx.take();
    }

    fn convert_channel(f: f32) -> u8 {
        (f.min(1.0).max(0.0) * 255.0) as u8
    }

    // Convert the floating point f32 representation to bytes.
    fn lin_srgb_f32_to_bytes(lin_srgb: &LinSrgb) -> [u8; 3] {
        let r = convert_channel(lin_srgb.red);
        let g = convert_channel(lin_srgb.green);
        let b = convert_channel(lin_srgb.blue);
        [r, g, b]
    }

    // If we have a DMX source, send data over it!
    if let Some(ref dmx_source) = model.dmx.source {
        // First, send data to spotlights and washes on universe 1.
        model.dmx.buffer.clear();
        model.dmx.buffer.extend((0..DMX_ADDRS_PER_UNIVERSE).map(|_| 0u8));

        // Collect spot light dimming data.
        for (spot_ix, dim) in model.spot_lights.iter().enumerate() {
            let dimmer = convert_channel(*dim);
            let col: [u8; DMX_ADDRS_PER_SPOT as usize] = [dimmer];
            let start_addr = model.config.spot_dmx_addrs[spot_ix] as usize;
            let end_addr = start_addr + DMX_ADDRS_PER_SPOT as usize;
            let range = start_addr..std::cmp::min(end_addr, model.dmx.buffer.len());
            let col = &col[..range.len()];
            model.dmx.buffer[range].copy_from_slice(col);
        }

        // Collect wash light color data.
        for (wash_ix, col) in model.wash_colors.iter().enumerate() {
            let [r, g, b] = lin_srgb_f32_to_bytes(col);
            let amber = 0;
            let col: [u8; DMX_ADDRS_PER_WASH as usize] = [r, g, b, amber];
            let start_addr = model.config.wash_dmx_addrs[wash_ix] as usize;
            let end_addr = start_addr + DMX_ADDRS_PER_WASH as usize;
            let range = start_addr..std::cmp::min(end_addr, model.dmx.buffer.len());
            let col = &col[..range.len()];
            model.dmx.buffer[range].copy_from_slice(col);
        }

        // Send spot and wash data.
        dmx_source
            .send(SPOT_AND_WASH_UNIVERSE, &model.dmx.buffer[..])
            .expect("failed to send DMX data");

        // Collect and send LED data.
        model.dmx.buffer.clear();
        let mut universe = LED_START_UNIVERSE;
        for col in model.led_colors.iter() {
            let col = lin_srgb_f32_to_bytes(col);
            model.dmx.buffer.extend(col.iter().cloned());
            // If we've filled a universe, send it.
            if model.dmx.buffer.len() >= DMX_ADDRS_PER_UNIVERSE as usize {
                let data = &model.dmx.buffer[..DMX_ADDRS_PER_UNIVERSE as usize];
                dmx_source.send(universe, data).expect("failed to send LED DMX data");
                model.dmx.buffer.drain(..DMX_ADDRS_PER_UNIVERSE as usize);
                universe += 1;
            }
        }
        let data = &model.dmx.buffer;
        dmx_source.send(universe, data).expect("failed to send LED DMX data");
    }

    // If we have an OSC sender, send data over it!
    if let Some(ref osc_tx) = model.osc.tx {
        // Send wash lights colors.
        let addr = "/cohen/wash_lights/";
        let mut args = Vec::with_capacity(model.wash_colors.len() * 4);
        for col in model.wash_colors.iter() {
            //let [r, g, b] = lin_srgb_f32_to_bytes(col);
            args.push(osc::Type::Float(col.red as _));
            args.push(osc::Type::Float(col.green as _));
            args.push(osc::Type::Float(col.blue as _));
            args.push(osc::Type::Float(0.0))
        }
        osc_tx.send((addr, args), &model.osc.addr).ok();

        // Send LED colors.
        let addr = "/cohen/leds/";
        let mut args = Vec::with_capacity(layout::LEDS_PER_METRE * 3);
        for (metre_ix, metre) in model.led_colors.chunks(layout::LEDS_PER_METRE).enumerate() {
            // TODO: Account for strip IDs etc here.
            args.clear();
            args.push(osc::Type::Int(metre_ix as _));
            for col in metre {
                let [r, g, b] = lin_srgb_f32_to_bytes(col);
                args.push(osc::Type::Int(r as _));
                args.push(osc::Type::Int(g as _));
                args.push(osc::Type::Int(b as _));
            }
            osc_tx.send((addr, args.clone()), &model.osc.addr).ok();
        }

        // Send Spot light dimmers.
        let addr = "/cohen/spot_light1/";
        let mut args = Vec::new();
        args.push(osc::Type::Float(model.spot_lights[0]));
        osc_tx.send((addr, args), &model.osc.addr).ok();

        let addr = "/cohen/spot_light2/";
        let mut args = Vec::new();
        args.push(osc::Type::Float(model.spot_lights[1]));
        osc_tx.send((addr, args), &model.osc.addr).ok();
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
            .map(|((_row, x, h), &c)| {
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

fn exit(app: &App, model: Model) {
    let assets = app
        .assets_path()
        .expect("failed to find project `assets` directory");
    let config_path = conf::path(&assets);
    save_to_json(config_path, &model.config).expect("failed to save config");
}
