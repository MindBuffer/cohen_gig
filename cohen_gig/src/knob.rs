use nannou_conrod::prelude::*;
use std::f64::consts::PI;

const START_ANGLE: f64 = 5.0 * PI / 4.0; // 225° (7 o'clock)
const ANGLE_RANGE: f64 = 3.0 * PI / 2.0; // 270° sweep to 5 o'clock
const ARC_SEGMENTS: usize = 32;
const SCALAR: f64 = 0.008;

pub struct Knob<'a> {
    common: widget::CommonBuilder,
    value: f32,
    min: f32,
    max: f32,
    label: &'a str,
    color: color::Color,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Style;

pub struct State {
    ids: Ids,
    // Normalized value when the drag started.
    drag_start_norm: Option<f64>,
}

widget_ids! {
    struct Ids {
        bg,
        track,
        arc,
        indicator,
        label,
    }
}

impl<'a> Knob<'a> {
    pub fn new(value: f32, min: f32, max: f32) -> Self {
        Knob {
            common: widget::CommonBuilder::default(),
            value,
            min,
            max,
            label: "",
            color: color::rgb(0.176, 0.513, 0.639),
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn color(mut self, color: color::Color) -> Self {
        self.color = color;
        self
    }
}

impl<'a> widget::Common for Knob<'a> {
    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }
    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }
}

impl<'a> Widget for Knob<'a> {
    type State = State;
    type Style = Style;
    type Event = Option<f32>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
            drag_start_norm: None,
        }
    }

    fn style(&self) -> Self::Style {
        Style
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs {
            id, state, rect, ui, ..
        } = args;
        let Knob {
            value,
            min,
            max,
            label,
            color: knob_color,
            ..
        } = self;

        let range = max - min;
        let norm = ((value - min) / range) as f64;

        // --- Input ---
        let mut new_value = None;
        let input = ui.widget_input(id);

        // On press: snapshot the current normalized value.
        for press in input.presses().mouse().left() {
            state.update(|s| s.drag_start_norm = Some(norm));
        }

        // On drag: compute new value from total_delta_xy relative to drag start.
        for drag in input.drags().left() {
            if let Some(start_norm) = state.drag_start_norm {
                let delta_norm = drag.total_delta_xy[1] * SCALAR;
                let new_norm = (start_norm + delta_norm).max(0.0).min(1.0);
                let v = min + new_norm as f32 * range;
                new_value = Some(v);
            }
        }

        // On release: clear drag state.
        for _release in input.releases().mouse().left() {
            state.update(|s| s.drag_start_norm = None);
        }

        let current = new_value.unwrap_or(value);
        let display_norm = ((current - min) / range) as f64;

        // --- Layout ---
        let cx = rect.x();
        let label_h = 14.0;
        let cy = rect.y() + label_h * 0.5;
        let radius = (rect.w().min(rect.h() - label_h)) * 0.45;
        let track_r = radius * 0.78;

        // Background circle
        widget::Circle::fill(radius)
            .color(color::rgb(0.15, 0.15, 0.15))
            .x_y(cx, cy)
            .parent(id)
            .graphics_for(id)
            .set(state.ids.bg, ui);

        // Track arc (dim, shows full range)
        let track: Vec<[Scalar; 2]> = (0..=ARC_SEGMENTS)
            .map(|i| {
                let t = i as f64 / ARC_SEGMENTS as f64;
                let a = START_ANGLE - t * ANGLE_RANGE;
                [cx + track_r * a.cos(), cy + track_r * a.sin()]
            })
            .collect();
        widget::PointPath::abs(track.iter().cloned())
            .color(color::rgb(0.3, 0.3, 0.3))
            .thickness(2.5)
            .parent(id)
            .graphics_for(id)
            .set(state.ids.track, ui);

        // Value arc (bright, from min to current)
        if display_norm > 0.005 {
            let n = (ARC_SEGMENTS as f64 * display_norm).max(2.0) as usize;
            let arc: Vec<[Scalar; 2]> = (0..=n)
                .map(|i| {
                    let t = i as f64 / n as f64;
                    let a = START_ANGLE - t * display_norm * ANGLE_RANGE;
                    [cx + track_r * a.cos(), cy + track_r * a.sin()]
                })
                .collect();
            widget::PointPath::abs(arc.iter().cloned())
                .color(knob_color)
                .thickness(3.0)
                .parent(id)
                .graphics_for(id)
                .set(state.ids.arc, ui);
        }

        // Indicator line
        let angle = START_ANGLE - display_norm * ANGLE_RANGE;
        let inner = radius * 0.25;
        let outer = radius * 0.9;
        widget::Line::abs(
            [cx + inner * angle.cos(), cy + inner * angle.sin()],
            [cx + outer * angle.cos(), cy + outer * angle.sin()],
        )
        .color(color::WHITE)
        .thickness(2.0)
        .parent(id)
        .graphics_for(id)
        .set(state.ids.indicator, ui);

        // Label
        if !label.is_empty() {
            widget::Text::new(label)
                .font_size(11)
                .color(color::WHITE)
                .mid_bottom_of(id)
                .parent(id)
                .graphics_for(id)
                .set(state.ids.label, ui);
        }

        new_value
    }
}
