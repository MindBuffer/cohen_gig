# Hover Preview Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Live shader/preset preview popup on hover in shader dropdowns and preset list.

**Architecture:** A single `HoverPreviewRequest` enum flows from GUI → worker thread → back to GUI as a rendered color buffer → wgpu texture → floating image widget. Both shader dropdown hover (Phase 1) and preset list hover (Phase 2) share the same buffer, texture, and rendering path.

**Tech Stack:** Rust, nannou, nannou_conrod (conrod UI), wgpu, rayon, shader_shared

**Design doc:** `docs/plans/2026-03-27-hover-preview-design.md`

---

### Task 1: Add HoverPreviewRequest type and plumb through worker communication

Add the enum, thread it through shared input/output/runtime, and wire up the
copy-back path. No rendering or GUI changes yet — just the data plumbing.

**Files:**
- Modify: `cohen_gig/src/main.rs`

**Step 1: Add the HoverPreviewRequest enum and extend shared structs**

After the `LastPresetChange` struct (~line 102), add:

```rust
#[derive(Clone)]
enum HoverPreviewRequest {
    Shader(shader_shared::Shader),
    Preset(conf::Preset),
}
```

Then add fields to these structs:

`LedWorkerSharedInput` (line 149) — add:
```rust
hover_preview_request: Option<HoverPreviewRequest>,
```

`LedWorkerSharedOutput` (line 156) — add:
```rust
led_colors_hover: Vec<LinSrgb>,
```

`LedWorkerRuntime` (line 1098) — add:
```rust
led_colors_hover: Vec<LinSrgb>,
```

`Model` (line 64) — add:
```rust
led_colors_hover: Vec<LinSrgb>,
hover_preview_request: Option<HoverPreviewRequest>,
```

**Step 2: Update initialization sites**

`LedWorker::new()` (line 389) — add to `LedWorkerSharedInput` init:
```rust
hover_preview_request: None,
```

`LedWorker::new()` (line 395) — add to `LedWorkerSharedOutput` init:
```rust
led_colors_hover: Vec::new(),
```

`LedWorkerRuntime::new()` (line 1124) — add alongside other buffers:
```rust
led_colors_hover: black_led_buffer(led_count),
```

`Model` construction in `main()` (line 552) — add:
```rust
led_colors_hover: black_led_buffer(initial_led_count),
hover_preview_request: None,
```

**Step 3: Update buffer resize sites**

`sync_led_worker_buffers()` (line 1148) — inside the `if runtime.led_colors.len() != led_count` block, add:
```rust
runtime.led_colors_hover.resize(led_count, lin_srgb(0.0, 0.0, 0.0));
```

`sync_led_buffers()` (line 907) — inside the `if model.led_colors.len() != led_count` block, add:
```rust
model.led_colors_hover.resize(led_count, lin_srgb(0.0, 0.0, 0.0));
```

**Step 4: Wire up worker thread communication**

`run_led_worker()` (line 1202) — in the input lock block, extract hover request alongside existing fields:
```rust
let (state, pending_shader, pending_preset_change, hover_preview_request, shutdown) = {
    let mut input = match shared_input.lock() {
        Ok(input) => input,
        Err(_) => break,
    };
    (
        input.latest_state.clone(),
        input.pending_shader.take(),
        input.pending_preset_change.take(),
        input.hover_preview_request.clone(),
        input.shutdown,
    )
};
```

Then pass `hover_preview_request` into `render_led_worker_frame` by adding it to
`LedWorkerInputState` — actually, simpler: just pass it as a separate field to
`render_led_worker_frame`. Change the function signature:

```rust
fn render_led_worker_frame(
    state: &LedWorkerInputState,
    runtime: &mut LedWorkerRuntime,
    hover_preview_request: &Option<HoverPreviewRequest>,
)
```

In the output lock block (~line 1236), add after existing clones:
```rust
output.led_colors_hover.clone_from(&runtime.led_colors_hover);
```

**Step 5: Wire up copy-back to Model**

`apply_led_worker_output()` (line 1042) — add after existing clones:
```rust
model.led_colors_hover.clone_from(&shared_output.led_colors_hover);
```

**Step 6: Wire up GUI → worker hover request**

`queue_led_worker_update()` (line 1023) — inside the lock block, add:
```rust
shared_input.hover_preview_request = model.hover_preview_request.clone();
```

**Step 7: Build and verify compilation**

Run: `cargo build 2>&1 | head -40`

