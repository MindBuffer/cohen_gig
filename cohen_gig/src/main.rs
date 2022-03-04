use korg_nano_kontrol_2 as korg;
use midir;
use nannou::prelude::*;
use nannou_conrod as ui;
use nannou_conrod::Ui;
use nannou_osc as osc;
use shader_shared::{Light, MixingInfo, Uniforms, Vertex};
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc;
use lerp::Lerp;

mod conf;
mod gui;
mod layout;
mod shader;
mod midi_osc;
mod lerp;

use crate::conf::Config;
use crate::shader::{Shader, ShaderFnPtr, ShaderReceiver};
use crate::midi_osc::MidiOsc;

const WINDOW_PAD: i32 = 20;
const GUI_WINDOW_X: i32 = WINDOW_PAD;
const GUI_WINDOW_Y: i32 = WINDOW_PAD;
const LED_STRIP_WINDOW_X: i32 = GUI_WINDOW_X + gui::WINDOW_WIDTH as i32 + WINDOW_PAD;
const LED_STRIP_WINDOW_Y: i32 = GUI_WINDOW_Y;
//const LED_STRIP_WINDOW_W: u32 = 1920 / 2 - WINDOW_PAD as u32 * 3 - gui::WINDOW_WIDTH;
const LED_STRIP_WINDOW_W: u32 = 1920 / 2 - WINDOW_PAD as u32 * 3;
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

pub const LED_PPM: f32 = 144.0;

pub const LED_SHADER_RESOLUTION_X: f32 = 720.0;
pub const LED_SHADER_RESOLUTION_Y: f32 = 450.0;

pub const SPOT_COUNT: usize = 2;
pub const DMX_ADDRS_PER_SPOT: u8 = 1;
pub const DMX_ADDRS_PER_WASH: u8 = 7;
pub const DMX_ADDRS_PER_LED: u8 = 3;
pub const DMX_ADDRS_PER_UNIVERSE: u16 = 512;

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
    controller: Controller,
    target_slider_values: Vec<f32>,
    target_pot_values: Vec<f32>,
    smoothing_speed: f32,
    // Colours output via the shader.
    wash_colors: Box<[LinSrgb; layout::WASH_COUNT]>,
    // Shader output with fade-to-black applied.
    wash_outputs: Box<[LinSrgb; layout::WASH_COUNT]>,
    // Colours output via the shader.
    // Starts from top left, one row at a time.
    led_colors: Box<[LinSrgb; layout::LED_COUNT]>,
    // Shader output with fade-to-black applied.
    led_outputs: Box<[LinSrgb; layout::LED_COUNT]>,
    last_preset_change: Option<LastPresetChange>,
    ui: Ui,
    ids: gui::Ids,
    midi_osc: MidiOsc,
    midi_cv_phase_amp: f32,
}

type LastPresetChange = (std::time::Instant, Box<[LinSrgb; layout::LED_COUNT]>);

struct ButtonState {
    pub last_pressed: std::time::Instant,
    pub state: korg::State,
}

struct Dmx {
    source: Option<sacn::DmxSource>,
    buffer: Vec<u8>,
}

pub struct Osc {
    tx: Option<osc::Sender>,
    addr: std::net::SocketAddr,
}

// The known state of the Korg at any point in time.
struct Controller {
    slider1: f32, // BW param 1
    slider2: f32, // BW param 2
    slider3: f32, // Colour param 1
    slider4: f32, // Colour param 2
    slider5: f32, // Wash param 1
    slider6: f32, // Wash param 2
    pot6: f32,    // Red / Hue
    pot7: f32,    // Green / Saturation
    pot8: f32,    // Blue / Value
    buttons: HashMap<shader_shared::Button, ButtonState>,
}

// The known state of the Korg at any point in time.
// struct Controller {
//     slider1: f32, // BW param 1
//     slider2: f32, // BW param 2
//     slider3: f32, // Colour param 1
//     slider4: f32, // Colour param 2
//     slider5: f32, // Midi Osc smoothing speed
//     slider6: f32, // midi_cv app.time phase add
//     slider7: f32, // LED fade to black
//     slider8: f32, // Left / Right Blend Mix
    // pot1: f32,    // BW param 1 (midi_cv amp)
    // pot2: f32,    // BW param 2 (midi_cv amp)
    // pot3: f32,    // Colour param 1 (midi_cv amp)
    // pot4: f32,    // Colour param 2 (midi_cv amp)
