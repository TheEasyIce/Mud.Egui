mod image_phase;
pub(crate) mod mud_image;

pub(crate) use image_phase::ImageLoadPhase;

pub use mud_image::{
    compute_fit_size, compute_position_offset, elevation_shadow, resolve_alloc_size,
    resolve_fit, MudImage, MAX_ELEVATION,
};