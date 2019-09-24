use crate::shader;
use nannou::prelude::*;
use nannou::ui::conrod_core::widget_ids;
use nannou::ui::prelude::*;
use std::f64::consts::PI;

pub const COLUMN_W: Scalar = 240.0;
//pub const DEFAULT_WIDGET_H: Scalar = 30.0;
pub const PAD: Scalar = 20.0;
pub const WINDOW_WIDTH: u32 = (COLUMN_W + PAD * 2.0) as u32;
pub const WINDOW_HEIGHT: u32 = 720;

widget_ids! {
    pub struct Ids {
        background,
        title_text,
        shader_title_text,
        shader_state_text,
    }
}

/// Update the user interface.
pub fn update(
    ref mut ui: UiCell,
    ids: &Ids,
    since_start: std::time::Duration,
    shader_activity: shader::Activity,
) {
    widget::Canvas::new()
        .border(0.0)
        .rgb(0.1, 0.1, 0.1)
        .pad(PAD)
        .set(ids.background, ui);

    text("COHEN GIG")
        .mid_top_of(ids.background)
        .set(ids.title_text, ui);

    text("Shader State")
        .mid_left_of(ids.background)
        .down(PAD * 1.5)
        .set(ids.shader_title_text, ui);

    let (string, color) = match shader_activity {
        shader::Activity::Incoming => {
            let s = "Compiling".into();
            let l = (since_start.secs() * 2.0 * PI).sin() * 0.35 + 0.5;
            let c = ui::color::YELLOW.with_luminance(l as _);
            (s, c)
        },
        shader::Activity::LastIncoming(last) => match last {
            shader::LastIncoming::Succeeded => {
                let s = "Succeeded".into();
                let c = ui::color::GREEN;
                (s, c)
            },
            shader::LastIncoming::Failed(_err) => {
                let s = format!("Compilation Failed");
                let c = ui::color::RED;
                (s, c)
            },
        }
    };
    text(&string)
        .color(color)
        .down(PAD)
        .set(ids.shader_state_text, ui);
}

fn text(s: &str) -> widget::Text {
    widget::Text::new(s).color(color::WHITE)
}
