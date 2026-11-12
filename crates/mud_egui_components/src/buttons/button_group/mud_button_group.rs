//! A layout panel that aggregates related [`MudButton`] children into a unified
//! structural block.
//!
//! Port of `MudButtonGroup.cs`. Renders buttons in a horizontal or vertical row,
//! sharing `color`, `size`, `variant` and `drop_shadow` across the group when
//! `override_styles` is `true`, and inserts a 1 px hairline separator between
//! adjacent buttons for every variant except `Outlined`.
//!
//! # Design notes
//!
//! * **Collection pattern (like `MudGrid`).** `show` runs the caller's closure
//!   once to collect `MudButton`s, applies group styling, then renders them in
//!   a single pass. This is the only way to know the button count upfront —
//!   required for `full_width` distribution and separator placement.
//! * **`override_styles` is boolean, not per-property.** egui has no binding
//!   priorities; the C# "local values win" rule cannot be expressed. The port
//!   simplifies to: `override_styles = true` overwrites every collected button's
//!   `variant` / `color` / `size` / `drop_shadow`; `false` leaves them alone.
//! * **Corner radius is not distributed** per head/tail button — `MudButton`
//!   does not currently expose a per-instance `corner_radius` builder. The field
//!   is stored for API parity with MudBlazor and documented as a limitation.
//! * **Separator color** resolves through the active palette, falling back to
//!   `Ui::visuals().widgets.noninteractive.bg_stroke.color`.

use std::hash::Hash;

use egui::{
    Color32, Id, InnerResponse, Response, Sense, Ui, Vec2, vec2, Widget,
};

use crate::buttons::button::mud_button::MudButton;
use mud_egui_core::base::component_base::ComponentBase;
use crate::foundation::utilities::Utilities;
use mud_egui_core::enums::{Color, Size, Variant};
use mud_egui_theming::theme::MudTheme;

/// Hairline thickness between two adjacent buttons.
const SEPARATOR_THICKNESS: f32 = 1.0;

/// Default group corner radius (matches MudBlazor).
const DEFAULT_CORNER_RADIUS: f32 = 4.0;

// =============================================================================
// Component
// =============================================================================

/// A horizontal or vertical group of buttons that share styling and are visually
/// joined by hairline separators.
///
/// ```ignore
/// MudButtonGroup::new()
///     .variant(Variant::Filled)
///     .color(Color::Primary)
///     .show(ui, |group| {
///         group.button(MudButton::new("Reply"));
///         group.button(MudButton::new("Reply All"));
///         group.button(MudButton::new("Forward"));
///     });
/// ```
#[must_use = "MudButtonGroup does nothing until you call `.show(ui, |g| ...)`"]
pub struct MudButtonGroup {
    /// Shared component state (utility classes).
    pub base: ComponentBase,
    /// Variant pushed to every collected button when `override_styles` is on.
    /// Also controls whether separators are drawn (`Outlined` → none).
    pub variant: Variant,
    /// Palette role pushed to every collected button when `override_styles` is on.
    pub color: Color,
    /// Size pushed to every collected button when `override_styles` is on.
    pub size: Size,
    /// Stack top-to-bottom (true) or side-by-side (false).
    pub vertical: bool,
    /// Stretch to fill the offered width, splitting it evenly across buttons.
    pub full_width: bool,
    /// Whether member buttons cast an elevation shadow.
    pub drop_shadow: bool,
    /// Reverse visual order in the horizontal layout.
    pub right_to_left: bool,
    /// Whether the group overwrites each button's own variant/color/size/drop_shadow.
    pub override_styles: bool,
    /// Group corner radius. **Currently has no effect** in this port — see module docs.
    pub corner_radius: f32,
    /// Stable identity for stacks sharing an auto-id.
    pub id_salt: Option<Id>,
}

impl Default for MudButtonGroup {
    fn default() -> Self {
        Self {
            base: ComponentBase::new(),
            variant: Variant::Text,
            color: Color::Primary,
            size: Size::Medium,
            vertical: false,
            full_width: false,
            drop_shadow: true,
            right_to_left: false,
            override_styles: true,
            corner_radius: DEFAULT_CORNER_RADIUS,
            id_salt: None,
        }
    }
}

