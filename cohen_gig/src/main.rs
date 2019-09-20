use nannou::prelude::*;
use nannou::Ui;
use shader_shared::Uniforms;

mod arch;
mod gui;
mod strip;
mod wash;

const WINDOW_PAD: i32 = 20;
const GUI_WINDOW_X: i32 = WINDOW_PAD;
const GUI_WINDOW_Y: i32 = WINDOW_PAD;
const VIS_WINDOW_X: i32 = GUI_WINDOW_X + gui::WINDOW_WIDTH as i32 + WINDOW_PAD;
const VIS_WINDOW_Y: i32 = GUI_WINDOW_Y;
const VIS_WINDOW_W: u32 = 1024;
const VIS_WINDOW_H: u32 = 720;

pub const FAR_Z: f32 = 0.0;
pub const CLOSE_Z: f32 = 1.0;
pub const LEFT_X: f32 = -1.0;
pub const RIGHT_X: f32 = 1.0;
pub const FLOOR_Y: f32 = -1.0;
pub const ROOF_Y: f32 = 1.0;

pub const LED_PPM: f32 = 60.0;

struct Model {
    gui_window: window::Id,
    vis_window: window::Id,
    dmx_source: Option<sacn::DmxSource>,
    ui: Ui,
    ids: gui::Ids,
    shader_watch: hotlib::Watch,
    shader: Shader,
}

type Universe = u16;
type Address = u16;

struct Shader {
    lib: libloading::Library,
}

struct LedStrip {
    start: (Universe, Address),
    end: (Universe, Address),
}

impl Shader {
    /// Load the shader function.
    pub fn get_fn(&self) -> libloading::Symbol<fn(Vector3, &Uniforms) -> LinSrgb> {
        unsafe {
            self.lib.get("shader".as_bytes()).expect("failed to load shader fn symbol")
        }
    }
}

impl From<libloading::Library> for Shader {
    fn from(lib: libloading::Library) -> Self {
        Shader { lib }
    }
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let gui_window = app
        .new_window()
        .with_title("COHEN GIG GUI")
        .with_dimensions(gui::WINDOW_WIDTH, gui::WINDOW_HEIGHT)
        .view(gui_view)
        .build()
        .expect("failed to build GUI window");

    let vis_window = app
        .new_window()
        .with_title("COHEN GIG PREVIS")
        .with_dimensions(VIS_WINDOW_W, VIS_WINDOW_H)
        .view(vis_view)
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
    }

    let dmx_source = None;
    let shader_watch = hotlib::watch(&shader_toml_path()).expect("failed to start watching shader");
    let shader_lib = shader_watch.build().expect("initial shader lib build failed");
    let shader = Shader::from(shader_lib);

    Model {
        gui_window,
        vis_window,
        dmx_source,
        ui,
        ids,
        shader_watch,
        shader,
    }
}

fn shader_toml_path() -> std::path::PathBuf {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = path.parent().expect("could not find workspace dir");
    workspace_dir.join("shader").join("Cargo").with_extension("toml")
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let ui = model.ui.set_widgets();
    gui::update(ui, &model.ids);

    // Check for an update to the shader.
    match model.shader_watch.try_next() {
        Err(err) => eprintln!("an error occurred watching the shader lib: {}", err),
        Ok(None) => (),
        Ok(Some(lib)) => model.shader = Shader::from(lib),
    };

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
        let shader = model.shader.get_fn();
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
    let vis_y_offset = w.h() * -0.2;
    let front_arch_scale = w.right().min(w.top()) * 4.0 / 7.0;
    let perspective_scale = 0.66;
    let total_dist = (arch::COUNT - 1) as f32 * arch::Z_GAP * front_arch_scale;
    let shader = model.shader.get_fn();

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
