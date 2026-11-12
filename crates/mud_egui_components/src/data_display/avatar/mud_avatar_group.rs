//! `MudAvatarGroup` — stacks a row of [`MudAvatar`] children with configurable
//! overlap, overflow handling (`+N` badge), and shared styling.
//!
//! Port of `MudAvatarGroup.razor.cs` (MudBlazor 8.13.0).
//!
//! # Parameters (parity with MudBlazor)
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `Max`               | [`MudAvatarGroup::max`] | `5` |
//! | `Total`             | [`MudAvatarGroup::total`] | `None` |
//! | `Spacing`           | [`MudAvatarGroup::spacing`] | `3` |
//! | `Size`              | [`MudAvatarGroup::size`] | [`Size::Medium`] |
//! | `Variant`           | [`MudAvatarGroup::variant`] | [`Variant::Filled`] |
//! | `Color`             | [`MudAvatarGroup::color`] | [`Color::Default`] |
//! | `Square`            | [`MudAvatarGroup::square`] | `false` |
//! | `Rounded`           | [`MudAvatarGroup::rounded`] | `false` |
//! | `Elevation`         | [`MudAvatarGroup::elevation`] | `0` |
//!
//! `max = 0` or `max = usize::MAX` shows every avatar (no `+N` badge).
//!
//! # Overlap
//!
//! Each `spacing` unit contributes 4 px of horizontal overlap — matching
//! MudBlazor's `ml-nX` margin-left classes (`spacing(3)` → 12 px).
//!
//! # Ring
//!
//! When [`variant`](MudAvatarGroup::variant) is [`Variant::Outlined`], each
//! child avatar is rendered `ringed`, which adds a 2 px surface-coloured
//! ring to every avatar (total +4 px per cell). [`avatar_cell_for`] accounts
//! for this so the row width and positions stay correct.

use egui::{vec2, pos2, Rect, Response, Sense, Ui};
use mud_egui_core::enums::{Color, Size, Variant};

use crate::data_display::avatar::mud_avatar::{MudAvatar, RING_THICKNESS};

/// Pixels contributed to overlap per `spacing` unit.
pub(crate) const OVERLAP_PER_SPACING_UNIT: f32 = 4.0;

/// Extra size added to each cell for the [`Variant::Outlined`] ring.
pub(crate) const RING_EXTRA: f32 = RING_THICKNESS * 2.0;

/// One avatar item to be displayed inside the group.
pub struct AvatarItem {
    /// The content of this avatar.
    pub content: AvatarContentInput,
    /// Optional color override; falls back to the group color when `None`.
    pub color: Option<Color>,
}

/// Content of one avatar in the group.
pub enum AvatarContentInput {
    /// A Material-style SVG path (MudBlazor's `Svg`).
    Svg(String),
    /// Short text (e.g. initials).
    Text(String),
}

impl AvatarItem {
    /// Creates a text avatar (initials).
    pub fn text(t: impl Into<String>) -> Self {
        Self {
            content: AvatarContentInput::Text(t.into()),
            color: None,
        }
    }

    /// Creates an SVG avatar (Material icon path).
    pub fn svg(svg: impl Into<String>) -> Self {
        Self {
            content: AvatarContentInput::Svg(svg.into()),
            color: None,
        }
    }

    /// Deprecated alias for [`svg`](Self::svg).
    #[deprecated(note = "renamed to `svg` for MudBlazor parity")]
    pub fn icon(svg: impl Into<String>) -> Self {
        Self::svg(svg)
    }

