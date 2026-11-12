//! A collapsible panel with a clickable header.
//!
//! Port of `MudExpansionPanel.razor.cs` (MudBlazor 8.13.0). The public parameter
//! surface mirrors the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `ChildContent` | the closure passed to [`MudExpansionPanel::show`] |
//! | `Dense` | [`MudExpansionPanel::dense`] |
//! | `Disabled` | [`MudExpansionPanel::disabled`] |
//! | `Expanded` | caller-owned `&mut bool` passed to `show` |
//! | `ExpandedChanged` | [`PanelResponse::expanded_changed`] |
//! | `Gutters` | [`MudExpansionPanel::gutters`] |
//! | `HeaderClass` | [`MudExpansionPanel::header_class`] (stored, not resolved) |
//! | `HideIcon` | [`MudExpansionPanel::hide_icon`] |
//! | `Icon` | [`MudExpansionPanel::icon`] (default `filled::EXPAND_MORE`) |
//! | `KeepContentAlive` | [`MudExpansionPanel::keep_content_alive`] — inert |
//! | `MaxHeight` | [`MudExpansionPanel::max_height`] |
//! | `Text` | [`MudExpansionPanel::new`] / [`MudExpansionPanel::text`] |
//! | `TitleContent` | [`MudExpansionPanel::title_content`] (overrides `Text`) |
//!
//! `Style` and `UserAttributes` are not ported (no CSS strings, no HTML tag).
//!
//! # Two rendering modes
//!
//! | Mode | Entry point | Surface |
//! |------|-------------|---------|
//! | Standalone | [`MudExpansionPanel::show`] | Own fill, own 1 pt outline, 4 pt radius |
//! | In a container | [`super::MudExpansionPanels::show`] | Fill only; the container draws the outer border, shadow and radius; the panel draws a 1 pt divider when it is not the last |
//!
//! Splitting it this way is what lets the container look like one rounded card
//! with internal rules, which is how MudBlazor's `.mud-expansion-panels` reads.
//!
//! # Header
//!
//! ```
//! [ pad_x ][ title (Text or TitleContent) …… ][ gap ][ icon 24 ][ pad_x ]
//! ```
//!
//! The header is 48 pt tall (32 pt when dense) and the icon rotates 180° as the
//! panel expands, animated with the same duration as the content.
//!
//! # Content
//!
//! The content region is delegated to [`MudCollapse`], so the expand/collapse
//! animation, the height cache and the `MaxHeight` clamp are shared with that
//! component rather than reimplemented here.

use std::any::Any;
use std::ops::Deref;

use egui::{
    pos2, vec2, Align, Color32, CornerRadius, CursorIcon, Frame, Layout, Margin, Rect, Response,
    Sense, Stroke, StrokeKind, Ui, UiBuilder,
};

use crate::surfaces::collapse::MudCollapse;
use crate::{MudIcon, MudText};
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::Typo;
use mud_egui_theming::theme::MudTheme;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Header height in logical pixels for a standard panel.
const HEADER_HEIGHT: f32 = 48.0;

/// Header height in logical pixels for a dense panel.
const DENSE_HEADER_HEIGHT: f32 = 32.0;

/// Vertical content padding in logical pixels for a standard panel.
const CONTENT_PADDING_Y: f32 = 16.0;

/// Vertical content padding in logical pixels for a dense panel.
const DENSE_CONTENT_PADDING_Y: f32 = 8.0;

/// Left and right padding in logical pixels, applied when `gutters` is on.
const GUTTER_PX: f32 = 16.0;

/// Expand-icon square side in logical pixels for a standard panel.
const ICON_SIZE: f32 = 24.0;

/// Expand-icon square side in logical pixels for a dense panel.
const DENSE_ICON_SIZE: f32 = 20.0;

/// Gap between the title area and the expand icon, in logical pixels.
const ICON_GAP: f32 = 8.0;

/// Corner radius of a standalone panel, in logical pixels.
const PANEL_RADIUS: f32 = 4.0;

/// Corner radius of a `MudExpansionPanels` container.
pub(crate) const CONTAINER_RADIUS: f32 = 4.0;

