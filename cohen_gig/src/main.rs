use lerp::Lerp;
use nannou::prelude::*;
use nannou_conrod as ui;
use nannou_conrod::Ui;
use rayon::prelude::*;
use sacn::packet::{ACN_SDT_MULTICAST_PORT, E131_DEFAULT_PRIORITY, UNIVERSE_CHANNEL_CAPACITY};
use sacn::source::SacnSource;
use shader_shared::{Light, MixingInfo, ShaderParams, Uniforms, Vertex};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

mod audio_input;
mod audio_widgets;
mod conf;
mod gui;
pub mod knob;
mod layout;
mod lerp;
mod mad_mapper;
mod midi;
pub mod mod_slider;
mod sacn_sender;
mod shader;

use crate::conf::GlobalConfig;
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

struct MidiTargetState {
    target: f32,
    smoothed: f32,
    active: bool,
}

struct PreviewImages {
    left_id: ui::image::Id,
    right_id: ui::image::Id,
    colourise_id: ui::image::Id,
    width: u32,
    height: u32,
}

struct Model {
    _gui_window: window::Id,
    led_strip_window: window::Id,
    preview_window_visible: bool,
    led_worker: LedWorker,
    dmx: Dmx,
    midi_manager: midi::MidiManager,
    midi_mapping: midi::mapping::MidiMapping,
    midi_learn: midi::learn::LearnState,
    midi_values: HashMap<midi::mapping::MidiTarget, MidiTargetState>,
    shader_rx: ShaderReceiver,
    global_config: GlobalConfig,
    presets: conf::Presets,
    smoothing_speed: f32,
    colour_channels: [f32; 3],
    buttons: HashMap<shader_shared::Button, ButtonState>,
    led_colors: Vec<LinSrgb>,
    led_colors_left: Vec<LinSrgb>,
    led_colors_right: Vec<LinSrgb>,
    led_colors_colourise: Vec<LinSrgb>,
    led_outputs: Vec<LinSrgb>,
    last_preset_change: Option<LastPresetChange>,
    ui: Ui,
    ids: gui::Ids,
    left_panel_tab: gui::LeftPanelTab,
    audio_input: audio_input::AudioInput,
    runtime_stats: RuntimeStats,
    mad_project: Option<mad_mapper::MadProject>,
    resolved_layout: Option<layout::ResolvedLayout>,
    pending_file_dialog: Option<std::sync::mpsc::Receiver<Option<std::path::PathBuf>>>,
    preview_images: Option<PreviewImages>,
}

#[derive(Clone)]
struct LastPresetChange {
    started_at: Instant,
    preset: conf::Preset,
}

struct PresetTransitionState {
    started_at: Instant,
    preset: conf::Preset,
    led_colors: Vec<LinSrgb>,
    led_color_buffer: Vec<LinSrgb>,
}

#[derive(Copy, Clone)]
pub struct CachedLedShaderInput {
    pub position: Point3,
    pub light: Light,
}

#[derive(Clone)]
struct ButtonState {
    pub last_pressed: std::time::Instant,
    pub state: shader_shared::State,
}

struct Dmx {
    error: Option<String>,
    last_send_route: Option<DmxSendRoute>,
    monitor: SacnOutputMonitor,
}

struct DmxRuntime {
    source: Option<DmxOutputTransport>,
    requested_interface_ip: Option<Ipv4Addr>,
    error: Option<String>,
    last_send_route: Option<DmxSendRoute>,
    last_send_attempt_at: Option<Instant>,
    monitor: SacnOutputMonitor,
}

struct RuntimeStats {
    app_fps: f32,
}

struct LedWorker {
    shared_input: Arc<Mutex<LedWorkerSharedInput>>,
    shared_output: Arc<Mutex<LedWorkerSharedOutput>>,
    last_applied_frame_id: u64,
    thread: Option<thread::JoinHandle<()>>,
}

struct LedWorkerSharedInput {
    latest_state: LedWorkerInputState,
    pending_preset_change: Option<LastPresetChange>,
    pending_shader: Option<Shader>,
    shutdown: bool,
}

struct LedWorkerSharedOutput {
    frame_id: u64,
    led_colors: Vec<LinSrgb>,
    led_colors_left: Vec<LinSrgb>,
    led_colors_right: Vec<LinSrgb>,
    led_colors_colourise: Vec<LinSrgb>,
    led_outputs: Vec<LinSrgb>,
    monitor: LedWorkerMonitorSnapshot,
    dmx_error: Option<String>,
    last_send_route: Option<DmxSendRoute>,
}

#[derive(Clone)]
struct LedWorkerInputState {
    app_time: f32,
    snapshot_at: Instant,
    config: LedWorkerConfig,
    colour_channels: [f32; 3],
    audio_envelope: f32,
    buttons: HashMap<shader_shared::Button, ButtonState>,
    capture_output_monitor: bool,
}

#[derive(Clone)]
struct LedWorkerConfig {
    dmx_on: bool,
    sacn_interface_ip: String,
    led_output_fps: conf::LedOutputFps,
    led_start_universe: u16,
    fade_to_black_led: f32,
    preset_lerp_secs: f32,
    led_layout: conf::LedLayout,
    preset: conf::Preset,
    /// Resolved layout from MadMapper, if active.
    resolved_layout: Option<layout::ResolvedLayout>,
}

#[derive(Clone, Default)]
struct LedWorkerMonitorSnapshot {
    universes: Vec<SacnUniverseSnapshot>,
    total_frames_sent: u64,
    total_packets_sent: u64,
    total_payload_bytes_sent: u64,
    smoothed_frame_fps: f32,
    last_sent_at: Option<Instant>,
    last_send_error: Option<String>,
}

