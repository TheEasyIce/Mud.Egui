//! A horizontal toolbar layout panel.
//!
//! Port of `MudToolBar.cs`. Stacks children left-to-right, optionally wraps to
//! multiple rows, and applies Material Design height (64px or 48px dense) and
//! horizontal gutters (16px). Includes CSS utility class support via `ComponentBase`.
//!
//! # MudBlazor API parity
//!
//! The current MudBlazor `MudToolBar` exposes `ChildContent`, `Dense`, `Gutters`
//! and `WrapContent`, plus the `MudComponentBase` members `Class`, `Style`, `Tag`,
//! `UserAttributes` and `FieldId`. It has no value, no events, no variants, no
//! elevation (elevation belongs to `MudAppBar`) and no adornment slots, so this
//! port does not add any.
//!
//! | MudBlazor      | Rust                                | Default |
//! |----------------|-------------------------------------|---------|
//! | `ChildContent` | the closure of [`MudToolBar::show`] | — |
//! | `Dense`        | [`MudToolBar::dense`]               | `false` |
//! | `Gutters`      | [`MudToolBar::gutters`]             | `true`  |
//! | `WrapContent`  | [`MudToolBar::wrap_content`]        | `false` |
//! | `Class`        | [`MudToolBar::classes`]             | none    |
//! | `Tag`          | [`MudToolBar::tag`]                 | none    |
//! | `Style`, `UserAttributes`, `FieldId` | — | no CSS strings / HTML attributes in egui |
//!
//! Extension kept from the original port: [`MudToolBar::justify_space_between`]
//! (and the resolver selector [`MudToolBar::flavor`]).
//!
//! # Architecture
//!
//! The toolbar is a thin shell around [`MudStack`]:
//!
//! 1. resolve the utility-class `Frame` (Bootstrap or Tailwind flavor);
//! 2. add the 16 pt gutters unless the utility classes already set padding;
//! 3. inside the frame, force the content to span the full width and at least the
//!    toolbar height, then hand the children to a horizontal `MudStack`
//!    (centered, 8 pt gap, optional wrap / space-between);
//! 4. reserve the toolbar's rectangle in the parent layout.
//!
//! The numeric decisions (heights, gutters, content minimums, used width) are pure
//! functions covered by unit tests; a few headless frame tests cover the glue.
//!
//! # Example
//!
//! ```ignore
//! // Plain usage: `StackUi` derefs to `Ui`.
//! MudToolBar::new().show(ui, |ui| {
//!     ui.heading("Inbox");
//!     ui.button("Compose");
//! });
//!
//! // Space-between needs item boundaries, so wrap children in `StackUi::item`.
//! MudToolBar::new()
//!     .justify_space_between(true)
//!     .show(ui, |stack| {
//!         stack.item(|ui| { ui.heading("Inbox"); });
//!         stack.item(|ui| { ui.button("Compose"); });
//!     });
//! ```

use std::any::Any;
use std::sync::Arc;

