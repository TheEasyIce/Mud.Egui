//! Material Design elevation and shadow utilities for `mud_egui`.
//!
//! `MudShadows` pre-computes 25 levels of drop shadows (0 to 24) to simulate
//! depth along the z-axis.
//!
//! - **Level 0**: flat (text, flat buttons).
//! - **Level 1-8**: low-resting elements (cards, dropdowns, app bars).
//! - **Level 16-24**: floating elements (dialogs, modals, FABs).
//!
//! Written for egui 0.31+, where `Shadow` stores `offset: [i8; 2]`,
//! `blur: u8` and `spread: u8`.

use egui::{epaint::Shadow, Color32};

/// A pre-computed collection of `egui` shadows representing UI elevation.
#[derive(Debug, Clone)]
pub struct MudShadows {
    /// Index `0` is no shadow, index `24` is maximum elevation.
    pub elevations: Vec<Shadow>,
}

impl MudShadows {
    /// Highest valid elevation level.
    pub const MAX_LEVEL: usize = 24;

    /// Standard drop shadows for a light theme (soft, semi-transparent black).
    pub fn new_light() -> Self {
        Self::build(Color32::from_black_alpha(32))
    }

    /// Standard shadows for a dark theme. Black shadows vanish on dark
    /// surfaces, so a faint ivory glow is used to separate elevated surfaces.
    pub fn new_dark() -> Self {
        Self::build(Color32::from_rgba_unmultiplied(250, 249, 245, 12))
    }

    /// Returns the shadow for an elevation level. Levels above 24 are clamped.
    pub fn get(&self, level: usize) -> Shadow {
        self.elevations[level.min(Self::MAX_LEVEL)]
    }

    fn build(color: Color32) -> Self {
        let mut elevations = Vec::with_capacity(Self::MAX_LEVEL + 1);

        // Level 0: flush with the background.
        elevations.push(Shadow::NONE);

        // Levels 1..=24: blur, spread and Y offset grow with elevation.
        for i in 1..=Self::MAX_LEVEL {
            let level = i as f32;
            elevations.push(Shadow {
                offset: [0, (level * 0.5).round() as i8],
                blur: (level * 1.5).round() as u8,
                spread: (level * 0.5).round() as u8,
                color,
            });
        }

        Self { elevations }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_25_levels_and_clamps() {
        let s = MudShadows::new_light();
        assert_eq!(s.elevations.len(), 25);
        assert_eq!(s.get(0), Shadow::NONE);
        assert_eq!(s.get(99), s.get(24));
    }
}