enum DmxOutputTransport {
    Network(SacnSource),
    Localhost(sacn_sender::LocalhostSacnSender),
    Auto {
        multicast: Option<SacnSource>,
        localhost: sacn_sender::LocalhostSacnSender,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DmxSendRoute {
    Multicast,
    Localhost,
}

impl DmxOutputTransport {
    fn send(&mut self, universe: u16, payload: &[u8]) -> Result<DmxSendRoute, String> {
        match self {
            Self::Network(source) => send_multicast_payload(source, universe, payload),
            Self::Localhost(sender) => sender
                .send_property_values(universe, payload)
                .map(|()| DmxSendRoute::Localhost)
                .map_err(|err| {
                    format!(
                        "Couldn't send localhost sACN universe {}: {}",
                        universe, err
                    )
                }),
            Self::Auto {
                multicast,
                localhost,
            } => {
                if let Some(source) = multicast.as_mut() {
                    match send_multicast_payload(source, universe, payload) {
                        Ok(route) => return Ok(route),
                        Err(_) => {
                            *multicast = None;
                        }
                    }
                }

                localhost
                    .send_property_values(universe, payload)
                    .map(|()| DmxSendRoute::Localhost)
                    .map_err(|err| {
                        format!(
                            "Couldn't send localhost sACN universe {}: {}",
                            universe, err
                        )
                    })
            }
        }
    }
}

#[derive(Clone)]
pub struct SacnUniverseSnapshot {
    pub universe: u16,
    pub payload: Vec<u8>,
    pub packets_sent: u64,
    pub last_sent_at: Option<Instant>,
}

#[derive(Default)]
pub struct SacnOutputMonitor {
    pub universes: Vec<SacnUniverseSnapshot>,
    pub selected_universe: Option<u16>,
    pub total_frames_sent: u64,
    pub total_packets_sent: u64,
    pub total_payload_bytes_sent: u64,
    pub smoothed_frame_fps: f32,
    pub last_sent_at: Option<Instant>,
    pub last_send_error: Option<String>,
}

impl SacnOutputMonitor {
    pub fn available_universe_labels(&self) -> Vec<String> {
        self.universes
            .iter()
            .map(|snapshot| format!("Universe {}", snapshot.universe))
            .collect()
    }

    pub fn selected_universe_index(&self) -> Option<usize> {
        let selected_universe = self.selected_universe?;
        self.universes
            .iter()
            .position(|snapshot| snapshot.universe == selected_universe)
    }

    pub fn select_universe(&mut self, index: usize) -> Option<u16> {
        let universe = self.universes.get(index)?.universe;
        self.selected_universe = Some(universe);
        Some(universe)
    }

    pub fn selected_universe_snapshot(&self) -> Option<&SacnUniverseSnapshot> {
        let selected_universe = self.selected_universe?;
        self.universes
            .iter()
            .find(|snapshot| snapshot.universe == selected_universe)
    }

    fn record_successful_frame_stats(
        &mut self,
        packet_count: usize,
        payload_bytes_sent: usize,
        now: Instant,
    ) {
        if let Some(previous_frame_at) = self.last_sent_at {
            if let Some(sample) = fps_from_duration(now.duration_since(previous_frame_at)) {
                self.smoothed_frame_fps = smooth_fps(self.smoothed_frame_fps, sample);
            }
        }

        self.total_frames_sent += 1;
        self.total_packets_sent += packet_count as u64;
        self.total_payload_bytes_sent += payload_bytes_sent as u64;
        self.last_sent_at = Some(now);
        self.last_send_error = None;
    }

    fn record_universe_snapshots(&mut self, payloads: &[(u16, Vec<u8>)], now: Instant) {
        self.universes.retain(|snapshot| {
            payloads
                .iter()
                .any(|(universe, _)| *universe == snapshot.universe)
        });

        for (universe, payload) in payloads {
            if let Some(snapshot) = self
                .universes
                .iter_mut()
                .find(|snapshot| snapshot.universe == *universe)
            {
                snapshot.payload = payload.clone();
                snapshot.packets_sent += 1;
                snapshot.last_sent_at = Some(now);
            } else {
                self.universes.push(SacnUniverseSnapshot {
                    universe: *universe,
                    payload: payload.clone(),
                    packets_sent: 1,
                    last_sent_at: Some(now),
                });
            }
        }

        self.universes
            .sort_by(|left, right| left.universe.cmp(&right.universe));

        if !self
            .selected_universe
            .map(|selected| {
                self.universes
                    .iter()
                    .any(|snapshot| snapshot.universe == selected)
            })
            .unwrap_or(false)
        {
            self.selected_universe = self.universes.first().map(|snapshot| snapshot.universe);
        }
    }

    fn record_send_error(&mut self, error: String) {
        self.last_send_error = Some(error);
    }
}

impl LedWorkerMonitorSnapshot {
    fn from_monitor(monitor: &SacnOutputMonitor) -> Self {
        Self {
            universes: monitor.universes.clone(),
            total_frames_sent: monitor.total_frames_sent,
            total_packets_sent: monitor.total_packets_sent,
            total_payload_bytes_sent: monitor.total_payload_bytes_sent,
            smoothed_frame_fps: monitor.smoothed_frame_fps,
            last_sent_at: monitor.last_sent_at,
            last_send_error: monitor.last_send_error.clone(),
        }
    }
}

impl LedWorker {
    fn new(initial_state: LedWorkerInputState) -> Self {
        let shared_input = Arc::new(Mutex::new(LedWorkerSharedInput {
            latest_state: initial_state,
            pending_preset_change: None,
            pending_shader: None,
            shutdown: false,
        }));
        let shared_output = Arc::new(Mutex::new(LedWorkerSharedOutput {
            frame_id: 0,
            led_colors: Vec::new(),
            led_colors_left: Vec::new(),
            led_colors_right: Vec::new(),
            led_colors_colourise: Vec::new(),
            led_outputs: Vec::new(),
            monitor: LedWorkerMonitorSnapshot::default(),
            dmx_error: None,
            last_send_route: None,
        }));

        let worker_input = Arc::clone(&shared_input);
        let worker_output = Arc::clone(&shared_output);
        let thread = thread::spawn(move || run_led_worker(worker_input, worker_output));

        Self {
            shared_input,
            shared_output,
            last_applied_frame_id: 0,
            thread: Some(thread),
        }
    }
}

impl RuntimeStats {
    fn record_app_frame(&mut self, since_last: Duration) {
        if let Some(sample) = fps_from_duration(since_last) {
            self.app_fps = smooth_fps(self.app_fps, sample);
        }
    }
}

fn main() {
    nannou::app(model).update(update).exit(exit).run();
}

fn model(app: &App) -> Model {
    let assets = app
        .assets_path()
        .expect("failed to find project `assets` directory");

    let (mut global_config, mut presets) = conf::load(&assets);
    global_config.led_layout.normalise();
    for preset in &mut presets.list {
        preset.migrate_legacy();
        gui::normalise_preset_shader_mod_amounts(preset);
    }

    let gui_window = app
        .new_window()
        .title("COHEN GIG - GUI")
        .size(gui::WINDOW_WIDTH, gui::WINDOW_HEIGHT)
        .surface_conf_builder(
            nannou::window::SurfaceConfigurationBuilder::new()
                .present_mode(nannou::wgpu::PresentMode::Immediate),
        )
        .raw_event(raw_window_event)
        .key_pressed(key_pressed)
        .view(gui_view)
        .build()
        .expect("failed to build GUI window");

    let led_strip_window = app
        .new_window()
        .title("COHEN GIG - PREVIS")
        .size(LED_STRIP_WINDOW_W, LED_STRIP_WINDOW_H)
        .surface_conf_builder(
            nannou::window::SurfaceConfigurationBuilder::new()
                .present_mode(nannou::wgpu::PresentMode::Immediate),
        )
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
        w.set_visible(global_config.preview_window_on);
    }

    let dmx = Dmx {
        error: None,
        last_send_route: None,
        monitor: SacnOutputMonitor::default(),
    };

    let shader_rx = shader::spawn_watch();

    let mad_project = global_config
        .madmapper_project_path
        .as_ref()
        .and_then(|path| match mad_mapper::parse(path) {
            Ok(project) => {
                eprintln!(
                    "Loaded MadMapper project: {} fixtures, {} pixels",
                    project.fixtures.len(),
                    project.total_pixels()
                );
                Some(project)
            }
            Err(e) => {
                eprintln!("Failed to parse MadMapper project: {}", e);
                None
            }
        });

    let initial_led_count = mad_project
        .as_ref()
        .map(|p| p.total_pixels())
        .unwrap_or_else(|| global_config.led_layout.led_count());
    let led_colors = black_led_buffer(initial_led_count);
    let led_outputs = black_led_buffer(initial_led_count);

    let midi_manager = midi::MidiManager::new();

    // Load MIDI mapping preset (or default empty).
    let midi_mappings_dir = assets.join("midi_mappings");
    let midi_mapping = midi::mapping::MidiMappingPreset::list_presets(&midi_mappings_dir)
        .into_iter()
        .next()
        .and_then(|name| {
            let path = midi_mappings_dir.join(format!("{name}.json"));
            midi::mapping::MidiMappingPreset::load(&path).ok()
        })
        .map(midi::mapping::MidiMapping::new)
        .unwrap_or_default();

    let audio_input = audio_input::AudioInput::new(128, global_config.audio_input_device.clone());
    let colour_channels = [1.0, 0.0, 1.0]; // R/H, G/S, B/V defaults

    let resolved_layout = mad_project.as_ref().map(layout::resolve_from_mad_project);

    let last_preset_change = None;
    let led_worker = LedWorker::new(build_led_worker_input_state(
        0.0,
        &global_config,
        &presets,
        &audio_input,
        colour_channels,
        gui::LeftPanelTab::Live,
        &resolved_layout,
    ));

    Model {
        _gui_window: gui_window,
        led_strip_window,
        preview_window_visible: global_config.preview_window_on,
        led_worker,
        dmx,
        midi_manager,
        midi_mapping,
        midi_learn: midi::learn::LearnState::default(),
        midi_values: HashMap::new(),
        shader_rx,
        global_config,
        presets,
        smoothing_speed: 0.05,
        colour_channels,
        buttons: Default::default(),
        led_colors,
        led_colors_left: black_led_buffer(initial_led_count),
        led_colors_right: black_led_buffer(initial_led_count),
        led_colors_colourise: black_led_buffer(initial_led_count),
        led_outputs,
        last_preset_change,
        ui,
        ids,
        left_panel_tab: gui::LeftPanelTab::Live,
        audio_input,
        runtime_stats: RuntimeStats { app_fps: 0.0 },
        resolved_layout,
        mad_project,
        pending_file_dialog: None,
        preview_images: None,
    }
}

fn black_led_buffer(led_count: usize) -> Vec<LinSrgb> {
    vec![lin_srgb(0.0, 0.0, 0.0); led_count]
}

fn led_colors_to_rgba(colors: &[LinSrgb], width: u32, height: u32) -> Vec<u8> {
    let pixel_count = (width * height) as usize;
    let mut rgba = vec![0u8; pixel_count * 4];
    for (i, color) in colors.iter().take(pixel_count).enumerate() {
        let offset = i * 4;
        rgba[offset] = (color.red.clamp(0.0, 1.0) * 255.0) as u8;
        rgba[offset + 1] = (color.green.clamp(0.0, 1.0) * 255.0) as u8;
        rgba[offset + 2] = (color.blue.clamp(0.0, 1.0) * 255.0) as u8;
        rgba[offset + 3] = 255;
    }
    rgba
}

fn preview_dimensions(
    led_layout: &conf::LedLayout,
    mad_project: Option<&mad_mapper::MadProject>,
) -> (u32, u32) {
    if let Some(project) = mad_project {
        let fixtures = project.fixtures_by_row();
        let rows = fixtures.len() as u32;
        let max_cols = fixtures
            .iter()
            .map(|f| f.pixel_count as u32)
            .max()
            .unwrap_or(1);
        (max_cols, rows.max(1))
    } else {
        (
            led_layout.leds_per_row() as u32,
            led_layout.row_count as u32,
        )
    }
}

fn update_preview_textures(app: &App, model: &mut Model) {
    let (width, height) =
        preview_dimensions(&model.global_config.led_layout, model.mad_project.as_ref());
    if width == 0 || height == 0 {
        return;
    }

    let window = match app.window(model._gui_window) {
        Some(w) => w,
        None => return,
    };

    let needs_recreate = match &model.preview_images {
        Some(pi) => pi.width != width || pi.height != height,
        None => true,
    };

    if needs_recreate {
        let device = window.device();

        let create_preview_texture = |label: &'static str| {
            device.create_texture(&nannou::wgpu::TextureDescriptor {
                label: Some(label),
                size: nannou::wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: nannou::wgpu::TextureDimension::D2,
                format: nannou::wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: nannou::wgpu::TextureUsages::COPY_DST
                    | nannou::wgpu::TextureUsages::TEXTURE_BINDING,
            })
        };

        let left_tex = create_preview_texture("preview_left");
        let right_tex = create_preview_texture("preview_right");
        let colourise_tex = create_preview_texture("preview_colourise");

        let left_img = ui::conrod_wgpu::Image {
            texture: left_tex,
            texture_format: nannou::wgpu::TextureFormat::Rgba8UnormSrgb,
            width,
            height,
        };
        let right_img = ui::conrod_wgpu::Image {
            texture: right_tex,
            texture_format: nannou::wgpu::TextureFormat::Rgba8UnormSrgb,
            width,
            height,
        };
        let colourise_img = ui::conrod_wgpu::Image {
            texture: colourise_tex,
            texture_format: nannou::wgpu::TextureFormat::Rgba8UnormSrgb,
            width,
            height,
        };

        if let Some(old) = model.preview_images.take() {
            model.ui.image_map.remove(old.left_id);
            model.ui.image_map.remove(old.right_id);
            model.ui.image_map.remove(old.colourise_id);
        }

        let left_id = model.ui.image_map.insert(left_img);
        let right_id = model.ui.image_map.insert(right_img);
        let colourise_id = model.ui.image_map.insert(colourise_img);
        model.preview_images = Some(PreviewImages {
            left_id,
            right_id,
            colourise_id,
            width,
            height,
        });
    }