/// Rotation, in degrees, of the expand icon when the panel is expanded.
const ICON_ROTATION_DEG: f32 = 180.0;

/// Highest elevation accepted by the container; values above this are clamped.
pub(crate) const MAX_ELEVATION: u8 = 24;

/// Width of the panel outline and of the in-container divider, in logical px.
pub(crate) const HAIRLINE: f32 = 1.0;

/// Duration of the icon rotation, in seconds.
const DEFAULT_DURATION: f32 = 0.2;

// ---------------------------------------------------------------------------
// MudExpansionPanel
// ---------------------------------------------------------------------------

/// A collapsible panel with a clickable header.
///
/// Mirrors `MudExpansionPanel` in MudBlazor 8.13.0. The Blazor two-way binding
/// `Expanded` / `ExpandedChanged` becomes a caller-owned `&mut bool` plus a
/// [`PanelResponse::expanded_changed`] report, the same split `MudCollapse` and
/// `MudAlert` use.
#[must_use = "MudExpansionPanel does nothing until you call `.show(..)`"]
pub struct MudExpansionPanel {
    /// Shared component state (utility classes). Stored for parity.
    pub base: ComponentBase,
    /// Header caption. Ignored when [`title_content`](Self::title_content) is
    /// set. Mirrors MudBlazor `Text`.
    pub text: String,
    /// Custom header content. Overrides [`text`](Self::text). Mirrors
    /// MudBlazor `TitleContent`.
    pub title_content: Option<Box<dyn FnMut(&mut Ui) + 'static>>,
    /// Compact header and content padding. Mirrors MudBlazor `Dense`.
    pub dense: bool,
    /// Ignores header clicks. Mirrors MudBlazor `Disabled`.
    pub disabled: bool,
    /// Left and right padding. Mirrors MudBlazor `Gutters` (default `true`).
    pub gutters: bool,
    /// CSS classes for the header. Stored, not resolved. Mirrors MudBlazor
    /// `HeaderClass`.
    pub header_class: Option<String>,
    /// Hides the expand icon. Mirrors MudBlazor `HideIcon`.
    pub hide_icon: bool,
    /// Expand glyph. Mirrors MudBlazor `Icon` (default `ExpandMore`).
    pub icon: Option<String>,
    /// Kept for parity; has no effect in egui. See the module docs.
    pub keep_content_alive: bool,
    /// Visible content height cap, in points. Mirrors MudBlazor `MaxHeight`.
    pub max_height: Option<f32>,
    /// Stable identity for the collapse animation. Required when two panels
    /// share a parent, which is why the container always sets it.
    pub id_salt: Option<egui::Id>,
    /// Arbitrary user data. Mirrors MudBlazor `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl Default for MudExpansionPanel {
    /// Empty caption, expanded layout, gutters on, `Dense`/`Disabled` off.
    fn default() -> Self {
        Self {
            base: ComponentBase::new(),
            text: String::new(),
            title_content: None,
            dense: false,
            disabled: false,
            gutters: true,
            header_class: None,
            hide_icon: false,
            icon: None,
            keep_content_alive: false,
            max_height: None,
            id_salt: None,
            tag: None,
        }
    }
}

impl MudExpansionPanel {
    /// Creates a panel whose header shows `text`.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }

    /// Replaces the header caption. Mirrors `Text`.
    #[inline]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Replaces the header with custom content. Overrides
    /// [`text`](Self::text). Mirrors `TitleContent`.
    #[inline]
    pub fn title_content(mut self, f: impl FnMut(&mut Ui) + 'static) -> Self {
        self.title_content = Some(Box::new(f));
        self
    }

    /// Enables compact padding. Mirrors `Dense`.
    #[inline]
    pub fn dense(mut self, b: bool) -> Self {
        self.dense = b;
        self
    }

    /// Ignores header clicks. Mirrors `Disabled`.
    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// Enables left and right padding. Mirrors `Gutters`.
    #[inline]
    pub fn gutters(mut self, b: bool) -> Self {
        self.gutters = b;
        self
    }

