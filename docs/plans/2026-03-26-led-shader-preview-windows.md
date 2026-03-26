# LED Shader Preview Windows Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Embed small LED preview images inside the GUI columns under "LED Shader Left" and "LED Shader Right" headings, showing each shader's isolated output before crossfading/blending — like a DJ deck preview.

**Architecture:** The worker thread computes left-only and right-only LED colors alongside the existing blended output by running the shader with isolated MixingInfo (Add blend, full xfade on one side, zero on the other). These buffers are synced to the main thread and rendered as GPU textures displayed via Conrod's `widget::Image` in each column.

**Tech Stack:** nannou 0.18, nannou_conrod 0.18, conrod_core 0.76.1, conrod_wgpu 0.76.1, wgpu (via nannou), rayon

---

### Task 1: Add left/right color buffers to worker thread

**Files:**
- Modify: `cohen_gig/src/main.rs` — `LedWorkerRuntime`, `LedWorkerSharedOutput`, `LedWorkerRuntime::new`, `sync_led_worker_buffers`, `run_led_worker`

**Step 1: Add buffers to `LedWorkerRuntime`**

In `LedWorkerRuntime` (line ~895), add two new buffers:

```rust
struct LedWorkerRuntime {
    shader: Option<Shader>,
    led_colors: Vec<LinSrgb>,
    led_color_buffer: Vec<LinSrgb>,
    led_outputs: Vec<LinSrgb>,
    led_colors_left: Vec<LinSrgb>,   // NEW
    led_colors_right: Vec<LinSrgb>,  // NEW
    led_shader_inputs: Vec<CachedLedShaderInput>,
    // ... rest unchanged
}
```

**Step 2: Add buffers to `LedWorkerSharedOutput`**

In `LedWorkerSharedOutput` (line ~132), add:

```rust
struct LedWorkerSharedOutput {
    frame_id: u64,
    led_colors: Vec<LinSrgb>,
    led_outputs: Vec<LinSrgb>,
    led_colors_left: Vec<LinSrgb>,   // NEW
    led_colors_right: Vec<LinSrgb>,  // NEW
    monitor: LedWorkerMonitorSnapshot,
    // ... rest unchanged
}
```

**Step 3: Initialize new buffers in `LedWorkerRuntime::new`**

Add to the `Self { ... }` block (line ~918):

```rust
led_colors_left: black_led_buffer(led_count),
led_colors_right: black_led_buffer(led_count),
```

**Step 4: Initialize new buffers in `LedWorker::new`**

Add to the `LedWorkerSharedOutput` construction (line ~368):

```rust
led_colors_left: Vec::new(),
led_colors_right: Vec::new(),
```

**Step 5: Resize new buffers in `sync_led_worker_buffers`**

Inside the `if runtime.led_colors.len() != led_count` block (line ~948), add:

```rust
runtime.led_colors_left.resize(led_count, lin_srgb(0.0, 0.0, 0.0));
runtime.led_colors_right.resize(led_count, lin_srgb(0.0, 0.0, 0.0));
```

**Step 6: Publish new buffers in `run_led_worker`**

Inside the `if let Ok(mut output) = shared_output.lock()` block (line ~1013), add:

```rust
output.led_colors_left.clone_from(&runtime.led_colors_left);
output.led_colors_right.clone_from(&runtime.led_colors_right);
```

**Step 7: Build and verify it compiles**

Run: `cargo build -p cohen_gig 2>&1 | tail -5`
Expected: compiles (warnings about unused fields are fine)

**Step 8: Commit**

```
git add cohen_gig/src/main.rs
git commit -m "Add left/right LED color buffers to worker thread"
```

---

### Task 2: Compute isolated left/right shader outputs

**Files:**
- Modify: `cohen_gig/src/main.rs` — `render_led_worker_frame`

**Step 1: Add isolated left/right shader passes after the main shader pass**

After the existing `std::mem::swap(&mut runtime.led_colors, &mut runtime.led_color_buffer)` (line ~1140), and before the preset crossfade lerp section, add two additional passes that re-run the shader with isolated mixing:

```rust
// Compute isolated left preview (Add blend, left only).
{
    let left_only_mix = MixingInfo {
        left: mix_info.left,
        right: mix_info.right,
        colourise: mix_info.colourise,
        blend_mode: shader_shared::BlendMode::Add,
        xfade_left: 1.0,
        xfade_right: 0.0,
    };
    let left_uniforms = Uniforms {
        mix: left_only_mix,
        ..uniforms.clone()
    };
    runtime
        .led_colors_left
        .par_iter_mut()
        .zip(runtime.led_shader_inputs.par_iter())
        .zip(runtime.led_colors.par_iter())
        .for_each(|((color, led_input), &last_color)| {
            let vertex = Vertex {
                position: led_input.position,
                light: led_input.light,
                last_color,
            };
            *color = shader(vertex, &left_uniforms);
        });
}

// Compute isolated right preview (Add blend, right only).
{
    let right_only_mix = MixingInfo {
        left: mix_info.left,
        right: mix_info.right,
        colourise: mix_info.colourise,
        blend_mode: shader_shared::BlendMode::Add,
        xfade_left: 0.0,
        xfade_right: 1.0,
    };
    let right_uniforms = Uniforms {
        mix: right_only_mix,
        ..uniforms.clone()
    };
    runtime
        .led_colors_right
        .par_iter_mut()
        .zip(runtime.led_shader_inputs.par_iter())
        .zip(runtime.led_colors.par_iter())
        .for_each(|((color, led_input), &last_color)| {
            let vertex = Vertex {
                position: led_input.position,
                light: led_input.light,
                last_color,
            };
            *color = shader(vertex, &right_uniforms);
        });
}
```

**Note:** This requires `Uniforms` to implement `Clone`. Check if it does — if not, `uniforms` will need to be reconstructed or the fields copied manually. The `buttons` field is a `HashMap` which is `Clone`, so this should work. If `Uniforms` doesn't derive `Clone`, we'll need to add it to `shader_shared`.

**Step 2: Check if `Uniforms` derives `Clone`**

Search `shader_shared/src/lib.rs` for the `Uniforms` struct definition. If it doesn't derive `Clone`, add `Clone` to its derive list.

Run: `cargo build -p cohen_gig 2>&1 | tail -5`
Expected: compiles successfully

**Step 3: Commit**

```
git add cohen_gig/src/main.rs shader_shared/src/lib.rs
git commit -m "Compute isolated left/right shader outputs in worker thread"
```

---

### Task 3: Sync left/right colors to main thread Model

**Files:**
- Modify: `cohen_gig/src/main.rs` — `Model`, `model()`, `apply_led_worker_output`, `sync_led_buffers`

**Step 1: Add left/right buffers to `Model`**

In the `Model` struct (line ~56), add:

```rust
led_colors_left: Vec<LinSrgb>,
led_colors_right: Vec<LinSrgb>,
```

**Step 2: Initialize in `model()` function**

In the `Model { ... }` construction (line ~522), add:

```rust
led_colors_left: black_led_buffer(initial_led_count),
led_colors_right: black_led_buffer(initial_led_count),
```

**Step 3: Sync in `apply_led_worker_output`**

After `model.led_outputs.clone_from(&shared_output.led_outputs)` (line ~859), add:

```rust
model.led_colors_left.clone_from(&shared_output.led_colors_left);
model.led_colors_right.clone_from(&shared_output.led_colors_right);
```

**Step 4: Resize in `sync_led_buffers`**

Find `sync_led_buffers` and add resizing for the new buffers alongside the existing ones. Search for where `led_colors` and `led_outputs` are resized and add matching lines for `led_colors_left` and `led_colors_right`.

**Step 5: Build and verify**

Run: `cargo build -p cohen_gig 2>&1 | tail -5`
Expected: compiles successfully

**Step 6: Commit**

```
git add cohen_gig/src/main.rs
git commit -m "Sync left/right LED preview colors to main thread"
```

---

### Task 4: Create preview textures and Conrod image IDs

**Files:**
- Modify: `cohen_gig/src/main.rs` — `Model`, `model()`
- Modify: `cohen_gig/src/gui.rs` — `widget_ids!`

**Step 1: Add widget IDs for preview images**

In `gui.rs` `widget_ids!` macro (line ~31), add two new IDs:

```rust
led_preview_left_image,
led_preview_right_image,
```

**Step 2: Add preview state to Model**

Add a struct and fields to `Model` for managing the preview textures:

```rust
struct PreviewImages {
    left_id: conrod_core::image::Id,
    right_id: conrod_core::image::Id,
    width: u32,
    height: u32,
}
```

Add to `Model`:

```rust
preview_images: Option<PreviewImages>,
```

Initialize as `None` in `model()`.

**Step 3: Build and verify**

Run: `cargo build -p cohen_gig 2>&1 | tail -5`
Expected: compiles successfully

