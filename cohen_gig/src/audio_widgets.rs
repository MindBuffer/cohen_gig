use crate::audio_input::AudioInput;
use crate::gui::{self, slider, COLUMN_W, PAD, TEXT_COLOR};
use nannou_conrod::prelude::*;
use std::collections::VecDeque;

const SCOPE_H: Scalar = 120.0;

pub fn set_widgets(ui: &mut UiCell, ids: &gui::Ids, audio: &mut AudioInput) {
    widget::Text::new("AUDIO INPUT")
        .mid_left_of(ids.column_1_id)
        .down(PAD * 1.5)
        .color(TEXT_COLOR)
        .font_size(14)
        .set(ids.audio_input_text, ui);

    // --- Waveform scope ---
    widget::Rectangle::fill([COLUMN_W, SCOPE_H])
        .down(5.0)
        .color(color::rgb(0.05, 0.05, 0.1))
        .set(ids.audio_scope_bg, ui);

    draw_scope(ui, ids.audio_scope_bg, ids.audio_scope, &audio.peak_history);

    // Threshold line
    if let Some(bg) = ui.rect_of(ids.audio_scope_bg) {
        let thresh_y = bg.bottom() + audio.threshold as Scalar * bg.h();
        widget::Line::abs(
            [bg.left(), thresh_y],
            [bg.right(), thresh_y],
        )
        .color(color::rgba(1.0, 0.3, 0.3, 0.7))
        .thickness(1.0)
        .set(ids.audio_threshold_line, ui);
    }

    // --- Sliders ---
    let label = format!("Threshold: {:.3}", audio.threshold);
    for v in slider(audio.threshold, 0.0, 1.0)
        .down_from(ids.audio_scope_bg, 5.0)
        .label(&label)
        .set(ids.audio_threshold_slider, ui)
    {
        audio.threshold = v;
    }

    let label = format!("Attack: {:.3}s", audio.attack);
    for v in slider(audio.attack, 0.001, 1.0)
        .down(5.0)
        .label(&label)
        .set(ids.audio_attack_slider, ui)
    {
        audio.attack = v;
    }

    let label = format!("Hold: {:.3}s", audio.hold);
    for v in slider(audio.hold, 0.0, 1.0)
        .down(5.0)
        .label(&label)
        .set(ids.audio_hold_slider, ui)
    {
        audio.hold = v;
    }

    let label = format!("Release: {:.3}s", audio.release);
    for v in slider(audio.release, 0.001, 2.0)
        .down(5.0)
        .label(&label)
        .set(ids.audio_release_slider, ui)
    {
        audio.release = v;
    }

    // --- Envelope scope ---
    widget::Rectangle::fill([COLUMN_W, SCOPE_H])
        .down(5.0)
        .color(color::rgb(0.05, 0.05, 0.1))
        .set(ids.audio_envelope_scope_bg, ui);

    draw_scope(
        ui,
        ids.audio_envelope_scope_bg,
        ids.audio_envelope_scope,
        &audio.envelope_history,
    );
}

fn draw_scope(
    ui: &mut UiCell,
    bg_id: widget::Id,
    path_id: widget::Id,
    history: &VecDeque<f32>,
) {
    let len = history.len();
    if len < 2 {
        return;
    }
    let bg_rect = match ui.rect_of(bg_id) {
        Some(r) => r,
        None => return,
    };
    let w = bg_rect.w();
    let h = bg_rect.h();

    // Points in absolute coordinates: bottom of scope = bg bottom, top = bg top.
    let points: Vec<[Scalar; 2]> = history
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = bg_rect.left() + (i as Scalar / (len - 1) as Scalar) * w;
            let y = bg_rect.bottom() + v as Scalar * h;
            [x, y]
        })
        .collect();

    widget::PointPath::abs(points.iter().cloned())
        .color(color::rgb(0.2, 0.8, 0.4))
        .thickness(1.0)
        .set(path_id, ui);
}