    /// Sets a per-item color override.
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

/// A layout panel that stacks [`MudAvatar`] members horizontally with
/// configurable overlap.
#[must_use = "MudAvatarGroup does nothing until you call `.show(ui, |g| ...)`"]
pub struct MudAvatarGroup {
    /// Maximum number of avatars shown before the `+N` overflow badge
    /// appears. `0` and `usize::MAX` mean “show all”.
    pub max: usize,
    /// Total people in the group; if larger than the number of items, the
    /// overflow badge shows `total - visible`.
    pub total: Option<usize>,
    /// Overlap spacing in units of 4 px.
    pub spacing: u8,
    /// Shared color applied to all avatars unless overridden per-item.
    pub color: Color,
    /// Shared size for all avatars.
    pub size: Size,
    /// Shared variant.
    pub variant: Variant,
    /// Shared elevation.
    pub elevation: u8,
    /// Shared rounded flag.
    pub rounded: bool,
    /// Shared square flag.
    pub square: bool,
}

impl Default for MudAvatarGroup {
    fn default() -> Self {
        Self {
            max: 5,
            total: None,
            spacing: 3,
            color: Color::Default,
            size: Size::Medium,
            variant: Variant::Filled,
            elevation: 0,
            rounded: false,
            square: false,
        }
    }
}

impl MudAvatarGroup {
    /// Creates a group with defaults matching MudBlazor (`max = 5`,
    /// `spacing = 3`, `Size::Medium`, `Variant::Filled`).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of visible avatars. `0` or `usize::MAX`
    /// shows all.
    #[inline]
    pub fn max(mut self, n: usize) -> Self {
        self.max = n;
        self
    }

    /// Sets the total number of people (may exceed the number of items).
    #[inline]
    pub fn total(mut self, n: usize) -> Self {
        self.total = Some(n);
        self
    }

    /// Sets the overlap spacing in units of 4 px.
    #[inline]
    pub fn spacing(mut self, units: u8) -> Self {
        self.spacing = units;
        self
    }

    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    #[inline]
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[inline]
    pub fn elevation(mut self, e: u8) -> Self {
        self.elevation = e;
        self
    }

    #[inline]
    pub fn rounded(mut self, b: bool) -> Self {
        self.rounded = b;
        self
    }

    #[inline]
    pub fn square(mut self, b: bool) -> Self {
        self.square = b;
        self
    }

    /// Draws the group horizontally.
    ///
    /// The closure receives a builder context where you can push items via
    /// [`MudAvatarGroupCtx::item`].
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        add_items: impl FnOnce(&mut MudAvatarGroupCtx) -> R,
    ) -> Response {
        let mut ctx = MudAvatarGroupCtx { items: Vec::new() };
        let _ = add_items(&mut ctx);
        let items = ctx.items;

        let overlap = overlap_for(self.spacing);
        let visible_limit = visible_count_for(self.max, items.len());
        let total_count = self.total.unwrap_or(items.len());
        let overflow_count = total_count.saturating_sub(visible_limit);

        let cell = avatar_cell_for(self.size, self.variant);
        let visible_slots = visible_limit + if overflow_count > 0 { 1 } else { 0 };
        let total_width = row_width_for(cell, visible_slots, overlap);

        let (rect, resp) = ui.allocate_exact_size(vec2(total_width, cell), Sense::hover());

        // Paint avatars starting at the row's top-left corner.
        let mut x = rect.min.x;
        let y = rect.min.y;
        for item in items.iter().take(visible_limit) {
            draw_avatar_at(ui, x, y, item, &self);
            x += cell - overlap;
        }
        if overflow_count > 0 {
            draw_overflow_at(ui, x, y, overflow_count, &self);
        }

        resp
    }
}

/// Builder context for adding items inside [`MudAvatarGroup::show`].
pub struct MudAvatarGroupCtx {
    items: Vec<AvatarItem>,
}

impl MudAvatarGroupCtx {
    /// Adds one avatar item.
    pub fn item(&mut self, item: AvatarItem) {
        self.items.push(item);
    }
}

// =============================================================================
// Pure helpers (testable without egui::Context)
// =============================================================================

/// Overlap contributed by `spacing` units of 4 px each.
#[inline]
pub(crate) fn overlap_for(spacing: u8) -> f32 {
    spacing as f32 * OVERLAP_PER_SPACING_UNIT
}

/// Diameter of an avatar for the given [`Size`]. Mirrors
/// [`AvatarMetrics::resolve`](super::mud_avatar::AvatarMetrics).
#[inline]
pub(crate) fn avatar_diameter_for(size: Size) -> f32 {
    match size {
        Size::Small => 24.0,
        Size::Large => 56.0,
        _ => 40.0,
    }
}

/// Extra padding added by the [`Variant::Outlined`] ring. Zero for other
/// variants.
#[inline]
pub(crate) fn ring_extra_for(variant: Variant) -> f32 {
    if variant == Variant::Outlined {
        RING_EXTRA
    } else {
        0.0
    }
}