    /// Sets header CSS classes. Mirrors `HeaderClass`.
    #[inline]
    pub fn header_class(mut self, class: impl Into<String>) -> Self {
        self.header_class = Some(class.into());
        self
    }

    /// Hides the expand icon. Mirrors `HideIcon`.
    #[inline]
    pub fn hide_icon(mut self, b: bool) -> Self {
        self.hide_icon = b;
        self
    }

    /// Overrides the expand glyph. Mirrors `Icon`.
    #[inline]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Stored for parity; no effect in egui. Mirrors `KeepContentAlive`.
    #[inline]
    pub fn keep_content_alive(mut self, b: bool) -> Self {
        self.keep_content_alive = b;
        self
    }

    /// Caps the visible content height in points. Mirrors `MaxHeight`.
    #[inline]
    pub fn max_height(mut self, px: f32) -> Self {
        self.max_height = Some(px);
        self
    }

    /// Sets a stable identity for the collapse animation.
    #[inline]
    pub fn id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_salt = Some(egui::Id::new(salt));
        self
    }

    /// Adds utility classes to the panel's [`ComponentBase`].
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Attaches arbitrary user data. Mirrors `Tag`.
    #[inline]
    pub fn tag<T: Any + Send + Sync>(mut self, tag: T) -> Self {
        self.tag = Some(Box::new(tag));
        self
    }

    /// Reads user data attached with [`tag`](Self::tag).
    #[inline]
    pub fn tag_ref<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.tag.as_ref().and_then(|b| b.downcast_ref::<T>())
    }

    /// Draws the panel on its own: surface fill, 1 pt outline, 4 pt radius.
    ///
    /// Use this when the panel is not inside [`super::MudExpansionPanels`].
    /// Inside a container, use the container's `show` so the panels share one
    /// border and one shadow.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        expanded: &mut bool,
        add_content: impl FnOnce(&mut Ui) -> R,
    ) -> PanelResponse<R> {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let fill: Color32 = palette.surface.into();
        let border = ui.visuals().widgets.noninteractive.bg_stroke.color;

        Frame::new()
            .fill(fill)
            .stroke(Stroke::new(HAIRLINE, border))
            .corner_radius(CornerRadius::same(radius_u8(PANEL_RADIUS)))
            .show(ui, |ui| {
                self.render(ui, PanelChrome::standalone(), expanded, add_content)
            })
            .inner
    }

    /// Draws the panel without its own surface.
    ///
    /// `pub(crate)` because only [`super::MudExpansionPanels`] should call it:
    /// the container owns the border, shadow and radius, and asks the panel for
    /// a divider instead of an outline.
    pub(crate) fn render<R>(
        mut self,
        ui: &mut Ui,
        chrome: PanelChrome,
        expanded: &mut bool,
        add_content: impl FnOnce(&mut Ui) -> R,
    ) -> PanelResponse<R> {
        let dense = self.dense || chrome.force_dense;
        let gutters = self.gutters && chrome.allow_gutters;
        let was_expanded = *expanded;

        // --- header -------------------------------------------------------
        let width = placed_width(ui.available_width());
        let hh = header_height(dense);
        let sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let header_start = ui.cursor().min;
        let (header_rect, header_resp) =
            ui.allocate_exact_size(vec2(width, hh), sense);
        let header_resp = if self.disabled {
            header_resp
        } else {
            header_resp.on_hover_cursor(CursorIcon::PointingHand)
        };

        let clicked = header_resp.clicked();
        *expanded = next_expanded(*expanded, clicked, self.disabled);
        let toggled = *expanded != was_expanded;

        // Header hover wash, behind the content.
        if header_resp.hovered() && !self.disabled {
            ui.painter().rect_filled(
                header_rect,
                0.0,
                ui.visuals().widgets.hovered.weak_bg_fill,
            );
        }

        let pad_x = horizontal_padding(gutters);
        let icon_px = icon_size(dense);
        let icon_gap = if self.hide_icon { 0.0 } else { ICON_GAP };
        let icon_w = if self.hide_icon { 0.0 } else { icon_px };
        let title_right = (header_rect.right() - pad_x - icon_w - icon_gap).max(header_rect.left());
        let title_rect = Rect::from_min_max(
            pos2(header_rect.left() + pad_x, header_rect.top()),
            pos2(title_right, header_rect.bottom()),
        );

        if let Some(render) = self.title_content.as_mut() {
            let mut child = ui.new_child(
                UiBuilder::new()
                    .max_rect(title_rect)
                    .layout(Layout::left_to_right(Align::Center)),
            );
            render(&mut child);
        } else {
            let mut child = ui.new_child(
                UiBuilder::new()
                    .max_rect(title_rect)
                    .layout(Layout::left_to_right(Align::Center)),
            );
            child.add(MudText::new(std::mem::take(&mut self.text)).typo(Typo::Subtitle1));
        }

        if !self.hide_icon {
            let progress = if self.disabled {
                bool_progress(ui.ctx(), header_resp.id.with("icon_rot"), *expanded)
            } else {
                bool_progress(ui.ctx(), header_resp.id.with("icon_rot"), *expanded)
            };
            let glyph = self
                .icon
                .take()
                .unwrap_or_else(|| mud_egui_icons_material::filled::EXPAND_MORE.to_owned());
            let icon_rect = Rect::from_center_size(
                pos2(
                    header_rect.right() - pad_x - icon_w * 0.5,
                    header_rect.center().y,
                ),
                vec2(icon_px, icon_px),
            );
            ui.put(
                icon_rect,
                MudIcon::new(glyph)
                    .size_px(icon_px)
                    .rotation(ICON_ROTATION_DEG * progress),
            );
        }

        // --- content ------------------------------------------------------
        let mut collapse = MudCollapse::new()
            .expanded(*expanded)
            .duration(DEFAULT_DURATION)
            .id_salt(header_resp.id.with("content"));
        if let Some(px) = self.max_height {
            collapse = collapse.max_height(px);
        }
        let collapse_out = collapse.show(ui, |ui| {
            egui::Frame::new()
                .inner_margin(Margin::symmetric(
                    0,
                    padding_i8(content_padding_y(dense)),
                ))
                .show(ui, add_content)
                .inner
        });

        // --- in-container divider -----------------------------------------
        if chrome.in_container && !chrome.last {
            let y = ui.cursor().min.y;
            ui.painter().rect_filled(
                Rect::from_min_size(pos2(header_start.x, y - HAIRLINE), vec2(width, HAIRLINE)),
                0.0,
                ui.visuals().widgets.noninteractive.bg_stroke.color,
            );
        }

        PanelResponse {
            response: header_resp,
            inner: collapse_out.inner,
            expanded: *expanded,
            toggled,
            expanded_changed: toggled.then_some(*expanded),
        }
    }
}