//     pot5: f32,    // Shaders smoothing speed
//     pot6: f32,    // Red / Hue
//     pot7: f32,    // Green / Saturation
//     pot8: f32,    // Blue / Value
//     buttons: HashMap<shader_shared::Button, ButtonState>,
// }

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
        .title("COHEN GIG - GUI")
        .size(gui::WINDOW_WIDTH, gui::WINDOW_HEIGHT)
        .raw_event(raw_window_event)
        .key_pressed(key_pressed)
        .view(gui_view)
        .build()
        .expect("failed to build GUI window");

    let led_strip_window = app
        .new_window()
        .title("COHEN GIG - PREVIS")
        .size(LED_STRIP_WINDOW_W, LED_STRIP_WINDOW_H)
        .key_pressed(key_pressed)
        .view(led_strip_view)
        .build()
        .unwrap();

    let topdown_window = app
        .new_window()
        .title("COHEN GIG - TOPDOWN")
        .size(TOPDOWN_WINDOW_W, TOPDOWN_WINDOW_H)
        .key_pressed(key_pressed)
        .view(topdown_view)
        .build()
        .unwrap();

    let mut ui = ui::builder(app)
        .window(gui_window)
        .build()
        .expect("failed to build `Ui` for GUI window");
    let ids = gui::Ids::new(ui.widget_id_generator());

    app.window(gui_window)
        .expect("GUI window closed unexpectedly")
        .set_outer_position_pixels(GUI_WINDOW_X, GUI_WINDOW_Y);

    {
        let w = app
            .window(led_strip_window)
            .expect("visualisation window closed unexpectedly");
        w.set_outer_position_pixels(LED_STRIP_WINDOW_X, LED_STRIP_WINDOW_Y);
        let w = app
            .window(topdown_window)
            .expect("visualisation window closed unexpectedly");
        w.set_outer_position_pixels(TOPDOWN_WINDOW_X, TOPDOWN_WINDOW_Y);
    }

    let dmx = Dmx {
        source: None,
        buffer: vec![],
    };

    let shader = None;
    let shader_rx = shader::spawn_watch();

    // Bind an `osc::Sender` and connect it to the target address.
    let tx = None;
    let addr = conf::default::osc_addr_textbox_string().parse().unwrap();
    let osc = Osc { tx, addr };

    let wash_colors = Box::new([lin_srgb(0.0, 0.0, 0.0); layout::WASH_COUNT]);
    let wash_outputs = Box::new([lin_srgb(0.0, 0.0, 0.0); layout::WASH_COUNT]);
    let led_colors = Box::new([lin_srgb(0.0, 0.0, 0.0); layout::LED_COUNT]);
    let led_outputs = Box::new([lin_srgb(0.0, 0.0, 0.0); layout::LED_COUNT]);

    // Setup MIDI Input
    let midi_in = midir::MidiInput::new("Korg Nano Kontrol 2").unwrap();

    // A channel for sending events to the main thread.
    let (midi_tx, midi_rx) = std::sync::mpsc::channel();

    let mut midi_inputs = Vec::new();

    // For each port used by the nano kontrol 2, check for events.
    for i in 0..midi_in.port_count() {
        let name = midi_in.port_name(i).unwrap();
        let midi_tx = midi_tx.clone();
        let midi_in = midir::MidiInput::new(&name).unwrap();
        println!("midi_in = {:?}", name);
        if name == "nanoKONTROL2 SLIDER/KNOB" {
            let input = midi_in
                .connect(
                    i,
                    "nanoKONTROL2 SLIDER/KNOB",
                    move |_stamp, msg, _| {
                        if let Some(event) = korg::Event::from_midi(msg) {
                            midi_tx.send(event).unwrap();
                        }
                    },
                    (),
                )
                .unwrap();
            midi_inputs.push(input);
        }
    }

    let controller = Controller {
        slider1: 0.5, // BW param 1
        slider2: 0.5, // BW param 2
        slider3: 0.5, // Colour param 1
        slider4: 0.5, // Colour param 2
        slider5: 0.5, // Shaders smoothing speed
        slider6: 0.5, // midi_cv app.time phase add
        // slider7: 0.0, // LED fade to black
        // slider8: 0.5, // Left / Right Blend Mix
        // pot1: 0.0,    // BW param 1 (midi_cv amp)
        // pot2: 0.0,    // BW param 2 (midi_cv amp)
        // pot3: 0.0,    // Colour param 1 (midi_cv amp)
        // pot4: 0.0,    // Colour param 2 (midi_cv amp)
        // pot5: 0.0,    // Midi Osc smoothing speed 
        pot6: 1.0,    // Red / Hue
        pot7: 0.0,    // Green / Saturation
        pot8: 1.0,    // Blue / Value
        buttons: Default::default(),
    };

    let midi_osc = MidiOsc::new();

    let last_preset_change = None;

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
        config,
        controller,
        target_slider_values: vec![0.5; 4],     // First 4 Sliders
        target_pot_values: vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0], // Last 3 Pots
        smoothing_speed: 0.05,
        wash_colors,
        wash_outputs,
        led_colors,
        led_outputs,
        last_preset_change,
        ui,
        ids,
        midi_osc,
        midi_cv_phase_amp: 0.0,
    }
}

