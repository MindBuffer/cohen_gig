use korg_nano_kontrol_2 as korg;
use lerp::Lerp;
use nannou::prelude::*;
use nannou_conrod as ui;
use nannou_conrod::Ui;
use sacn::packet::{E131_DEFAULT_PRIORITY, UNIVERSE_CHANNEL_CAPACITY};
use sacn::source::SacnSource;
use shader_shared::{Light, MixingInfo, Uniforms, Vertex};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::mpsc;

mod audio_input;
mod audio_widgets;
mod conf;
mod gui;
pub mod knob;
mod layout;
mod lerp;
pub mod mod_slider;
mod shader;

use crate::conf::Config;
use crate::shader::{Shader, ShaderFnPtr, ShaderReceiver};

const WINDOW_PAD: i32 = 20;
const GUI_WINDOW_X: i32 = WINDOW_PAD;
const GUI_WINDOW_Y: i32 = WINDOW_PAD;
const LED_STRIP_WINDOW_X: i32 = GUI_WINDOW_X + gui::WINDOW_WIDTH as i32 + WINDOW_PAD;
const LED_STRIP_WINDOW_Y: i32 = GUI_WINDOW_Y;
//const LED_STRIP_WINDOW_W: u32 = 1920 / 2 - WINDOW_PAD as u32 * 3 - gui::WINDOW_WIDTH;
const LED_STRIP_WINDOW_W: u32 = 1920 / 2 - WINDOW_PAD as u32 * 3;
const LED_STRIP_WINDOW_H: u32 = 480;
pub const FAR_Z: f32 = 0.0;
pub const CLOSE_Z: f32 = 1.0;
pub const LEFT_X: f32 = -1.0;
pub const RIGHT_X: f32 = 1.0;
pub const FLOOR_Y: f32 = -1.0;
pub const ROOF_Y: f32 = 1.0;

pub const DMX_ADDRS_PER_LED: u8 = 3;
pub const DMX_ADDRS_PER_UNIVERSE: u16 = 512;

struct Model {
    _gui_window: window::Id,
    led_strip_window: window::Id,
    dmx: Dmx,
    _midi_inputs: Vec<midir::MidiInputConnection<()>>,
    midi_rx: mpsc::Receiver<korg::Event>,
    shader_rx: ShaderReceiver,
    shader: Option<Shader>,
    config: Config,
    controller: Controller,
    target_slider_values: Vec<f32>,
    target_pot_values: Vec<f32>,
    smoothing_speed: f32,
    // Colours output via the shader.
    // Starts from top left, one row at a time.
    led_colors: Vec<LinSrgb>,
    // Shader output with fade-to-black applied.
    led_outputs: Vec<LinSrgb>,
    last_preset_change: Option<LastPresetChange>,
    ui: Ui,
    ids: gui::Ids,
    audio_input: audio_input::AudioInput,
    midi_cv_phase_amp: f32,
}

type LastPresetChange = (std::time::Instant, Vec<LinSrgb>);

struct ButtonState {
    pub last_pressed: std::time::Instant,
    pub state: korg::State,
}

struct Dmx {
    source: Option<SacnSource>,
    requested_interface_ip: Option<Ipv4Addr>,
    bind_error: Option<String>,
}

// The known state of the Korg at any point in time.
struct Controller {
    slider1: f32, // BW param 1
    slider2: f32, // BW param 2
    slider3: f32, // Colour param 1
    slider4: f32, // Colour param 2
    slider5: f32, // Shader param 5
    slider6: f32, // Shader param 6
    pot6: f32,    // Red / Hue
    pot7: f32,    // Green / Saturation
    pot8: f32,    // Blue / Value
    buttons: HashMap<shader_shared::Button, ButtonState>,
}

fn main() {
    nannou::app(model).update(update).exit(exit).run();
}