Fix any compilation errors (likely just unused variable warnings for `hover_preview_request` in `render_led_worker_frame`). The function body doesn't use it yet — that's Task 2.

**Step 8: Commit**

```
feat: add HoverPreviewRequest plumbing through worker thread
```

---

### Task 2: Render hover preview in worker thread

Add the conditional render pass in `render_led_worker_frame` that generates
`led_colors_hover` when a request is active.

**Files:**
- Modify: `cohen_gig/src/main.rs:1256` (`render_led_worker_frame`)

**Step 1: Add hover render pass**

At the end of `render_led_worker_frame()`, after the colourise preview block
(~line 1376) and before the preset transition logic (~line 1378), add:

```rust
// Hover preview: render only when a request is active.
if let Some(ref request) = hover_preview_request {
    let hover_uniforms = match request {
        HoverPreviewRequest::Shader(s) => {
            let mix = MixingInfo {
                left: *s,
                right: *s,
                colourise: shader_shared::Shader::SolidRgbColour,
                blend_mode: shader_shared::BlendMode::Add,
                xfade_left: 1.0,
                xfade_right: 0.0,
                params_left: ShaderParams::default(),
                params_right: ShaderParams::default(),
                params_colourise: white_colourise,
            };
            Uniforms { mix, ..uniforms.clone() }
        }
        HoverPreviewRequest::Preset(preset) => preset_uniforms(state, preset),
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

Note: `white_colourise` is already defined earlier in the function (~line 1273).
`uniforms` is already computed (~line 1262). Both are in scope.

**Step 2: Build and verify**

Run: `cargo build 2>&1 | head -20`

Expected: clean build.

**Step 3: Commit**

```
feat: render hover preview in worker thread
```

---

### Task 3: Add hover preview texture

Create, upload, and manage the hover preview wgpu texture alongside the existing
three preview textures.

**Files:**
- Modify: `cohen_gig/src/main.rs` (`PreviewImages`, `update_preview_textures`)

**Step 1: Add hover_id to PreviewImages**

In `PreviewImages` (line 56), add:
```rust
hover_id: ui::image::Id,
```

**Step 2: Create hover texture in update_preview_textures**

In `update_preview_textures()`, inside the `if needs_recreate` block (line 642):

After `let colourise_tex = ...` (line 664), add:
```rust
let hover_tex = create_preview_texture("preview_hover");
```

After the colourise `ui::conrod_wgpu::Image` (line 678), add:
```rust
let hover_img = ui::conrod_wgpu::Image {
    texture: hover_tex,
    texture_format: nannou::wgpu::TextureFormat::Rgba8UnormSrgb,
    width,
    height,
};
```

In the old image removal block (line 685), add:
```rust
model.ui.image_map.remove(old.hover_id);
```

After `let colourise_id = ...` (line 693), add:
```rust
let hover_id = model.ui.image_map.insert(hover_img);
```

Add `hover_id` to the `PreviewImages` construction.

**Step 3: Upload hover texture data**

After the colourise `queue.write_texture` block (~line 758), add:
```rust
if !model.led_colors_hover.is_empty() {
    let hover_rgba = led_colors_to_rgba(&model.led_colors_hover, pi.width, pi.height);
    if let Some(img) = model.ui.image_map.get(&pi.hover_id) {
        queue.write_texture(
            nannou::wgpu::ImageCopyTexture {
                texture: &img.texture,
                mip_level: 0,
                origin: nannou::wgpu::Origin3d::ZERO,
                aspect: nannou::wgpu::TextureAspect::All,
            },
            &hover_rgba,
            layout,
            size,
        );
    }
}
```

**Step 4: Pass hover image ID to GUI**

In `UpdateContext` (`gui.rs` line 172), add:
```rust
pub preview_hover_image_id: Option<ui::image::Id>,
```

In the `gui::update` destructure (`gui.rs` ~line 1206), add:
```rust
preview_hover_image_id,
```

In `update()` in `main.rs` (~line 1713), add to `UpdateContext` construction:
```rust
preview_hover_image_id: model.preview_images.as_ref().map(|pi| pi.hover_id),
```

**Step 5: Build and verify**

Run: `cargo build 2>&1 | head -20`

Expected: clean build (unused variable warning for `preview_hover_image_id` is fine).

**Step 6: Commit**

```
feat: add hover preview texture creation and upload
```

---

### Task 4: Add widget IDs and dropdown state for custom shader dropdowns

Add the new conrod widget IDs and the `ShaderDropdownState` struct. Wire the
dropdown state through `UpdateContext`.

**Files:**
- Modify: `cohen_gig/src/gui.rs` (Ids, state structs, UpdateContext)
- Modify: `cohen_gig/src/main.rs` (Model, update)

**Step 1: Add widget IDs**

In the `widget_ids!` block (`gui.rs` line 33), after `led_shader_right_ddl` (line 85), add:
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

**Step 2: Add ShaderDropdownState**

After `PresetListDragState` (`gui.rs` ~line 164), add:
```rust
#[derive(Clone, Copy, Debug, Default)]
pub struct ShaderDropdownState {
    pub is_open: bool,
}
```

**Step 3: Add hover state struct for GUI**

After `ShaderDropdownState`, add:
```rust
#[derive(Clone, Debug, Default)]
pub struct HoverPreviewState {
    /// The rect of the hovered item, for positioning the floating preview.
    pub hovered_rect: Option<nannou_conrod::Rect>,
}
```

**Step 4: Add to UpdateContext**

In `UpdateContext` (`gui.rs` line 172), add:
```rust
pub hover_preview_request: &'a mut Option<crate::HoverPreviewRequest>,
pub shader_left_dropdown: &'a mut ShaderDropdownState,
pub shader_right_dropdown: &'a mut ShaderDropdownState,
pub hover_preview_state: &'a mut HoverPreviewState,
```

Update the destructure in `gui::update()` (~line 1182) to include these new fields.

**Step 5: Add to Model and wire through update()**

In `Model` (`main.rs` line 64), add:
```rust
shader_left_dropdown: gui::ShaderDropdownState,
shader_right_dropdown: gui::ShaderDropdownState,
hover_preview_state: gui::HoverPreviewState,
```

Initialize in Model construction (~line 552):
```rust
shader_left_dropdown: gui::ShaderDropdownState::default(),
shader_right_dropdown: gui::ShaderDropdownState::default(),
hover_preview_state: gui::HoverPreviewState::default(),
```

In `update()` `UpdateContext` construction (~line 1692), add:
```rust
hover_preview_request: &mut model.hover_preview_request,
shader_left_dropdown: &mut model.shader_left_dropdown,
shader_right_dropdown: &mut model.shader_right_dropdown,
hover_preview_state: &mut model.hover_preview_state,
```

**Step 6: Build and verify**

Run: `cargo build 2>&1 | head -20`

**Step 7: Commit**

```
feat: add widget IDs and dropdown state for hover preview
```

---

### Task 5: Implement custom shader_dropdown function (Phase 1 core)

Replace `widget::DropDownList` with a custom dropdown that detects per-item
hover and emits `HoverPreviewRequest::Shader`.

**Files:**
- Modify: `cohen_gig/src/gui.rs`

**Step 1: Write the shader_dropdown function**

Add this function before `set_presets_widgets` in gui.rs:

```rust
/// Custom shader dropdown with per-item hover detection.
///
/// Returns `Some(shader)` when a selection is made.
fn shader_dropdown(
    ui: &mut UiCell,
    button_id: widget::Id,
    list_id: widget::Id,
    scrollbar_id: widget::Id,
    state: &mut ShaderDropdownState,
    current: Shader,
    shader_names: &[&str],
    hover_request: &mut Option<crate::HoverPreviewRequest>,
    hover_preview_state: &mut HoverPreviewState,
) -> Option<Shader> {
    let current_name = current.name();

    // Toggle button.
    if widget::Button::new()
        .w_h(COLUMN_W, PAD * 2.0)
        .down(10.0)
        .rgb(0.176, 0.513, 0.639)
        .label(current_name)
        .label_font_size(15)
        .label_rgb(1.0, 1.0, 1.0)
        .set(button_id, ui)
        .was_clicked()
    {
        state.is_open = !state.is_open;
    }

    if !state.is_open {
        return None;
    }

    let item_h = PAD * 2.0;
    let max_visible = 15;
    let visible_count = shader_names.len().min(max_visible);
    let list_h = item_h * visible_count as f64;

    let (mut events, scrollbar) = widget::ListSelect::single(shader_names.len())
        .flow_down()
        .item_size(item_h)
        .scrollbar_on_top()
        .w_h(COLUMN_W, list_h)
        .down(0.0)
        .set(list_id, ui);

    let mut selected = None;

    while let Some(event) = events.next(ui, |i| i == current.to_index()) {
        use nannou_conrod::widget::list_select::Event;
        match event {
            Event::Item(item) => {
                let label = shader_names[item.i];
                let is_current = item.i == current.to_index();
                let color = if is_current {
                    nannou_conrod::color::rgb(0.25, 0.6, 0.75)
                } else {
                    nannou_conrod::color::rgb(0.176, 0.513, 0.639)
                };
                let btn = widget::Button::new()
                    .border(0.0)
                    .color(color)
                    .label(label)
                    .label_font_size(13)
                    .label_color(nannou_conrod::color::WHITE);
                item.set(btn, ui);

                // Hover detection.
                if ui.widget_input(item.widget_id).mouse().is_some() {
                    if let Some(shader) = Shader::from_index(item.i) {
                        *hover_request = Some(crate::HoverPreviewRequest::Shader(shader));
                        hover_preview_state.hovered_rect = ui.rect_of(item.widget_id);
                    }
                }
            }
            Event::Selection(idx) => {
                selected = Shader::from_index(idx);
                state.is_open = false;
                *hover_request = None;
                hover_preview_state.hovered_rect = None;
            }
            _ => {}
        }
    }

    if let Some(sb) = scrollbar {
        sb.set(scrollbar_id, ui);
    }

    // Close on click outside: if mouse clicked but not on our list or button.
    if state.is_open {
        let mouse = ui.global_input().current.mouse.buttons.left();
        if mouse.is_down() {
            let mouse_xy = ui.global_input().current.mouse.xy;
            let over_button = ui.rect_of(button_id).map_or(false, |r| r.is_over(mouse_xy));
            let over_list = ui.rect_of(list_id).map_or(false, |r| r.is_over(mouse_xy));
            if !over_button && !over_list {
                state.is_open = false;
                *hover_request = None;
                hover_preview_state.hovered_rect = None;
            }
        }
    }

    selected
}
```

**Step 2: Replace left shader DropDownList with shader_dropdown**

In `gui::update()`, find the left shader dropdown (~line 1341-1359). Replace:

```rust
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
```

With:
```rust
let shader_names: Vec<_> = shader_shared::ALL_SHADERS
    .iter()
    .map(|s| s.name())
    .collect();