fn raw_window_event(app: &App, model: &mut Model, event: &ui::RawWindowEvent) {
    model.ui.handle_raw_event(app, event);
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => {
            let button = shader_shared::Button::Cycle;
            update_korg_button(&mut model.controller, button, korg::State::On);
        }
        _ => (),
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    model.midi_osc.update();
    
    // Apply the GUI update.
    let ui = model.ui.set_widgets();
    let assets = app.assets_path().expect("failed to find assets directory");
    gui::update(
        ui,
        &mut model.config,
        &mut model.midi_osc,
        &mut model.osc,
        update.since_start,
        model.shader_rx.activity(),
        &model.led_colors,
        &mut model.last_preset_change,
        &assets,
        &mut model.ids,
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

    for event in model.midi_rx.try_iter() {
        //println!("{:?}", &event);
        match event {
            korg::Event::VerticalSlider(strip, value) => match strip {
                korg::Strip::A => {
                    model.target_slider_values[0] = map_range(value as f32, 0.0, 127.0, 0.0, 1.0)
                }
                korg::Strip::B => {
                    model.target_slider_values[1] = map_range(value as f32, 0.0, 127.0, 0.0, 1.0)
                }
                korg::Strip::C => {
                    model.target_slider_values[2] = map_range(value as f32, 0.0, 127.0, 0.0, 1.0)
                }
                korg::Strip::D => {
                    model.target_slider_values[3] = map_range(value as f32, 0.0, 127.0, 0.0, 1.0)
                }
                korg::Strip::E => {
                    model.smoothing_speed = map_range(value as f32, 0.0, 127.0, 0.0008, 0.08)
                }
                korg::Strip::F => {
                    model.midi_cv_phase_amp = map_range(value as f32, 0.0, 127.0, 0.0, 4.0);
                }
                korg::Strip::G => {
                    model.config.fade_to_black.led = map_range(value as f32, 0.0, 127.0, 0.0, 1.0)
                }
                korg::Strip::H => {
                    model.config.presets.selected_mut().left_right_mix =
                        map_range(value as f32, 0.0, 127.0, -1.0, 1.0)
                }
            },
            korg::Event::RotarySlider(strip, value) => match strip {
                korg::Strip::A => {
                    model.target_pot_values[0] = map_range(value as f32, 0.0, 127.0, 0.0, 1.0)
                }
                korg::Strip::B => {
                    model.target_pot_values[1] = map_range(value as f32, 0.0, 127.0, 0.0, 1.0)
                }
                korg::Strip::C => {
                    model.target_pot_values[2] = map_range(value as f32, 0.0, 127.0, 0.0, 1.0)
                }
                korg::Strip::D => {
                    model.target_pot_values[3] = map_range(value as f32, 0.0, 127.0, 0.0, 1.0)
                }
                korg::Strip::E => {
                    model.midi_osc.smoothing_speed = map_range(value as f32, 0.0, 127.0, 0.0008, 0.99)
                }
                korg::Strip::F => {
                    model.target_pot_values[5] = map_range(value as f32, 0.0, 127.0, 0.0, 1.0)
                }
                korg::Strip::G => {
                    model.target_pot_values[6] = map_range(value as f32, 0.0, 127.0, 0.0, 1.0)
                }
                korg::Strip::H => {
                    model.target_pot_values[7] = map_range(value as f32, 0.0, 127.0, 0.0, 1.0)
                }
            },

            // Updates for button events.
            korg::Event::Button(row, strip, state) => {
                let button = shader_shared::Button::Row(row, strip);
                update_korg_button(&mut model.controller, button, state);
            }
            korg::Event::TrackButton(tb, state) => {
                let button = shader_shared::Button::Track(tb);
                update_korg_button(&mut model.controller, button, state);
            }
            korg::Event::CycleButton(state) => {
                let button = shader_shared::Button::Cycle;
                update_korg_button(&mut model.controller, button, state);
            }
            korg::Event::MarkerButton(mb, state) => {
                let button = shader_shared::Button::Marker(mb);
                update_korg_button(&mut model.controller, button, state);
            }
            korg::Event::TransportButton(t, state) => {
                let button = shader_shared::Button::Transport(t);
                update_korg_button(&mut model.controller, button, state);
            }
        }
    }

    model.controller.slider1 = model.controller.slider1 * (1.0 - model.smoothing_speed)
        + model.target_slider_values[0] * model.smoothing_speed;
    model.controller.slider2 = model.controller.slider2 * (1.0 - model.smoothing_speed)
        + model.target_slider_values[1] * model.smoothing_speed;
    model.controller.slider3 = model.controller.slider3 * (1.0 - model.smoothing_speed)
        + model.target_slider_values[2] * model.smoothing_speed;
    model.controller.slider4 = model.controller.slider4 * (1.0 - model.smoothing_speed)
        + model.target_slider_values[3] * model.smoothing_speed;

    model.midi_osc.mod_amp1 = model.midi_osc.mod_amp1 * (1.0 - model.smoothing_speed)
        + model.target_pot_values[0] * model.smoothing_speed;
    model.midi_osc.mod_amp2 = model.midi_osc.mod_amp2 * (1.0 - model.smoothing_speed)
        + model.target_pot_values[1] * model.smoothing_speed;
    model.midi_osc.mod_amp3 = model.midi_osc.mod_amp3 * (1.0 - model.smoothing_speed)
        + model.target_pot_values[2] * model.smoothing_speed;
    model.midi_osc.mod_amp4 = model.midi_osc.mod_amp4 * (1.0 - model.smoothing_speed)
        + model.target_pot_values[3] * model.smoothing_speed;

    model.controller.pot6 = model.controller.pot6 * (1.0 - model.smoothing_speed)
        + model.target_pot_values[5] * model.smoothing_speed;
    model.controller.pot7 = model.controller.pot7 * (1.0 - model.smoothing_speed)
        + model.target_pot_values[6] * model.smoothing_speed;
    model.controller.pot8 = model.controller.pot8 * (1.0 - model.smoothing_speed)
        + model.target_pot_values[7] * model.smoothing_speed;

    /*
    when t is -1, volumes[0] = 0, volumes[1] = 1
    when t = 0, volumes[0] = 0.707, volumes[1] = 0.707 (equal-power cross fade)
    when t = 1, volumes[0] = 1, volumes[1] = 0
    // Equal power xfade taken from https://dsp.stackexchange.com/questions/14754/equal-power-crossfade
    */

    let lr_mix = model.config.presets.selected().left_right_mix;
    let xfade_left = (0.5 * (1.0 + lr_mix)).sqrt();
    let xfade_right = (0.5 * (1.0 - lr_mix)).sqrt();
    let preset = model.config.presets.selected();
    let mix_info = MixingInfo {
        left: preset.shader_left,
        right: preset.shader_right,
        colourise: preset.colourise,
        blend_mode: preset.blend_mode,
        xfade_left,
        xfade_right,
    };

    let piano_mod = (model.midi_osc.midi_cv * model.midi_osc.mod_amp1) - (model.midi_osc.mod_amp1 / 2.0);
    let bw_param1 = clamp(model.controller.slider1 + piano_mod, 0.0, 1.0);

    let piano_mod = (model.midi_osc.midi_cv * model.midi_osc.mod_amp2) - (model.midi_osc.mod_amp2 / 2.0);
    let bw_param2 = clamp(model.controller.slider2 + piano_mod, 0.0, 1.0);

    let piano_mod = (model.midi_osc.midi_cv * model.midi_osc.mod_amp3) - (model.midi_osc.mod_amp3 / 2.0);
    let colour_param1 = clamp(model.controller.slider3 + piano_mod, 0.0, 1.0);

    let piano_mod = (model.midi_osc.midi_cv * model.midi_osc.mod_amp4) - (model.midi_osc.mod_amp4 / 2.0);
    let colour_param2 = clamp(model.controller.slider4 + piano_mod, 0.0, 1.0);

    // Collect the data that is uniform across all lights that will be passed into the shaders.
    let shader_params = preset.shader_params.clone();
    let buttons = model
        .controller
        .buttons
        .iter()
        .map(|(&b, b_state)| {
            let secs = b_state.last_pressed.elapsed().secs() as f32;
            let state = b_state.state;
            let state = shader_shared::ButtonState { secs, state };
            (b, state)
        })
        .collect();
    let uniforms = Uniforms {
        time: app.time + (model.midi_osc.midi_cv * model.midi_cv_phase_amp),
        resolution: vec2(LED_SHADER_RESOLUTION_X, LED_SHADER_RESOLUTION_Y),
        use_midi: model.config.midi_on,
        slider1: bw_param1, // BW param 1
        slider2: bw_param2, // BW param 2
        slider3: colour_param1, // Colour param 1
        slider4: colour_param2, // Colour param 2
        slider5: model.controller.slider5, // Wash param 1
        slider6: model.controller.slider6, // Wash param 2
        pot6: model.controller.pot6,       // Red / Hue
        pot7: model.controller.pot7,       // Green / Saturation
        pot8: model.controller.pot8,       // Blue / Value
        params: shader_params,
        wash_lerp_amt: preset.wash_lerp_amt,
        mix: mix_info,
        buttons,
    };

    // Apply the shader for the washes.
    // for wash_ix in 0..model.wash_colors.len() {
    //     let trg_m = layout::wash_index_to_topdown_target_position_metres(wash_ix);
    //     let trg_h = layout::wash_index_to_target_height_metres(wash_ix);
    //     let trg_s = pm_to_ps(trg_m, trg_h);
    //     let light = Light::Wash { index: wash_ix };
    //     let last_color = model.wash_colors[wash_ix];
    //     let position = trg_s;
    //     let vertex = Vertex {
    //         position,
    //         light,
    //         last_color,
    //     };
    //     model.wash_colors[wash_ix] = shader(vertex, &uniforms);
    // }

    // let vertices: Vec<Vertex> = layout::led_positions_metres()
    //     .enumerate()
    //     .map(| (led_ix, (row, x, h)) | {
    //         let ps = pm_to_ps(pt2(x, layout::SHADER_ORIGIN_METRES[1]), h);
    //         let index = led_ix;
    //         let col = led_ix % layout::LEDS_PER_ROW;
    //         let col_row = [col, row];
    //         let n_x = (col as f32 / (layout::LEDS_PER_ROW - 1) as f32) * 2.0 - 1.0;
    //         let n_y = (row as f32 / (layout::LED_ROW_COUNT - 1) as f32) * 2.0 - 1.0;
    //         let normalised_coords = vec2(n_x, n_y);
    //         let light = Light::Led {
    //             index,
    //             col_row,
    //             normalised_coords,
    //         };
    //         let last_color = model.led_colors[led_ix];
    //         let position = ps;
    //         let vertex = Vertex {
    //             position,
    //             light,
    //             last_color,
    //         };
    //         vertex
    //     }).collect();

    // Apply the shader for the LEDs.
    for (led_ix, (row, x, h)) in layout::led_positions_metres().enumerate() {
        let ps = pm_to_ps(pt2(x, layout::SHADER_ORIGIN_METRES[1]), h);
        let index = led_ix;
        let col = led_ix % layout::LEDS_PER_ROW;
        let col_row = [col, row];
        let n_x = (col as f32 / (layout::LEDS_PER_ROW - 1) as f32) * 2.0 - 1.0;
        let n_y = (row as f32 / (layout::LED_ROW_COUNT - 1) as f32) * 2.0 - 1.0;
        let normalised_coords = vec2(n_x, n_y);
        let light = Light::Led {
            index,
            col_row,
            normalised_coords,
        };
        let last_color = model.led_colors[led_ix];
        let position = ps;
        let vertex = Vertex {
            position,
            light,
            last_color,
        };
        model.led_colors[led_ix] = shader(vertex, &uniforms);
    }

    // If we recently changed presets, interpolate from the previous state.
    let (prev_output, lerp_amt) = match model.last_preset_change {
        None => (&[][..], 1.0),
        Some((ref inst, ref prev_output)) => {
            let elapsed_secs = inst.elapsed().as_secs_f32();
            if elapsed_secs < model.config.preset_lerp_secs {
                let diff = model.config.preset_lerp_secs - elapsed_secs;
                let amt = 1.0 - diff / model.config.preset_lerp_secs;
                (&prev_output[..], amt)
            } else {
                (&[][..], 1.0)
            }
        }
    };

    // Write the colours to the output buffer with the fade applied.
    // let ftb = model.config.fade_to_black.wash;
    // let w_ftb = lin_srgb(ftb, ftb, ftb);
    // for (output, &colour) in model.wash_outputs.iter_mut().zip(model.wash_colors.iter()) {
    //     *output = colour * w_ftb;
    // }
    
    // Write the colours to the output buffer with the fade applied.
    let ftb = model.config.fade_to_black.led;
    let l_ftb = lin_srgb(ftb, ftb, ftb);
    for (i, (output, &colour)) in model.led_outputs.iter_mut().zip(model.led_colors.iter()).enumerate() {
        let new = colour * l_ftb;
        *output = match prev_output.get(i) {
            None => new,
            Some(prev) => prev.lerp(&new, lerp_amt),
        };

        //*output = colour * l_ftb;
    }

    // Ensure we are connected to a DMX source if enabled.
    if model.config.dmx_on && model.dmx.source.is_none() {
        // let source =
        //    sacn::DmxSource::new("Cohen Pre-vis").expect("failed to connect to DMX source");
        let source =
            sacn::DmxSource::with_ip("Cohen Pre-vis", "10.0.0.100").expect("failed to connect to DMX source");
        model.dmx.source = Some(source);
    } else if !model.config.dmx_on && model.dmx.source.is_some() {
        model.dmx.source.take();
    }

    // Ensure we are connected to an OSC source if enabled.
    if model.config.osc_on && model.osc.tx.is_none() {
        let tx = osc::sender().expect("failed to create OSC sender");
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

    // Update dimming control of the 2 house spot lights
    let spot_lights = [
        model.config.fade_to_black.spot1,
        model.config.fade_to_black.spot2,
    ];

    // If we have a DMX source, send data over it!
    if let Some(ref dmx_source) = model.dmx.source {
        // // First, send data to spotlights and washes on universe 1.
        // model.dmx.buffer.clear();
        // model
        //     .dmx
        //     .buffer
        //     .extend((0..DMX_ADDRS_PER_UNIVERSE).map(|_| 0u8));

        // // Collect spot light dimming data.
        // for (spot_ix, dim) in spot_lights.iter().enumerate() {
        //     let dimmer = convert_channel(*dim);
        //     let col: [u8; DMX_ADDRS_PER_SPOT as usize] = [dimmer];
        //     let start_addr = model.config.spot_dmx_addrs[spot_ix] as usize;
        //     let end_addr = start_addr + DMX_ADDRS_PER_SPOT as usize;
        //     let range = start_addr..std::cmp::min(end_addr, model.dmx.buffer.len());
        //     let col = &col[..range.len()];
        //     model.dmx.buffer[range].copy_from_slice(col);
        // }

        // // Collect wash light color data.
        // for (wash_ix, col) in model.wash_outputs.iter().enumerate() {
        //     let [r, g, b] = lin_srgb_f32_to_bytes(col);
        //     let intensity = 255; // should this be 255?
        //     let col: [u8; DMX_ADDRS_PER_WASH as usize] = [intensity, r, g, b, 0, 0, 0];
        //     let start_addr = model.config.wash_dmx_addrs[wash_ix] as usize;
        //     let end_addr = start_addr + DMX_ADDRS_PER_WASH as usize;
        //     let range = start_addr..std::cmp::min(end_addr, model.dmx.buffer.len());
        //     let col = &col[..range.len()];
        //     model.dmx.buffer[range].copy_from_slice(col);
        // }

        // // Send spot and wash data.
        // dmx_source
        //     .send(model.config.wash_spot_universe, &model.dmx.buffer[..])
        //     .expect("failed to send DMX data");

        // Collect and send LED data.
        model.dmx.buffer.clear();
        let mut universe = model.config.led_start_universe;
        for col in model.led_outputs.iter() {
            let col = lin_srgb_f32_to_bytes(col);
            model.dmx.buffer.extend(col.iter().cloned());
            
            // If we've filled a universe, send it.
            if model.dmx.buffer.len() >= (DMX_ADDRS_PER_UNIVERSE as usize - 2) {
                // We need to pack in 2 empty bytes so colour values aren't spilit over universes!
                model.dmx.buffer.push(0);
                model.dmx.buffer.push(0);
            // if model.dmx.buffer.len() >= (DMX_ADDRS_PER_UNIVERSE as usize) {
            //     // We need to pack in 2 empty bytes so colour values aren't spilit over universes!
            //     model.dmx.buffer.push(0);
            //     model.dmx.buffer.push(0);

                let data = &model.dmx.buffer[..DMX_ADDRS_PER_UNIVERSE as usize];
                dmx_source
                    .send(universe, data)
                    .expect("failed to send LED DMX data");

                model.dmx.buffer.drain(..DMX_ADDRS_PER_UNIVERSE as usize);
                universe += 1;
            }
        }
        
        dmx_source
            .send(universe, &model.dmx.buffer)
            .expect("failed to send LED DMX data");
    }

    // If we have an OSC sender, send data over it!
    // if let Some(ref osc_tx) = model.osc.tx {
    //     // Send wash lights colors.
    //     let addr = "/cohen/wash_lights/";
    //     let mut args = Vec::with_capacity(model.wash_outputs.len() * 4);
    //     for col in model.wash_outputs.iter() {
    //         //let [r, g, b] = lin_srgb_f32_to_bytes(col);
    //         args.push(osc::Type::Float(col.red as _));
    //         args.push(osc::Type::Float(col.green as _));
    //         args.push(osc::Type::Float(col.blue as _));
    //         args.push(osc::Type::Float(0.0))
    //     }
    //     osc_tx.send((addr, args), &model.osc.addr).ok();

    //     // Send LED colors.
    //     let addr = "/cohen/leds/";
    //     let mut args = Vec::with_capacity(layout::LEDS_PER_METRE * 3);
    //     for (metre_ix, metre) in model.led_outputs.chunks(layout::LEDS_PER_METRE).enumerate() {
    //         // TODO: Account for strip IDs etc here.
    //         args.clear();
    //         args.push(osc::Type::Int(metre_ix as _));
    //         for col in metre {
    //             let [r, g, b] = lin_srgb_f32_to_bytes(col);
    //             args.push(osc::Type::Int(r as _));
    //             args.push(osc::Type::Int(g as _));
    //             args.push(osc::Type::Int(b as _));
    //         }
    //         osc_tx.send((addr, args.clone()), &model.osc.addr).ok();
    //     }

    //     // Send Spot light dimmers.
    //     let addr = "/cohen/spot_light1/";
    //     let mut args = Vec::new();
    //     args.push(osc::Type::Float(spot_lights[0]));
    //     osc_tx.send((addr, args), &model.osc.addr).ok();

    //     let addr = "/cohen/spot_light2/";
    //     let mut args = Vec::new();
    //     args.push(osc::Type::Float(spot_lights[1]));
    //     osc_tx.send((addr, args), &model.osc.addr).ok();
    // }
}

fn gui_view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(GREEN);
    draw.ellipse()
        .x_y(app.mouse.x, app.mouse.y)
        .radius(20.0)
        .color(RED);
    draw.to_frame(app, &frame).unwrap();

    model
        .ui
        .draw_to_frame(app, &frame)
        .expect("failed to draw `Ui` to `Frame`");
}

fn topdown_view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    let w = app.window(model.topdown_window).unwrap().rect();

    // Functions ready for metres <-> point translations.
    let metres_to_points_scale = 15.0;

    let m_to_p = |m| m * metres_to_points_scale;
    let p_to_m = |p| p / metres_to_points_scale;
    let pm_to_pp = |pm: [f32; 2]| pt2(m_to_p(pm[0]), m_to_p(pm[1]));
    let pp_to_pm = |pp: [f32; 2]| pt2(p_to_m(pp[0]), p_to_m(pp[1]));

    // Topdown metres <-> shader coords.
    let pm_to_ps =
        |pm: [f32; 2], h: f32| layout::topdown_metres_to_shader_coords(Vec2::from_slice(&pm), h);

    // Draw the walls.
    let ps = layout::WALL_METRES.iter().cloned().map(pm_to_pp);
    draw.path().fill().points(ps).rgb(0.1, 0.1, 0.1);

    // Draw the wash target ellipses.
    for wash_ix in 0..layout::WASH_COUNT {
        let trg_m = layout::wash_index_to_topdown_target_position_metres(wash_ix);
        let trg_p = pm_to_pp(trg_m.to_array());
        let r_m = 3.0;
        let r = m_to_p(r_m);
        let color = model.wash_outputs[wash_ix];
        let alpha = 0.2;
        let c = nannou::color::Alpha { color, alpha };
        draw.ellipse().xy(trg_p).radius(r).color(c);
    }

    // Draw the wash source indices.
    for wash_ix in 0..layout::WASH_COUNT {
        let src_m = layout::wash_index_to_topdown_source_position_metres(wash_ix);
        let src_p = pm_to_pp(src_m.to_array());
        let trg_m = layout::wash_index_to_topdown_target_position_metres(wash_ix);
        let trg_p = pm_to_pp(trg_m.to_array());
        let color = model.wash_outputs[wash_ix];
        draw.line().color(color).points(src_p, trg_p);
        draw.text(&format!("{}", wash_ix)).font_size(16).xy(src_p);
    }

    // Draw blackness outside the walls as an adhoc crop.
    let crop_p = Vec2::from_slice(&layout::WALL_METRES[0]) - pt2(0.0, 20.0);
    let crop_bl = crop_p - pt2(20.0, 0.0);
    let crop_tl = crop_bl + pt2(0.0, 50.0);
    let crop_tr = crop_tl + pt2(50.0, 0.0);
    let crop_br = crop_tr - pt2(0.0, 50.0);
    let crop = [crop_p, crop_bl, crop_tl, crop_tr, crop_br, crop_p];
    let crop_points = layout::WALL_METRES
        .iter()
        .cloned()
        .map(|p| Vec2::from_slice(&p))
        .chain(Some(Vec2::from_slice(&layout::WALL_METRES[0])))
        .chain(crop.iter().cloned())
        .map(|p| pm_to_pp(p.to_array()));
    draw.polygon().points(crop_points).color(BLACK);

    // Draw the mouse position in shader coords.
    if app.window_id() == model.topdown_window && app.keys.down.contains(&Key::LShift) {
        let mouse_p = app.mouse.position();
        let mouse_m = pp_to_pm(mouse_p.to_array());
        let mouse_s = pm_to_ps(mouse_m.to_array(), 0.0);
        let coords_text = format!("{:.2}x, {:.2}z", mouse_s.x, mouse_s.z);
        draw.text(&coords_text)
            .x(mouse_p.x)
            .y(mouse_p.y + 16.0)
            .font_size(16);
    }

    draw_hotload_feedback(app, model, &draw, w);

    draw.to_frame(app, &frame).unwrap();
}

