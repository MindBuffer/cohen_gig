use nannou::prelude::*;
use nannou::ui::conrod_core::widget_ids;
use nannou::ui::prelude::*;

pub const COLUMN_W: Scalar = 240.0;
pub const DEFAULT_WIDGET_H: Scalar = 30.0;
pub const PAD: Scalar = 20.0;
pub const WINDOW_WIDTH: u32 = (COLUMN_W + PAD * 2.0) as u32;
pub const WINDOW_HEIGHT: u32 = 720;

widget_ids! {
    pub struct Ids {
        background,
        title_text,
    }
}

/// Update the user interface.
pub fn update(
    ref mut ui: UiCell,
    ids: &Ids,
) {
    widget::Canvas::new()
        .border(0.0)
        .rgb(0.1, 0.1, 0.1)
        .pad(PAD)
        .set(ids.background, ui);

    text("COHEN GIG")
        .mid_top_of(ids.background)
        .set(ids.title_text, ui);
}

fn text(s: &str) -> widget::Text {
    widget::Text::new(s).color(color::WHITE)
}