impl egui::Widget for MudExpansionPanel {
    /// Draws an empty panel in the expanded state. Prefer
    /// [`MudExpansionPanel::show`] so the state is yours to own.
    fn ui(self, ui: &mut Ui) -> Response {
        let mut expanded = true;
        self.show(ui, &mut expanded, |_| ()).response
    }
}

// ---------------------------------------------------------------------------
// PanelChrome
// ---------------------------------------------------------------------------

/// Container-supplied rendering overrides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PanelChrome {
    /// Container `Dense`, OR-ed onto each panel.
    pub force_dense: bool,
    /// Container `Gutters`, AND-ed onto each panel.
    pub allow_gutters: bool,
    /// Whether the panel is inside a container at all.
    pub in_container: bool,
    /// Whether this is the last panel (skips the divider).
    pub last: bool,
}

impl PanelChrome {
    /// The mode used by [`MudExpansionPanel::show`].
    #[inline]
    pub(crate) const fn standalone() -> Self {
        Self {
            force_dense: false,
            allow_gutters: true,
            in_container: false,
            last: true,
        }
    }
}

// ---------------------------------------------------------------------------
// PanelResponse
// ---------------------------------------------------------------------------

/// What [`MudExpansionPanel::show`] did this frame.
pub struct PanelResponse<R> {
    /// The header's `Response`. `PanelResponse` dereferences to it.
    pub response: Response,
    /// The content closure's value; `None` while the content is fully
    /// collapsed.
    pub inner: Option<R>,
    /// The panel's expansion state after this frame.
    pub expanded: bool,
    /// `true` when the header toggled the state this frame.
    pub toggled: bool,
    /// `Some(new)` on the frame the header toggled the state. Mirrors
    /// MudBlazor `ExpandedChanged`.
    pub expanded_changed: Option<bool>,
}