    if let Some(ref pi) = model.preview_images {
        let queue = window.queue();
        let size = nannou::wgpu::Extent3d {
            width: pi.width,
            height: pi.height,
            depth_or_array_layers: 1,
        };
        let layout = nannou::wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(pi.width * 4),
            rows_per_image: std::num::NonZeroU32::new(pi.height),
        };

        let left_rgba = led_colors_to_rgba(&model.led_colors_left, pi.width, pi.height);
        if let Some(img) = model.ui.image_map.get(&pi.left_id) {
            queue.write_texture(
                nannou::wgpu::ImageCopyTexture {
                    texture: &img.texture,
                    mip_level: 0,
                    origin: nannou::wgpu::Origin3d::ZERO,
                    aspect: nannou::wgpu::TextureAspect::All,
                },
                &left_rgba,
                layout,
                size,
            );
        }

        let right_rgba = led_colors_to_rgba(&model.led_colors_right, pi.width, pi.height);
        if let Some(img) = model.ui.image_map.get(&pi.right_id) {
            queue.write_texture(
                nannou::wgpu::ImageCopyTexture {
                    texture: &img.texture,
                    mip_level: 0,
                    origin: nannou::wgpu::Origin3d::ZERO,
                    aspect: nannou::wgpu::TextureAspect::All,
                },
                &right_rgba,
                layout,
                size,
            );
        }

        let colourise_rgba = led_colors_to_rgba(&model.led_colors_colourise, pi.width, pi.height);
        if let Some(img) = model.ui.image_map.get(&pi.colourise_id) {
            queue.write_texture(
                nannou::wgpu::ImageCopyTexture {
                    texture: &img.texture,
                    mip_level: 0,
                    origin: nannou::wgpu::Origin3d::ZERO,
                    aspect: nannou::wgpu::TextureAspect::All,
                },
                &colourise_rgba,
                layout,
                size,
            );
        }
    }
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

fn build_sacn_payloads(
    dmx_map: Option<&layout::DmxMap>,
    fallback_start_universe: u16,
    led_outputs: &[LinSrgb],
) -> Vec<(u16, Vec<u8>)> {
    match dmx_map {
        Some(layout::DmxMap::PerFixture(entries)) => {
            build_per_fixture_payloads(entries, led_outputs)
        }
        Some(layout::DmxMap::Sequential { start_universe }) => build_led_sacn_payloads(
            *start_universe,
            led_outputs.iter().map(lin_srgb_f32_to_bytes),
        ),
        None => build_led_sacn_payloads(
            fallback_start_universe,
            led_outputs.iter().map(lin_srgb_f32_to_bytes),
        ),
    }
}

fn build_per_fixture_payloads(
    entries: &[layout::FixtureDmxEntry],
    led_outputs: &[LinSrgb],
) -> Vec<(u16, Vec<u8>)> {
    let mut all_payloads: Vec<(u16, Vec<u8>)> = Vec::new();
    for entry in entries {
        let end = (entry.led_offset + entry.led_count).min(led_outputs.len());
        let fixture_leds = &led_outputs[entry.led_offset..end];
        let fixture_payloads = build_led_sacn_payloads(
            entry.start_universe,
            fixture_leds.iter().map(lin_srgb_f32_to_bytes),
        );
        all_payloads.extend(fixture_payloads);
    }
    all_payloads
}

fn normalised_led_coord(index: usize, count: usize) -> f32 {
    if count <= 1 {
        0.0
    } else {
        (index as f32 / (count - 1) as f32) * 2.0 - 1.0
    }
}

pub fn rebuild_led_shader_inputs(led_layout: &conf::LedLayout) -> Vec<CachedLedShaderInput> {
    layout::led_positions_metres(led_layout)
        .enumerate()
        .map(|(led_ix, (row, x, h))| {
            let position = layout::topdown_metres_to_shader_coords(
                pt2(x, layout::SHADER_ORIGIN_METRES[1]),
                h,
                led_layout,
            );
            let col = led_ix % led_layout.leds_per_row();
            let light = Light::Led {
                index: led_ix,
                col_row: [col, row],
                normalised_coords: vec2(
                    normalised_led_coord(col, led_layout.leds_per_row()),
                    normalised_led_coord(row, led_layout.row_count),
                ),
            };
            CachedLedShaderInput { position, light }
        })
        .collect()
}