fn led_strip_view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    let w = app.window(model.led_strip_window).unwrap().rect();

    let metres_to_points_scale = (w.h() / layout::TOP_LED_ROW_FROM_GROUND as f32)
        .min(w.w() / layout::METRES_PER_LED_ROW as f32)
        * 0.8;
    let m_to_p = |m| m * metres_to_points_scale;
    let p_to_m = |p| p / metres_to_points_scale;
    let x_offset_m = layout::SHADER_ORIGIN_METRES[0];
    let y_offset_m = layout::TOP_LED_ROW_FROM_GROUND as f32 * 0.5;
    let pm_to_pp = |x: f32, h: f32| pt2(m_to_p(x - x_offset_m), m_to_p(h - y_offset_m));
    let pp_to_pm = |pp: Point2| (p_to_m(pp.x) + x_offset_m, p_to_m(pp.y) + y_offset_m);
    let pm_to_ps = |x: f32, h: f32| layout::topdown_metres_to_shader_coords(pt2(x, 0.0), h);

    // Draw the LEDs one row at a time.
    let mut leds = layout::led_positions_metres().zip(model.led_outputs.iter());
    for _ in 0..layout::LED_ROW_COUNT {
        let vs = leds
            .by_ref()
            .take(layout::LEDS_PER_ROW)
            .map(|((_row, x, h), &c)| {
                let pp = pm_to_pp(x, h);
                (pp, c)
            });
        draw.polyline().weight(5.0).points_colored(vs);
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
fn draw_hotload_feedback(app: &App, model: &Model, draw: &Draw, w: geom::Rect) {
    // If we only recently loaded a new shader, flash the screen a little.
    let secs_since_load = model.shader_rx.last_timestamp().elapsed().secs();
    if secs_since_load < 1.0 {
        let flash_alpha = (1.0 - secs_since_load).powi(8);
        let flash_color = match model.shader_rx.last_incoming() {
            shader::LastIncoming::Succeeded => GREEN,
            shader::LastIncoming::Failed(_) => RED,
        };
        let color = nannou::color::Alpha {
            color: flash_color,
            alpha: flash_alpha,
        };
        draw.rect().wh(w.wh()).color(color);
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

fn save_config(assets: &Path, config: &Config) {
    let config_path = conf::path(&assets);
    save_to_json(config_path, config).expect("failed to save config");
}

fn exit(app: &App, model: Model) {
    let assets = app
        .assets_path()
        .expect("failed to find project `assets` directory");
    save_config(&assets, &model.config);
}

// A function for updating the controller's button states based on a button event.
fn update_korg_button(
    controller: &mut Controller,
    button: shader_shared::Button,
    state: korg::State,
) {
    let now = std::time::Instant::now();
    let b_state = controller.buttons.entry(button).or_insert_with(|| {
        let last_pressed = now;
        ButtonState {
            last_pressed,
            state,
        }
    });
    b_state.state = state;
    if state == korg::State::On {
        b_state.last_pressed = now;
    }
}