**Step 4: Commit**

```
git add cohen_gig/src/main.rs cohen_gig/src/gui.rs
git commit -m "Add preview image widget IDs and Model state"
```

---

### Task 5: Create and update preview textures each frame

**Files:**
- Modify: `cohen_gig/src/main.rs` — `update()`

This is the core texture pipeline. Each frame we need to:
1. Determine preview dimensions from the LED layout
2. On first frame (or layout change): create GPU textures and insert into image_map
3. Every frame: convert `Vec<LinSrgb>` to RGBA bytes and upload to the textures

**Step 1: Add a helper to convert LED colors to RGBA pixel bytes**

Add a function in `main.rs`:

```rust
fn led_colors_to_rgba(colors: &[LinSrgb], width: u32, height: u32) -> Vec<u8> {
    let pixel_count = (width * height) as usize;
    let mut rgba = vec![0u8; pixel_count * 4];
    for (i, color) in colors.iter().take(pixel_count).enumerate() {
        let offset = i * 4;
        rgba[offset] = (color.red.clamp(0.0, 1.0) * 255.0) as u8;
        rgba[offset + 1] = (color.green.clamp(0.0, 1.0) * 255.0) as u8;
        rgba[offset + 2] = (color.blue.clamp(0.0, 1.0) * 255.0) as u8;
        rgba[offset + 3] = 255;
    }
    rgba
}
```

**Step 2: Add a helper to compute preview dimensions from layout**

```rust
fn preview_dimensions(
    led_layout: &conf::LedLayout,
    mad_project: Option<&mad_mapper::MadProject>,
) -> (u32, u32) {
    if let Some(project) = mad_project {
        let fixtures = project.fixtures_by_row();
        let rows = fixtures.len() as u32;
        let max_cols = fixtures.iter().map(|f| f.pixel_count as u32).max().unwrap_or(1);
        (max_cols, rows.max(1))
    } else {
        (led_layout.leds_per_row() as u32, led_layout.row_count as u32)
    }
}
```

**Step 3: Add texture creation/update logic in `update()`**

After `apply_led_worker_output(model)` and before the GUI update, add:

```rust
update_preview_textures(app, model);
```

Implement the function:

```rust
fn update_preview_textures(app: &App, model: &mut Model) {
    let (width, height) = preview_dimensions(
        &model.global_config.led_layout,
        model.mad_project.as_ref(),
    );
    if width == 0 || height == 0 {
        return;
    }

    let window = match app.window(model._gui_window) {
        Some(w) => w,
        None => return,
    };

    // Recreate textures if dimensions changed or first time.
    let needs_recreate = match &model.preview_images {
        Some(pi) => pi.width != width || pi.height != height,
        None => true,
    };

    if needs_recreate {
        let device = window.device();

        let create_texture = |label: &'static str| -> wgpu::TextureHandle {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            })
        };

        let left_tex = create_texture("preview_left");
        let right_tex = create_texture("preview_right");

        let left_img = conrod_wgpu::Image {
            texture: left_tex,
            texture_format: wgpu::TextureFormat::Rgba8UnormSrgb,
            width,
            height,
        };
        let right_img = conrod_wgpu::Image {
            texture: right_tex,
            texture_format: wgpu::TextureFormat::Rgba8UnormSrgb,
            width,
            height,
        };

        // Remove old images if they exist.
        if let Some(old) = model.preview_images.take() {
            model.ui.image_map.remove(old.left_id);
            model.ui.image_map.remove(old.right_id);
        }

        let left_id = model.ui.image_map.insert(left_img);
        let right_id = model.ui.image_map.insert(right_img);
        model.preview_images = Some(PreviewImages {
            left_id,
            right_id,
            width,
            height,
        });
    }

    // Upload current LED colors to textures.
    if let Some(ref pi) = model.preview_images {
        let queue = window.queue();
        let size = wgpu::Extent3d {
            width: pi.width,
            height: pi.height,
            depth_or_array_layers: 1,
        };

        let left_rgba = led_colors_to_rgba(&model.led_colors_left, pi.width, pi.height);
        if let Some(left_img) = model.ui.image_map.get(&pi.left_id) {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &left_img.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &left_rgba,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(pi.width * 4),
                    rows_per_image: std::num::NonZeroU32::new(pi.height),
                },
                size,
            );
        }

        let right_rgba = led_colors_to_rgba(&model.led_colors_right, pi.width, pi.height);
        if let Some(right_img) = model.ui.image_map.get(&pi.right_id) {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &right_img.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &right_rgba,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(pi.width * 4),
                    rows_per_image: std::num::NonZeroU32::new(pi.height),
                },
                size,
            );
        }
    }
}
```

