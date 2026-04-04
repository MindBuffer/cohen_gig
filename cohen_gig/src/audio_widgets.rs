use crate::audio_input::{AudioInput, MAX_INPUT_GAIN_DB};
use crate::gui::{self, slider, COLUMN_ONE_SECTION_GAP, COLUMN_W, TEXT_COLOR};
use crate::mod_slider::SmoothedSlider;
use nannou_conrod::prelude::*;
use std::collections::VecDeque;

const SCOPE_H: Scalar = 120.0;

pub fn set_widgets(
    ui: &mut UiCell,
    ids: &gui::Ids,
    audio: &mut AudioInput,
    preferred_device_name: &mut String,
    smoothing_speed: &mut f32,
    master_speed: &mut f32,
    smoothed_master_speed: f32,
    anchor_id: widget::Id,
) {
    widget::Text::new("AUDIO INPUT")
        .down_from(anchor_id, COLUMN_ONE_SECTION_GAP)
        .align_left_of(ids.column_1_id)
        .color(TEXT_COLOR)
        .font_size(14)
        .set(ids.audio_input_text, ui);

    let audio_device_labels = audio.available_device_labels();
    let selected_audio_device = audio.selected_device_index();
    if !audio_device_labels.is_empty() {
        if let Some(selected_idx) =
            widget::DropDownList::new(&audio_device_labels, selected_audio_device)
                .w_h(COLUMN_W, gui::DEFAULT_WIDGET_H)
                .down(5.0)
                .max_visible_items(8)
                .rgb(0.176, 0.513, 0.639)
                .label("Audio Input")
                .label_font_size(14)
                .label_rgb(1.0, 1.0, 1.0)
                .scrollbar_on_top()
                .set(ids.audio_device_ddl, ui)
        {
            if let Some(selected_name) = audio.select_device(selected_idx) {
                *preferred_device_name = selected_name;
            }
        }
    } else {
        widget::Rectangle::fill([COLUMN_W, gui::DEFAULT_WIDGET_H])
            .down(5.0)
            .color(color::DARK_CHARCOAL)
            .set(ids.audio_device_placeholder, ui);
    }

    if let Some(error) = audio.device_error() {
        widget::Text::new(error)
            .down(5.0)
            .w(COLUMN_W)
            .font_size(10)
            .color(color::LIGHT_RED)
            .left_justify()
            .set(ids.audio_device_error_text, ui);
    }

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

    widget::Text::new("GLOBAL PARAMS")
        .down_from(ids.audio_envelope_scope_bg, COLUMN_ONE_SECTION_GAP)
        .align_left_of(ids.column_1_id)
        .color(TEXT_COLOR)
        .font_size(14)
        .set(ids.global_params_text, ui);

    let label = format!("Smoothing Speed: {:.4}", *smoothing_speed);
    if let Some(v) = slider(*smoothing_speed, 0.0008, 0.08)
        .down(5.0)
        .label(&label)
        .set(ids.smoothing_speed_slider, ui)
    {
        *smoothing_speed = v;
    }

    let label = format!("Master Speed: {:.3}", *master_speed);
    if let Some(v) = SmoothedSlider::new(*master_speed, smoothed_master_speed, 0.0, 1.0)
        .down(5.0)
        .label(&label)
        .set(ids.master_speed_slider, ui)
    {
        *master_speed = v;
    }
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
