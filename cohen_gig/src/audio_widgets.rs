use crate::audio_input::{AudioInput, MAX_INPUT_GAIN_DB};
use crate::gui::{self, slider, COLUMN_ONE_SECTION_GAP, COLUMN_W, TEXT_COLOR};
use nannou_conrod::prelude::*;
use std::collections::VecDeque;

const SCOPE_H: Scalar = 120.0;

pub fn set_widgets(ui: &mut UiCell, ids: &gui::Ids, audio: &mut AudioInput) {
    widget::Text::new("AUDIO INPUT")
        .mid_left_of(ids.column_1_id)
        .down(COLUMN_ONE_SECTION_GAP)
        .color(TEXT_COLOR)
        .font_size(14)
        .set(ids.audio_input_text, ui);

    // --- Waveform scope ---
    widget::Rectangle::fill([COLUMN_W, SCOPE_H])
        .down(5.0)
        .color(color::rgb(0.05, 0.05, 0.1))
        .set(ids.audio_scope_bg, ui);

    draw_waveform(
        ui,
        ids.audio_scope_bg,
        ids.audio_scope,
        ids.audio_scope_neg,
        &audio.waveform_history,
    );

    // Center and threshold lines
    if let Some(bg) = ui.rect_of(ids.audio_scope_bg) {
        let centre_y = bg.y();
        let thresh_offset = audio.threshold as Scalar * bg.h() * 0.5;
        widget::Line::abs([bg.left(), centre_y], [bg.right(), centre_y])
            .color(color::rgba(1.0, 1.0, 1.0, 0.12))
            .thickness(1.0)
            .set(ids.audio_scope_midline, ui);
        widget::Line::abs(
            [bg.left(), centre_y + thresh_offset],
            [bg.right(), centre_y + thresh_offset],
        )
        .color(color::rgba(1.0, 0.3, 0.3, 0.7))
        .thickness(1.0)
        .set(ids.audio_threshold_line, ui);
        widget::Line::abs(
            [bg.left(), centre_y - thresh_offset],
            [bg.right(), centre_y - thresh_offset],
        )
        .color(color::rgba(1.0, 0.3, 0.3, 0.45))
        .thickness(1.0)
        .set(ids.audio_threshold_line_neg, ui);
    }

    // --- Sliders ---
    let gain_multiplier = audio.gain_multiplier();
    let label = format!("Gain: +{:.1} dB ({:.2}x)", audio.gain_db, gain_multiplier);
    if let Some(v) = slider(audio.gain_db, 0.0, MAX_INPUT_GAIN_DB)
        .down_from(ids.audio_scope_bg, 5.0)
        .label(&label)
        .set(ids.audio_gain_slider, ui)
    {
        audio.gain_db = v;
    }

    let label = format!("Threshold: {:.3}", audio.threshold);
    if let Some(v) = slider(audio.threshold, 0.0, 1.0)
        .down_from(ids.audio_gain_slider, 5.0)
        .label(&label)
        .set(ids.audio_threshold_slider, ui)
    {
        audio.threshold = v;
    }

    let label = format!("Attack: {:.3}s", audio.attack);
    if let Some(v) = slider(audio.attack, 0.001, 1.0)
        .down(5.0)
        .label(&label)
        .set(ids.audio_attack_slider, ui)
    {
        audio.attack = v;
    }

    let label = format!("Hold: {:.3}s", audio.hold);
    if let Some(v) = slider(audio.hold, 0.0, 1.0)
        .down(5.0)
        .label(&label)
        .set(ids.audio_hold_slider, ui)
    {
        audio.hold = v;
    }

    let label = format!("Release: {:.3}s", audio.release);
    if let Some(v) = slider(audio.release, 0.001, 2.0)
        .down(5.0)
        .label(&label)
        .set(ids.audio_release_slider, ui)
    {
        audio.release = v;
    }

    // --- Envelope scope ---
    widget::Rectangle::fill([COLUMN_W, SCOPE_H])
        .down_from(ids.audio_release_slider, 5.0)
        .color(color::rgb(0.05, 0.05, 0.1))
        .set(ids.audio_envelope_scope_bg, ui);

    draw_positive_scope(
        ui,
        ids.audio_envelope_scope_bg,
        ids.audio_envelope_scope,
        &audio.envelope_history,
    );
}

fn draw_waveform(
    ui: &mut UiCell,
    bg_id: widget::Id,
    upper_path_id: widget::Id,
    lower_path_id: widget::Id,
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
    let half_h = bg_rect.h() * 0.5;
    let centre_y = bg_rect.y();
    let bucket_count = w.max(2.0).round() as usize;
    let samples: Vec<f32> = history.iter().copied().collect();

    let (upper_points, lower_points): (Vec<[Scalar; 2]>, Vec<[Scalar; 2]>) = (0..bucket_count)
        .map(|bucket| {
            let start = bucket * len / bucket_count;
            let end = ((bucket + 1) * len / bucket_count).max(start + 1).min(len);
            let slice = &samples[start..end];
            let min_sample = slice
                .iter()
                .fold(1.0f32, |min_value, &sample| min_value.min(sample))
                .clamp(-1.0, 1.0);
            let max_sample = slice
                .iter()
                .fold(-1.0f32, |max_value, &sample| max_value.max(sample))
                .clamp(-1.0, 1.0);
            let x = bg_rect.left() + (bucket as Scalar / (bucket_count - 1) as Scalar) * w;
            (
                [x, centre_y + max_sample as Scalar * half_h],
                [x, centre_y + min_sample as Scalar * half_h],
            )
        })
        .collect();

    widget::PointPath::abs(upper_points.iter().cloned())
        .color(color::rgba(0.2, 0.8, 0.4, 0.95))
        .thickness(1.25)
        .set(upper_path_id, ui);

    widget::PointPath::abs(lower_points.iter().cloned())
        .color(color::rgba(0.2, 0.8, 0.4, 0.95))
        .thickness(1.25)
        .set(lower_path_id, ui);
}

fn draw_positive_scope(
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