impl MudButtonGroup {
    /// Creates a default (horizontal, text-variant, primary) group.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    // =========================================================================
    // Builder Methods
    // =========================================================================

    #[inline]
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    #[inline]
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    #[inline]
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    #[inline]
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    #[inline]
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    #[inline]
    pub fn drop_shadow(mut self, drop_shadow: bool) -> Self {
        self.drop_shadow = drop_shadow;
        self
    }

    #[inline]
    pub fn right_to_left(mut self, rtl: bool) -> Self {
        self.right_to_left = rtl;
        self
    }

    #[inline]
    pub fn override_styles(mut self, override_styles: bool) -> Self {
        self.override_styles = override_styles;
        self
    }

    #[inline]
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Add utility classes: `p-*`, `m-*`, `bg-*`.
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Distinguishes two groups that would otherwise share an auto id.
    #[inline]
    pub fn id_salt(mut self, salt: impl Hash + std::fmt::Debug) -> Self {
        #[allow(deprecated)]
        let id = Id::new(salt);
        self.id_salt = Some(id);
        self
    }

    // =========================================================================
    // Public API
    // =========================================================================

    /// Renders the group. `add_buttons` runs exactly once, collecting
    /// [`MudButton`] instances; each is then rendered into the group's layout.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_buttons: impl FnOnce(&mut ButtonGroupUi<'_>) -> R,
    ) -> InnerResponse<R> {
        let mut buttons = Vec::new();
        let value = {
            let mut group_ui = ButtonGroupUi {
                buttons: &mut buttons,
                group: &self,
            };
            add_buttons(&mut group_ui)
        };

        // Empty group: return a zero-sized hover response at the current cursor.
        if buttons.is_empty() {
            let rect = ui.available_rect_before_wrap();
            let response = ui.allocate_rect(rect, Sense::hover());
            return InnerResponse::new(value, response);
        }

        let utilities = self.utilities();
        let response = if utilities.is_empty() {
            self.render(ui, buttons)
        } else {
            let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
            let palette = theme.current_palette();

            let mut frame = egui::Frame::new()
                .inner_margin(utilities.padding.to_margin())
                .outer_margin(utilities.margin.to_margin());
            if let Some(bg) = utilities.background.and_then(|r| r.resolve(palette)) {
                frame = frame.fill(bg);
            }

            frame.show(ui, |ui| self.render(ui, buttons)).inner
        };

        InnerResponse::new(value, response)
    }

    // =========================================================================
    // Internal
    // =========================================================================

    fn utilities(&self) -> Utilities {
        Utilities::parse(self.base.class_names().iter().map(AsRef::as_ref))
    }

    /// Applies group styling to `button` when `override_styles` is on.
    fn apply_styles(&self, button: MudButton) -> MudButton {
        if !self.override_styles {
            return button;
        }
        button
            .variant(self.variant)
            .color(self.color)
            .size(self.size)
            .drop_shadow(self.drop_shadow)
    }

    /// Whether this variant renders separators between buttons.
    /// `Outlined` groups draw no separators (they rely on border overlap).
    #[inline]
    fn needs_separators(&self) -> bool {
        !matches!(self.variant, Variant::Outlined)
    }

    /// Resolves the separator color from the active palette.
    fn resolve_separator_color(&self, ui: &Ui) -> Color32 {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        Color::Default
            .resolve(palette)
            .unwrap_or_else(|| ui.visuals().widgets.noninteractive.bg_stroke.color)
    }

    /// Splits rendering into horizontal / vertical, natural / full-width.
    fn render(&self, ui: &mut Ui, buttons: Vec<MudButton>) -> Response {
        let sep_color = self.resolve_separator_color(ui);

        if self.vertical {
            self.render_vertical(ui, buttons, sep_color)
        } else if self.full_width {
            self.render_horizontal_full(ui, buttons, sep_color)
        } else {
            self.render_horizontal_natural(ui, buttons, sep_color)
        }
    }

    fn render_horizontal_natural(
        &self,
        ui: &mut Ui,
        buttons: Vec<MudButton>,
        sep_color: Color32,
    ) -> Response {
        let show_sep = self.needs_separators();
        let rtl = self.right_to_left;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            let h = ui.available_height();

            let iter: Box<dyn Iterator<Item = MudButton>> = if rtl {
                Box::new(buttons.into_iter().rev())
            } else {
                Box::new(buttons.into_iter())
            };

            let mut first = true;
            for button in iter {
                if !first && show_sep {
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(SEPARATOR_THICKNESS, h), Sense::hover());
                    ui.painter().rect_filled(rect, 0.0, sep_color);
                }
                ui.add(button);
                first = false;
            }
        })
            .response
    }

    fn render_horizontal_full(
        &self,
        ui: &mut Ui,
        buttons: Vec<MudButton>,
        sep_color: Color32,
    ) -> Response {
        let show_sep = self.needs_separators();
        let rtl = self.right_to_left;
        let n = buttons.len();
        let sep_count = if show_sep && n > 1 { n - 1 } else { 0 };

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            let avail_w = ui.available_width();
            let avail_h = ui.available_height();
            let sep_total = SEPARATOR_THICKNESS * sep_count as f32;
            let seg_w = ((avail_w - sep_total) / n.max(1) as f32).max(0.0);

            let iter: Box<dyn Iterator<Item = MudButton>> = if rtl {
                Box::new(buttons.into_iter().rev())
            } else {
                Box::new(buttons.into_iter())
            };

            let mut first = true;
            for button in iter {
                if !first && show_sep {
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(SEPARATOR_THICKNESS, avail_h), Sense::hover());
                    ui.painter().rect_filled(rect, 0.0, sep_color);
                }
                let btn = button.full_width(true);
                let layout = egui::Layout::left_to_right(egui::Align::Center);
                ui.allocate_ui_with_layout(vec2(seg_w, avail_h), layout, |ui| {
                    ui.add(btn);
                });
                first = false;
            }
        })
            .response
    }

    fn render_vertical(
        &self,
        ui: &mut Ui,
        buttons: Vec<MudButton>,
        sep_color: Color32,
    ) -> Response {
        let show_sep = self.needs_separators();
        let full = self.full_width;

        // 1. Measure max intrinsic width if not full_width.
        //    We render each button in an invisible child UI to get its natural
        //    width without affecting the parent layout.
        let target_width = if full {
            ui.available_width()
        } else {
            let mut max_w = 0.0_f32; // <-- Explicitly type this as f32
            let measure_rect = egui::Rect::from_min_size(
                ui.next_widget_position(),
                egui::vec2(ui.available_width(), 0.0),
            );
            for btn in &buttons {
                let mut child_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(measure_rect)
                        .layout(egui::Layout::top_down(egui::Align::LEFT))
                        .invisible(),
                );
                // Force full_width(false) to measure the natural width.
                child_ui.add(btn.clone().full_width(false));
                max_w = max_w.max(child_ui.min_rect().width());
            }
            max_w
        };

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            let mut first = true;
            for button in buttons {
                if !first && show_sep {
                    let (rect, _) = ui.allocate_exact_size(
                        vec2(target_width, SEPARATOR_THICKNESS),
                        Sense::hover(),
                    );
                    ui.painter().rect_filled(rect, 0.0, sep_color);
                }

                // Force full_width(true) and render inside a fixed-width child.
                let btn = button.full_width(true);
                ui.allocate_ui_with_layout(
                    vec2(target_width, 0.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.add(btn);
                    },
                );
                first = false;
            }
        })
            .response
    }
}