if let Some(shader) = shader_dropdown(
    ui,
    ids.shader_left_button,
    ids.shader_left_list,
    ids.shader_left_scrollbar,
    shader_left_dropdown,
    preset.shader_left,
    &shader_names,
    hover_preview_request,
    hover_preview_state,
) {
    preset.shader_left = shader;
}
```

**Step 3: Replace right shader DropDownList with shader_dropdown**

Find the right shader dropdown (~line 1456-1469). Replace:

```rust
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
```

With:
```rust
if let Some(shader) = shader_dropdown(
    ui,
    ids.shader_right_button,
    ids.shader_right_list,
    ids.shader_right_scrollbar,
    shader_right_dropdown,
    preset.shader_right,
    &shader_names,
    hover_preview_request,
    hover_preview_state,
) {
    preset.shader_right = shader;
}
```

**Step 4: Build and verify**

Run: `cargo build 2>&1 | head -20`

**Step 5: Run the app and test**

Run: `cargo run`

Verify:
- Left/right shader dropdowns show current shader name as a button
- Clicking opens a scrollable list of shader names
- Clicking a shader name selects it and closes the list
- Clicking outside the list closes it
- The existing preview strips still render correctly

**Step 6: Commit**

```
feat: replace shader DropDownList with custom dropdown supporting hover
```

---

### Task 6: Draw floating hover preview image

Render the hover preview texture as a floating `widget::Image` positioned next
to the hovered dropdown item.

**Files:**
- Modify: `cohen_gig/src/gui.rs` (~line 1206, after destructure, near end of update)

**Step 1: Clear hover request at frame start**

At the top of `gui::update()`, right after the destructure (~line 1206), reset
the hover state so it's only set when actually hovering this frame:

```rust
// Clear previous frame's hover state — it gets re-set by dropdown/list
// hover detection below if still hovering.
*hover_preview_request = None;
hover_preview_state.hovered_rect = None;
```

Note: The `shader_dropdown` function and preset list hover detection will set
these back to `Some` if the mouse is currently over an item.

**Step 2: Draw floating preview at end of update**

At the very end of `gui::update()` (after all shader/preset widget code), add:

```rust
// Floating hover preview image.
if let (Some(image_id), Some(rect)) = (preview_hover_image_id, hover_preview_state.hovered_rect) {
    widget::Image::new(image_id)
        .w(COLUMN_W)
        .h(COLUMN_W * 0.3)
        .floating(true)
        .x_y(rect.right() + COLUMN_W * 0.5 + PAD, rect.y())
        .set(ids.hover_preview_image, ui);
}
```

**Step 3: Build and test**

Run: `cargo run`

Verify:
- Open a shader dropdown and hover over items
- A live preview image should appear to the right of the hovered item
- The preview shows the hovered shader rendered on the LED layout
- Moving the mouse to a different item updates the preview
- Closing the dropdown hides the preview

**Step 4: Commit**

```
feat: draw floating hover preview image for shader dropdowns
```

---

### Task 7: Add preset list hover preview (Phase 2)

Add hover detection to the existing preset `ListSelect` to emit
`HoverPreviewRequest::Preset`.

**Files:**
- Modify: `cohen_gig/src/gui.rs` (`set_presets_widgets`)

**Step 1: Thread hover state through set_presets_widgets**

Update the `set_presets_widgets` signature to accept hover state:

```rust
pub fn set_presets_widgets(
    ui: &mut UiCell,
    ids: &Ids,
    global_config: &mut GlobalConfig,
    presets: &mut crate::conf::Presets,
    preset_list_drag: &mut PresetListDragState,
    last_preset_change: &mut Option<crate::LastPresetChange>,
    _led_colors: &LedColors,
    assets: &Path,
    hover_preview_request: &mut Option<crate::HoverPreviewRequest>,
    hover_preview_state: &mut HoverPreviewState,
) {
```

Update the call site in `gui::update()` (~line 1286) to pass the new args.

**Step 2: Add hover detection in Event::Item handler**

In `set_presets_widgets`, inside `Event::Item(item)` (~line 2507), after
`item.set(button, ui);` (line 2532), add:

```rust
// Hover detection for preset preview.
if ui.widget_input(item.widget_id).mouse().is_some() {
    if item.i < presets.list.len() {
        *hover_preview_request =
            Some(crate::HoverPreviewRequest::Preset(presets.list[item.i].clone()));
        hover_preview_state.hovered_rect = ui.rect_of(item.widget_id);
    }
}
```

**Step 3: Build and test**

Run: `cargo run`

Verify:
- Hover over preset list items
- The floating preview appears showing the full graph output of the hovered preset
- The preview shows left+right shaders blended through the preset's color post-processing
- Moving to a different preset updates the preview
- Audio envelope modulation is visible in the preview

**Step 4: Commit**

```
feat: add preset list hover preview (Phase 2)
```

---

### Task 8: Polish and edge cases

Handle edge cases: dropdown close-on-blur cleanup, ensure only one hover source
active at a time, and clean up any dead widget IDs.

**Files:**
- Modify: `cohen_gig/src/gui.rs`

**Step 1: Close shader dropdown when switching tabs**

At the start of the tab-switch handling in `gui::update()` (~line 1241-1272),
when a tab button is clicked, close any open dropdowns:

```rust
if ... .was_clicked() {
    *left_panel_tab = LeftPanelTab::Live;
    shader_left_dropdown.is_open = false;
    shader_right_dropdown.is_open = false;
}
```

Apply to all three tab buttons.

**Step 2: Close other dropdown when one opens**

In `shader_dropdown`, when toggling open, close the sibling. The simplest
approach: in `gui::update()`, after each `shader_dropdown` call, if that
dropdown just opened, close the other:

Actually, this is handled naturally because only one dropdown renders its list
at a time, and click-outside detection closes the open one. No extra code needed
unless testing reveals an issue.

**Step 3: Remove old DropDownList widget IDs if no longer used**

Check if `ids.led_shader_left_ddl` and `ids.led_shader_right_ddl` are still
referenced anywhere. If the only uses were the removed `DropDownList` calls,
remove them from the `widget_ids!` block to keep things clean.

**Step 4: Full test pass**

Run: `cargo run`

Test matrix:
- [ ] Left shader dropdown: open, hover, select, close-on-outside-click
- [ ] Right shader dropdown: same
- [ ] Preview appears and disappears correctly
- [ ] Preview position is reasonable (not off-screen)
- [ ] Preset list hover shows full graph preview
- [ ] Existing preview strips (left/right/colourise) still work
- [ ] Preset selection still works (click selects, transitions work)
- [ ] Preset drag-reorder still works
- [ ] No noticeable performance degradation

**Step 5: Commit**

```
feat: polish hover preview edge cases and cleanup
```
