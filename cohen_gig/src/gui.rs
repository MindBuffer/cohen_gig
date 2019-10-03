use crate::{shader, Osc, State};
use nannou::prelude::*;
use nannou::ui::conrod_core::widget_ids;
use nannou::ui::prelude::*;
use std::f64::consts::PI;
use std::net::SocketAddr;

pub const COLUMN_W: Scalar = 240.0;
pub const DEFAULT_WIDGET_H: Scalar = 30.0;
pub const PAD: Scalar = 20.0;
pub const WINDOW_WIDTH: u32 = ((COLUMN_W + PAD * 2.0) as u32);
pub const WINDOW_HEIGHT: u32 = 1080 - PAD as u32;
pub const WIDGET_W: Scalar = COLUMN_W;
pub const HALF_WIDGET_W: Scalar = WIDGET_W * 0.5 - PAD * 0.25;

widget_ids! {
    pub struct Ids {
        background,
        title_text,
        dmx_button,
        osc_button,
        osc_address_text,
        osc_address_text_box,
        shader_title_text,
        shader_state_text,

        led_shader_left_text,
        led_shader_left_ddl,

        led_shader_right_text,
        led_shader_right_ddl,

        wash_shader_text,
        wash_shader_ddl,

        colour_post_process_text,
        colour_post_process_ddl,

        blend_mode_text,
        blend_mode_ddl,
    }
}

/// Update the user interface.
pub fn update(
    ref mut ui: UiCell,
    ids: &Ids,
    state: &mut State,
    osc: &mut Osc,
    since_start: std::time::Duration,
    shader_activity: shader::Activity,
) {
    widget::Canvas::new()
        .border(0.0)
        .rgb(0.1, 0.1, 0.1)
        .pad(PAD)
        .x(-(WINDOW_WIDTH as f64 / 2.0) + COLUMN_W / 2.0 + PAD)
        .w_h(COLUMN_W + (PAD * 2.0), WINDOW_HEIGHT as f64)
        .set(ids.background, ui);

    text("COHEN GIG")
        .mid_top_of(ids.background)
        .set(ids.title_text, ui);

    if button()
        .color(toggle_color(state.dmx_on))
        .label("DMX")
        .w(HALF_WIDGET_W)
        .mid_left_of(ids.background)
        .down(PAD * 1.5)
        .set(ids.dmx_button, ui)
        .was_clicked()
    {
        state.dmx_on = !state.dmx_on;
    }

    if button()
        .color(toggle_color(state.osc_on))
        .label("OSC")
        .right(PAD * 0.5)
        .w(HALF_WIDGET_W)
        .set(ids.osc_button, ui)
        .was_clicked()
    {
        state.osc_on = !state.osc_on;
    }

    text("OSC Address")
        .mid_left_of(ids.background)
        .down(PAD * 1.5)
        .set(ids.osc_address_text, ui);

    let color = match state.osc_addr_textbox_string.parse::<SocketAddr>() {
        Ok(socket) => {
            match osc.addr == socket {
                true => color::BLACK,
                false => color::DARK_GREEN.with_luminance(0.1),
            }
        }
        Err(_) => color::DARK_RED.with_luminance(0.1),
    };
    for event in widget::TextBox::new(&state.osc_addr_textbox_string)
        .w_h(WIDGET_W, DEFAULT_WIDGET_H)
        .border(0.0)
        .color(color)
        .text_color(color::WHITE)
        .font_size(14)
        .set(ids.osc_address_text_box, ui)
    {
        match event {
            widget::text_box::Event::Update(string) => state.osc_addr_textbox_string = string,
            widget::text_box::Event::Enter => {
                if let Ok(socket) = state.osc_addr_textbox_string.parse() {
                    osc.addr = socket;
                }
            },
        }
    }

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

    //---------------------- LED SHADER LEFT 
    text("LED Shader Left")
        .down(20.0)
        .color(color::WHITE)
        .set(ids.led_shader_left_text, ui);

    for selected_idx in widget::DropDownList::new(&state.led_shader_names, state.led_shader_idx_left)
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
        state.led_shader_idx_left = Some(selected_idx);
    }

    //---------------------- LED SHADER RIGHT 
    text("LED Shader Right")
        .down(20.0)
        .color(color::WHITE)
        .set(ids.led_shader_right_text, ui);

    for selected_idx in widget::DropDownList::new(&state.led_shader_names, state.led_shader_idx_right)
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
        state.led_shader_idx_right = Some(selected_idx);
    }

    //---------------------- WASH SHADER
    text("Wash Shader")
        .down(20.0)
        .color(color::WHITE)
        .set(ids.wash_shader_text, ui);

    for selected_idx in widget::DropDownList::new(&state.wash_shader_names, state.wash_shader_idx)
        .w_h(COLUMN_W, PAD * 2.0)
        .down(10.0)
        .max_visible_items(15)
        .rgb(0.176, 0.513, 0.639)
        .label("Wash Shader Preset")
        .label_font_size(15)
        .label_rgb(1.0, 1.0, 1.0)
        .scrollbar_on_top()
        .set(ids.wash_shader_ddl, ui)
    {
        state.wash_shader_idx = Some(selected_idx);
    }

    //---------------------- COLOUT POST PROCESS SHADER
    text("Colour Post Process")
        .down(20.0)
        .color(color::WHITE)
        .set(ids.colour_post_process_text, ui);

    for selected_idx in widget::DropDownList::new(&state.solid_colour_names, state.solid_colour_idx)
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
        state.solid_colour_idx = Some(selected_idx);
    }

    //---------------------- BLEND MODES
    text("LED Blend Mode")
        .down(20.0)
        .color(color::WHITE)
        .set(ids.blend_mode_text, ui);

    for selected_idx in widget::DropDownList::new(&state.blend_mode_names, state.blend_mode_idx)
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
        state.blend_mode_idx = Some(selected_idx);
    }
}

fn text(s: &str) -> widget::Text {
    widget::Text::new(s).color(color::WHITE)
}

fn toggle_color(on: bool) -> ui::Color {
    match on {
        true => color::BLUE,
        false => color::BLACK,
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