**Important note on wgpu types:** The `wgpu` types used above refer to the raw wgpu crate types. In nannou, `nannou::wgpu` re-exports `nannou_wgpu`, which re-exports from `wgpu`. However, `conrod_wgpu::Image` holds a raw `wgpu::Texture` (wgpu 0.11). The implementation will need to use the correct wgpu types that match `conrod_wgpu`'s expectations. If there are type mismatches, the texture may need to be created through a compatible path. The `image_map.get()` method is available via `Deref` to the underlying `HashMap` (see `conrod_core::image::Map`'s `Deref` impl).

**Step 4: Build and verify**

Run: `cargo build -p cohen_gig 2>&1 | tail -20`
Expected: compiles (may need adjustments to wgpu type imports)

**Step 5: Commit**

```
git add cohen_gig/src/main.rs
git commit -m "Create and update preview textures each frame"
```

---

### Task 6: Display preview images in GUI columns

**Files:**
- Modify: `cohen_gig/src/gui.rs` — `update()`, `UpdateContext`

**Step 1: Pass preview image IDs to the GUI**

Add to `UpdateContext`:

```rust
pub preview_left_image_id: Option<conrod_core::image::Id>,
pub preview_right_image_id: Option<conrod_core::image::Id>,
```

Update the `gui::update()` destructure to extract these fields.

Update the call site in `main.rs` `update()` to pass:

```rust
preview_left_image_id: model.preview_images.as_ref().map(|pi| pi.left_id),
preview_right_image_id: model.preview_images.as_ref().map(|pi| pi.right_id),
```

**Step 2: Add Image widgets in the LED Shader Left column**

After the "LED Shader Left" heading text (line ~1303) and before the dropdown (line ~1314), insert:

```rust
if let Some(image_id) = preview_left_image_id {
    widget::Image::new(image_id)
        .w(COLUMN_W)
        .h(COLUMN_W * 0.3)  // aspect ratio ~3:1 for typical LED strip layouts
        .down(10.0)
        .set(ids.led_preview_left_image, ui);
}
```

**Step 3: Add Image widget in the LED Shader Right column**

After the "LED Shader Right" heading text (line ~1402) and before the dropdown (line ~1408), insert:

```rust
if let Some(image_id) = preview_right_image_id {
    widget::Image::new(image_id)
        .w(COLUMN_W)
        .h(COLUMN_W * 0.3)
        .down(10.0)
        .set(ids.led_preview_right_image, ui);
}
```

**Step 4: Build and verify**

Run: `cargo build -p cohen_gig 2>&1 | tail -5`
Expected: compiles successfully

**Step 5: Run the application and visually verify**

Run: `cargo run -p cohen_gig`

Verify:
- Two small preview images appear in columns 3 and 4
- Left preview shows only the left shader's output
- Right preview shows only the right shader's output
- Moving the left/right mix slider doesn't affect the previews (they show isolated outputs)
- Changing shaders updates the corresponding preview
- The main preview window continues to show the blended output

**Step 6: Commit**

```
git add cohen_gig/src/gui.rs cohen_gig/src/main.rs
git commit -m "Display left/right shader previews in GUI columns"
```

---

## Implementation Notes

### wgpu Type Compatibility
`conrod_wgpu` v0.76.1 depends on `wgpu` v0.11. `nannou_wgpu` v0.18.0 also wraps wgpu. These should resolve to the same version in the dependency tree, but if there are type mismatches when creating textures for `conrod_wgpu::Image`, you may need to use `nannou::wgpu` types exclusively and verify the `conrod_wgpu::Image.texture` field type matches.

### Performance Considerations
The preview computation runs the shader 3x per LED per frame (main + left + right). For typical LED counts (100-1000), this is well within budget on modern CPUs with rayon parallelism. If performance becomes an issue, the previews could be computed at a reduced rate (every Nth frame).

### Texture Format
Using `Rgba8UnormSrgb` to match the sRGB color space of `LinSrgb` values. If colors look wrong, try `Rgba8Unorm` instead.

### Preview Aspect Ratio
The `h(COLUMN_W * 0.3)` gives a ~3:1 preview. Adjust based on actual LED layouts. For MadMapper layouts with many rows, a taller preview might be better. Could compute dynamically from `preview_dimensions()`.
