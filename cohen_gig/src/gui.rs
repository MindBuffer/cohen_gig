use crate::conf::Config;
use crate::shader;
use nannou::prelude::*;

use nannou_conrod as ui;
use nannou_conrod::prelude::*;
use nannou_conrod::Color;

use shader_shared::{BlendMode, Shader, ShaderParams};
use std::f64::consts::PI;
use std::path::Path;

pub const NUM_COLUMNS: u32 = 4;
pub const COLUMN_W: Scalar = 240.0;
pub const DEFAULT_WIDGET_H: Scalar = 30.0;
pub const DEFAULT_SLIDER_H: Scalar = 20.0;
pub const TEXT_BOX_H: Scalar = DEFAULT_WIDGET_H / 1.5;
pub const PAD: Scalar = 20.0;
pub const COLUMN_ONE_SECTION_GAP: Scalar = 6.0;
pub const WINDOW_WIDTH: u32 =
    (COLUMN_W as u32 * NUM_COLUMNS) + (PAD * 2.0 + PAD * (NUM_COLUMNS - 1) as Scalar) as u32;
pub const WINDOW_HEIGHT: u32 = 1050 - (2.0 * PAD) as u32;
pub const WIDGET_W: Scalar = COLUMN_W;
pub const HALF_WIDGET_W: Scalar = WIDGET_W * 0.5 - PAD * 0.25;
pub const BUTTON_COLOR: Color = Color::Rgba(0.11, 0.39, 0.4, 1.0); // teal
pub const TEXT_COLOR: Color = Color::Rgba(1.0, 1.0, 1.0, 1.0);
pub const PRESET_LIST_COLOR: Color = Color::Rgba(0.16, 0.32, 0.6, 1.0); // blue
pub const PRESET_LIST_SELECTED_COLOR: Color = Color::Rgba(0.28, 0.54, 1.0, 1.0); // light blue
pub const PRESET_ENTRY_COLOR: Color = Color::Rgba(0.05, 0.1, 0.2, 1.0); // dark blue

widget_ids! {
    pub struct Ids {
        background,
        column_1_id,
        column_2_id,
        column_3_id,
        column_4_id,

        scrollbar,
        title_text,
        live_tab_button,
        output_tab_button,
        dmx_button,
        midi_button,
        output_fps_text,
        output_fps_ddl,
        output_fps_status_text,
        audio_device_ddl,
        audio_device_placeholder,
        audio_device_error_text,
        sacn_interface_ip_text,
        sacn_interface_ip_help_text,
        sacn_interface_ip_text_box,
        sacn_interface_ip_error_text,
        shader_title_text,
        shader_state_text,

        presets_text,
        presets_lerp_slider,
        presets_duplicate,
        presets_new_button,
        presets_save_button,
        presets_delete_button,
        presets_text_box,
        presets_list,
        enter_preset_name_text,

        universe_starts_text,
        led_start_universe_dialer,
        led_layout_text,
        led_pixels_per_metre_dialer,
        led_metres_per_row_dialer,
        led_row_count_dialer,
        led_layout_stats_text,

        led_shader_left_text,
        led_shader_left_ddl,

        led_shader_right_text,
        led_shader_right_ddl,

        shader_mod_sliders[],
        shader_int_sliders[],
        shader_buttons[],

        colour_post_process_text,
        colour_post_process_ddl,

        blend_mode_text,
        blend_mode_ddl,

        shader_mix_left_right,
        led_fade_to_black,

        audio_input_text,
        audio_scope_bg,
        audio_scope,
        audio_scope_neg,
        audio_scope_midline,
        audio_threshold_line,
        audio_threshold_line_neg,
        audio_gain_slider,
        audio_threshold_slider,
        audio_attack_slider,
        audio_hold_slider,
        audio_release_slider,
        audio_envelope_scope_bg,
        audio_envelope_scope,

        sacn_output_title_text,
        sacn_output_status_text,
        sacn_output_universe_text,
        sacn_output_universe_ddl,
        sacn_output_universe_placeholder,
        sacn_output_grid_help_text,
        sacn_output_grid_bg,
        sacn_output_grid_cells[],
        sacn_output_grid_cell_values[],
        sacn_output_grid_summary_text,
        sacn_output_slot_preview_text,

    }
}

type LedColors = [LinSrgb];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeftPanelTab {
    Live,
    Output,
}

pub struct UpdateContext<'a> {
    pub config: &'a mut Config,
    pub audio_input: &'a mut crate::audio_input::AudioInput,
    pub left_panel_tab: &'a mut LeftPanelTab,
    pub sacn_output_monitor: &'a mut crate::SacnOutputMonitor,
    pub sacn_error: Option<&'a str>,
    pub sacn_transport_label: Option<&'a str>,
    pub since_start: std::time::Duration,
    pub shader_activity: shader::Activity<'a>,
    pub led_colors: &'a LedColors,
    pub last_preset_change: &'a mut Option<crate::LastPresetChange>,
    pub assets: &'a Path,
    pub ids: &'a mut Ids,
}

/// Implemented for all sets of shader parameters to allow for generic GUI layout.
trait Params {
    /// The total number of parameters.
    fn param_count(&self) -> usize;
    /// The parameter at the given index.
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_>;
}

struct ParamMut<'a> {
    name: &'static str,
    kind: ParamKindMut<'a>,
}