impl<R> Deref for PanelResponse<R> {
    type Target = Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (no `egui::Context` required)
// ---------------------------------------------------------------------------

/// Header height for the given density.
#[inline]
pub(crate) fn header_height(dense: bool) -> f32 {
    if dense {
        DENSE_HEADER_HEIGHT
    } else {
        HEADER_HEIGHT
    }
}

/// Vertical content padding for the given density.
#[inline]
pub(crate) fn content_padding_y(dense: bool) -> f32 {
    if dense {
        DENSE_CONTENT_PADDING_Y
    } else {
        CONTENT_PADDING_Y
    }
}

/// Left and right padding for the given gutter setting.
#[inline]
pub(crate) fn horizontal_padding(gutters: bool) -> f32 {
    if gutters {
        GUTTER_PX
    } else {
        0.0
    }
}

/// Expand-icon square side for the given density.
#[inline]
pub(crate) fn icon_size(dense: bool) -> f32 {
    if dense {
        DENSE_ICON_SIZE
    } else {
        ICON_SIZE
    }
}

/// The state after a header click.
///
/// Mirrors MudBlazor's `ToggleExpansionAsync`: a disabled panel never changes,
/// and an enabled panel always flips.
#[inline]
pub(crate) fn next_expanded(expanded: bool, clicked: bool, disabled: bool) -> bool {
    if clicked && !disabled {
        !expanded
    } else {
        expanded
    }
}

/// Icon rotation in degrees for the given expansion state.
#[inline]
fn icon_rotation(progress: f32) -> f32 {
    ICON_ROTATION_DEG * progress.clamp(0.0, 1.0)
}

/// Clamps a pixel value into `u8` for `CornerRadius`.
#[inline]
pub(crate) fn radius_u8(px: f32) -> u8 {
    if px.is_finite() {
        px.clamp(0.0, 255.0) as u8
    } else {
        0
    }
}

/// Clamps a pixel value into `i8` for `Margin`.
#[inline]
pub(crate) fn padding_i8(px: f32) -> i8 {
    radius_u8(px).min(127) as i8
}

/// Replaces a non-finite width with `0`.
#[inline]
pub(crate) fn placed_width(width: f32) -> f32 {
    if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    }
}