fn model(app: &App) -> Model {
    let assets = app
        .assets_path()
        .expect("failed to find project `assets` directory");

    let config_path = conf::path(&assets);
    let mut config: Config = load_from_json(config_path)
        .ok()
        .unwrap_or_else(Config::default);
    config.led_layout.normalise();
    for preset in &mut config.presets.list {
        gui::normalise_preset_shader_mod_amounts(preset);
    }

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
    }

    let dmx = Dmx {
        source: None,
        requested_interface_ip: None,
        bind_error: None,
    };

    let shader = None;
    let shader_rx = shader::spawn_watch();

    let led_colors = black_led_buffer(config.led_layout.led_count());
    let led_outputs = black_led_buffer(config.led_layout.led_count());

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
        slider5: 0.5, // Shader param 5
        slider6: 0.5, // Shader param 6
        // slider7: 0.0, // LED fade to black
        // slider8: 0.5, // Left / Right Blend Mix
        // pot1: 0.0,    // BW param 1 (midi_cv amp)
        // pot2: 0.0,    // BW param 2 (midi_cv amp)
        // pot3: 0.0,    // Colour param 1 (midi_cv amp)
        // pot4: 0.0,    // Colour param 2 (midi_cv amp)
        // pot5: 0.0,    // Reserved smoothing control
        pot6: 1.0, // Red / Hue
        pot7: 0.0, // Green / Saturation
        pot8: 1.0, // Blue / Value
        buttons: Default::default(),
    };

    let audio_input = audio_input::AudioInput::new(128);

    let last_preset_change = None;

    Model {
        _gui_window: gui_window,
        led_strip_window,
        dmx,
        _midi_inputs: midi_inputs,
        midi_rx,
        shader_rx,
        shader,
        config,
        controller,
        target_slider_values: vec![0.5; 4], // First 4 Sliders
        target_pot_values: vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0], // Last 3 Pots
        smoothing_speed: 0.05,
        led_colors,
        led_outputs,
        last_preset_change,
        ui,
        ids,
        audio_input,
        midi_cv_phase_amp: 0.0,
    }
}

fn black_led_buffer(led_count: usize) -> Vec<LinSrgb> {
    vec![lin_srgb(0.0, 0.0, 0.0); led_count]
}

fn convert_channel(f: f32) -> u8 {
    (f.clamp(0.0, 1.0) * 255.0) as u8
}

// Convert the floating point f32 representation to bytes.
fn lin_srgb_f32_to_bytes(lin_srgb: &LinSrgb) -> [u8; 3] {
    let r = convert_channel(lin_srgb.red);
    let g = convert_channel(lin_srgb.green);
    let b = convert_channel(lin_srgb.blue);
    [r, g, b]
}

fn build_led_sacn_payloads(
    start_universe: u16,
    rgb_triplets: impl IntoIterator<Item = [u8; 3]>,
) -> Vec<(u16, Vec<u8>)> {
    let mut payloads = Vec::new();
    let mut universe = start_universe;
    let mut payload = vec![0];

    for rgb in rgb_triplets {
        payload.extend(rgb);

        // Full universes carry 170 RGB pixels = 510 DMX slots. Reserve the
        // last 2 slots as zeros so the next RGB triplet always starts on a new
        // universe boundary instead of being split 2/1 across universes.
        if payload.len() >= (UNIVERSE_CHANNEL_CAPACITY - 2) {
            payload.push(0);
            payload.push(0);
            payloads.push((universe, payload));
            universe += 1;
            payload = vec![0];
        }
    }

    // Intentionally do not emit a trailing empty universe when the LED data
    // lands exactly on a 170-pixel boundary. The old loop did that
    // accidentally by always sending after draining the last full packet.
    if payload.len() > 1 {
        payloads.push((universe, payload));
    }

    payloads
}

fn normalised_led_coord(index: usize, count: usize) -> f32 {
    if count <= 1 {
        0.0
    } else {
        (index as f32 / (count - 1) as f32) * 2.0 - 1.0
    }
}

fn sync_led_buffers(model: &mut Model) {
    let led_count = model.config.led_layout.led_count();
    if model.led_colors.len() != led_count {
        model.led_colors.resize(led_count, lin_srgb(0.0, 0.0, 0.0));
        model.led_outputs.resize(led_count, lin_srgb(0.0, 0.0, 0.0));
        model.last_preset_change = None;
    }
}

