//! `MudLayout` — the full-screen shell that hosts an app bar, one or more
//! drawers, and the main content.
//!
//! Port of `MudLayout.razor.cs` (MudBlazor 8.13.0). MudBlazor's `MudLayout`
//! exposes **only** `ChildContent` plus the base members; it is a
//! `MudDrawerContainer` whose job is to provide the screen-sized area and to
//! let the children coordinate through CSS variables.
//!
//! The egui port replaces the CSS variables with a caller-owned
//! [`LayoutState`]: `show` resets it, publishes the shell's full height, and
//! hands the caller a screen-sized, gap-free `Ui`. The sub-components publish
//! their own measurements into `state` as they are declared.
//!
//! # `Class` resolution
//!
//! `Class` is a real `MudLayout` base parameter, so `self.base` is resolved
//! through `BsStyle` / `TwStyle` and applied to the shell rect: `bg-*` fills,
//! `p-*` pads the shell (shrinking the area all children see), `rounded-*`
//! rounds. `bg-background` and `bg-body` both resolve to
//! `MudPaletteBackground` — the token-to-key converter turns `background` into
//! the same key `bg-body` aliases to.
//!
//! The fill is painted directly at the screen rect (not via `Frame::show`,
//! which allocates only the content's natural size and would paint a thin
//! strip instead of the whole shell).
//!
//! # Gap-free stack
//!
//! `show` sets `item_spacing = (0, 0)` on the shell `Ui` so the app bar and
//! the row that holds the drawer and main content abut, and so the drawer and
//! main content inside a plain `ui.horizontal(..)` abut too — no stray gaps
//! that the layout arithmetic would fail to account for.
//!
//! # Composition
//!
//! ```ignore
//! MudLayout::new().show(ui, &mut self.layout, |ui, layout| {
//!     MudAppBar::new().show(ui, layout, |ui| { /* title, actions */ });
//!     ui.horizontal(|ui| {
//!         self.drawer.show(ui, &mut self.drawer_state, layout, |ui| { /* nav */ });
//!         MudMainContent::new().show(ui, layout, |ui| { /* page body */ });
//!     });
//! });
//! ```

use egui::{vec2, Align, Color32, Layout, Rect, Sense, Stroke, StrokeKind, Ui, UiBuilder};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::StyleFlavor;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

use super::layout_state::LayoutState;

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// A full-screen shell that hosts an app bar, drawers, and the main content.
#[must_use = "MudLayout does nothing until you call `.show(ui, state, ..)`"]
pub struct MudLayout {
    /// Shared component state (utility classes). Resolved into the shell
    /// fill / stroke / padding.
    pub base: ComponentBase,
    /// Bootstrap vs Tailwind resolver. Not a MudBlazor parameter — mirrors
    /// `MudDrawer` / `MudPopover`'s flavour selector.
    pub flavor: StyleFlavor,
}

impl Default for MudLayout {
    fn default() -> Self {
        Self { base: ComponentBase::new(), flavor: StyleFlavor::default() }
    }
}

impl MudLayout {
    /// Creates a layout with MudBlazor's default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the classes. Resolved through `flavor` — `bg-background`,
    /// `bg-body`, `p-*`, `rounded-*`, … all work.
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Sets the resolver flavour (Bootstrap by default).
    #[inline]
    pub fn flavor(mut self, flavor: StyleFlavor) -> Self {
        self.flavor = flavor;
        self
    }

    /// Resets `state`, publishes the shell's height, and runs `body` with a
    /// screen-sized `Ui` and `&mut state`.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        state: &mut LayoutState,
        body: impl FnOnce(&mut Ui, &mut LayoutState) -> R,
    ) -> R {
        state.reset();
        let screen = ui.ctx().input(|i| i.content_rect());
        state.shell_height = screen.height();

        // Resolve `self.base` into a Frame — this is what makes `bg-*` /
        // `p-*` / `rounded-*` contribute.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);
        let frame = match self.flavor {
            StyleFlavor::Bootstrap => {
                let mut bs = BsStyle::default();
                bs.build_frame(&self.base, &lookup)
            }
            StyleFlavor::Tailwind => {
                let mut tw_state = TwState::default();
                tw_state.dark = ui.visuals().dark_mode;
                let mut tw = TwStyle::default();
                tw.build_frame(&self.base, tw_state, REM_SIZE_PT, &lookup)
            }
        };

        // 1. Paint the resolved fill / stroke **at the full screen rect**.
        //
        // `Frame::show` allocates only the content's natural size, so relying
        // on it would paint a thin strip at the top of the shell instead of
        // filling the whole screen. Paint manually here, then pad the content
        // by `frame.inner_margin` below.
        if ui.is_rect_visible(screen) {
            if frame.fill != Color32::TRANSPARENT {
                ui.painter()
                    .rect_filled(screen, frame.corner_radius, frame.fill);
            }
            if frame.stroke != Stroke::NONE {
                ui.painter().rect_stroke(
                    screen,
                    frame.corner_radius,
                    frame.stroke,
                    StrokeKind::Inside,
                );
            }
        }

        // 2. Reserve the full screen in the parent so nothing else overlaps.
        let _ = ui.allocate_rect(screen, Sense::hover());

        // 3. Content child, inset by `frame.inner_margin` so `p-*` works.
        let inner_rect = Rect::from_min_max(
            screen.min
                + vec2(
                frame.inner_margin.left as f32,
                frame.inner_margin.top as f32,
            ),
            screen.max
                - vec2(
                frame.inner_margin.right as f32,
                frame.inner_margin.bottom as f32,
            ),
        );
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(inner_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        // Gapless stack: the app bar and the content row abut, and a plain
        // `ui.horizontal(..)` inside the row is gap-free too. Each child owns
        // its own internal padding.
        child.spacing_mut().item_spacing = vec2(0.0, 0.0);
        body(&mut child, state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_empty_base() {
        let l: MudLayout = MudLayout::new();
        assert!(l.base.class_names().is_empty());
    }

    #[test]
    fn classes_are_stored() {
        let l = MudLayout::new().classes(["bg-surface"]);
        assert!(l.base.has_class("bg-surface"));
    }

    #[test]
    fn classes_accumulate_across_calls() {
        let l = MudLayout::new()
            .classes(["bg-background"])
            .classes(["p-4"]);
        assert_eq!(l.base.class_names().len(), 2);
    }

    #[test]
    fn flavor_defaults_to_bootstrap() {
        let l = MudLayout::new();
        assert!(matches!(l.flavor, StyleFlavor::Bootstrap));
    }

    #[test]
    fn flavor_builder_stores() {
        let l = MudLayout::new().flavor(StyleFlavor::Tailwind);
        assert!(matches!(l.flavor, StyleFlavor::Tailwind));
    }
}