/// Animated `bool` progress in `0..=1`, keyed by `id`.
#[inline]
fn bool_progress(ctx: &egui::Context, id: egui::Id, value: bool) -> f32 {
    ctx.animate_bool_with_time(id, value, DEFAULT_DURATION)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_height_follows_density() {
        assert_eq!(header_height(false), HEADER_HEIGHT);
        assert_eq!(header_height(true), DENSE_HEADER_HEIGHT);
        assert!(header_height(true) < header_height(false));
    }

    #[test]
    fn content_padding_follows_density() {
        assert_eq!(content_padding_y(false), CONTENT_PADDING_Y);
        assert_eq!(content_padding_y(true), DENSE_CONTENT_PADDING_Y);
        assert!(content_padding_y(true) < content_padding_y(false));
    }

    #[test]
    fn horizontal_padding_follows_gutters() {
        assert_eq!(horizontal_padding(true), GUTTER_PX);
        assert_eq!(horizontal_padding(false), 0.0);
    }

    #[test]
    fn icon_size_follows_density() {
        assert_eq!(icon_size(false), ICON_SIZE);
        assert_eq!(icon_size(true), DENSE_ICON_SIZE);
        assert!(icon_size(true) < icon_size(false));
    }

    #[test]
    fn next_expanded_truth_table() {
        assert!(next_expanded(false, true, false));
        assert!(!next_expanded(true, true, false));
        assert!(!next_expanded(false, false, false));
        assert!(next_expanded(true, false, false));
    }

    #[test]
    fn next_expanded_ignores_clicks_when_disabled() {
        assert!(!next_expanded(false, true, true));
        assert!(next_expanded(true, true, true));
    }

    #[test]
    fn icon_rotation_spans_zero_to_one_eighty() {
        assert_eq!(icon_rotation(0.0), 0.0);
        assert_eq!(icon_rotation(0.5), ICON_ROTATION_DEG * 0.5);
        assert_eq!(icon_rotation(1.0), ICON_ROTATION_DEG);
    }

    #[test]
    fn icon_rotation_clamps_and_sanitizes() {
        assert_eq!(icon_rotation(-1.0), 0.0);
        assert_eq!(icon_rotation(2.0), ICON_ROTATION_DEG);
        assert_eq!(icon_rotation(f32::NAN), 0.0);
    }

    #[test]
    fn radius_helpers_sanitize() {
        assert_eq!(radius_u8(4.0), 4);
        assert_eq!(radius_u8(-1.0), 0);
        assert_eq!(radius_u8(300.0), 255);
        assert_eq!(radius_u8(f32::NAN), 0);
        assert_eq!(padding_i8(200.0), 127);
        assert_eq!(padding_i8(8.0), 8);
    }

    #[test]
    fn placed_width_sanitizes() {
        assert_eq!(placed_width(400.0), 400.0);
        assert_eq!(placed_width(-1.0), 0.0);
        assert_eq!(placed_width(f32::NAN), 0.0);
        assert_eq!(placed_width(f32::INFINITY), 0.0);
    }

    #[test]
    fn chrome_standalone_has_no_container_overrides() {
        let c = PanelChrome::standalone();
        assert!(!c.force_dense);
        assert!(c.allow_gutters);
        assert!(!c.in_container);
        assert!(c.last);
    }

    #[test]
    fn defaults_match_mudblazor() {
        let p = MudExpansionPanel::new("Details");
        assert_eq!(p.text, "Details");
        assert!(p.title_content.is_none());
        assert!(!p.dense);
        assert!(!p.disabled);
        assert!(p.gutters);
        assert!(p.header_class.is_none());
        assert!(!p.hide_icon);
        assert!(p.icon.is_none());
        assert!(!p.keep_content_alive);
        assert!(p.max_height.is_none());
        assert!(p.id_salt.is_none());
        assert!(p.tag.is_none());
        assert!(p.base.class_names().is_empty());
    }

    #[test]
    fn builders_set_every_field() {
        let p = MudExpansionPanel::new("start")
            .text("Title")
            .title_content(|ui| {
                ui.add(MudText::new("custom"));
            })
            .dense(true)
            .disabled(true)
            .gutters(false)
            .header_class("hdr")
            .hide_icon(true)
            .icon("M0 0")
            .keep_content_alive(true)
            .max_height(120.0)
            .id_salt("p1")
            .classes(["mb-2"])
            .tag("panel");
        assert_eq!(p.text, "Title");
        assert!(p.title_content.is_some());
        assert!(p.dense && p.disabled && p.hide_icon && p.keep_content_alive);
        assert!(!p.gutters);
        assert_eq!(p.header_class.as_deref(), Some("hdr"));
        assert_eq!(p.icon.as_deref(), Some("M0 0"));
        assert_eq!(p.max_height, Some(120.0));
        assert!(p.id_salt.is_some());
        assert!(!p.base.class_names().is_empty());
        assert_eq!(p.tag_ref::<&str>(), Some(&"panel"));
        assert!(p.tag_ref::<u32>().is_none());
    }

    #[test]
    fn id_salt_is_stable_across_calls() {
        let a = MudExpansionPanel::new("x").id_salt("s");
        let b = MudExpansionPanel::new("x").id_salt("s");
        assert_eq!(a.id_salt, b.id_salt);
        assert_ne!(a.id_salt, MudExpansionPanel::new("x").id_salt("t").id_salt);
    }
}