fn build_led_worker_input_state(
    app_time: f32,
    global_config: &GlobalConfig,
    presets: &conf::Presets,
    audio_input: &audio_input::AudioInput,
    colour_channels: [f32; 3],
    left_panel_tab: gui::LeftPanelTab,
    resolved_layout: &Option<layout::ResolvedLayout>,
) -> LedWorkerInputState {
    let resolved_layout = resolved_layout.clone();
    LedWorkerInputState {
        app_time,
        snapshot_at: Instant::now(),
        config: LedWorkerConfig {
            dmx_on: global_config.dmx_on,
            sacn_interface_ip: global_config.sacn_interface_ip.clone(),
            led_output_fps: global_config.led_output_fps,
            led_start_universe: global_config.led_start_universe,
            fade_to_black_led: global_config.fade_to_black.led,
            preset_lerp_secs: global_config.preset_lerp_secs,
            led_layout: global_config.led_layout.clone(),
            preset: presets.selected().clone(),
            resolved_layout,
        },
        colour_channels,
        audio_envelope: audio_input.envelope,
        buttons: Default::default(),
        capture_output_monitor: left_panel_tab == gui::LeftPanelTab::Output,
    }
}

fn sync_led_buffers(model: &mut Model) {
    let led_count = model
        .mad_project
        .as_ref()
        .map(|p| p.total_pixels())
        .unwrap_or_else(|| model.global_config.led_layout.led_count());
    if model.led_colors.len() != led_count {
        model.led_colors.resize(led_count, lin_srgb(0.0, 0.0, 0.0));
        model
            .led_colors_left
            .resize(led_count, lin_srgb(0.0, 0.0, 0.0));
        model
            .led_colors_right
            .resize(led_count, lin_srgb(0.0, 0.0, 0.0));
        model
            .led_colors_colourise
            .resize(led_count, lin_srgb(0.0, 0.0, 0.0));
        model.led_outputs.resize(led_count, lin_srgb(0.0, 0.0, 0.0));
        model.last_preset_change = None;
    }
}

/// Copy the shader-specific fields from `src` into `dst` for the given shader type.

fn apply_midi_values(model: &mut Model) {
    use midi::mapping::MidiTarget;
    for (&target, state) in &model.midi_values {
        if !state.active {
            continue;
        }
        let v = state.smoothed;
        match target {
            MidiTarget::SmoothingSpeed => {
                model.smoothing_speed = map_range(v, 0.0, 1.0, 0.0008, 0.08);
            }
            MidiTarget::FadeToBlack => {
                model.global_config.fade_to_black.led = v;
            }
            MidiTarget::LeftRightMix => {
                model.presets.selected_mut().left_right_mix = map_range(v, 0.0, 1.0, -1.0, 1.0);
            }
            MidiTarget::AudioGain => {
                model.audio_input.gain_db =
                    map_range(v, 0.0, 1.0, 0.0, audio_input::MAX_INPUT_GAIN_DB);
            }
            MidiTarget::AudioThreshold => {
                model.audio_input.threshold = v;
            }
            MidiTarget::AudioAttack => {
                model.audio_input.attack = map_range(v, 0.0, 1.0, 0.001, 1.0);
            }
            MidiTarget::AudioHold => {
                model.audio_input.hold = map_range(v, 0.0, 1.0, 0.0, 2.0);
            }
            MidiTarget::AudioRelease => {
                model.audio_input.release = map_range(v, 0.0, 1.0, 0.01, 2.0);
            }
            MidiTarget::ColourChannel1 => {
                model.colour_channels[0] = v;
            }
            MidiTarget::ColourChannel2 => {
                model.colour_channels[1] = v;
            }
            MidiTarget::ColourChannel3 => {
                model.colour_channels[2] = v;
            }
            MidiTarget::ColourPalette => {
                model
                    .presets
                    .selected_mut()
                    .shader_params_colourise
                    .colour_palettes
                    .interval = map_range(v, 0.0, 1.0, 0.0, 1.0);
            }
            MidiTarget::ShaderLeftParam(n) => {
                let preset = model.presets.selected_mut();
                let shader = preset.shader_left;
                let params: &mut dyn gui::Params =
                    gui::shader_params(shader, &mut preset.shader_params_left);
                if (n as usize) < params.param_count() {
                    let p = params.param_mut(n as usize);
                    if let gui::ParamKindMut::F32 { value, max } = p.kind {
                        *value = v * max;
                    }
                }
            }
            MidiTarget::ShaderRightParam(n) => {
                let preset = model.presets.selected_mut();
                let shader = preset.shader_right;
                let params: &mut dyn gui::Params =
                    gui::shader_params(shader, &mut preset.shader_params_right);
                if (n as usize) < params.param_count() {
                    let p = params.param_mut(n as usize);
                    if let gui::ParamKindMut::F32 { value, max } = p.kind {
                        *value = v * max;
                    }
                }
            }
            MidiTarget::ShaderLeftMod(n) => {
                let preset = model.presets.selected_mut();
                let idx = n as usize;
                if idx < preset.shader_mod_amounts_left.len() {
                    preset.shader_mod_amounts_left[idx] = v;
                }
            }
            MidiTarget::ShaderRightMod(n) => {
                let preset = model.presets.selected_mut();
                let idx = n as usize;
                if idx < preset.shader_mod_amounts_right.len() {
                    preset.shader_mod_amounts_right[idx] = v;
                }
            }
        }
    }
}

fn queue_led_worker_update(app: &App, model: &mut Model) {
    if let Ok(mut shared_input) = model.led_worker.shared_input.lock() {
        shared_input.latest_state = build_led_worker_input_state(
            app.time,
            &model.global_config,
            &model.presets,
            &model.audio_input,
            model.colour_channels,
            model.left_panel_tab,
            &model.resolved_layout,
        );
        shared_input.latest_state.buttons = model.buttons.clone();

        if let Some(last_preset_change) = model.last_preset_change.take() {
            shared_input.pending_preset_change = Some(last_preset_change);
        }
    }
}

fn apply_led_worker_output(model: &mut Model) {
    let Ok(shared_output) = model.led_worker.shared_output.lock() else {
        return;
    };

    if shared_output.frame_id == model.led_worker.last_applied_frame_id {
        return;
    }

    model.led_worker.last_applied_frame_id = shared_output.frame_id;
    model.led_colors.clone_from(&shared_output.led_colors);
    model
        .led_colors_left
        .clone_from(&shared_output.led_colors_left);
    model
        .led_colors_right
        .clone_from(&shared_output.led_colors_right);
    model
        .led_colors_colourise
        .clone_from(&shared_output.led_colors_colourise);
    model.led_outputs.clone_from(&shared_output.led_outputs);
    model.dmx.error = shared_output.dmx_error.clone();
    model.dmx.last_send_route = shared_output.last_send_route;

    let selected_universe = model.dmx.monitor.selected_universe;
    model
        .dmx
        .monitor
        .universes
        .clone_from(&shared_output.monitor.universes);
    model.dmx.monitor.total_frames_sent = shared_output.monitor.total_frames_sent;
    model.dmx.monitor.total_packets_sent = shared_output.monitor.total_packets_sent;
    model.dmx.monitor.total_payload_bytes_sent = shared_output.monitor.total_payload_bytes_sent;
    model.dmx.monitor.smoothed_frame_fps = shared_output.monitor.smoothed_frame_fps;
    model.dmx.monitor.last_sent_at = shared_output.monitor.last_sent_at;
    model.dmx.monitor.last_send_error = shared_output.monitor.last_send_error.clone();

    model.dmx.monitor.selected_universe = selected_universe
        .filter(|selected| {
            model
                .dmx
                .monitor
                .universes
                .iter()
                .any(|snapshot| snapshot.universe == *selected)
        })
        .or_else(|| {
            model
                .dmx
                .monitor
                .universes
                .first()
                .map(|snapshot| snapshot.universe)
        });
}