enum ParamKindMut<'a> {
    F32 { value: &'a mut f32, max: f32 },
    Bool(&'a mut bool),
    Usize { value: &'a mut usize, max: usize },
}

struct ShaderWidgetState<'a> {
    mod_slider_ix: &'a mut usize,
    int_slider_ix: &'a mut usize,
    button_ix: &'a mut usize,
    mod_amounts: &'a mut Vec<f32>,
    envelope: f32,
}

impl Params for shader_shared::AcidGradient {
    fn param_count(&self) -> usize {
        3
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "zoom",
                kind: ParamKindMut::F32 {
                    value: &mut self.zoom,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "offset",
                kind: ParamKindMut::F32 {
                    value: &mut self.offset,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::BlinkyCircles {
    fn param_count(&self) -> usize {
        3
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "zoom",
                kind: ParamKindMut::F32 {
                    value: &mut self.zoom,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "offset",
                kind: ParamKindMut::F32 {
                    value: &mut self.offset,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::BwGradient {
    fn param_count(&self) -> usize {
        5
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "dc",
                kind: ParamKindMut::F32 {
                    value: &mut self.dc,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "amp",
                kind: ParamKindMut::F32 {
                    value: &mut self.amp,
                    max: 1.0,
                },
            },
            3 => ParamMut {
                name: "freq",
                kind: ParamKindMut::F32 {
                    value: &mut self.freq,
                    max: 1.0,
                },
            },
            4 => ParamMut {
                name: "mirror",
                kind: ParamKindMut::Bool(&mut self.mirror),
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::ColourGrid {
    fn param_count(&self) -> usize {
        2
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "zoom_amount",
                kind: ParamKindMut::F32 {
                    value: &mut self.zoom_amount,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::EscherTilings {
    fn param_count(&self) -> usize {
        3
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "scale",
                kind: ParamKindMut::F32 {
                    value: &mut self.scale,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "shape_iter",
                kind: ParamKindMut::F32 {
                    value: &mut self.shape_iter,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::GilmoreAcid {
    fn param_count(&self) -> usize {
        9
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "displace",
                kind: ParamKindMut::F32 {
                    value: &mut self.displace,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "colour_offset",
                kind: ParamKindMut::F32 {
                    value: &mut self.colour_offset,
                    max: 1.0,
                },
            },
            3 => ParamMut {
                name: "grid_size",
                kind: ParamKindMut::F32 {
                    value: &mut self.grid_size,
                    max: 1.0,
                },
            },
            4 => ParamMut {
                name: "wave",
                kind: ParamKindMut::F32 {
                    value: &mut self.wave,
                    max: 1.0,
                },
            },
            5 => ParamMut {
                name: "zoom_amount",
                kind: ParamKindMut::F32 {
                    value: &mut self.zoom_amount,
                    max: 1.0,
                },
            },
            6 => ParamMut {
                name: "rotation_amount",
                kind: ParamKindMut::F32 {
                    value: &mut self.rotation_amount,
                    max: 1.0,
                },
            },
            7 => ParamMut {
                name: "brightness",
                kind: ParamKindMut::F32 {
                    value: &mut self.brightness,
                    max: 1.0,
                },
            },
            8 => ParamMut {
                name: "saturation",
                kind: ParamKindMut::F32 {
                    value: &mut self.saturation,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::JustRelax {
    fn param_count(&self) -> usize {
        3
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "shape_offset",
                kind: ParamKindMut::F32 {
                    value: &mut self.shape_offset,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "iter",
                kind: ParamKindMut::F32 {
                    value: &mut self.iter,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::LifeLedWall {
    fn param_count(&self) -> usize {
        7
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "size",
                kind: ParamKindMut::F32 {
                    value: &mut self.size,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "red",
                kind: ParamKindMut::F32 {
                    value: &mut self.red,
                    max: 1.0,
                },
            },
            3 => ParamMut {
                name: "green",
                kind: ParamKindMut::F32 {
                    value: &mut self.green,
                    max: 1.0,
                },
            },
            4 => ParamMut {
                name: "blue",
                kind: ParamKindMut::F32 {
                    value: &mut self.blue,
                    max: 1.0,
                },
            },
            5 => ParamMut {
                name: "saturation",
                kind: ParamKindMut::F32 {
                    value: &mut self.saturation,
                    max: 1.0,
                },
            },
            6 => ParamMut {
                name: "colour_offset",
                kind: ParamKindMut::F32 {
                    value: &mut self.colour_offset,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::LineGradient {
    fn param_count(&self) -> usize {
        5
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "num_stripes",
                kind: ParamKindMut::F32 {
                    value: &mut self.num_stripes,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "stripe_width",
                kind: ParamKindMut::F32 {
                    value: &mut self.stripe_width,
                    max: 1.0,
                },
            },
            3 => ParamMut {
                name: "angle",
                kind: ParamKindMut::F32 {
                    value: &mut self.angle,
                    max: 1.0,
                },
            },
            4 => ParamMut {
                name: "smooth_width",
                kind: ParamKindMut::F32 {
                    value: &mut self.smooth_width,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::Metafall {
    fn param_count(&self) -> usize {
        5
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "scale",
                kind: ParamKindMut::F32 {
                    value: &mut self.scale,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "red",
                kind: ParamKindMut::F32 {
                    value: &mut self.red,
                    max: 1.0,
                },
            },
            3 => ParamMut {
                name: "green",
                kind: ParamKindMut::F32 {
                    value: &mut self.green,
                    max: 1.0,
                },
            },
            4 => ParamMut {
                name: "blue",
                kind: ParamKindMut::F32 {
                    value: &mut self.blue,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::ParticleZoom {
    fn param_count(&self) -> usize {
        4
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "density",
                kind: ParamKindMut::F32 {
                    value: &mut self.density,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "shape",
                kind: ParamKindMut::F32 {
                    value: &mut self.shape,
                    max: 1.0,
                },
            },
            3 => ParamMut {
                name: "tau",
                kind: ParamKindMut::F32 {
                    value: &mut self.tau,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::RadialLines {
    fn param_count(&self) -> usize {
        2
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "zoom_amount",
                kind: ParamKindMut::F32 {
                    value: &mut self.zoom_amount,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::SatisSpiraling {
    fn param_count(&self) -> usize {
        4
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "loops",
                kind: ParamKindMut::F32 {
                    value: &mut self.loops,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "mirror",
                kind: ParamKindMut::Bool(&mut self.mirror),
            },
            3 => ParamMut {
                name: "rotate",
                kind: ParamKindMut::Bool(&mut self.rotate),
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::SpiralIntersect {
    fn param_count(&self) -> usize {
        6
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "g1",
                kind: ParamKindMut::F32 {
                    value: &mut self.g1,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "g2",
                kind: ParamKindMut::F32 {
                    value: &mut self.g2,
                    max: 1.0,
                },
            },
            3 => ParamMut {
                name: "rot1",
                kind: ParamKindMut::F32 {
                    value: &mut self.rot1,
                    max: 1.0,
                },
            },
            4 => ParamMut {
                name: "rot2",
                kind: ParamKindMut::F32 {
                    value: &mut self.rot2,
                    max: 1.0,
                },
            },
            5 => ParamMut {
                name: "colours",
                kind: ParamKindMut::F32 {
                    value: &mut self.colours,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::SquareTunnel {
    fn param_count(&self) -> usize {
        4
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "rotation_speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.rotation_speed,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "rotation_offset",
                kind: ParamKindMut::F32 {
                    value: &mut self.rotation_offset,
                    max: 1.0,
                },
            },
            3 => ParamMut {
                name: "zoom",
                kind: ParamKindMut::F32 {
                    value: &mut self.zoom,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::ThePulse {
    fn param_count(&self) -> usize {
        4
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "scale",
                kind: ParamKindMut::F32 {
                    value: &mut self.scale,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "colour_iter",
                kind: ParamKindMut::F32 {
                    value: &mut self.colour_iter,
                    max: 1.0,
                },
            },
            3 => ParamMut {
                name: "thickness",
                kind: ParamKindMut::F32 {
                    value: &mut self.thickness,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::TunnelProjection {
    fn param_count(&self) -> usize {
        2
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "res",
                kind: ParamKindMut::F32 {
                    value: &mut self.res,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::VertColourGradient {
    fn param_count(&self) -> usize {
        6
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "scale",
                kind: ParamKindMut::F32 {
                    value: &mut self.scale,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "colour_iter",
                kind: ParamKindMut::F32 {
                    value: &mut self.colour_iter,
                    max: 1.0,
                },
            },
            3 => ParamMut {
                name: "line_amp",
                kind: ParamKindMut::F32 {
                    value: &mut self.line_amp,
                    max: 1.0,
                },
            },
            4 => ParamMut {
                name: "diag_amp",
                kind: ParamKindMut::F32 {
                    value: &mut self.diag_amp,
                    max: 1.0,
                },
            },
            5 => ParamMut {
                name: "border_amp",
                kind: ParamKindMut::F32 {
                    value: &mut self.boarder_amp,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::MitchWash {
    fn param_count(&self) -> usize {
        2
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "pulse_speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.pulse_speed,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::ShapeEnvelopes {
    fn param_count(&self) -> usize {
        4
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "pulse_speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.pulse_speed,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "line_thickness",
                kind: ParamKindMut::F32 {
                    value: &mut self.line_thickness,
                    max: 1.0,
                },
            },
            3 => ParamMut {
                name: "shape_thickness",
                kind: ParamKindMut::F32 {
                    value: &mut self.shape_thickness,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::RowTest {
    fn param_count(&self) -> usize {
        1
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "row",
                kind: ParamKindMut::F32 {
                    value: &mut self.row,
                    max: 8.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::BarTest {
    fn param_count(&self) -> usize {
        1
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "row",
                kind: ParamKindMut::F32 {
                    value: &mut self.row,
                    max: 8.0,
                },
            },
            1 => ParamMut {
                name: "bar",
                kind: ParamKindMut::F32 {
                    value: &mut self.bar,
                    max: 8.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::SolidHsvColour {
    fn param_count(&self) -> usize {
        3
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "hue",
                kind: ParamKindMut::F32 {
                    value: &mut self.hue,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "saturation",
                kind: ParamKindMut::F32 {
                    value: &mut self.saturation,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "value",
                kind: ParamKindMut::F32 {
                    value: &mut self.value,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::SolidRgbColour {
    fn param_count(&self) -> usize {
        3
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "red",
                kind: ParamKindMut::F32 {
                    value: &mut self.red,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "green",
                kind: ParamKindMut::F32 {
                    value: &mut self.green,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "blue",
                kind: ParamKindMut::F32 {
                    value: &mut self.blue,
                    max: 1.0,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

impl Params for shader_shared::ColourPalettes {
    fn param_count(&self) -> usize {
        3
    }
    fn param_mut(&mut self, ix: usize) -> ParamMut<'_> {
        match ix {
            0 => ParamMut {
                name: "speed",
                kind: ParamKindMut::F32 {
                    value: &mut self.speed,
                    max: 1.0,
                },
            },
            1 => ParamMut {
                name: "interval",
                kind: ParamKindMut::F32 {
                    value: &mut self.interval,
                    max: 1.0,
                },
            },
            2 => ParamMut {
                name: "selected",
                kind: ParamKindMut::Usize {
                    value: &mut self.selected,
                    max: 16,
                },
            },
            _ => panic!("no parameter for index {}: check `param_count` impl", ix),
        }
    }
}

/// Update the user interface.
pub fn update(ui: &mut UiCell, ctx: UpdateContext<'_>) {
    let UpdateContext {
        config,
        audio_input,
        left_panel_tab,
        sacn_output_monitor,
        sacn_error,
        sacn_transport_label,
        since_start,
        shader_activity,
        led_colors,
        last_preset_change,
        assets,
        ids,
    } = ctx;

    widget::Canvas::new()
        .pad(PAD)
        .border(0.0)
        .rgb(0.1, 0.1, 0.1)
        .set(ids.background, ui);

    // Column canvasses.

    column_canvas(ids.background)
        .top_left_of(ids.background)
        .set(ids.column_1_id, ui);

    column_canvas(ids.background)
        .align_top_of(ids.column_1_id)
        .right_from(ids.column_1_id, PAD)
        .set(ids.column_2_id, ui);

    column_canvas(ids.background)
        .align_top_of(ids.column_1_id)
        .right_from(ids.column_2_id, PAD)
        .set(ids.column_3_id, ui);

    column_canvas(ids.background)
        .align_top_of(ids.column_1_id)
        .right_from(ids.column_3_id, PAD)
        .set(ids.column_4_id, ui);

    text("COHEN GIG")
        .mid_top_of(ids.column_1_id)
        .set(ids.title_text, ui);

    if button()
        .color(tab_button_color(*left_panel_tab == LeftPanelTab::Live))
        .label("Live")
        .w(HALF_WIDGET_W)
        .mid_left_of(ids.column_1_id)
        .down(PAD * 1.5)
        .set(ids.live_tab_button, ui)
        .was_clicked()
    {
        *left_panel_tab = LeftPanelTab::Live;
    }

    if button()
        .color(tab_button_color(*left_panel_tab == LeftPanelTab::Output))
        .label("Output")
        .right(PAD * 0.5)
        .w(HALF_WIDGET_W)
        .set(ids.output_tab_button, ui)
        .was_clicked()
    {
        *left_panel_tab = LeftPanelTab::Output;
    }

    match *left_panel_tab {
        LeftPanelTab::Live => {
            let audio_anchor =
                set_live_sidebar_widgets(ui, ids, config, since_start, shader_activity);
            crate::audio_widgets::set_widgets(
                ui,
                ids,
                audio_input,
                &mut config.audio_input_device,
                audio_anchor,
            );
            set_presets_widgets(ui, ids, config, last_preset_change, led_colors, assets);
        }
        LeftPanelTab::Output => {
            set_output_sidebar_widgets(ui, ids, config, sacn_error, sacn_output_monitor);
            set_output_monitor_widgets(
                ui,
                ids,
                config,
                sacn_output_monitor,
                sacn_error,
                sacn_transport_label,
            );
        }
    }

    // Now that preset selection is done, get easier access to the selected preset.
    let preset = config.presets.selected_mut();

    //---------------------- LED SHADER LEFT

    text("LED Shader Left")
        .top_left_of(ids.column_3_id)
        .color(color::WHITE)
        .set(ids.led_shader_left_text, ui);

    let shader_names: Vec<_> = shader_shared::ALL_SHADERS
        .iter()
        .map(|s| s.name())
        .collect();
    let shader_idx = preset.shader_left.to_index();

    if let Some(selected_idx) = widget::DropDownList::new(&shader_names, Some(shader_idx))
        .w_h(COLUMN_W, PAD * 2.0)
        .down(10.0)
        .max_visible_items(15)
        .rgb(0.176, 0.513, 0.639)
        .label("LED Shader Preset")
        .label_font_size(15)
        .label_rgb(1.0, 1.0, 1.0)
        .scrollbar_on_top()
        .set(ids.led_shader_left_ddl, ui)
    {
        preset.shader_left = Shader::from_index(selected_idx).unwrap();
    }

    let mut mod_slider_ix = 0;
    let mut int_slider_ix = 0;
    let mut button_ix = 0;

    {
        let params = shader_params(preset.shader_left, &mut preset.shader_params);
        set_shader_widgets(
            ui,
            ids,
            params,
            ShaderWidgetState {
                mod_slider_ix: &mut mod_slider_ix,
                int_slider_ix: &mut int_slider_ix,
                button_ix: &mut button_ix,
                mod_amounts: &mut preset.shader_mod_amounts,
                envelope: audio_input.envelope,
            },
        );
    }

    //---------------------- COLOUR POST PROCESS SHADER
    text("Colour Post Process")
        .down(20.0)
        .color(color::WHITE)
        .set(ids.colour_post_process_text, ui);

    let colour_names: Vec<_> = shader_shared::SOLID_COLOUR_SHADERS
        .iter()
        .map(|s| s.name())
        .collect();
    let colourise_idx = preset.colourise.to_index();

    if let Some(selected_idx) = widget::DropDownList::new(&colour_names, Some(colourise_idx))
        .w_h(COLUMN_W, PAD * 2.0)
        .down(10.0)
        .max_visible_items(15)
        .rgb(0.176, 0.513, 0.639)
        .label("Wash Shader Preset")
        .label_font_size(15)
        .label_rgb(1.0, 1.0, 1.0)
        .scrollbar_on_top()
        .set(ids.colour_post_process_ddl, ui)
    {
        preset.colourise = Shader::from_index(selected_idx).unwrap();
    }

    {
        let params = shader_params(preset.colourise, &mut preset.shader_params);
        set_shader_widgets(
            ui,
            ids,
            params,
            ShaderWidgetState {
                mod_slider_ix: &mut mod_slider_ix,
                int_slider_ix: &mut int_slider_ix,
                button_ix: &mut button_ix,
                mod_amounts: &mut preset.shader_mod_amounts,
                envelope: audio_input.envelope,
            },
        );
    }

    //---------------------- LED SHADER RIGHT

    text("LED Shader Right")
        .top_left_of(ids.column_4_id)
        .color(color::WHITE)
        .set(ids.led_shader_right_text, ui);

    let shader_idx = preset.shader_right.to_index();
    if let Some(selected_idx) = widget::DropDownList::new(&shader_names, Some(shader_idx))
        .w_h(COLUMN_W, PAD * 2.0)
        .down(10.0)
        .max_visible_items(15)
        .rgb(0.176, 0.513, 0.639)
        .label("LED Shader Preset")
        .label_font_size(15)
        .label_rgb(1.0, 1.0, 1.0)
        .scrollbar_on_top()
        .set(ids.led_shader_right_ddl, ui)
    {
        preset.shader_right = Shader::from_index(selected_idx).unwrap();
    }

    {
        let params = shader_params(preset.shader_right, &mut preset.shader_params);
        set_shader_widgets(
            ui,
            ids,
            params,
            ShaderWidgetState {
                mod_slider_ix: &mut mod_slider_ix,
                int_slider_ix: &mut int_slider_ix,
                button_ix: &mut button_ix,
                mod_amounts: &mut preset.shader_mod_amounts,
                envelope: audio_input.envelope,
            },
        );
    }

    preset.shader_mod_amounts.truncate(mod_slider_ix);

    //---------------------- BLEND MODES
    text("LED Blend Mode")
        .down(20.0)
        .color(color::WHITE)
        .set(ids.blend_mode_text, ui);

    let blend_mode_names: Vec<_> = shader_shared::ALL_BLEND_MODES
        .iter()
        .map(|blend_mode| blend_mode.name())
        .collect();
    let blend_mode_idx = preset.blend_mode as usize;
    if let Some(selected_idx) = widget::DropDownList::new(&blend_mode_names, Some(blend_mode_idx))
        .w_h(COLUMN_W, PAD * 2.0)
        .down(10.0)
        .max_visible_items(15)
        .rgb(0.176, 0.513, 0.639)
        .label("Wash Shader Preset")
        .label_font_size(15)
        .label_rgb(1.0, 1.0, 1.0)
        .scrollbar_on_top()
        .set(ids.blend_mode_ddl, ui)
    {
        preset.blend_mode = BlendMode::from_index(selected_idx).unwrap();
    }

    if let Some(value) = slider(preset.left_right_mix, 1.0, -1.0)
        .down(10.0)
        .label("Left Right Mix")
        .set(ids.shader_mix_left_right, ui)
    {
        preset.left_right_mix = value;
    }

    if let Some(value) = slider(config.fade_to_black.led, 0.0, 1.0)
        .down(10.0)
        .label("LED Fade to Black")
        .set(ids.led_fade_to_black, ui)
    {
        config.fade_to_black.led = value;
    }

    // A scrollbar for the canvas.
    //widget::Scrollbar::y_axis(ids.background).auto_hide(true).set(ids.scrollbar, ui);
}

fn set_live_sidebar_widgets(
    ui: &mut UiCell,
    ids: &Ids,
    config: &mut Config,
    since_start: std::time::Duration,
    shader_activity: shader::Activity<'_>,
) -> widget::Id {
    if button()
        .color(toggle_color(config.dmx_on))
        .label("DMX")
        .w(HALF_WIDGET_W)
        .mid_left_of(ids.column_1_id)
        .down_from(ids.live_tab_button, PAD * 0.5)
        .set(ids.dmx_button, ui)
        .was_clicked()
    {
        config.dmx_on = !config.dmx_on;
    }

    if button()
        .color(toggle_color(config.midi_on))
        .label("MIDI")
        .right(PAD * 0.5)
        .w(HALF_WIDGET_W)
        .set(ids.midi_button, ui)
        .was_clicked()
    {
        config.midi_on = !config.midi_on;
    }

    text("Shader State")
        .mid_left_of(ids.column_1_id)
        .down(COLUMN_ONE_SECTION_GAP)
        .set(ids.shader_title_text, ui);

    let (string, color) = match shader_activity {
        shader::Activity::Incoming => {
            let s = "Compiling".into();
            let l = (since_start.secs() * 2.0 * PI).sin() * 0.35 + 0.5;
            let c = ui::color::YELLOW.with_luminance(l as _);
            (s, c)
        }
        shader::Activity::LastIncoming(last) => match last {
            shader::LastIncoming::Succeeded => {
                let s = "Succeeded".into();
                let c = ui::color::GREEN;
                (s, c)
            }
            shader::LastIncoming::Failed(_err) => {
                let s = "Compilation Failed".to_string();
                let c = ui::color::RED;
                (s, c)
            }
        },
    };
    text(&string)
        .color(color)
        .down(PAD)
        .set(ids.shader_state_text, ui);

    ids.shader_state_text
}

fn set_output_sidebar_widgets(
    ui: &mut UiCell,
    ids: &Ids,
    config: &mut Config,
    sacn_error: Option<&str>,
    sacn_output_monitor: &crate::SacnOutputMonitor,
) {
    text("LED Output FPS")
        .mid_left_of(ids.column_1_id)
        .down_from(ids.live_tab_button, PAD * 0.5)
        .set(ids.output_fps_text, ui);

    let output_fps_labels: Vec<_> = crate::conf::LedOutputFps::ALL
        .iter()
        .map(|mode| mode.label())
        .collect();
    let selected_output_fps = Some(config.led_output_fps.to_index());
    if let Some(selected_idx) = widget::DropDownList::new(&output_fps_labels, selected_output_fps)
        .w_h(WIDGET_W, DEFAULT_WIDGET_H)
        .down(5.0)
        .max_visible_items(output_fps_labels.len())
        .rgb(0.176, 0.513, 0.639)
        .label("LED Output")
        .label_font_size(14)
        .label_rgb(1.0, 1.0, 1.0)
        .scrollbar_on_top()
        .set(ids.output_fps_ddl, ui)
    {
        if let Some(output_fps) = crate::conf::LedOutputFps::from_index(selected_idx) {
            config.led_output_fps = output_fps;
        }
    }

    let measured_output_fps = if config.dmx_on {
        format_measured_fps(
            sacn_output_monitor.smoothed_frame_fps,
            sacn_output_monitor.total_frames_sent,
        )
    } else {
        "Disabled".to_string()
    };
    let output_fps_status = format!(
        "LED Output: {} (Cap {})",
        measured_output_fps,
        config.led_output_fps.label()
    );
    widget::Text::new(&output_fps_status)
        .down(5.0)
        .w(WIDGET_W)
        .font_size(10)
        .color(TEXT_COLOR)
        .left_justify()
        .set(ids.output_fps_status_text, ui);

    text("sACN Interface IP")
        .mid_left_of(ids.column_1_id)
        .down_from(ids.output_fps_status_text, COLUMN_ONE_SECTION_GAP)
        .set(ids.sacn_interface_ip_text, ui);

    widget::Text::new(
        "Use this computer's IP on the PixLite network, e.g. 10.0.0.100. Leave blank for Auto; it falls back to localhost preview if multicast is unavailable.",
    )
    .down(5.0)
    .w(WIDGET_W)
    .font_size(10)
    .color(TEXT_COLOR)
    .left_justify()
    .set(ids.sacn_interface_ip_help_text, ui);

    let color = if sacn_error.is_some() {
        color::DARK_RED.with_luminance(0.1)
    } else {
        match crate::conf::parse_sacn_interface_ip(&config.sacn_interface_ip) {
            Ok(Some(_)) => color::DARK_GREEN.with_luminance(0.1),
            Ok(None) => color::BLACK,
            Err(_) => color::DARK_RED.with_luminance(0.1),
        }
    };
    for event in widget::TextBox::new(&config.sacn_interface_ip)
        .w_h(WIDGET_W, DEFAULT_WIDGET_H)
        .down(5.0)
        .border(0.0)
        .color(color)
        .text_color(color::WHITE)
        .font_size(14)
        .set(ids.sacn_interface_ip_text_box, ui)
    {
        match event {
            widget::text_box::Event::Update(string) => config.sacn_interface_ip = string,
            widget::text_box::Event::Enter => {
                config.sacn_interface_ip = config.sacn_interface_ip.trim().to_string();
            }
        }
    }

    if let Some(error) = sacn_error {
        widget::Text::new(error)
            .down(5.0)
            .w(WIDGET_W)
            .font_size(10)
            .color(color::LIGHT_RED)
            .left_justify()
            .set(ids.sacn_interface_ip_error_text, ui);
    }

    text("Universes")
        .mid_left_of(ids.column_1_id)
        .down(COLUMN_ONE_SECTION_GAP)
        .set(ids.universe_starts_text, ui);

    let min_universe = 1.0;
    let max_universe = 99.0;
    let precision = 0;
    let v = config.led_start_universe;
    if let Some(v) = widget::NumberDialer::new(v as f32, min_universe, max_universe, precision)
        .border(0.0)
        .label("Start Universe")
        .label_color(color::WHITE)
        .label_font_size(14)
        .down(PAD)
        .w(WIDGET_W)
        .h(DEFAULT_WIDGET_H)
        .color(color::DARK_CHARCOAL)
        .set(ids.led_start_universe_dialer, ui)
    {
        config.led_start_universe = v as u16;
    }

    text("LED Layout")
        .mid_left_of(ids.column_1_id)
        .down(COLUMN_ONE_SECTION_GAP)
        .set(ids.led_layout_text, ui);

    if let Some(v) = widget::NumberDialer::new(
        config.led_layout.leds_per_metre as f32,
        1.0,
        288.0,
        precision,
    )
    .border(0.0)
    .label("LEDs / Metre")
    .label_color(color::WHITE)
    .label_font_size(14)
    .down(PAD)
    .w(WIDGET_W)
    .h(DEFAULT_WIDGET_H)
    .color(color::DARK_CHARCOAL)
    .set(ids.led_pixels_per_metre_dialer, ui)
    {
        config.led_layout.leds_per_metre = v as usize;
    }

    if let Some(v) = widget::NumberDialer::new(
        config.led_layout.metres_per_row as f32,
        1.0,
        32.0,
        precision,
    )
    .border(0.0)
    .label("Row Length (m)")
    .label_color(color::WHITE)
    .label_font_size(14)
    .down(5.0)
    .w(WIDGET_W)
    .h(DEFAULT_WIDGET_H)
    .color(color::DARK_CHARCOAL)
    .set(ids.led_metres_per_row_dialer, ui)
    {
        config.led_layout.metres_per_row = v as usize;
    }

    if let Some(v) =
        widget::NumberDialer::new(config.led_layout.row_count as f32, 1.0, 32.0, precision)
            .border(0.0)
            .label("Rows")
            .label_color(color::WHITE)
            .label_font_size(14)
            .down(5.0)
            .w(WIDGET_W)
            .h(DEFAULT_WIDGET_H)
            .color(color::DARK_CHARCOAL)
            .set(ids.led_row_count_dialer, ui)
    {
        config.led_layout.row_count = v as usize;
    }

    config.led_layout.normalise();

    let total_leds = config.led_layout.led_count();
    let leds_per_universe =
        (crate::DMX_ADDRS_PER_UNIVERSE as usize - 2) / crate::DMX_ADDRS_PER_LED as usize;
    let universe_count = ((total_leds.saturating_sub(1)) / leds_per_universe) + 1;
    let start_universe = config.led_start_universe;
    let end_universe = start_universe.saturating_add(universe_count.saturating_sub(1) as u16);
    let layout_stats = format!(
        "{} LEDs across {} universes (U{}-U{})",
        total_leds, universe_count, start_universe, end_universe
    );
    widget::Text::new(&layout_stats)
        .down(5.0)
        .w(WIDGET_W)
        .font_size(10)
        .color(TEXT_COLOR)
        .left_justify()
        .set(ids.led_layout_stats_text, ui);
}

fn set_output_monitor_widgets(
    ui: &mut UiCell,
    ids: &mut Ids,
    config: &Config,
    sacn_output_monitor: &mut crate::SacnOutputMonitor,
    sacn_error: Option<&str>,
    sacn_transport_label: Option<&str>,
) {
    widget::Text::new("sACN OUTPUT")
        .top_left_of(ids.column_2_id)
        .color(TEXT_COLOR)
        .set(ids.sacn_output_title_text, ui);

    let status_text = if !config.dmx_on {
        "DMX output is disabled on the Live tab.".to_string()
    } else if let Some(error) = sacn_error.or(sacn_output_monitor.last_send_error.as_deref()) {
        format!(
            "LED output: {} (Cap {})\nsACN error:\n{}",
            format_measured_fps(
                sacn_output_monitor.smoothed_frame_fps,
                sacn_output_monitor.total_frames_sent
            ),
            config.led_output_fps.label(),
            error
        )
    } else if let Some(last_sent_at) = sacn_output_monitor.last_sent_at {
        format!(
            "Route: {}\nLED output: {} (Cap {})\nLast send: {:.2}s ago\nFrames sent: {}\nPackets sent: {}\nPayload bytes: {}",
            sacn_transport_label.unwrap_or("Unknown"),
            format_measured_fps(
                sacn_output_monitor.smoothed_frame_fps,
                sacn_output_monitor.total_frames_sent
            ),
            config.led_output_fps.label(),
            last_sent_at.elapsed().as_secs_f32(),
            sacn_output_monitor.total_frames_sent,
            sacn_output_monitor.total_packets_sent,
            sacn_output_monitor.total_payload_bytes_sent
        )
    } else {
        format!(
            "LED output: Waiting (Cap {})\nWaiting for the first successful sACN packet.",
            config.led_output_fps.label()
        )
    };
    widget::Text::new(&status_text)
        .down(10.0)
        .w(WIDGET_W)
        .font_size(11)
        .color(TEXT_COLOR)
        .left_justify()
        .set(ids.sacn_output_status_text, ui);

    widget::Text::new("Universe View")
        .down(10.0)
        .color(TEXT_COLOR)
        .font_size(12)
        .set(ids.sacn_output_universe_text, ui);

    let universe_labels = sacn_output_monitor.available_universe_labels();
    let selected_universe = sacn_output_monitor.selected_universe_index();
    if !universe_labels.is_empty() {
        if let Some(selected_idx) = widget::DropDownList::new(&universe_labels, selected_universe)
            .w_h(WIDGET_W, DEFAULT_WIDGET_H)
            .down(5.0)
            .max_visible_items(8)
            .rgb(0.176, 0.513, 0.639)
            .label("Last Sent Universe")
            .label_font_size(14)
            .label_rgb(1.0, 1.0, 1.0)
            .scrollbar_on_top()
            .set(ids.sacn_output_universe_ddl, ui)
        {
            let _ = sacn_output_monitor.select_universe(selected_idx);
        }
    } else {
        widget::Rectangle::fill([WIDGET_W, DEFAULT_WIDGET_H])
            .down(5.0)
            .color(color::DARK_CHARCOAL)
            .set(ids.sacn_output_universe_placeholder, ui);
    }

    widget::Text::new("Channels 1-512 run left to right, top to bottom. Start code omitted.")
        .down(5.0)
        .w(WIDGET_W)
        .font_size(10)
        .color(TEXT_COLOR)
        .left_justify()
        .set(ids.sacn_output_grid_help_text, ui);

    widget::Rectangle::fill([WIDGET_W, 390.0])
        .down(5.0)
        .color(color::rgb(0.05, 0.05, 0.1))
        .set(ids.sacn_output_grid_bg, ui);

    let selected_snapshot = sacn_output_monitor.selected_universe_snapshot();
    let data_slots = selected_snapshot
        .map(|snapshot| snapshot.payload.get(1..).unwrap_or(&[]))
        .unwrap_or(&[]);
    draw_sacn_output_grid(ui, ids, data_slots);

    let summary = match selected_snapshot {
        Some(snapshot) => {
            let data_slot_count = snapshot.payload.len().saturating_sub(1);
            let pad_bytes = if data_slot_count == crate::DMX_ADDRS_PER_UNIVERSE as usize {
                2
            } else {
                0
            };
            let rgb_pixels = data_slot_count.saturating_sub(pad_bytes) / 3;
            let non_zero_slots = data_slots.iter().filter(|&&value| value != 0).count();
            format!(
                "U{}: {} packets, {} slots, {} RGB pixels, {} non-zero slots{}",
                snapshot.universe,
                snapshot.packets_sent,
                data_slot_count,
                rgb_pixels,
                non_zero_slots,
                if pad_bytes > 0 {
                    ", 2 pad bytes reserved for RGB alignment"
                } else {
                    ""
                }
            )
        }
        None => "No successful sACN payload captured yet.".to_string(),
    };
    widget::Text::new(&summary)
        .down_from(ids.sacn_output_grid_bg, 5.0)
        .w(WIDGET_W)
        .font_size(10)
        .color(TEXT_COLOR)
        .left_justify()
        .set(ids.sacn_output_grid_summary_text, ui);

    let slot_preview = format_slot_preview(data_slots);
    widget::Text::new(&slot_preview)
        .down(5.0)
        .w(WIDGET_W)
        .font_size(10)
        .color(TEXT_COLOR)
        .left_justify()
        .set(ids.sacn_output_slot_preview_text, ui);
}

fn draw_sacn_output_grid(ui: &mut UiCell, ids: &mut Ids, data_slots: &[u8]) {
    const GRID_COLS: usize = 16;
    const GRID_ROWS: usize = 32;
    const GRID_CELL_GAP: Scalar = 1.0;

    let Some(bg_rect) = ui.rect_of(ids.sacn_output_grid_bg) else {
        return;
    };

    let cell_w = (bg_rect.w() - GRID_CELL_GAP * (GRID_COLS as Scalar + 1.0)) / GRID_COLS as Scalar;
    let cell_h = (bg_rect.h() - GRID_CELL_GAP * (GRID_ROWS as Scalar + 1.0)) / GRID_ROWS as Scalar;

    if ids.sacn_output_grid_cells.len() < GRID_COLS * GRID_ROWS {
        ids.sacn_output_grid_cells
            .resize(GRID_COLS * GRID_ROWS, &mut ui.widget_id_generator());
    }
    if ids.sacn_output_grid_cell_values.len() < GRID_COLS * GRID_ROWS {
        ids.sacn_output_grid_cell_values
            .resize(GRID_COLS * GRID_ROWS, &mut ui.widget_id_generator());
    }

    let font_size = 7;

    for idx in 0..(GRID_COLS * GRID_ROWS) {
        let value = data_slots.get(idx).copied().unwrap_or(0);
        let norm = value as f32 / 255.0;
        let color = color::rgb(0.08 + norm * 0.82, 0.08 + norm * 0.5, 0.12 + norm * 0.2);
        let text_color = if norm > 0.55 {
            color::BLACK
        } else {
            color::WHITE
        };
        let row = idx / GRID_COLS;
        let col = idx % GRID_COLS;
        let x = bg_rect.left()
            + GRID_CELL_GAP
            + cell_w * 0.5
            + col as Scalar * (cell_w + GRID_CELL_GAP);
        let y =
            bg_rect.top() - GRID_CELL_GAP - cell_h * 0.5 - row as Scalar * (cell_h + GRID_CELL_GAP);

        widget::Rectangle::fill([cell_w, cell_h])
            .x_y(x, y)
            .color(color)
            .set(ids.sacn_output_grid_cells[idx], ui);

        widget::Text::new(&value.to_string())
            .x_y(x, y)
            .w_h(cell_w, cell_h)
            .font_size(font_size)
            .color(text_color)
            .center_justify()
            .set(ids.sacn_output_grid_cell_values[idx], ui);
    }
}

fn format_slot_preview(data_slots: &[u8]) -> String {
    if data_slots.is_empty() {
        return "Slots 1-16: waiting for sACN output.".to_string();
    }

    let first_eight = data_slots
        .iter()
        .take(8)
        .map(|value| format!("{:03}", value))
        .collect::<Vec<_>>()
        .join(" ");
    let second_eight = data_slots
        .iter()
        .skip(8)
        .take(8)
        .map(|value| format!("{:03}", value))
        .collect::<Vec<_>>()
        .join(" ");

    format!("Slots 1-8: {}\nSlots 9-16: {}", first_eight, second_eight)
}

fn format_measured_fps(smoothed_fps: f32, total_frames: u64) -> String {
    match total_frames {
        0 => "Waiting".to_string(),
        1 => "Measuring...".to_string(),
        _ => format!("{:.1} FPS", smoothed_fps),
    }
}

pub fn set_presets_widgets(
    ui: &mut UiCell,
    ids: &Ids,
    config: &mut Config,
    last_preset_change: &mut Option<crate::LastPresetChange>,
    led_colors: &LedColors,
    assets: &Path,
) {
    const PRESET_ACTION_GAP: Scalar = 2.0;

    widget::Text::new("PRESETS")
        .top_left_of(ids.column_2_id)
        .color(TEXT_COLOR)
        .set(ids.presets_text, ui);

    let label = format!("Lerp Duration: {:.2} secs", config.preset_lerp_secs);
    if let Some(v) = slider(config.preset_lerp_secs, 0.0, 6.0)
        .down(10.0)
        .w_h(WIDGET_W, DEFAULT_WIDGET_H)
        .label(&label)
        .set(ids.presets_lerp_slider, ui)
    {
        config.preset_lerp_secs = v;
    }

    for _click in button()
        .down(10.0)
        .label("Save")
        .w_h(WIDGET_W, DEFAULT_WIDGET_H)
        .color(BUTTON_COLOR)
        .set(ids.presets_save_button, ui)
    {
        config.presets.selected_mut().name = config.presets.selected_preset_name.clone();
        super::save_config(assets, config);
    }

    for _click in button()
        .down(PRESET_ACTION_GAP)
        .label("Delete")
        .w_h(WIDGET_W, DEFAULT_WIDGET_H)
        .color(BUTTON_COLOR)
        .set(ids.presets_delete_button, ui)
    {
        config
            .presets
            .list
            .remove(config.presets.selected_preset_idx);

        // Ensure there's always at least one preset.
        if config.presets.list.is_empty() {
            config.presets.list.push(Default::default());
        }

        // Ensure the selected index points at a valid preset.
        if config.presets.selected_preset_idx >= config.presets.list.len() {
            config.presets.selected_preset_idx -= 1;
        }

        // Update selected preset name.
        config.presets.selected_preset_name = config.presets.selected().name.clone();
    }

    for _click in button()
        .down(PRESET_ACTION_GAP)
        .label("New")
        .w_h(WIDGET_W, DEFAULT_WIDGET_H)
        .color(BUTTON_COLOR)
        .set(ids.presets_new_button, ui)
    {
        let new_preset = crate::conf::Preset::default();
        config.presets.selected_preset_name = new_preset.name.clone();
        config.presets.list.push(new_preset);
        config.presets.selected_preset_idx = config.presets.list.len() - 1;
    }

    for _click in button()
        .down(PRESET_ACTION_GAP)
        .label("Duplicate")
        .w_h(WIDGET_W, DEFAULT_WIDGET_H)
        .color(BUTTON_COLOR)
        .set(ids.presets_duplicate, ui)
    {
        let mut new_preset = config.presets.selected().clone();
        new_preset.name = config.presets.selected_preset_name.clone();
        config.presets.list.push(new_preset);
        config.presets.selected_preset_idx = config.presets.list.len() - 1;
    }

    widget::Text::new("Enter Preset Name")
        .down(10.0)
        .font_size(10)
        .color(TEXT_COLOR)
        .set(ids.enter_preset_name_text, ui);

    for event in widget::TextBox::new(&config.presets.selected_preset_name)
        .down(10.0)
        .w_h(WIDGET_W, TEXT_BOX_H)
        .color(PRESET_ENTRY_COLOR)
        .text_color(TEXT_COLOR)
        .font_size(14)
        .set(ids.presets_text_box, ui)
    {
        use nannou_conrod::widget::text_box::Event;
        match event {
            Event::Update(text) => config.presets.selected_preset_name = text,
            Event::Enter => {
                config.presets.selected_mut().name = config.presets.selected_preset_name.clone();
                super::save_config(assets, config);
            }
        }
    }

    let names: Vec<_> = config.presets.list.iter().map(|p| p.name.clone()).collect();

    // Instantiate the `ListSelect` widget.
    let font_size = TEXT_BOX_H as ui::FontSize / 2;
    let (mut events, presets_scrollbar) = widget::ListSelect::single(names.len())
        .flow_down()
        .item_size(TEXT_BOX_H)
        .scrollbar_next_to()
        .w_h(WIDGET_W, 500.0)
        .down_from(ids.presets_text_box, 10.0)
        .align_left()
        .set(ids.presets_list, ui);

    // Handle the `ListSelect`s events.
    while let Some(event) = events.next(ui, |i| i == config.presets.selected_preset_idx) {
        use nannou_conrod::widget::list_select::Event;
        match event {
            // For the `Item` events we instantiate the `List`'s items.
            Event::Item(item) => {
                let label = &names[item.i];
                let (color, label_color) = if item.i == config.presets.selected_preset_idx {
                    (PRESET_LIST_SELECTED_COLOR, nannou_conrod::color::BLACK)
                } else {
                    (PRESET_LIST_COLOR, TEXT_COLOR)
                };
                let button = widget::Button::new()
                    .border(0.0)
                    .color(color)
                    .w_h(WIDGET_W, TEXT_BOX_H)
                    .label(label)
                    .label_font_size(font_size)
                    .label_color(label_color);
                item.set(button, ui);
            }

            // The selection has changed.
            Event::Selection(selection) => {
                if selection < config.presets.list.len() {
                    config.presets.selected_preset_idx = selection;
                    config.presets.selected_preset_name = config.presets.selected().name.clone();
                    let now = std::time::Instant::now();
                    *last_preset_change = Some((now, led_colors.to_vec()));
                }
            }
            _ => (),
        }
    }

    if let Some(sb) = presets_scrollbar {
        sb.set(ui);
    }
}

fn set_shader_widgets(
    ui: &mut UiCell,
    ids: &mut Ids,
    params: &mut dyn Params,
    state: ShaderWidgetState<'_>,
) {
    use crate::mod_slider::ModSlider;
    let ShaderWidgetState {
        mod_slider_ix,
        int_slider_ix,
        button_ix,
        mod_amounts,
        envelope,
    } = state;

    for ix in 0..params.param_count() {
        let ParamMut { name, kind } = params.param_mut(ix);

        match kind {
            ParamKindMut::F32 { value, max } => {
                if ids.shader_mod_sliders.len() <= *mod_slider_ix {
                    ids.shader_mod_sliders
                        .resize(*mod_slider_ix + 1, &mut ui.widget_id_generator());
                }
                if mod_amounts.len() <= *mod_slider_ix {
                    mod_amounts.resize(*mod_slider_ix + 1, 0.0);
                }
                let id = ids.shader_mod_sliders[*mod_slider_ix];
                let mod_amt = mod_amounts[*mod_slider_ix];

                if let Some((v, m)) = ModSlider::new(*value, mod_amt, envelope, 0.0, max)
                    .label(name)
                    .w_h(COLUMN_W, 30.0)
                    .down(10.0)
                    .set(id, ui)
                {
                    *value = v;
                    mod_amounts[*mod_slider_ix] = m;
                }

                *mod_slider_ix += 1;
            }

            ParamKindMut::Usize { value, max } => {
                if ids.shader_int_sliders.len() <= *int_slider_ix {
                    ids.shader_int_sliders
                        .resize(*int_slider_ix + 1, &mut ui.widget_id_generator());
                }
                let id = ids.shader_int_sliders[*int_slider_ix];

                if let Some(v) = slider(*value as f32, 0.0, max as f32)
                    .down(10.0)
                    .label(name)
                    .set(id, ui)
                {
                    *value = v as usize;
                }

                *int_slider_ix += 1;
            }

            ParamKindMut::Bool(value) => {
                if ids.shader_buttons.len() <= *button_ix {
                    ids.shader_buttons
                        .resize(*button_ix + 1, &mut ui.widget_id_generator());
                }
                let id = ids.shader_buttons[*button_ix];

                for v in toggle(*value)
                    .down(10.0)
                    .label(name)
                    .w_h(COLUMN_W, PAD)
                    .set(id, ui)
                {
                    *value = v;
                }

                *button_ix += 1;
            }
        }
    }
}

fn text(s: &str) -> widget::Text<'_> {
    widget::Text::new(s).color(color::WHITE)
}

fn toggle_color(on: bool) -> ui::Color {
    match on {
        true => color::BLUE,
        false => color::BLACK,
    }
}

fn tab_button_color(selected: bool) -> ui::Color {
    match selected {
        true => PRESET_LIST_SELECTED_COLOR,
        false => color::DARK_CHARCOAL,
    }
}

fn button() -> widget::Button<'static, widget::button::Flat> {
    widget::Button::new()
        .w_h(COLUMN_W, DEFAULT_WIDGET_H)
        .label_font_size(12)
        .color(color::DARK_CHARCOAL)
        .label_color(color::WHITE)
        .border(0.0)
}

// Shorthand for the toggle style we'll use.
fn toggle(b: bool) -> widget::Toggle<'static> {
    widget::Toggle::new(b)
        .w_h(COLUMN_W, DEFAULT_SLIDER_H)
        .label_font_size(14)
        .rgb(0.176, 0.513, 0.639)
        .label_rgb(1.0, 1.0, 1.0)
        .border(0.0)
}

// Shorthand for the slider style we'll use
pub fn slider(val: f32, min: f32, max: f32) -> widget::Slider<'static, f32> {
    widget::Slider::new(val, min, max)
        .w_h(COLUMN_W, DEFAULT_SLIDER_H)
        .label_font_size(14)
        .rgb(0.176, 0.513, 0.639)
        .label_rgb(1.0, 1.0, 1.0)
        .border(0.0)
}

fn column_canvas(background: widget::Id) -> widget::Canvas<'static> {
    widget::Canvas::new()
        .border(0.0)
        .rgb(0.1, 0.1, 0.1)
        .pad(0.0)
        .parent(background)
        .w(COLUMN_W)
        .h_of(background)
        .scroll_kids_vertically()
}

/// Apply envelope modulation to shader params, matching the same iteration
/// order as set_shader_widgets so mod_slider_ix lines up with mod_amounts.
pub fn apply_shader_modulation(
    shader: Shader,
    params: &mut ShaderParams,
    mod_slider_ix: &mut usize,
    mod_amounts: &[f32],
    envelope: f32,
) {
    let p: &mut dyn Params = shader_params(shader, params);
    for ix in 0..p.param_count() {
        let ParamMut { kind, .. } = p.param_mut(ix);
        match kind {
            ParamKindMut::F32 { value, max } => {
                if let Some(&mod_amt) = mod_amounts.get(*mod_slider_ix) {
                    let offset = (envelope * mod_amt) - (mod_amt / 2.0);
                    *value = (*value + offset).max(0.0).min(max);
                }
                *mod_slider_ix += 1;
            }
            ParamKindMut::Usize { .. } | ParamKindMut::Bool(_) => {}
        }
    }
}

pub fn normalise_preset_shader_mod_amounts(preset: &mut crate::conf::Preset) {
    let left_count = shader_modulation_slot_count(preset.shader_left, &mut preset.shader_params);
    let colourise_count = shader_modulation_slot_count(preset.colourise, &mut preset.shader_params);
    let right_count = shader_modulation_slot_count(preset.shader_right, &mut preset.shader_params);
    let mod_slot_count = left_count + colourise_count + right_count;
    preset.shader_mod_amounts.resize(mod_slot_count, 0.0);
}

fn shader_modulation_slot_count(shader: Shader, params: &mut ShaderParams) -> usize {
    let p: &mut dyn Params = shader_params(shader, params);
    let mut count = 0;
    for ix in 0..p.param_count() {
        let ParamMut { kind, .. } = p.param_mut(ix);
        if let ParamKindMut::F32 { .. } = kind {
            count += 1;
        }
    }
    count
}

fn shader_params(shader: Shader, params: &mut ShaderParams) -> &mut dyn Params {
    match shader {
        Shader::AcidGradient => &mut params.acid_gradient,
        Shader::BlinkyCircles => &mut params.blinky_circles,
        Shader::BwGradient => &mut params.bw_gradient,
        Shader::ColourGrid => &mut params.colour_grid,
        Shader::EscherTilings => &mut params.escher_tilings,
        Shader::GilmoreAcid => &mut params.gilmore_acid,
        Shader::JustRelax => &mut params.just_relax,
        Shader::LifeLedWall => &mut params.life_led_wall,
        Shader::LineGradient => &mut params.line_gradient,
        Shader::Metafall => &mut params.metafall,
        Shader::ParticleZoom => &mut params.particle_zoom,
        Shader::RadialLines => &mut params.radial_lines,
        Shader::SatisSpiraling => &mut params.satis_spiraling,
        Shader::SpiralIntersect => &mut params.spiral_intersect,
        Shader::SquareTunnel => &mut params.square_tunnel,
        Shader::ThePulse => &mut params.the_pulse,
        Shader::TunnelProjection => &mut params.tunnel_projection,
        Shader::VertColourGradient => &mut params.vert_colour_gradient,
        Shader::SolidHsvColour => &mut params.solid_hsv_colour,
        Shader::SolidRgbColour => &mut params.solid_rgb_colour,
        Shader::ColourPalettes => &mut params.colour_palettes,
        Shader::MitchWash => &mut params.mitch_wash,
        Shader::ShapeEnvelopes => &mut params.shape_envelopes,
        Shader::RowTest => &mut params.row_test,
        Shader::BarTest => &mut params.bar_test,
    }
}
