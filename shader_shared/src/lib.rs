//! Items shared between the hotloaded shader file and the `cohen_gig` executable. This is
//! important in order to ensure types are laid out the same way between the dynamic library and
//! the exe.

/// Data that is uniform across all shader calls for a single frame.
#[repr(C)]
pub struct Uniforms {
    pub time: f32,
}
