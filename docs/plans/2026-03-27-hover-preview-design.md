# Live Hover Preview Design

## Goal

Show a live-rendered shader preview popup when hovering over items in shader
dropdowns (Phase 1) and preset list (Phase 2). Both phases share a single
preview buffer, texture, and rendering path.

## Architecture

```
GUI thread                     LED worker thread              GUI thread
───────────                    ────────────────              ───────────
detect hovered item ──write──▶ Option<HoverPreviewRequest>
                               if Some → render into
                               led_colors_hover buffer
                               ◀──read── copy to model
                                         upload to hover texture
                                         draw floating Image at
                                         hovered item position
```

## Data Types

```rust
/// Sent from GUI → worker via LedWorkerSharedInput.
enum HoverPreviewRequest {
    /// Phase 1: isolated single shader with default params.
    Shader(shader_shared::Shader),
    /// Phase 2: full preset graph with saved params + live audio mod.
    Preset(conf::Preset),
}
```

## Changes by File

### main.rs

**LedWorkerSharedInput** — add field:
```rust
hover_preview_request: Option<HoverPreviewRequest>,
```

**LedWorkerSharedOutput** — add field:
```rust
led_colors_hover: Vec<LinSrgb>,
```

**LedWorkerRuntime** — add field:
```rust
led_colors_hover: Vec<LinSrgb>,
```
Initialized alongside existing preview buffers in `LedWorkerRuntime::new()`.
Resized in `sync_led_worker_buffers()`.

**render_led_worker_frame()** — after existing preview renders, add:
```rust
if let Some(ref request) = state.hover_preview_request {
    let hover_uniforms = match request {
        HoverPreviewRequest::Shader(s) => {
            // Isolated shader: Add blend, full left, zero right, white colourise.
            let mix = MixingInfo {
                left: *s,
                right: *s,
                colourise: Shader::SolidRgbColour,
                blend_mode: BlendMode::Add,
                xfade_left: 1.0,
                xfade_right: 0.0,
                params_left: ShaderParams::default(),
                params_right: ShaderParams::default(),
                params_colourise: white_colourise,
            };
            Uniforms { mix, ..uniforms.clone() }
        }
        HoverPreviewRequest::Preset(preset) => {
            preset_uniforms(state, preset)
        }
    };
    render_preset_graph(
        shader,
        &runtime.led_shader_inputs,
        &hover_uniforms,
        &runtime.led_colors,
        &mut runtime.led_colors_hover,
    );
}
```

**run_led_worker()** — read `hover_preview_request` from shared input alongside
existing fields. Write `led_colors_hover` to shared output alongside existing
buffers.

**apply_led_worker_output()** — copy `led_colors_hover` to `Model`.

**Model** — add field:
```rust
led_colors_hover: Vec<LinSrgb>,
```

**PreviewImages** — add field:
```rust
hover_id: Option<ui::image::Id>,
```

**update_preview_textures()** — create/update the hover texture using the same
pattern as left/right/colourise. Only write to texture when `led_colors_hover`
is non-empty (i.e. a hover request was active).

**UpdateContext** — add field:
```rust
preview_hover_image_id: Option<ui::image::Id>,
```

### gui.rs

**Ids** — add:
```rust
// Custom shader dropdown (left).
shader_left_button,
shader_left_list,
shader_left_scrollbar,
// Custom shader dropdown (right).
shader_right_button,
shader_right_list,
shader_right_scrollbar,
// Floating hover preview.
hover_preview_image,
```

**New state struct:**
```rust
pub struct ShaderDropdownState {
    pub is_open: bool,
}
```

Two instances added to Model (or passed through UpdateContext).

**New function — `shader_dropdown()`:**

Replaces `widget::DropDownList` for shader selectors. Implementation:

1. **Button** showing current shader name. On click, toggle `is_open`.
2. When open, **`widget::ListSelect::single()`** renders shader names (same
   styling as current dropdown: max 15 visible, scrollbar on top, same colors).
3. For each `Event::Item(item)`, check hover:
   ```rust
   let is_hovered = ui.widget_input(item.widget_id)
       .mouse()
       .map_or(false, |m| m.is_over());
   ```
4. If hovered, write `Some(HoverPreviewRequest::Shader(shader))` + store the
   item's absolute rect for positioning the floating preview.
5. On `Event::Selection`, return `Some(selected_shader)` and close dropdown.
6. Click outside → close dropdown, clear hover request.

**Floating preview widget:**

When a hover request is active and the preview texture is available:
```rust
if let (Some(image_id), Some(rect)) = (preview_hover_image_id, hovered_item_rect) {
    widget::Image::new(image_id)
        .w(COLUMN_W)
        .h(COLUMN_W * 0.3)
        .floating(true)
        .x_y(rect.right() + COLUMN_W * 0.5 + PAD, rect.y())
        .set(ids.hover_preview_image, ui);
}
```

Positioned to the right of the hovered item. Floats above all other widgets.

**Phase 2 — preset list hover:**

In `set_presets_widgets()`, the existing `Event::Item(item)` handler already
iterates items with widget IDs. Add the same hover detection:
```rust
let is_hovered = ui.widget_input(item.widget_id)
    .mouse()
    .map_or(false, |m| m.is_over());
if is_hovered {
    hover_request = Some(HoverPreviewRequest::Preset(presets.list[item.i].clone()));
    hovered_item_rect = Some(ui.rect_of(item.widget_id));
}
```

Same floating preview image, same texture, same buffer. The only difference is
the `HoverPreviewRequest` variant.

### gui.rs UpdateContext additions

```rust
pub hover_preview_request: &'a mut Option<HoverPreviewRequest>,
pub preview_hover_image_id: Option<ui::image::Id>,
pub shader_left_dropdown: &'a mut ShaderDropdownState,
pub shader_right_dropdown: &'a mut ShaderDropdownState,
```

## Shared Logic

Both phases share:
- The same `led_colors_hover` buffer (one at a time — you can't hover a shader
  dropdown and preset list simultaneously)
- The same wgpu texture and `ui::image::Id`
- The same floating `widget::Image` rendering code
- The same hover detection pattern (`ui.widget_input(id).mouse()`)
- The same worker-thread render call (`render_preset_graph`)

## Performance

- Zero cost when not hovering: the worker skips the hover render when
  `hover_preview_request` is `None`
- Single additional `render_preset_graph` call when hovering (same cost as one
  of the existing left/right/colourise previews — already proven fast enough)
- No additional threads, no additional GPU resources beyond one texture

## Cleanup

When hover ends (dropdown closes, mouse leaves list), GUI sets
`hover_preview_request = None`. Worker stops rendering the hover buffer.
Texture stays allocated but isn't written to — negligible memory cost.
