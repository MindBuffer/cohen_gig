use crate::knob::Knob;
use nannou_conrod::prelude::*;

const MOD_BAR_H: Scalar = 4.0;
const GAP: Scalar = 4.0;

pub struct ModSlider<'a> {
    common: widget::CommonBuilder,
    value: f32,
    smoothed_value: f32,
    mod_amount: f32,
    modulation: f32,
    min: f32,
    max: f32,
    label: &'a str,
}

pub struct SmoothedSlider<'a> {
    common: widget::CommonBuilder,
    value: f32,
    smoothed_value: f32,
    min: f32,
    max: f32,
    label: &'a str,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Style;

pub struct State {
    ids: Ids,
}

widget_ids! {
    struct Ids {
        knob,
        slider,
        mod_bar_bg,
        mod_bar,
    }
}

impl<'a> ModSlider<'a> {
    pub fn new(
        value: f32,
        smoothed_value: f32,
        mod_amount: f32,
        modulation: f32,
        min: f32,
        max: f32,
    ) -> Self {
        ModSlider {
            common: widget::CommonBuilder::default(),
            value,
            smoothed_value,
            mod_amount,
            modulation,
            min,
            max,
            label: "",
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }
}

impl<'a> widget::Common for ModSlider<'a> {
    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }
    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }
}

impl<'a> SmoothedSlider<'a> {
    pub fn new(value: f32, smoothed_value: f32, min: f32, max: f32) -> Self {
        SmoothedSlider {
            common: widget::CommonBuilder::default(),
            value,
            smoothed_value,
            min,
            max,
            label: "",
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }
}

impl<'a> widget::Common for SmoothedSlider<'a> {
    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }
    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }
}

impl<'a> Widget for ModSlider<'a> {
    type State = State;
    type Style = Style;
    type Event = Option<(f32, f32)>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        Style
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs {
            id,
            state,
            rect,
            ui,
            ..
        } = args;
        let ModSlider {
            value,
            smoothed_value,
            mod_amount,
            modulation,
            min,
            max,
            label,
            ..
        } = self;

        let mut new_value = None;
        let mut new_mod = None;

        // Layout: knob left (square, same height as slider), slider right, mod bar above.
        let slider_h = rect.h() - MOD_BAR_H - 2.0;
        let knob_size = slider_h;
        let slider_w = rect.w() - knob_size - GAP;
        let slider_left = rect.left() + knob_size + GAP;
        let slider_cx = slider_left + slider_w / 2.0;
        let slider_cy = rect.bottom() + slider_h / 2.0;
        let knob_cx = rect.left() + knob_size / 2.0;

        // Knob (mod depth)
        if let Some(v) = Knob::new(mod_amount, 0.0, 1.0)
            .w_h(knob_size, knob_size)
            .x_y(knob_cx, slider_cy)
            .parent(id)
            .set(state.ids.knob, ui)
        {
            new_mod = Some(v);
        }

        // Slider (base value)
        if let Some(v) = widget::Slider::new(value, min, max)
            .w_h(slider_w, slider_h)
            .x_y(slider_cx, slider_cy)
            .label(label)
            .label_font_size(11)
            .label_color(color::WHITE)
            .color(color::rgb(0.176, 0.513, 0.639))
            .border(0.0)
            .parent(id)
            .set(state.ids.slider, ui)
        {
            new_value = Some(v);
        }

        // Mod bar: sits just above the slider, shows final output.
        let cur_mod = new_mod.unwrap_or(mod_amount);
        let cur_val = smoothed_value;
        let offset = (modulation * cur_mod) - (cur_mod / 2.0);
        let final_val = (cur_val + offset).max(min).min(max);
        let final_norm = ((final_val - min) / (max - min)) as Scalar;

        let bar_y = slider_cy + slider_h / 2.0 + MOD_BAR_H / 2.0 + 1.0;

        // Bar background
        widget::Rectangle::fill([slider_w, MOD_BAR_H])
            .x_y(slider_cx, bar_y)
            .color(color::rgb(0.08, 0.08, 0.08))
            .parent(id)
            .graphics_for(id)
            .set(state.ids.mod_bar_bg, ui);

        // Bar fill
        let fill_w = (final_norm * slider_w).max(0.0);
        if fill_w > 0.5 {
            widget::Rectangle::fill([fill_w, MOD_BAR_H])
                .x_y(slider_left + fill_w / 2.0, bar_y)
                .color(color::rgb(0.2, 0.8, 0.4))
                .parent(id)
                .graphics_for(id)
                .set(state.ids.mod_bar, ui);
        }

        if new_value.is_some() || new_mod.is_some() {
            Some((new_value.unwrap_or(value), new_mod.unwrap_or(mod_amount)))
        } else {
            None
        }
    }
}

impl<'a> Widget for SmoothedSlider<'a> {
    type State = State;
    type Style = Style;
    type Event = Option<f32>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        Style
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs {
            id,
            state,
            rect,
            ui,
            ..
        } = args;
        let SmoothedSlider {
            value,
            smoothed_value,
            min,
            max,
            label,
            ..
        } = self;

        let slider_h = rect.h() - MOD_BAR_H - 2.0;
        let slider_w = rect.w();
        let slider_cx = rect.x();
        let slider_cy = rect.bottom() + slider_h / 2.0;
        let slider_left = rect.left();

        let new_value = widget::Slider::new(value, min, max)
            .w_h(slider_w, slider_h)
            .x_y(slider_cx, slider_cy)
            .label(label)
            .label_font_size(11)
            .label_color(color::WHITE)
            .color(color::rgb(0.176, 0.513, 0.639))
            .border(0.0)
            .parent(id)
            .set(state.ids.slider, ui);

        let final_norm = ((smoothed_value - min) / (max - min)).clamp(0.0, 1.0) as Scalar;
        let bar_y = slider_cy + slider_h / 2.0 + MOD_BAR_H / 2.0 + 1.0;

        widget::Rectangle::fill([slider_w, MOD_BAR_H])
            .x_y(slider_cx, bar_y)
            .color(color::rgb(0.08, 0.08, 0.08))
            .parent(id)
            .graphics_for(id)
            .set(state.ids.mod_bar_bg, ui);

        let fill_w = (final_norm * slider_w).max(0.0);
        if fill_w > 0.5 {
            widget::Rectangle::fill([fill_w, MOD_BAR_H])
                .x_y(slider_left + fill_w / 2.0, bar_y)
                .color(color::rgb(0.2, 0.8, 0.4))
                .parent(id)
                .graphics_for(id)
                .set(state.ids.mod_bar, ui);
        }

        new_value
    }
}
