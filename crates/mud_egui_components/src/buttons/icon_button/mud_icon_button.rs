//! An icon-only button built on top of [`MudButton`].
//!
//! Port of `MudIconButton.razor.cs` (MudBlazor 8.13.0). Text content and
//! secondary icons are intentionally impossible to set — the widget only
//! exposes what MudBlazor's `MudIconButton` actually has.

use egui::{Response, Ui, Widget};

use crate::buttons::button::mud_button::MudButton;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, Size, Variant};

#[must_use = "MudIconButton does nothing until you call `.show(ui)` or `ui.add(..)`"]
pub struct MudIconButton {
    pub icon: String,
    pub base: ComponentBase,
    pub variant: Variant,
    pub color: Color,
    pub icon_color: Color,
    pub size: Size,
    pub disabled: bool,
    pub drop_shadow: bool,
}

impl MudIconButton {
    pub fn new(icon: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            base: ComponentBase::new(),
            variant: Variant::Filled,
            color: Color::Primary,
            icon_color: Color::Default,
            size: Size::Medium,
            disabled: false,
            drop_shadow: true,
        }
    }

    #[inline] pub fn variant(mut self, variant: Variant) -> Self { self.variant = variant; self }
    #[inline] pub fn color(mut self, color: Color) -> Self { self.color = color; self }
    #[inline] pub fn icon_color(mut self, color: Color) -> Self { self.icon_color = color; self }
    #[inline] pub fn size(mut self, size: Size) -> Self { self.size = size; self }
    #[inline] pub fn disabled(mut self, disabled: bool) -> Self { self.disabled = disabled; self }
    #[inline] pub fn drop_shadow(mut self, drop_shadow: bool) -> Self { self.drop_shadow = drop_shadow; self }
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Maps a size token to the minimum width/height of the button.
    #[inline]
    fn resolve_dimension(size: Size) -> f32 {
        match size {
            Size::ExtraSmall => 32.0,
            Size::Small => 40.0,
            Size::Medium => 48.0,
            Size::Large => 56.0,
            Size::ExtraLarge => 64.0,
        }
    }

    /// Resolves the tint color based on `icon_color` + variant.
    ///
    /// Mirrors `MudIconButton.cs::ApplyIconTint`:
    /// * If `icon_color` is anything other than `Color::Default`, that wins.
    /// * Otherwise `Text` / `Outlined` uses the button's own colour.
    /// * `Filled` uses `Color::Inherit` so the icon picks up the button's
    ///   contrast foreground.
    #[inline]
    fn resolve_tint(&self) -> Color {
        if !matches!(self.icon_color, Color::Default) {
            return self.icon_color;
        }
        if matches!(self.variant, Variant::Text | Variant::Outlined) {
            self.color
        } else {
            Color::Inherit
        }
    }

    /// Converts this icon button into the underlying [`MudButton`].
    pub(crate) fn into_button(self) -> MudButton {
        let tint = self.resolve_tint();
        let classes: Vec<&str> = self.base.class_names().iter().map(|s| s.as_str()).collect();
        MudButton::new("")
            .start_icon(self.icon)
            .variant(self.variant)
            .color(self.color)
            .size(self.size)
            .disabled(self.disabled)
            .drop_shadow(self.drop_shadow)
            .icon_color(tint)
            .classes(classes)
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let dim = Self::resolve_dimension(self.size);
        let button = self.into_button().full_width(true);

        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = dim;
            ui.add_sized(egui::vec2(dim, dim), button)
        })
            .inner
    }
}

impl Widget for MudIconButton {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_defaults() {
        let b = MudIconButton::new("M12 2L2 7");
        assert_eq!(b.icon, "M12 2L2 7");
        assert!(matches!(b.variant, Variant::Filled));
        assert!(matches!(b.color, Color::Primary));
        assert!(matches!(b.icon_color, Color::Default));
        assert!(matches!(b.size, Size::Medium));
        assert!(!b.disabled && b.drop_shadow);
        assert_eq!(b.base.class_names().len(), 0);
    }

    #[test]
    fn builder_chain() {
        let b = MudIconButton::new("X")
            .variant(Variant::Outlined)
            .color(Color::Error)
            .icon_color(Color::Warning)
            .size(Size::Large)
            .disabled(true)
            .drop_shadow(false)
            .classes(["p-2"]);
        assert!(matches!(b.variant, Variant::Outlined));
        assert!(matches!(b.color, Color::Error));
        assert!(matches!(b.icon_color, Color::Warning));
        assert!(matches!(b.size, Size::Large));
        assert!(b.disabled && !b.drop_shadow);
        assert_eq!(b.base.class_names().len(), 1);
    }

    #[test]
    fn classes_accumulate_across_calls() {
        let b = MudIconButton::new("X").classes(["p-1"]).classes(["m-2", "bg-primary"]);
        assert_eq!(b.base.class_names().len(), 3);
    }

    #[test]
    fn resolve_dimension_maps_correctly() {
        assert_eq!(MudIconButton::resolve_dimension(Size::ExtraSmall), 32.0);
        assert_eq!(MudIconButton::resolve_dimension(Size::Small), 40.0);
        assert_eq!(MudIconButton::resolve_dimension(Size::Medium), 48.0);
        assert_eq!(MudIconButton::resolve_dimension(Size::Large), 56.0);
        assert_eq!(MudIconButton::resolve_dimension(Size::ExtraLarge), 64.0);
    }

    #[test]
    fn resolve_tint_defaults_by_variant() {
        let text_btn = MudIconButton::new("X").variant(Variant::Text).color(Color::Error);
        assert!(matches!(text_btn.resolve_tint(), Color::Error));

        let out_btn = MudIconButton::new("X").variant(Variant::Outlined).color(Color::Warning);
        assert!(matches!(out_btn.resolve_tint(), Color::Warning));

        let fill_btn = MudIconButton::new("X").variant(Variant::Filled).color(Color::Info);
        assert!(matches!(fill_btn.resolve_tint(), Color::Inherit));
    }

    #[test]
    fn resolve_tint_respects_icon_color_override() {
        let b = MudIconButton::new("X")
            .variant(Variant::Filled)
            .color(Color::Info)
            .icon_color(Color::Warning);
        assert!(matches!(b.resolve_tint(), Color::Warning));
    }
}