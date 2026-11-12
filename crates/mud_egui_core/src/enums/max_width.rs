//! Container maximum width constraints for `mud_egui`.
//!
//! This module defines the `MaxWidth` enum, which is a direct port of MudBlazor's
//! max-width property. It is commonly used by layout containers, dialogs, cards,
//! and paper surfaces to restrict their maximum horizontal expansion on wide screens,
//! preserving readability and visual balance.

/// Represents standard responsive maximum width constraints for UI containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MaxWidth {
    /// Disables the maximum width constraint entirely, allowing the container
    /// to expand freely to fill 100% of the available horizontal space.
    False,

    /// Extra Small max-width constraint (typically ~444px).
    /// Ideal for compact dialogs, alerts, or small cards.
    Xs,

    /// Small max-width constraint (typically ~600px).
    Sm,

    /// Medium max-width constraint (typically ~960px).
    Md,

    /// Large max-width constraint (typically ~1280px).
    /// This is the standard default for general page layouts and content containers.
    #[default]
    Lg,

    /// Extra Large max-width constraint (typically ~1920px).
    Xl,

    /// Double Extra Large max-width constraint (typically ~2560px).
    /// Used for expansive dashboards or ultra-wide displays.
    Xxl,
}