use egui::{
    vec2, Color32, Frame, InnerResponse, Margin, Rect, Response, Sense, Ui, UiBuilder, Widget,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::align_items::AlignItems;
use mud_egui_core::enums::justify::Justify;
use mud_egui_core::enums::wrap::Wrap;
use mud_egui_core::enums::StyleFlavor;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

use crate::layout::stack::mud_stack::{MudStack, StackUi};

/// Base rem size (in points) passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Default height for a standard toolbar.
const DEFAULT_HEIGHT: f32 = 64.0;
/// Default height for a dense toolbar.
const DENSE_HEIGHT: f32 = 48.0;
/// Horizontal gutter size in points (`i8` because `egui::Margin` stores `i8`).
const GUTTER: i8 = 16;
/// Gap between toolbar children, in `MudStack` spacing units (4 px each → 8 px).
const STACK_SPACING: u8 = 2;

// =============================================================================
// Private pure helpers
// =============================================================================

/// Total toolbar height for the dense / standard variants.
#[inline]
fn toolbar_height(dense: bool) -> f32 {
    if dense {
        DENSE_HEIGHT
    } else {
        DEFAULT_HEIGHT
    }
}

/// Inner margin to use: the gutters when requested and the utility classes did
/// not already provide padding, otherwise the margin the classes resolved to.
#[inline]
fn resolve_inner_margin(current: Margin, gutters: bool) -> Margin {
    if gutters && current == Margin::ZERO {
        Margin {
            left: GUTTER,
            right: GUTTER,
            top: 0,
            bottom: 0,
        }
    } else {
        current
    }
}

#[inline]
fn vertical_extent_of(margin: Margin) -> f32 {
    f32::from(margin.top) + f32::from(margin.bottom)
}

/// Minimum height of the frame's *content* so that the whole framed toolbar
/// (content + inner margin + outer margin + stroke) is at least `total`.
///
/// Without this the frame (and any background utility class) would only be as
/// tall as its children while the toolbar reserved `total` in the parent.
#[inline]
fn content_min_height(total: f32, inner: Margin, outer: Margin, stroke_width: f32) -> f32 {
    let stroke = if stroke_width.is_finite() {
        stroke_width.max(0.0) * 2.0
    } else {
        0.0
    };
    (total - vertical_extent_of(inner) - vertical_extent_of(outer) - stroke).max(0.0)
}

/// Width to reserve in the parent: the available width when it is finite,
/// otherwise whatever the content used.
#[inline]
fn used_width(available: f32, content: f32) -> f32 {
    if available.is_finite() {
        available.max(0.0)
    } else if content.is_finite() {
        content.max(0.0)
    } else {
        0.0
    }
}

/// Height to reserve in the parent: what the framed content used, but never less
/// than the toolbar height.
#[inline]
fn used_height(content: f32, min_height: f32) -> f32 {
    if content.is_finite() {
        content.max(min_height)
    } else {
        min_height
    }
}

// =============================================================================
// Component
// =============================================================================

/// A horizontal toolbar layout panel.
///
/// # Example
///
/// ```ignore
/// MudToolBar::new().dense(true).show(ui, |ui| {
///     ui.button("Cut");
///     ui.button("Copy");
/// });
/// ```
#[must_use = "MudToolBar does nothing until you call `.show(ui, |ui| ...)`"]
pub struct MudToolBar {
    /// Uses the dense (48px) height instead of the default (64px).
    /// MudBlazor: `Dense`. Default: `false`.
    pub dense: bool,
    /// Applies 16px horizontal padding. MudBlazor: `Gutters`. Default: `true`.
    ///
    /// Skipped when the utility classes already resolve to an inner margin.
    pub gutters: bool,
    /// If `true`, children wrap to the next row if they exceed the width.
    /// MudBlazor: `WrapContent`. Default: `false`.
    pub wrap_content: bool,
    /// If `true`, free space is distributed evenly between children. *Extension.*
    ///
    /// Only affects children added with [`StackUi::item`] (it needs item
    /// boundaries) and is ignored while `wrap_content` is on, as in `MudStack`.
    pub justify_space_between: bool,

    /// Arbitrary user data. MudBlazor: `Tag`.
    tag: Option<Arc<dyn Any + Send + Sync>>,

    /// Shared component state (CSS-like utility classes, elevation, etc.).
    /// MudBlazor: `Class`.
    pub base: ComponentBase,
    /// Which utility-class system to resolve against.
    pub flavor: StyleFlavor,
    /// Bootstrap resolver cache.
    pub bs_style: BsStyle,
    /// Tailwind resolver cache.
    pub tw_style: TwStyle,
    /// State fed to the Tailwind resolver.
    pub tw_state: TwState,
}

impl Default for MudToolBar {
    fn default() -> Self {
        Self {
            dense: false,
            gutters: true,
            wrap_content: false,
            justify_space_between: false,
            tag: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }
}

impl MudToolBar {
    /// Creates a standard (64 pt) toolbar with gutters and no wrapping.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    // =========================================================================
    // Builder Methods
    // =========================================================================

    /// Uses the dense (48 pt) height.
    #[inline]
    pub fn dense(mut self, dense: bool) -> Self {
        self.dense = dense;
        self
    }

    /// Adds (or removes) the 16 pt left and right gutters.
    #[inline]
    pub fn gutters(mut self, gutters: bool) -> Self {
        self.gutters = gutters;
        self
    }

    /// Lets the content wrap onto several rows.
    #[inline]
    pub fn wrap_content(mut self, wrap_content: bool) -> Self {
        self.wrap_content = wrap_content;
        self
    }

    /// Distributes free space between children (children added with [`StackUi::item`]).
    #[inline]
    pub fn justify_space_between(mut self, justify: bool) -> Self {
        self.justify_space_between = justify;
        self
    }

    /// Selects the utility-class resolver.
    #[inline]
    pub fn flavor(mut self, flavor: StyleFlavor) -> Self {
        self.flavor = flavor;
        self
    }

    /// Adds utility classes.
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Attaches arbitrary user data to the component (MudBlazor: `Tag`).
    ///
    /// ```ignore
    /// let bar = MudToolBar::new().tag(3_u32);
    /// assert_eq!(bar.tag_ref::<u32>(), Some(&3));
    /// ```
    #[inline]
    pub fn tag<T: Any + Send + Sync>(mut self, tag: T) -> Self {
        self.tag = Some(Arc::new(tag));
        self
    }

    /// Returns the attached user data if it exists and has type `T`.
    #[inline]
    pub fn tag_ref<T: Any>(&self) -> Option<&T> {
        self.tag.as_deref().and_then(|t| t.downcast_ref::<T>())
    }

    // =========================================================================
    // Rendering
    // =========================================================================

    /// Draws the toolbar; the closure adds the children (MudBlazor `ChildContent`).
    ///
    /// The closure receives a [`StackUi`], which derefs to [`egui::Ui`], so plain widgets
    /// can be added directly. Wrap children in [`StackUi::item`] to make them measured
    /// units; `justify_space_between` needs that.
    ///
    /// ```ignore
    /// MudToolBar::new().justify_space_between(true).show(ui, |stack| {
    ///     stack.item(|ui| { ui.label("left"); });
    ///     stack.item(|ui| { ui.label("right"); });
    /// });
    /// ```
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_children: impl FnOnce(&mut StackUi<'_>) -> R,
    ) -> InnerResponse<R> {
        self.render(ui, |ui, stack| stack.show(ui, add_children))
    }

    fn build_stack(&self) -> MudStack {
        let justify = if self.justify_space_between {
            Justify::SpaceBetween
        } else {
            Justify::Start
        };
        let wrap = if self.wrap_content {
            Wrap::Wrap
        } else {
            Wrap::NoWrap
        };

        MudStack::horizontal()
            .justify(justify)
            .wrap(wrap)
            .align_items(AlignItems::Center)
            .spacing(STACK_SPACING)
    }

    fn render<R>(
        mut self,
        ui: &mut Ui,
        draw: impl FnOnce(&mut Ui, MudStack) -> InnerResponse<R>,
    ) -> InnerResponse<R> {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();

        // Resolve CSS utility frame
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);
        self.tw_state.dark = ui.visuals().dark_mode;

        let mut frame: Frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => {
                self.tw_style
                    .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup)
            }
        };

        // Apply gutters if no explicit CSS padding was provided
        frame = frame.inner_margin(resolve_inner_margin(frame.inner_margin, self.gutters));

        let total_height = toolbar_height(self.dense);
        let content_min = content_min_height(
            total_height,
            frame.inner_margin,
            frame.outer_margin,
            frame.stroke.width,
        );
        let stack = self.build_stack();

        let avail = ui.available_rect_before_wrap();
        let mut child_ui = ui.new_child(UiBuilder::new().max_rect(avail));

        let outer = frame.show(&mut child_ui, |ui| {
            // Make the framed area span the full width and the toolbar height,
            // so backgrounds from utility classes cover the whole toolbar.
            let width = ui.available_width();
            if width.is_finite() {
                ui.set_min_width(width);
            }
            ui.set_min_height(content_min);
            draw(ui, stack)
        });

        // Allocate the correct size in the *parent* layout.
        let used = child_ui.min_rect();
        let size = vec2(
            used_width(avail.width(), used.width()),
            used_height(used.height(), total_height),
        );
        let response = ui.allocate_rect(Rect::from_min_size(avail.min, size), Sense::hover());

        InnerResponse::new(outer.inner.inner, response)
    }
}