struct LedWorkerRuntime {
    shader: Option<Shader>,
    led_colors: Vec<LinSrgb>,
    led_colors_left: Vec<LinSrgb>,
    led_colors_right: Vec<LinSrgb>,
    led_colors_colourise: Vec<LinSrgb>,
    led_color_buffer: Vec<LinSrgb>,
    led_outputs: Vec<LinSrgb>,
    led_shader_inputs: Vec<CachedLedShaderInput>,
    cached_led_layout: conf::LedLayout,
    /// True when currently using a MadMapper resolved layout.
    using_mad_layout: bool,
    preset_transition: Option<PresetTransitionState>,
    dmx: DmxRuntime,
}

impl LedWorkerRuntime {
    fn new(config: &LedWorkerConfig) -> Self {
        let (led_count, shader_inputs, using_mad) = match &config.resolved_layout {
            Some(rl) => (rl.led_count, rl.shader_inputs.clone(), true),
            None => {
                let inputs = rebuild_led_shader_inputs(&config.led_layout);
                let count = inputs.len();
                (count, inputs, false)
            }
        };
        Self {
            shader: None,
            led_colors: black_led_buffer(led_count),
            led_colors_left: black_led_buffer(led_count),
            led_colors_right: black_led_buffer(led_count),
            led_colors_colourise: black_led_buffer(led_count),
            led_color_buffer: black_led_buffer(led_count),
            led_outputs: black_led_buffer(led_count),
            led_shader_inputs: shader_inputs,
            cached_led_layout: config.led_layout.clone(),
            using_mad_layout: using_mad,
            preset_transition: None,
            dmx: DmxRuntime {
                source: None,
                requested_interface_ip: None,
                error: None,
                last_send_route: None,
                last_send_attempt_at: None,
                monitor: SacnOutputMonitor::default(),
            },
        }
    }
}

fn sync_led_worker_buffers(runtime: &mut LedWorkerRuntime, config: &LedWorkerConfig) {
    let (led_count, new_inputs, now_mad) = match &config.resolved_layout {
        Some(rl) => (rl.led_count, Some(&rl.shader_inputs), true),
        None => (config.led_layout.led_count(), None, false),
    };

    let source_changed = now_mad != runtime.using_mad_layout
        || (!now_mad && runtime.cached_led_layout != config.led_layout);

    if runtime.led_colors.len() != led_count {
        runtime
            .led_colors
            .resize(led_count, lin_srgb(0.0, 0.0, 0.0));
        runtime
            .led_colors_left
            .resize(led_count, lin_srgb(0.0, 0.0, 0.0));
        runtime
            .led_colors_right
            .resize(led_count, lin_srgb(0.0, 0.0, 0.0));
        runtime
            .led_colors_colourise
            .resize(led_count, lin_srgb(0.0, 0.0, 0.0));
        runtime
            .led_color_buffer
            .resize(led_count, lin_srgb(0.0, 0.0, 0.0));
        runtime
            .led_outputs
            .resize(led_count, lin_srgb(0.0, 0.0, 0.0));
        runtime.preset_transition = None;
    }
    if source_changed || runtime.led_shader_inputs.len() != led_count {
        runtime.led_shader_inputs = match new_inputs {
            Some(inputs) => inputs.clone(),
            None => rebuild_led_shader_inputs(&config.led_layout),
        };
        runtime.cached_led_layout = config.led_layout.clone();
        runtime.using_mad_layout = now_mad;
        runtime.preset_transition = None;
    }
}

fn run_led_worker(
    shared_input: Arc<Mutex<LedWorkerSharedInput>>,
    shared_output: Arc<Mutex<LedWorkerSharedOutput>>,
) {
    let initial_state = {
        let input = shared_input
            .lock()
            .expect("led worker input lock poisoned during startup");
        input.latest_state.clone()
    };
    let mut runtime = LedWorkerRuntime::new(&initial_state.config);
    let mut frame_id = 0u64;

    loop {
        let (state, pending_shader, pending_preset_change, shutdown) = {
            let mut input = match shared_input.lock() {
                Ok(input) => input,
                Err(_) => break,
            };
            (
                input.latest_state.clone(),
                input.pending_shader.take(),
                input.pending_preset_change.take(),
                input.shutdown,
            )
        };

        if shutdown {
            break;
        }

        if let Some(shader) = pending_shader {
            runtime.shader = Some(shader);
        }
        if let Some(last_preset_change) = pending_preset_change {
            runtime.preset_transition = Some(PresetTransitionState {
                started_at: last_preset_change.started_at,
                preset: last_preset_change.preset,
                led_colors: runtime.led_colors.clone(),
                led_color_buffer: black_led_buffer(runtime.led_colors.len()),
            });
        }

        sync_led_worker_buffers(&mut runtime, &state.config);
        render_led_worker_frame(&state, &mut runtime);

        frame_id = frame_id.wrapping_add(1);
        if let Ok(mut output) = shared_output.lock() {
            output.frame_id = frame_id;
            output.led_colors.clone_from(&runtime.led_colors);
            output.led_colors_left.clone_from(&runtime.led_colors_left);
            output
                .led_colors_right
                .clone_from(&runtime.led_colors_right);
            output
                .led_colors_colourise
                .clone_from(&runtime.led_colors_colourise);
            output.led_outputs.clone_from(&runtime.led_outputs);
            output.monitor = LedWorkerMonitorSnapshot::from_monitor(&runtime.dmx.monitor);
            output.dmx_error = runtime.dmx.error.clone();
            output.last_send_route = runtime.dmx.last_send_route;
        }

        thread::yield_now();
    }
}