fn raw_window_event(app: &App, model: &mut Model, event: &ui::RawWindowEvent) {
    model.ui.handle_raw_event(app, event);
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    if key == Key::Space {
        let button = shader_shared::Button::Cycle;
        update_korg_button(&mut model.controller, button, korg::State::On);
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    model.audio_input.update();

    // Apply the GUI update.
    let mut ui = model.ui.set_widgets();
    let assets = app.assets_path().expect("failed to find assets directory");
    gui::update(
        &mut ui,
        gui::UpdateContext {
            config: &mut model.config,
            audio_input: &mut model.audio_input,
            dmx_bind_error: model.dmx.bind_error.as_deref(),
            since_start: update.since_start,
            shader_activity: model.shader_rx.activity(),
            led_colors: model.led_colors.as_slice(),
            last_preset_change: &mut model.last_preset_change,
            assets: assets.as_path(),
            ids: &mut model.ids,
        },
    );
    drop(ui);
    model.config.led_layout.normalise();
    sync_led_buffers(model);

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
    let led_layout = &model.config.led_layout;
    let pm_to_ps = |pm: Point2, h: f32| layout::topdown_metres_to_shader_coords(pm, h, led_layout);

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
                korg::Strip::E => {}
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

    model.audio_input.mod_amp1 = model.audio_input.mod_amp1 * (1.0 - model.smoothing_speed)
        + model.target_pot_values[0] * model.smoothing_speed;
    model.audio_input.mod_amp2 = model.audio_input.mod_amp2 * (1.0 - model.smoothing_speed)
        + model.target_pot_values[1] * model.smoothing_speed;
    model.audio_input.mod_amp3 = model.audio_input.mod_amp3 * (1.0 - model.smoothing_speed)
        + model.target_pot_values[2] * model.smoothing_speed;
    model.audio_input.mod_amp4 = model.audio_input.mod_amp4 * (1.0 - model.smoothing_speed)
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

    let env = model.audio_input.envelope;

    let piano_mod = (env * model.audio_input.mod_amp1) - (model.audio_input.mod_amp1 / 2.0);
    let bw_param1 = clamp(model.controller.slider1 + piano_mod, 0.0, 1.0);

    let piano_mod = (env * model.audio_input.mod_amp2) - (model.audio_input.mod_amp2 / 2.0);
    let bw_param2 = clamp(model.controller.slider2 + piano_mod, 0.0, 1.0);

    let piano_mod = (env * model.audio_input.mod_amp3) - (model.audio_input.mod_amp3 / 2.0);
    let colour_param1 = clamp(model.controller.slider3 + piano_mod, 0.0, 1.0);

    let piano_mod = (env * model.audio_input.mod_amp4) - (model.audio_input.mod_amp4 / 2.0);
    let colour_param2 = clamp(model.controller.slider4 + piano_mod, 0.0, 1.0);

    // Clone shader params and apply envelope modulation from the mod sliders.
    let mut shader_params = preset.shader_params;
    {
        let mut mod_slider_ix = 0;
        gui::apply_shader_modulation(
            preset.shader_left,
            &mut shader_params,
            &mut mod_slider_ix,
            &preset.shader_mod_amounts,
            env,
        );
        gui::apply_shader_modulation(
            preset.colourise,
            &mut shader_params,
            &mut mod_slider_ix,
            &preset.shader_mod_amounts,
            env,
        );
        gui::apply_shader_modulation(
            preset.shader_right,
            &mut shader_params,
            &mut mod_slider_ix,
            &preset.shader_mod_amounts,
            env,
        );
    }
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
        time: app.time + (env * model.midi_cv_phase_amp),
        resolution: layout::shader_resolution(led_layout),
        use_midi: model.config.midi_on,
        slider1: bw_param1,                // BW param 1
        slider2: bw_param2,                // BW param 2
        slider3: colour_param1,            // Colour param 1
        slider4: colour_param2,            // Colour param 2
        slider5: model.controller.slider5, // Shader param 5
        slider6: model.controller.slider6, // Shader param 6
        pot6: model.controller.pot6,       // Red / Hue
        pot7: model.controller.pot7,       // Green / Saturation
        pot8: model.controller.pot8,       // Blue / Value
        params: shader_params,
        mix: mix_info,
        buttons,
    };

    // Apply the shader for the LEDs.
    for (led_ix, (row, x, h)) in layout::led_positions_metres(led_layout).enumerate() {
        let ps = pm_to_ps(pt2(x, layout::SHADER_ORIGIN_METRES[1]), h);
        let index = led_ix;
        let col = led_ix % led_layout.leds_per_row();
        let col_row = [col, row];
        let n_x = normalised_led_coord(col, led_layout.leds_per_row());
        let n_y = normalised_led_coord(row, led_layout.row_count);
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
    let ftb = model.config.fade_to_black.led;
    let l_ftb = lin_srgb(ftb, ftb, ftb);
    for (i, (output, &colour)) in model
        .led_outputs
        .iter_mut()
        .zip(model.led_colors.iter())
        .enumerate()
    {
        let new = colour * l_ftb;
        *output = match prev_output.get(i) {
            None => new,
            Some(prev) => prev.lerp(&new, lerp_amt),
        };
    }

    // Ensure we are connected to a DMX source if enabled.
    if model.config.dmx_on {
        if let Ok(desired_interface_ip) =
            conf::parse_sacn_interface_ip(&model.config.sacn_interface_ip)
        {
            let should_refresh_source = model.dmx.source.is_none()
                || model.dmx.requested_interface_ip != desired_interface_ip;
            if should_refresh_source {
                match create_dmx_source(desired_interface_ip) {
                    Ok(source) => {
                        model.dmx.source = Some(source);
                        model.dmx.requested_interface_ip = desired_interface_ip;
                        model.dmx.bind_error = None;
                    }
                    Err(err) => {
                        model.dmx.requested_interface_ip = desired_interface_ip;
                        model.dmx.bind_error = Some(match desired_interface_ip {
                            Some(ip) => format!("Couldn't bind sACN to {}: {}", ip, err),
                            None => format!("Couldn't auto-bind sACN: {}", err),
                        });
                    }
                }
            }
        } else {
            model.dmx.requested_interface_ip = None;
            model.dmx.bind_error = None;
        }
    } else if model.dmx.source.is_some() {
        model.dmx.source.take();
        model.dmx.requested_interface_ip = None;
        model.dmx.bind_error = None;
    }

    // If we have a DMX source, send data over it!
    if let Some(ref mut dmx_source) = model.dmx.source {
        for (universe, payload) in build_led_sacn_payloads(
            model.config.led_start_universe,
            model.led_outputs.iter().map(lin_srgb_f32_to_bytes),
        ) {
            dmx_source
                .register_universe(universe)
                .expect("failed to register LED sACN universe");
            dmx_source
                .send(
                    &[universe],
                    &payload,
                    Some(E131_DEFAULT_PRIORITY),
                    None,
                    None,
                )
                .expect("failed to send LED DMX data");
        }
    }
}

fn create_dmx_source(interface_ip: Option<Ipv4Addr>) -> sacn::error::errors::Result<SacnSource> {
    let bind_ip = interface_ip.unwrap_or(Ipv4Addr::UNSPECIFIED);
    let bind_addr = SocketAddr::new(IpAddr::V4(bind_ip), 0);
    let mut source = SacnSource::with_ip("Cohen Pre-vis", bind_addr)?;
    // Preserve the old sender behaviour: data only, no source discovery chatter.
    source.set_is_sending_discovery(false);
    Ok(source)
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

fn led_strip_view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    let w = app.window(model.led_strip_window).unwrap().rect();
    let led_layout = &model.config.led_layout;

    let metres_to_points_scale = (w.h() / layout::top_led_row_from_ground(led_layout))
        .min(w.w() / led_layout.metres_per_row as f32)
        * 0.8;
    let m_to_p = |m| m * metres_to_points_scale;
    let p_to_m = |p| p / metres_to_points_scale;
    let x_offset_m = layout::SHADER_ORIGIN_METRES[0];
    let y_offset_m = layout::top_led_row_from_ground(led_layout) * 0.5;
    let pm_to_pp = |x: f32, h: f32| pt2(m_to_p(x - x_offset_m), m_to_p(h - y_offset_m));
    let pp_to_pm = |pp: Point2| (p_to_m(pp.x) + x_offset_m, p_to_m(pp.y) + y_offset_m);
    let pm_to_ps =
        |x: f32, h: f32| layout::topdown_metres_to_shader_coords(pt2(x, 0.0), h, led_layout);

    // Draw the LEDs one row at a time.
    let mut leds = layout::led_positions_metres(led_layout).zip(model.led_outputs.iter());
    for _ in 0..led_layout.row_count {
        let vs = leds
            .by_ref()
            .take(led_layout.leds_per_row())
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
            draw.text(s)
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
    let config_path = conf::path(assets);
    save_to_json(config_path, config).expect("failed to save config");
}

fn exit(app: &App, model: Model) {
    let assets = app
        .assets_path()
        .expect("failed to find project `assets` directory");
    save_config(assets.as_path(), &model.config);
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

#[cfg(test)]
mod tests {
    use super::{build_led_sacn_payloads, UNIVERSE_CHANNEL_CAPACITY};

    fn test_rgb_triplets(count: usize) -> Vec<[u8; 3]> {
        (0..count)
            .map(|i| {
                let base = (i * 3) as u8;
                [base, base.wrapping_add(1), base.wrapping_add(2)]
            })
            .collect()
    }

    #[test]
    fn payloads_include_dmx_start_code_at_byte_zero() {
        let payloads = build_led_sacn_payloads(7, vec![[12, 34, 56]]);

        assert_eq!(payloads, vec![(7, vec![0, 12, 34, 56])]);
    }

    #[test]
    fn full_universes_are_padded_with_two_zero_bytes_for_rgb_alignment() {
        let pixels = test_rgb_triplets(170);
        let payloads = build_led_sacn_payloads(1, pixels.iter().copied());

        assert_eq!(payloads.len(), 1);
        assert_eq!(payloads[0].0, 1);
        assert_eq!(payloads[0].1.len(), UNIVERSE_CHANNEL_CAPACITY);

        let expected: Vec<u8> = std::iter::once(0)
            .chain(pixels.iter().flat_map(|rgb| rgb.iter().copied()))
            .chain([0, 0])
            .collect();
        assert_eq!(payloads[0].1, expected);
    }

    #[test]
    fn rgb_data_splits_across_multiple_universes_without_splitting_triplets() {
        let pixels = test_rgb_triplets(171);
        let payloads = build_led_sacn_payloads(4, pixels.iter().copied());

        assert_eq!(payloads.len(), 2);
        assert_eq!(payloads[0].0, 4);
        assert_eq!(payloads[1].0, 5);
        assert_eq!(payloads[0].1.len(), UNIVERSE_CHANNEL_CAPACITY);
        assert_eq!(payloads[0].1[UNIVERSE_CHANNEL_CAPACITY - 2..], [0, 0]);
        assert_eq!(
            payloads[1].1,
            vec![0, pixels[170][0], pixels[170][1], pixels[170][2]]
        );
    }

    #[test]
    fn exact_universe_boundaries_do_not_emit_a_trailing_empty_universe() {
        let pixels = test_rgb_triplets(340);
        let payloads = build_led_sacn_payloads(9, pixels.iter().copied());

        assert_eq!(payloads.len(), 2);
        assert_eq!(payloads[0].0, 9);
        assert_eq!(payloads[1].0, 10);
        assert!(payloads.iter().all(|(_, payload)| !payload.is_empty()));
        assert!(payloads.iter().all(|(_, payload)| payload.len() > 1));
    }
}