impl Widget for MudToolBar {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui, |_| {}).response
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    const EPS: f32 = 0.01;

    /// Runs `f` inside a real egui frame on a headless 400x300 context.
    fn run_in_ui<R>(f: impl FnOnce(&mut Ui) -> R) -> R {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
            ..Default::default()
        };
        let mut f = Some(f);
        let mut out = None;
        let _ = ctx.run_ui(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(f) = f.take() {
                    out = Some(f(ui));
                }
            });
        });
        out.expect("the frame closure must run exactly once")
    }

    // ---- pure helpers -------------------------------------------------------

    #[test]
    fn heights_match_material_spec() {
        assert_eq!(toolbar_height(false), 64.0);
        assert_eq!(toolbar_height(true), 48.0);
    }

    #[test]
    fn gutters_apply_only_without_explicit_padding() {
        let m = resolve_inner_margin(Margin::ZERO, true);
        assert_eq!((m.left, m.right, m.top, m.bottom), (16, 16, 0, 0));

        assert_eq!(resolve_inner_margin(Margin::ZERO, false), Margin::ZERO);

        let custom = Margin::symmetric(4, 2);
        assert_eq!(resolve_inner_margin(custom, true), custom);
        assert_eq!(resolve_inner_margin(custom, false), custom);
    }

    #[test]
    fn vertical_extent_sums_top_and_bottom() {
        let m = Margin { left: 9, right: 9, top: 3, bottom: 5 };
        assert_eq!(vertical_extent_of(m), 8.0);
        assert_eq!(vertical_extent_of(Margin::ZERO), 0.0);
    }

    #[test]
    fn content_min_height_subtracts_decorations() {
        let inner = Margin { left: 0, right: 0, top: 4, bottom: 4 };
        let outer = Margin { left: 0, right: 0, top: 2, bottom: 2 };
        assert!((content_min_height(64.0, inner, outer, 1.0) - 46.0).abs() < EPS);
    }

    #[test]
    fn content_min_height_never_negative_and_stroke_is_sanitized() {
        let big = Margin { left: 0, right: 0, top: 100, bottom: 100 };
        assert_eq!(content_min_height(48.0, big, Margin::ZERO, 0.0), 0.0);
        assert_eq!(content_min_height(48.0, Margin::ZERO, Margin::ZERO, f32::NAN), 48.0);
        assert_eq!(content_min_height(48.0, Margin::ZERO, Margin::ZERO, -3.0), 48.0);
    }

    #[test]
    fn used_width_prefers_finite_available() {
        assert_eq!(used_width(300.0, 120.0), 300.0);
        assert_eq!(used_width(f32::INFINITY, 120.0), 120.0);
        assert_eq!(used_width(f32::INFINITY, f32::NAN), 0.0);
        assert_eq!(used_width(-5.0, 10.0), 0.0);
    }

    #[test]
    fn used_height_is_at_least_the_toolbar_height() {
        assert_eq!(used_height(30.0, 64.0), 64.0);
        assert_eq!(used_height(100.0, 64.0), 100.0);
        assert_eq!(used_height(f32::NAN, 64.0), 64.0);
    }

    // ---- builder ------------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let t = MudToolBar::new();
        assert!(!t.dense);
        assert!(t.gutters);
        assert!(!t.wrap_content);
        assert!(!t.justify_space_between);
        assert!(t.tag_ref::<u32>().is_none());
    }

    #[test]
    fn builder_sets_every_parameter() {
        let t = MudToolBar::new()
            .dense(true)
            .gutters(false)
            .wrap_content(true)
            .justify_space_between(true);
        assert!(t.dense && !t.gutters && t.wrap_content && t.justify_space_between);
    }

    #[test]
    fn tag_round_trips_by_type() {
        let t = MudToolBar::new().tag("main-bar");
        assert_eq!(t.tag_ref::<&str>(), Some(&"main-bar"));
        assert_eq!(t.tag_ref::<u32>(), None);
    }

    // ---- rendering (headless frames) ---------------------------------------

    #[test]
    fn standard_toolbar_reserves_64pt_and_full_width() {
        let r = run_in_ui(|ui| MudToolBar::new().show(ui, |ui| ui.label("x")));
        assert!((r.response.rect.height() - 64.0).abs() < 1.0);
        assert!(r.response.rect.width() > 300.0);
    }

    #[test]
    fn dense_toolbar_reserves_48pt() {
        let r = run_in_ui(|ui| MudToolBar::new().dense(true).show(ui, |ui| ui.label("x")));
        assert!((r.response.rect.height() - 48.0).abs() < 1.0);
    }

    #[test]
    fn show_returns_the_closure_value() {
        let r = run_in_ui(|ui| MudToolBar::new().show(ui, |_| 7));
        assert_eq!(r.inner, 7);
    }

    #[test]
    fn show_with_items_returns_the_closure_value() {
        let r = run_in_ui(|ui| {
            MudToolBar::new()
                .justify_space_between(true)
                .show(ui, |stack| {
                    stack.item(|ui| {
                        ui.label("a");
                    });
                    stack.item(|ui| {
                        ui.label("b");
                    });
                    5
                })
        });
        assert_eq!(r.inner, 5);
        assert!(r.response.rect.height() >= 63.0);
    }

    #[test]
    fn gutters_push_children_inwards() {
        let (left, with, without) = run_in_ui(|ui| {
            let left = ui.max_rect().left();
            let a = MudToolBar::new()
                .show(ui, |ui| ui.next_widget_position().x)
                .inner;
            let b = MudToolBar::new()
                .gutters(false)
                .show(ui, |ui| ui.next_widget_position().x)
                .inner;
            (left, a, b)
        });
        assert!(with >= left + 15.0);
        assert!(without < left + 8.0);
    }

    #[test]
    fn wrapping_toolbar_is_never_shorter_than_standard() {
        let r = run_in_ui(|ui| {
            MudToolBar::new().wrap_content(true).show(ui, |ui| {
                for i in 0..40 {
                    ui.label(format!("item {i}"));
                }
            })
        });
        assert!(r.response.rect.height() >= 63.0);
    }

    #[test]
    fn widget_trait_renders_an_empty_toolbar() {
        let r = run_in_ui(|ui| ui.add(MudToolBar::new()));
        assert!((r.rect.height() - 64.0).abs() < 1.0);
    }
}