fn render_led_worker_frame(state: &LedWorkerInputState, runtime: &mut LedWorkerRuntime) {
    let shader: ShaderFnPtr = runtime
        .shader
        .as_ref()
        .map(|shader| *shader.get_fn())
        .unwrap_or(shader::black);
    let uniforms = preset_uniforms(state, &state.config.preset);
    render_preset_graph(
        shader,
        &runtime.led_shader_inputs,
        &uniforms,
        &runtime.led_colors,
        &mut runtime.led_color_buffer,
    );
    std::mem::swap(&mut runtime.led_colors, &mut runtime.led_color_buffer);

    // White colourise params used to bypass post-processing in left/right previews.
    let white_colourise = ShaderParams {
        solid_rgb_colour: shader_shared::SolidRgbColour {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
        },
        ..ShaderParams::default()
    };

    // Compute isolated left preview (Add blend, left only, no colourise).
    {
        let left_only_mix = MixingInfo {
            colourise: shader_shared::Shader::SolidRgbColour,
            blend_mode: shader_shared::BlendMode::Add,
            xfade_left: 1.0,
            xfade_right: 0.0,
            params_colourise: white_colourise,
            ..uniforms.mix.clone()
        };
        let left_uniforms = Uniforms {
            mix: left_only_mix,
            ..uniforms.clone()
        };
        runtime
            .led_colors_left
            .par_iter_mut()
            .zip(runtime.led_shader_inputs.par_iter())
            .zip(runtime.led_colors.par_iter())
            .for_each(|((color, led_input), &last_color)| {
                let vertex = Vertex {
                    position: led_input.position,
                    light: led_input.light,
                    last_color,
                };
                *color = shader(vertex, &left_uniforms);
            });
    }

    // Compute isolated right preview (Add blend, right only, no colourise).
    {
        let right_only_mix = MixingInfo {
            colourise: shader_shared::Shader::SolidRgbColour,
            blend_mode: shader_shared::BlendMode::Add,
            xfade_left: 0.0,
            xfade_right: 1.0,
            params_colourise: white_colourise,
            ..uniforms.mix.clone()
        };
        let right_uniforms = Uniforms {
            mix: right_only_mix,
            ..uniforms.clone()
        };
        runtime
            .led_colors_right
            .par_iter_mut()
            .zip(runtime.led_shader_inputs.par_iter())
            .zip(runtime.led_colors.par_iter())
            .for_each(|((color, led_input), &last_color)| {
                let vertex = Vertex {
                    position: led_input.position,
                    light: led_input.light,
                    last_color,
                };
                *color = shader(vertex, &right_uniforms);
            });
    }

    // Compute isolated colourise preview.
    {
        use shader_shared::SolidRgbColour;
        let colourise_only_mix = MixingInfo {
            left: shader_shared::Shader::SolidRgbColour,
            blend_mode: shader_shared::BlendMode::Add,
            xfade_left: 1.0,
            xfade_right: 0.0,
            params_left: ShaderParams {
                solid_rgb_colour: SolidRgbColour {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                },
                ..ShaderParams::default()
            },
            params_right: ShaderParams::default(),
            ..uniforms.mix.clone()
        };
        let colourise_uniforms = Uniforms {
            mix: colourise_only_mix,
            ..uniforms.clone()
        };
        runtime
            .led_colors_colourise
            .par_iter_mut()
            .zip(runtime.led_shader_inputs.par_iter())
            .zip(runtime.led_colors.par_iter())
            .for_each(|((color, led_input), &last_color)| {
                let vertex = Vertex {
                    position: led_input.position,
                    light: led_input.light,
                    last_color,
                };
                *color = shader(vertex, &colourise_uniforms);
            });
    }

    let mut clear_transition = state.config.preset_lerp_secs <= 0.0;
    let (prev_output, lerp_amt) = match runtime.preset_transition.as_mut() {
        None => (&[][..], 1.0),
        Some(transition) => {
            let elapsed_secs = transition.started_at.elapsed().as_secs_f32();
            if elapsed_secs < state.config.preset_lerp_secs {
                let transition_uniforms = preset_uniforms(state, &transition.preset);
                render_preset_graph(
                    shader,
                    &runtime.led_shader_inputs,
                    &transition_uniforms,
                    &transition.led_colors,
                    &mut transition.led_color_buffer,
                );
                std::mem::swap(&mut transition.led_colors, &mut transition.led_color_buffer);
                (
                    &transition.led_colors[..],
                    ease_in_out((elapsed_secs / state.config.preset_lerp_secs).clamp(0.0, 1.0)),
                )
            } else {
                clear_transition = true;
                (&[][..], 1.0)
            }
        }
    };

    let ftb = state.config.fade_to_black_led;
    let l_ftb = lin_srgb(ftb, ftb, ftb);
    runtime
        .led_outputs
        .par_iter_mut()
        .zip(runtime.led_colors.par_iter())
        .enumerate()
        .for_each(|(i, (output, &colour))| {
            let new = colour * l_ftb;
            *output = match prev_output.get(i) {
                None => new,
                Some(prev) => prev.lerp(&new, lerp_amt),
            };
        });

    if clear_transition {
        runtime.preset_transition = None;
    }

    update_led_worker_dmx(state, runtime);
}

fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn preset_uniforms(state: &LedWorkerInputState, preset: &conf::Preset) -> Uniforms {
    let led_layout = &state.config.led_layout;

    /*
    when t is -1, volumes[0] = 0, volumes[1] = 1
    when t = 0, volumes[0] = 0.707, volumes[1] = 0.707 (equal-power cross fade)
    when t = 1, volumes[0] = 1, volumes[1] = 0
    // Equal power xfade taken from https://dsp.stackexchange.com/questions/14754/equal-power-crossfade
    */
    let lr_mix = preset.left_right_mix;
    let xfade_left = (0.5 * (1.0 + lr_mix)).sqrt();
    let xfade_right = (0.5 * (1.0 - lr_mix)).sqrt();
    let env = state.audio_envelope;

    let mut params_left = preset.shader_params_left;
    {
        let mut mod_ix = 0;
        gui::apply_shader_modulation(
            preset.shader_left,
            &mut params_left,
            &mut mod_ix,
            &preset.shader_mod_amounts_left,
            env,
        );
    }
    let mut params_colourise = preset.shader_params_colourise;
    {
        let mut mod_ix = 0;
        gui::apply_shader_modulation(
            preset.colourise,
            &mut params_colourise,
            &mut mod_ix,
            &preset.shader_mod_amounts_colourise,
            env,
        );
    }
    let mut params_right = preset.shader_params_right;
    {
        let mut mod_ix = 0;
        gui::apply_shader_modulation(
            preset.shader_right,
            &mut params_right,
            &mut mod_ix,
            &preset.shader_mod_amounts_right,
            env,
        );
    }

    let mix_info = MixingInfo {
        left: preset.shader_left,
        right: preset.shader_right,
        colourise: preset.colourise,
        blend_mode: preset.blend_mode,
        xfade_left,
        xfade_right,
        params_left,
        params_right,
        params_colourise,
    };

    let buttons = state
        .buttons
        .iter()
        .map(|(&button, button_state)| {
            let secs = button_state.last_pressed.elapsed().secs() as f32;
            let state = shader_shared::ButtonState {
                secs,
                state: button_state.state,
            };
            (button, state)
        })
        .collect();
    let time = state.app_time + state.snapshot_at.elapsed().as_secs_f32();
    Uniforms {
        time,
        resolution: layout::shader_resolution(led_layout),
        pot6: state.colour_channels[0],
        pot7: state.colour_channels[1],
        pot8: state.colour_channels[2],
        params: ShaderParams::default(),
        mix: mix_info,
        buttons,
    }
}

fn render_preset_graph(
    shader: ShaderFnPtr,
    led_shader_inputs: &[CachedLedShaderInput],
    uniforms: &Uniforms,
    previous_led_colors: &[LinSrgb],
    output: &mut [LinSrgb],
) {
    output
        .par_iter_mut()
        .zip(led_shader_inputs.par_iter())
        .zip(previous_led_colors.par_iter())
        .for_each(|((color, led_input), &last_color)| {
            let vertex = Vertex {
                position: led_input.position,
                light: led_input.light,
                last_color,
            };
            *color = shader(vertex, uniforms);
        });
}