/// Full width of one avatar’s cell, including the ring extension.
#[inline]
pub(crate) fn avatar_cell_for(size: Size, variant: Variant) -> f32 {
    avatar_diameter_for(size) + ring_extra_for(variant)
}

/// Total width of the row: `cell` per slot minus `overlap` between them.
#[inline]
pub(crate) fn row_width_for(cell: f32, count: usize, overlap: f32) -> f32 {
    if count == 0 {
        0.0
    } else {
        (cell * count as f32 - overlap * (count - 1) as f32).max(0.0)
    }
}

/// Number of avatars shown before the `+N` badge. `0` and `usize::MAX`
/// both mean "show all".
#[inline]
pub(crate) fn visible_count_for(max: usize, total: usize) -> usize {
    if max == 0 || max == usize::MAX {
        total
    } else {
        max.min(total)
    }
}

// =============================================================================
// Rendering
// =============================================================================

fn draw_avatar_at(ui: &mut Ui, x: f32, y: f32, item: &AvatarItem, group: &MudAvatarGroup) {
    let cell = avatar_cell_for(group.size, group.variant);
    let rect = Rect::from_min_size(pos2(x, y), vec2(cell, cell));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));

    let mut avatar = MudAvatar::new()
        .size(group.size)
        .variant(group.variant)
        .color(item.color.unwrap_or(group.color))
        .elevation(group.elevation)
        .rounded(group.rounded)
        .square(group.square);

    if group.variant == Variant::Outlined {
        avatar = avatar.ringed(true);
    }

    match &item.content {
        AvatarContentInput::Svg(svg) => {
            avatar = avatar.svg(svg.clone());
        }
        AvatarContentInput::Text(t) => {
            avatar = avatar.text(t.clone());
        }
    }

    let _ = avatar.show(&mut child);
}