impl Widget for MudButtonGroup {
    /// Renders an empty group (useful for its utility-class frame).
    /// Real content should go through [`MudButtonGroup::show`].
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui, |_| {}).response
    }
}

// =============================================================================
// ButtonGroupUi
// =============================================================================

/// Collector handle passed to [`MudButtonGroup::show`].
///
/// Every [`Self::button`] call applies the group's `override_styles` rules and
/// appends the button to the pending render list. The buttons are rendered in
/// order after the closure returns.
pub struct ButtonGroupUi<'a> {
    buttons: &'a mut Vec<MudButton>,
    group: &'a MudButtonGroup,
}

impl ButtonGroupUi<'_> {
    /// Registers one button. Its `variant` / `color` / `size` / `drop_shadow`
    /// are overwritten with the group's values when `override_styles` is on.
    pub fn button(&mut self, button: MudButton) {
        self.buttons.push(self.group.apply_styles(button));
    }

    /// Convenience: builds a button via `f` and registers it.
    pub fn button_with(&mut self, f: impl FnOnce(MudButton) -> MudButton) {
        let button = f(MudButton::new(""));
        self.button(button);
    }

    /// Number of buttons collected so far.
    #[inline]
    pub fn len(&self) -> usize {
        self.buttons.len()
    }

    /// Whether no buttons have been collected yet.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buttons.is_empty()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_mudblazor() {
        let g = MudButtonGroup::default();
        assert!(!g.vertical);
        assert!(!g.full_width);
        assert!(g.drop_shadow);
        assert!(!g.right_to_left);
        assert!(g.override_styles);
        assert_eq!(g.corner_radius, DEFAULT_CORNER_RADIUS);
        assert!(matches!(g.variant, Variant::Text));
        assert!(matches!(g.color, Color::Primary));
        assert!(matches!(g.size, Size::Medium));
        assert_eq!(g.base.class_names().len(), 0);
    }

    #[test]
    fn builder_chain_sets_all_fields() {
        let g = MudButtonGroup::new()
            .variant(Variant::Filled)
            .color(Color::Secondary)
            .size(Size::Large)
            .vertical(true)
            .full_width(true)
            .drop_shadow(false)
            .right_to_left(true)
            .override_styles(false)
            .corner_radius(8.0)
            .classes(["p-2"]);
        assert!(matches!(g.variant, Variant::Filled));
        assert!(matches!(g.color, Color::Secondary));
        assert!(matches!(g.size, Size::Large));
        assert!(g.vertical);
        assert!(g.full_width);
        assert!(!g.drop_shadow);
        assert!(g.right_to_left);
        assert!(!g.override_styles);
        assert_eq!(g.corner_radius, 8.0);
        assert_eq!(g.base.class_names().len(), 1);
    }

    #[test]
    fn apply_styles_overrides_when_on() {
        let g = MudButtonGroup::new()
            .variant(Variant::Filled)
            .color(Color::Error)
            .size(Size::Small)
            .drop_shadow(false);
        let btn = MudButton::new("x")
            .variant(Variant::Outlined)
            .color(Color::Info)
            .size(Size::Large)
            .drop_shadow(true);
        let applied = g.apply_styles(btn);
        assert!(matches!(applied.variant, Variant::Filled));
        assert!(matches!(applied.color, Color::Error));
        assert!(matches!(applied.size, Size::Small));
        assert!(!applied.drop_shadow);
    }

    #[test]
    fn apply_styles_preserves_when_off() {
        let g = MudButtonGroup::new().override_styles(false);
        let btn = MudButton::new("x")
            .variant(Variant::Outlined)
            .color(Color::Info);
        let applied = g.apply_styles(btn);
        assert!(matches!(applied.variant, Variant::Outlined));
        assert!(matches!(applied.color, Color::Info));
    }

    #[test]
    fn needs_separators_true_except_outlined() {
        assert!(MudButtonGroup::new().variant(Variant::Filled).needs_separators());
        assert!(MudButtonGroup::new().variant(Variant::Text).needs_separators());
        assert!(!MudButtonGroup::new().variant(Variant::Outlined).needs_separators());
    }

    #[test]
    fn classes_accumulate_across_calls() {
        let g = MudButtonGroup::new().classes(["p-2"]).classes(["m-2", "bg-primary"]);
        assert_eq!(g.base.class_names().len(), 3);
    }

    #[test]
    fn new_equals_default() {
        let a = MudButtonGroup::new();
        let b = MudButtonGroup::default();
        assert!(matches!(a.variant, Variant::Text));
        assert!(matches!(b.variant, Variant::Text));
        assert_eq!(a.corner_radius, b.corner_radius);
    }

    #[test]
    #[allow(deprecated)]
    fn id_salt_is_deterministic() {
        let a = MudButtonGroup::new().id_salt("x");
        let b = MudButtonGroup::new().id_salt("x");
        assert_eq!(a.id_salt, b.id_salt);
        assert_eq!(a.id_salt, Some(Id::new("x")));
    }
}