fn update_led_worker_dmx(state: &LedWorkerInputState, runtime: &mut LedWorkerRuntime) {
    if state.config.dmx_on {
        if let Ok(desired_interface_ip) =
            conf::parse_sacn_interface_ip(&state.config.sacn_interface_ip)
        {
            let should_refresh_source = runtime.dmx.source.is_none()
                || runtime.dmx.requested_interface_ip != desired_interface_ip;
            if should_refresh_source {
                match create_dmx_source(desired_interface_ip) {
                    Ok(source) => {
                        runtime.dmx.source = Some(source);
                        runtime.dmx.requested_interface_ip = desired_interface_ip;
                        runtime.dmx.error = None;
                        runtime.dmx.last_send_route = None;
                        runtime.dmx.last_send_attempt_at = None;
                    }
                    Err(err) => {
                        runtime.dmx.requested_interface_ip = desired_interface_ip;
                        let error = match desired_interface_ip {
                            Some(ip) => format!("Couldn't bind sACN to {}: {}", ip, err),
                            None => format!("Couldn't auto-bind sACN: {}", err),
                        };
                        runtime.dmx.monitor.record_send_error(error.clone());
                        runtime.dmx.error = Some(error);
                        runtime.dmx.last_send_route = None;
                        runtime.dmx.last_send_attempt_at = None;
                    }
                }
            }
        } else {
            runtime.dmx.requested_interface_ip = None;
            runtime.dmx.error = None;
            runtime.dmx.last_send_route = None;
            runtime.dmx.last_send_attempt_at = None;
        }
    } else if runtime.dmx.source.is_some() {
        runtime.dmx.source.take();
        runtime.dmx.requested_interface_ip = None;
        runtime.dmx.error = None;
        runtime.dmx.last_send_route = None;
        runtime.dmx.last_send_attempt_at = None;
    }

    let now = Instant::now();
    let should_send_output = should_send_led_output(
        state.config.led_output_fps,
        runtime.dmx.last_send_attempt_at,
        now,
    );
    let mut disconnect_source = false;
    if should_send_output {
        runtime.dmx.last_send_attempt_at = Some(now);
    }
    if should_send_output {
        if let Some(ref mut dmx_source) = runtime.dmx.source {
            let dmx_map = state.config.resolved_layout.as_ref().map(|rl| &rl.dmx_map);
            let payloads = build_sacn_payloads(
                dmx_map,
                state.config.led_start_universe,
                &runtime.led_outputs,
            );
            let mut sent_packet_count = 0usize;
            let mut sent_payload_bytes = 0usize;
            let mut sent_payloads = if state.capture_output_monitor {
                Some(Vec::with_capacity(payloads.len()))
            } else {
                None
            };
            let mut last_send_route = None;
            let mut send_error = None;

            for (universe, payload) in payloads {
                match dmx_source.send(universe, &payload) {
                    Ok(route) => {
                        last_send_route = Some(route);
                        sent_packet_count += 1;
                        sent_payload_bytes += payload.len();
                        if let Some(ref mut snapshots) = sent_payloads {
                            snapshots.push((universe, payload));
                        }
                    }
                    Err(error) => {
                        send_error = Some(error);
                        disconnect_source = true;
                        break;
                    }
                }
            }

            if sent_packet_count > 0 {
                let sent_at = Instant::now();
                runtime.dmx.monitor.record_successful_frame_stats(
                    sent_packet_count,
                    sent_payload_bytes,
                    sent_at,
                );
                if let Some(ref snapshots) = sent_payloads {
                    runtime
                        .dmx
                        .monitor
                        .record_universe_snapshots(snapshots, sent_at);
                }
            }

            if let Some(error) = send_error {
                runtime.dmx.monitor.record_send_error(error.clone());
                runtime.dmx.error = Some(error);
                runtime.dmx.last_send_route = None;
            } else if sent_packet_count > 0 {
                runtime.dmx.error = None;
                runtime.dmx.last_send_route = last_send_route;
            }
        }
    }

    if disconnect_source {
        runtime.dmx.source.take();
        runtime.dmx.last_send_route = None;
        runtime.dmx.last_send_attempt_at = None;
    }
}

fn sync_preview_window_visibility(app: &App, model: &mut Model) {
    if model.preview_window_visible == model.global_config.preview_window_on {
        return;
    }

    if let Some(window) = app.window(model.led_strip_window) {
        window.set_visible(model.global_config.preview_window_on);
    }
    model.preview_window_visible = model.global_config.preview_window_on;
}

fn raw_window_event(app: &App, model: &mut Model, event: &ui::RawWindowEvent) {
    model.ui.handle_raw_event(app, event);
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    if key == Key::Space {
        let button = shader_shared::Button::Cycle;
        update_button(&mut model.buttons, button, shader_shared::State::On);
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    model.audio_input.update();
    apply_led_worker_output(model);
    update_preview_textures(app, model);
    model.runtime_stats.record_app_frame(update.since_last);

    // Apply the GUI update.
    let mut ui = model.ui.set_widgets();
    let assets = app.assets_path().expect("failed to find assets directory");
    gui::update(
        &mut ui,
        gui::UpdateContext {
            global_config: &mut model.global_config,
            presets: &mut model.presets,
            audio_input: &mut model.audio_input,
            left_panel_tab: &mut model.left_panel_tab,
            sacn_output_monitor: &mut model.dmx.monitor,
            sacn_error: model.dmx.error.as_deref(),
            sacn_transport_label: model.dmx.last_send_route.map(dmx_send_route_label),
            since_start: update.since_start,
            shader_activity: model.shader_rx.activity(),
            led_colors: model.led_colors.as_slice(),
            last_preset_change: &mut model.last_preset_change,
            assets: assets.as_path(),
            ids: &mut model.ids,
            mad_project: &mut model.mad_project,
            resolved_layout: &mut model.resolved_layout,
            pending_file_dialog: &mut model.pending_file_dialog,
            midi_mapping: &mut model.midi_mapping,
            midi_learn: &mut model.midi_learn,
            midi_values: &mut model.midi_values,
            preview_left_image_id: model.preview_images.as_ref().map(|pi| pi.left_id),
            preview_right_image_id: model.preview_images.as_ref().map(|pi| pi.right_id),
            preview_colourise_image_id: model.preview_images.as_ref().map(|pi| pi.colourise_id),
        },
    );
    drop(ui);

    // Poll for async file dialog result.
    if let Some(ref rx) = model.pending_file_dialog {
        if let Ok(result) = rx.try_recv() {
            if let Some(path) = result {
                match mad_mapper::parse(&path) {
                    Ok(project) => {
                        eprintln!(
                            "Loaded MadMapper project: {} fixtures, {} pixels",
                            project.fixtures.len(),
                            project.total_pixels()
                        );
                        model.global_config.madmapper_project_path =
                            Some(path.to_string_lossy().into_owned());
                        model.resolved_layout = Some(layout::resolve_from_mad_project(&project));
                        model.mad_project = Some(project);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse MadMapper project: {}", e);
                    }
                }
            }
            model.pending_file_dialog = None;
        }
    }

    sync_preview_window_visibility(app, model);
    update_gui_window_title(app, model);
    model.global_config.led_layout.normalise();
    sync_led_buffers(model);

    // Check for an update to the shader.
    if let Some(shader) = model.shader_rx.update() {
        if let Ok(mut shared_input) = model.led_worker.shared_input.lock() {
            shared_input.pending_shader = Some(shader);
        }
    }

    // MIDI routing: CC → mapping lookup → target values → smoothing → apply.
    model.midi_manager.poll();
    for msg in model.midi_manager.drain().collect::<Vec<_>>() {
        // Check learn mode first.
        if model.midi_learn.is_listening() {
            if let Some((port, cc, target)) = model.midi_learn.receive(&msg) {
                model.midi_mapping.assign(port, cc, target);
            }
            continue;
        }
        // Normal routing.
        if let Some(target) = model.midi_mapping.target_for(&msg.port_name, msg.cc) {
            let normalized = 1.0 - (msg.value as f32 / 127.0);
            let state = model.midi_values.entry(target).or_insert(MidiTargetState {
                target: normalized,
                smoothed: normalized,
                active: false,
            });
            state.target = normalized;
            state.active = true;
        }
    }
    model.midi_learn.update();

    // Smooth active MIDI targets.
    let s = model.smoothing_speed;
    for (_target, state) in &mut model.midi_values {
        if state.active {
            state.smoothed = state.smoothed * (1.0 - s) + state.target * s;
        }
    }

    // Apply smoothed MIDI values to their destinations.
    apply_midi_values(model);

    queue_led_worker_update(app, model);
}

fn should_send_led_output(
    output_fps_mode: conf::LedOutputFps,
    last_send_attempt_at: Option<Instant>,
    now: Instant,
) -> bool {
    match output_fps_mode.fps_limit() {
        None => true,
        Some(fps_limit) => {
            let min_interval = Duration::from_secs_f64(1.0 / fps_limit as f64);
            last_send_attempt_at
                .map(|last_send_at| now.duration_since(last_send_at) >= min_interval)
                .unwrap_or(true)
        }
    }
}

fn create_dmx_source(interface_ip: Option<Ipv4Addr>) -> Result<DmxOutputTransport, String> {
    match interface_ip {
        Some(ip) if ip.is_loopback() => sacn_sender::LocalhostSacnSender::new("Cohen Pre-vis")
            .map(DmxOutputTransport::Localhost)
            .map_err(|err| format!("Couldn't create localhost sACN sender: {}", err)),
        Some(ip) => create_multicast_dmx_source(ip)
            .map(DmxOutputTransport::Network)
            .map_err(|err| err.to_string()),
        None => {
            let localhost = sacn_sender::LocalhostSacnSender::new("Cohen Pre-vis")
                .map_err(|err| format!("Couldn't create localhost sACN sender: {}", err))?;
            let multicast = create_multicast_dmx_source(Ipv4Addr::UNSPECIFIED).ok();
            Ok(DmxOutputTransport::Auto {
                multicast,
                localhost,
            })
        }
    }
}

fn create_multicast_dmx_source(interface_ip: Ipv4Addr) -> sacn::error::errors::Result<SacnSource> {
    let bind_addr = SocketAddr::new(IpAddr::V4(interface_ip), ACN_SDT_MULTICAST_PORT + 1);
    let mut source = SacnSource::with_ip("Cohen Pre-vis", bind_addr)?;
    // Preserve the old sender behaviour: data only, no source discovery chatter.
    source.set_is_sending_discovery(false);
    Ok(source)
}

fn send_multicast_payload(
    source: &mut SacnSource,
    universe: u16,
    payload: &[u8],
) -> Result<DmxSendRoute, String> {
    source
        .register_universe(universe)
        .map_err(|err| format!("Couldn't register sACN universe {}: {}", universe, err))?;
    source
        .send(
            &[universe],
            payload,
            Some(E131_DEFAULT_PRIORITY),
            None,
            None,
        )
        .map(|()| DmxSendRoute::Multicast)
        .map_err(|err| format!("Couldn't send sACN universe {}: {}", universe, err))
}

fn dmx_send_route_label(route: DmxSendRoute) -> &'static str {
    match route {
        DmxSendRoute::Multicast => "Network multicast",
        DmxSendRoute::Localhost => "Localhost preview",
    }
}