fn draw_overflow_at(ui: &mut Ui, x: f32, y: f32, count: usize, group: &MudAvatarGroup) {
    let cell = avatar_cell_for(group.size, group.variant);
    let rect = Rect::from_min_size(pos2(x, y), vec2(cell, cell));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));

    let mut avatar = MudAvatar::new()
        .size(group.size)
        .variant(group.variant)
        .color(group.color)
        .elevation(group.elevation)
        .rounded(group.rounded)
        .square(group.square)
        .text(format!("+{}", count));

    if group.variant == Variant::Outlined {
        avatar = avatar.ringed(true);
    }

    let _ = avatar.show(&mut child);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- spacing / overlap -------------------------------------------------

    #[test]
    fn overlap_scales_with_spacing() {
        assert_eq!(overlap_for(0), 0.0);
        assert_eq!(overlap_for(1), 4.0);
        assert_eq!(overlap_for(3), 12.0);
        assert_eq!(overlap_for(8), 32.0);
        assert_eq!(overlap_for(255), 1020.0);
    }

    // --- diameter ----------------------------------------------------------

    #[test]
    fn avatar_diameter_matches_avatar_metrics() {
        assert_eq!(avatar_diameter_for(Size::Small), 24.0);
        assert_eq!(avatar_diameter_for(Size::Medium), 40.0);
        assert_eq!(avatar_diameter_for(Size::Large), 56.0);
        // MudBlazor only defines S/M/L; other variants fall back to Medium.
        assert_eq!(avatar_diameter_for(Size::ExtraSmall), 40.0);
        assert_eq!(avatar_diameter_for(Size::ExtraLarge), 40.0);
    }

    // --- ring / cell -------------------------------------------------------

    #[test]
    fn ring_extra_only_for_outlined() {
        assert_eq!(ring_extra_for(Variant::Outlined), 4.0);
        assert_eq!(ring_extra_for(Variant::Filled), 0.0);
        assert_eq!(ring_extra_for(Variant::Text), 0.0);
    }

    #[test]
    fn avatar_cell_adds_ring_for_outlined() {
        assert_eq!(avatar_cell_for(Size::Medium, Variant::Outlined), 44.0);
        assert_eq!(avatar_cell_for(Size::Medium, Variant::Filled), 40.0);
        assert_eq!(avatar_cell_for(Size::Small, Variant::Outlined), 28.0);
    }

    // --- row width ---------------------------------------------------------

    #[test]
    fn row_width_zero_for_empty_group() {
        assert_eq!(row_width_for(40.0, 0, 12.0), 0.0);
    }

    #[test]
    fn row_width_single_avatar_is_cell() {
        assert_eq!(row_width_for(40.0, 1, 12.0), 40.0);
        assert_eq!(row_width_for(44.0, 1, 12.0), 44.0);
    }

    #[test]
    fn row_width_accounts_for_overlap() {
        // 3 cells × 40px, overlap 12px → 40 + 28 + 28 = 96
        assert_eq!(row_width_for(40.0, 3, 12.0), 96.0);
        // 5 cells × 40px, overlap 12px → 40 + 4 × 28 = 152
        assert_eq!(row_width_for(40.0, 5, 12.0), 152.0);
    }

    #[test]
    fn row_width_never_negative() {
        // Extreme overlap cannot drive width below zero.
        assert_eq!(row_width_for(10.0, 5, 100.0), 0.0);
    }

    // --- visible count -----------------------------------------------------

    #[test]
    fn visible_count_default_max_five() {
        assert_eq!(visible_count_for(5, 10), 5);
        assert_eq!(visible_count_for(5, 3), 3);
        assert_eq!(visible_count_for(5, 0), 0);
        assert_eq!(visible_count_for(5, 5), 5);
    }

    #[test]
    fn visible_count_zero_and_max_mean_all() {
        assert_eq!(visible_count_for(0, 10), 10);
        assert_eq!(visible_count_for(usize::MAX, 10), 10);
        assert_eq!(visible_count_for(0, 0), 0);
    }

    // --- builders ----------------------------------------------------------

    #[test]
    fn default_matches_mudblazor() {
        let g = MudAvatarGroup::new();
        assert_eq!(g.max, 5);
        assert_eq!(g.spacing, 3);
        assert!(g.total.is_none());
        assert!(matches!(g.size, Size::Medium));
        assert!(matches!(g.variant, Variant::Filled));
        assert!(matches!(g.color, Color::Default));
        assert_eq!(g.elevation, 0);
        assert!(!g.rounded);
        assert!(!g.square);
    }

    #[test]
    fn builders_set_all_fields() {
        let g = MudAvatarGroup::new()
            .max(3)
            .total(10)
            .spacing(2)
            .color(Color::Secondary)
            .size(Size::Large)
            .variant(Variant::Outlined)
            .elevation(6)
            .rounded(true)
            .square(true);
        assert_eq!(g.max, 3);
        assert_eq!(g.total, Some(10));
        assert_eq!(g.spacing, 2);
        assert!(matches!(g.color, Color::Secondary));
        assert!(matches!(g.size, Size::Large));
        assert!(matches!(g.variant, Variant::Outlined));
        assert_eq!(g.elevation, 6);
        assert!(g.rounded && g.square);
    }

    // --- item / ctx --------------------------------------------------------

    #[test]
    fn item_text_has_no_color_override() {
        let it = AvatarItem::text("AB");
        assert!(matches!(it.content, AvatarContentInput::Text(ref s) if s == "AB"));
        assert!(it.color.is_none());
    }

    #[test]
    fn item_svg_and_color() {
        let it = AvatarItem::svg("M0 0h24v24H0z").color(Color::Success);
        assert!(matches!(it.content, AvatarContentInput::Svg(ref s) if s == "M0 0h24v24H0z"));
        assert!(matches!(it.color, Some(Color::Success)));
    }

    #[test]
    fn ctx_preserves_insertion_order() {
        let mut ctx = MudAvatarGroupCtx { items: Vec::new() };
        ctx.item(AvatarItem::text("A"));
        ctx.item(AvatarItem::svg("M0 0"));
        ctx.item(AvatarItem::text("C"));
        assert_eq!(ctx.items.len(), 3);
        assert!(matches!(ctx.items[0].content, AvatarContentInput::Text(_)));
        assert!(matches!(ctx.items[1].content, AvatarContentInput::Svg(_)));
        assert!(matches!(ctx.items[2].content, AvatarContentInput::Text(_)));
    }
}