fn update_gui_window_title(app: &App, model: &Model) {
    let title = if model.runtime_stats.app_fps > 0.0 {
        format!("COHEN GIG - GUI - {:.1} FPS", model.runtime_stats.app_fps)
    } else {
        "COHEN GIG - GUI".to_string()
    };
    if let Some(window) = app.window(model._gui_window) {
        window.set_title(&title);
    }
}

fn fps_from_duration(duration: Duration) -> Option<f32> {
    let secs = duration.as_secs_f32();
    if secs <= 0.0 {
        return None;
    }
    let fps = 1.0 / secs;
    fps.is_finite().then_some(fps)
}

fn smooth_fps(current: f32, sample: f32) -> f32 {
    const FPS_SMOOTHING_FACTOR: f32 = 0.2;
    if current > 0.0 {
        current + (sample - current) * FPS_SMOOTHING_FACTOR
    } else {
        sample
    }
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

    if !model.global_config.preview_window_on {
        draw.to_frame(app, &frame).unwrap();
        return;
    }

    let w = app.window(model.led_strip_window).unwrap().rect();

    if model.mad_project.is_some() {
        led_strip_view_mad(&draw, &w, model);
    } else {
        led_strip_view_manual(app, &draw, &w, model);
    }

    draw_hotload_feedback(app, model, &draw, w);
    draw.to_frame(app, &frame).unwrap();
}

/// Preview rendering when a MadMapper project is active.
/// Uses normalised coords from the resolved layout, centered in the window.
fn led_strip_view_mad(draw: &Draw, w: &geom::Rect, model: &Model) {
    let project = model.mad_project.as_ref().unwrap();
    let fixtures = project.fixtures_by_row();
    let scale = w.h() * 0.4;

    let mut led_idx = 0usize;
    for fixture in &fixtures {
        let row_count = fixtures.len().max(1);
        let fixture_idx = fixtures
            .iter()
            .position(|f| std::ptr::eq(*f, *fixture))
            .unwrap_or(0);
        let y_norm = if row_count <= 1 {
            0.0
        } else {
            1.0 - (fixture_idx as f32 / (row_count - 1) as f32) * 2.0
        };
        let y_px = y_norm * scale;

        let x_scale = w.w() * 0.45;
        let vs = (0..fixture.pixel_count).filter_map(|pixel_ix| {
            let color = model.led_outputs.get(led_idx + pixel_ix)?;
            let x_norm = if fixture.pixel_count <= 1 {
                0.0
            } else {
                (pixel_ix as f32 / (fixture.pixel_count - 1) as f32) * 2.0 - 1.0
            };
            Some((pt2(x_norm * x_scale, y_px), *color))
        });
        draw.polyline().weight(5.0).points_colored(vs);
        led_idx += fixture.pixel_count;
    }
}

/// Preview rendering for manual LED layout (original path).
fn led_strip_view_manual(app: &App, draw: &Draw, w: &geom::Rect, model: &Model) {
    let led_layout = &model.global_config.led_layout;

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

fn save_global_config(assets: &Path, global_config: &GlobalConfig) {
    let path = conf::config_path(assets);
    save_to_json(path, global_config).expect("failed to save global config");
}

fn save_presets(assets: &Path, presets: &conf::Presets) {
    let path = conf::presets_path(assets);
    save_to_json(path, presets).expect("failed to save presets");
}

fn exit(_app: &App, mut model: Model) {
    if let Ok(mut shared_input) = model.led_worker.shared_input.lock() {
        shared_input.shutdown = true;
    }
    if let Some(thread) = model.led_worker.thread.take() {
        let _ = thread.join();
    }
}

fn update_button(
    buttons: &mut HashMap<shader_shared::Button, ButtonState>,
    button: shader_shared::Button,
    state: shader_shared::State,
) {
    let now = std::time::Instant::now();
    let b_state = buttons.entry(button).or_insert_with(|| ButtonState {
        last_pressed: now,
        state,
    });
    b_state.state = state;
    if state == shader_shared::State::On {
        b_state.last_pressed = now;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_led_sacn_payloads, build_per_fixture_payloads, should_send_led_output,
        UNIVERSE_CHANNEL_CAPACITY,
    };
    use crate::conf::LedOutputFps;
    use crate::layout::FixtureDmxEntry;
    use nannou::prelude::*;
    use std::time::{Duration, Instant};

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

    #[test]
    fn free_output_mode_never_throttles() {
        let now = Instant::now();

        assert!(should_send_led_output(LedOutputFps::Free, None, now));
        assert!(should_send_led_output(LedOutputFps::Free, Some(now), now));
    }

    #[test]
    fn capped_output_mode_waits_for_the_selected_interval() {
        let now = Instant::now();
        let too_soon = now.checked_sub(Duration::from_millis(10)).unwrap();
        let ready = now.checked_sub(Duration::from_millis(13)).unwrap();

        assert!(!should_send_led_output(
            LedOutputFps::Fps80,
            Some(too_soon),
            now
        ));
        assert!(should_send_led_output(
            LedOutputFps::Fps80,
            Some(ready),
            now
        ));
    }

    #[test]
    fn per_fixture_payloads_route_pixels_to_correct_universes() {
        // Two fixtures: 4 pixels on universe 5, 3 pixels on universe 10.
        let entries = vec![
            FixtureDmxEntry {
                led_offset: 0,
                led_count: 4,
                start_universe: 5,
                start_channel: 1,
                channels_per_pixel: 3,
            },
            FixtureDmxEntry {
                led_offset: 4,
                led_count: 3,
                start_universe: 10,
                start_channel: 1,
                channels_per_pixel: 3,
            },
        ];

        let led_outputs: Vec<LinSrgb> = (0..7)
            .map(|i| {
                let v = (i + 1) as f32 / 7.0;
                lin_srgb(v, v, v)
            })
            .collect();

        let payloads = build_per_fixture_payloads(&entries, &led_outputs);

        assert_eq!(payloads.len(), 2);
        assert_eq!(payloads[0].0, 5);
        assert_eq!(payloads[1].0, 10);

        // First fixture: 4 pixels * 3 channels = 12 bytes + 1 start code = 13.
        assert_eq!(payloads[0].1.len(), 13);
        assert_eq!(payloads[0].1[0], 0); // DMX start code

        // Second fixture: 3 pixels * 3 channels = 9 bytes + 1 start code = 10.
        assert_eq!(payloads[1].1.len(), 10);
        assert_eq!(payloads[1].1[0], 0);
    }
}
