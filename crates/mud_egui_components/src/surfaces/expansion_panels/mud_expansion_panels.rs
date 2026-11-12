//! A container that manages a set of [`MudExpansionPanel`]s.
//!
//! Port of `MudExpansionPanels.razor.cs` (MudBlazor 8.13.0). The public
//! parameter surface mirrors the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `ChildContent` | the closure passed to [`MudExpansionPanels::show`] |
//! | `Dense` | [`MudExpansionPanels::dense`] — OR-ed onto every panel |
//! | `Elevation` | [`MudExpansionPanels::elevation`] (default `1`) |
//! | `Gutters` | [`MudExpansionPanels::gutters`] — AND-ed onto every panel |
//! | `MultiExpansion` | [`MudExpansionPanels::multi_expansion`] |
//! | `Outlined` | [`MudExpansionPanels::outlined`] (default `true`) |
//! | `Square` | [`MudExpansionPanels::square`] |
//!
//! `Style` and `UserAttributes` are not ported.
//!
//! # Cascading
//!
//! `Dense` and `Gutters` are non-nullable `bool`s on both the container and the
//! panel, so the cascade is a boolean fold rather than an "unset means inherit"
//! rule:
//!
//! | Container | Panel | Effective |
//! |-----------|-------|-----------|
//! | `Dense(false)` | `Dense(false)` | standard |
//! | `Dense(true)` | `Dense(false)` | **dense** (container wins) |
//! | `Dense(false)` | `Dense(true)` | **dense** |
//! | `Gutters(true)` | `Gutters(true)` | padded |
//! | `Gutters(false)` | `Gutters(true)` | **no padding** (container wins) |
//! | `Gutters(true)` | `Gutters(false)` | **no padding** |
//!
//! A panel can therefore opt into density but not opt out of it, and can opt
//! out of gutters but not opt in. That is the only reading consistent with
//! MudBlazor's wording ("Uses compact padding for **all** panels" /
//! "Adds left and right padding **to all** panels") for non-nullable flags.
//!
//! # State
//!
//! Expansion state is caller-owned: a `&mut Vec<bool>` indexed by declaration
//! order. It is resized to the panel count at the start of every frame, so
//! adding or removing panels between frames is safe; reordering them shifts
//! which flag belongs to which panel, exactly as it does in Blazor.
//!
//! # `ExpandAllAsync` and friends
//!
//! The three bulk methods are ported as free functions over `&mut [bool]`:
//! [`expand_all`], [`collapse_all`], [`collapse_all_except`]. Call them on the
//! caller-owned vector before or after `show`:

use std::any::Any;

use egui::{pos2, vec2, Color32, CornerRadius, Frame, InnerResponse, Response, Stroke, Ui};

use super::mud_expansion_panel::{
    radius_u8, MudExpansionPanel, PanelChrome, CONTAINER_RADIUS, HAIRLINE, MAX_ELEVATION,
};
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_theming::theme::MudTheme;

// ---------------------------------------------------------------------------
// MudExpansionPanels
// ---------------------------------------------------------------------------

/// A container that manages a set of [`MudExpansionPanel`]s.
///
/// Mirrors `MudExpansionPanels` in MudBlazor 8.13.0. By default it is an
/// accordion: opening one panel closes the others. Set
/// [`multi_expansion`](Self::multi_expansion) to allow several at once.
#[must_use = "MudExpansionPanels does nothing until you call `.show(..)`"]
pub struct MudExpansionPanels {
    /// Shared component state (utility classes). Stored for parity.
    pub base: ComponentBase,
    /// Compact padding for every panel. Mirrors MudBlazor `Dense`.
    pub dense: bool,
    /// Drop-shadow depth for the whole stack, clamped to 24. Mirrors
    /// MudBlazor `Elevation` (default `1`).
    pub elevation: u8,
    /// Left and right padding for every panel. Mirrors MudBlazor `Gutters`
    /// (default `true`).
    pub gutters: bool,
    /// Allow several panels open at once. Mirrors MudBlazor `MultiExpansion`.
    pub multi_expansion: bool,
    /// Draw a border around the stack. Mirrors MudBlazor `Outlined`
    /// (default `true`).
    pub outlined: bool,
    /// Square corners on the stack. Mirrors MudBlazor `Square`.
    pub square: bool,
    /// Arbitrary user data. Mirrors MudBlazor `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl Default for MudExpansionPanels {
    /// Accordion, elevation 1, gutters on, outlined, rounded — MudBlazor's
    /// defaults.
    fn default() -> Self {
        Self {
            base: ComponentBase::new(),
            dense: false,
            elevation: 1,
            gutters: true,
            multi_expansion: false,
            outlined: true,
            square: false,
            tag: None,
        }
    }
}

impl MudExpansionPanels {
    /// Creates a container with MudBlazor's defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables compact padding on every panel. Mirrors `Dense`.
    #[inline]
    pub fn dense(mut self, b: bool) -> Self {
        self.dense = b;
        self
    }

    /// Sets the drop-shadow depth (clamped to 24). Mirrors `Elevation`.
    #[inline]
    pub fn elevation(mut self, e: u8) -> Self {
        self.elevation = e;
        self
    }

    /// Enables left and right padding on every panel. Mirrors `Gutters`.
    #[inline]
    pub fn gutters(mut self, b: bool) -> Self {
        self.gutters = b;
        self
    }

    /// Allows several panels to be open at once. Mirrors `MultiExpansion`.
    #[inline]
    pub fn multi_expansion(mut self, b: bool) -> Self {
        self.multi_expansion = b;
        self
    }

    /// Draws a border around the stack. Mirrors `Outlined`.
    #[inline]
    pub fn outlined(mut self, b: bool) -> Self {
        self.outlined = b;
        self
    }

    /// Removes the corner rounding. Mirrors `Square`.
    #[inline]
    pub fn square(mut self, b: bool) -> Self {
        self.square = b;
        self
    }

    /// Adds utility classes to the container's [`ComponentBase`].
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

    /// Draws the stack and reports which panel (if any) toggled this frame.
    ///
    /// `state` is resized to the number of registered panels before anything is
    /// drawn, so a panel added this frame starts collapsed.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        state: &mut Vec<bool>,
        add_panels: impl FnOnce(&mut ExpansionPanelsUi<'_, '_>) -> R,
    ) -> PanelSetResponse<R> {
        let mut entries: Vec<PanelEntry<'_>> = Vec::new();
        let inner = {
            let mut builder = ExpansionPanelsUi {
                entries: &mut entries,
            };
            add_panels(&mut builder)
        };

        sync_state_len(state, entries.len());

        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let fill: Color32 = palette.surface.into();
        let border = ui.visuals().widgets.noninteractive.bg_stroke.color;

        let mut frame = Frame::new()
            .fill(fill)
            .corner_radius(CornerRadius::same(container_radius(self.square)));
        if self.outlined {
            frame = frame.stroke(Stroke::new(HAIRLINE, border));
        }
        if let Some(shadow) = elevation_shadow(self.elevation) {
            frame = frame.shadow(shadow);
        }

        let count = entries.len();
        let multi = self.multi_expansion;
        let mut toggled: Option<usize> = None;

        let out: InnerResponse<R> = frame.show(ui, |ui| {
            for (index, entry) in entries.drain(..).enumerate() {
                let before = state[index];
                let chrome = PanelChrome {
                    force_dense: self.dense,
                    allow_gutters: self.gutters,
                    in_container: true,
                    last: index + 1 >= count,
                };
                let PanelEntry { panel, content } = entry;
                let res = panel.render(ui, chrome, &mut state[index], content);
                if res.toggled {
                    toggled = Some(index);
                    if !multi && res.expanded {
                        collapse_all_except(state, index);
                    }
                }
                let _ = before;
            }
            inner
        });

        PanelSetResponse {
            response: out.response,
            inner: out.inner,
            toggled,
        }
    }
}

// ---------------------------------------------------------------------------
// ExpansionPanelsUi
// ---------------------------------------------------------------------------

/// Collector passed to [`MudExpansionPanels::show`].
///
/// Panels are registered first and rendered afterwards, which is what makes
/// accordion enforcement and `outlined` border painting correct within a single
/// frame: the container knows how many panels there are before drawing the
/// first one.
pub struct ExpansionPanelsUi<'p, 'a> {
    entries: &'p mut Vec<PanelEntry<'a>>,
}

/// One registered panel plus its content closure.
///
/// The closure borrows for `'a` — the lifetime of the `show` call — so panels
/// can capture local state instead of being forced to `'static`.
struct PanelEntry<'a> {
    panel: MudExpansionPanel,
    content: Box<dyn FnOnce(&mut Ui) + 'a>,
}

impl<'p, 'a> ExpansionPanelsUi<'p, 'a> {
    /// Registers one panel and its content.
    ///
    /// Nothing is drawn yet; rendering happens after the closure returns.
    pub fn add(&mut self, panel: MudExpansionPanel, content: impl FnOnce(&mut Ui) + 'a) {
        self.entries.push(PanelEntry {
            panel,
            content: Box::new(content),
        });
    }

    /// Number of panels registered so far.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no panel has been registered yet.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ---------------------------------------------------------------------------
// PanelSetResponse
// ---------------------------------------------------------------------------

/// What [`MudExpansionPanels::show`] did this frame.
pub struct PanelSetResponse<R> {
    /// The container's `Response` (the whole stack, border included).
    pub response: Response,
    /// The builder closure's return value.
    pub inner: R,
    /// Index of the panel whose header toggled this frame, if any.
    pub toggled: Option<usize>,
}

// ---------------------------------------------------------------------------
// Bulk operations — ports of ExpandAllAsync / CollapseAllAsync / ...
// ---------------------------------------------------------------------------

/// Opens every panel. Port of `ExpandAllAsync`.
pub fn expand_all(state: &mut [bool]) {
    for flag in state.iter_mut() {
        *flag = true;
    }
}

/// Closes every panel. Port of `CollapseAllAsync`.
pub fn collapse_all(state: &mut [bool]) {
    for flag in state.iter_mut() {
        *flag = false;
    }
}

/// Closes every panel except `keep`. Port of `CollapseAllExceptAsync`.
///
/// Out-of-range indices are ignored (nothing is collapsed), which is the
/// behaviour a `@ref` to a panel that has not rendered yet would produce.
pub fn collapse_all_except(state: &mut [bool], keep: usize) {
    if keep >= state.len() {
        return;
    }
    for (index, flag) in state.iter_mut().enumerate() {
        *flag = index == keep;
    }
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Resizes `state` to `len`, defaulting new entries to collapsed.
#[inline]
fn sync_state_len(state: &mut Vec<bool>, len: usize) {
    state.resize(len, false);
}

/// Corner radius of the container in `u8`.
#[inline]
fn container_radius(square: bool) -> u8 {
    if square {
        0
    } else {
        radius_u8(CONTAINER_RADIUS)
    }
}

/// Drop shadow for the container, clamped to [`MAX_ELEVATION`].
///
/// `None` at elevation `0`.
fn elevation_shadow(elevation: u8) -> Option<egui::epaint::Shadow> {
    if elevation == 0 {
        return None;
    }
    let e = elevation.min(MAX_ELEVATION) as f32;
    Some(egui::epaint::Shadow {
        offset: [0, (e * 0.5) as i8],
        blur: (e * 1.5) as u8,
        spread: 0,
        color: Color32::from_black_alpha(((e * 6.0) as u8).min(96)),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surfaces::expansion_panels::mud_expansion_panel::{header_height, PanelChrome};

    #[test]
    fn defaults_match_mudblazor() {
        let c = MudExpansionPanels::new();
        assert!(!c.dense);
        assert_eq!(c.elevation, 1);
        assert!(c.gutters);
        assert!(!c.multi_expansion);
        assert!(c.outlined);
        assert!(!c.square);
        assert!(c.tag.is_none());
        assert!(c.base.class_names().is_empty());
    }

    #[test]
    fn builders_set_every_field() {
        let c = MudExpansionPanels::new()
            .dense(true)
            .elevation(6)
            .gutters(false)
            .multi_expansion(true)
            .outlined(false)
            .square(true)
            .classes(["mt-4"])
            .tag("faq");
        assert!(c.dense && c.multi_expansion && c.square);
        assert!(!c.gutters && !c.outlined);
        assert_eq!(c.elevation, 6);
        assert!(!c.base.class_names().is_empty());
        assert_eq!(c.tag_ref::<&str>(), Some(&"faq"));
        assert!(c.tag_ref::<u32>().is_none());
    }

    #[test]
    fn sync_state_len_grows_with_collapsed_defaults() {
        let mut state = Vec::new();
        sync_state_len(&mut state, 3);
        assert_eq!(state, vec![false, false, false]);
        state[0] = true;
        sync_state_len(&mut state, 2);
        assert_eq!(state, vec![true, false]);
        sync_state_len(&mut state, 4);
        assert_eq!(state, vec![true, false, false, false]);
    }

    #[test]
    fn expand_all_sets_every_flag() {
        let mut state = vec![false, false, false];
        expand_all(&mut state);
        assert_eq!(state, vec![true, true, true]);
        expand_all(&mut state);
        assert_eq!(state, vec![true, true, true]);
    }

    #[test]
    fn collapse_all_clears_every_flag() {
        let mut state = vec![true, true, true];
        collapse_all(&mut state);
        assert_eq!(state, vec![false, false, false]);
        collapse_all(&mut state);
        assert_eq!(state, vec![false, false, false]);
    }

    #[test]
    fn collapse_all_except_keeps_one() {
        let mut state = vec![true, false, true];
        collapse_all_except(&mut state, 1);
        assert_eq!(state, vec![false, true, false]);
        collapse_all_except(&mut state, 0);
        assert_eq!(state, vec![true, false, false]);
    }

    #[test]
    fn collapse_all_except_ignores_out_of_range() {
        let mut state = vec![true, true];
        collapse_all_except(&mut state, 5);
        assert_eq!(state, vec![true, true]);
        let mut empty: Vec<bool> = Vec::new();
        collapse_all_except(&mut empty, 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn container_radius_is_zero_when_square() {
        assert_eq!(container_radius(true), 0);
        assert_eq!(container_radius(false), 4);
    }

    #[test]
    fn elevation_shadow_is_none_at_zero_and_clamps() {
        assert!(elevation_shadow(0).is_none());
        let s4 = elevation_shadow(4).expect("elevation 4");
        let s8 = elevation_shadow(8).expect("elevation 8");
        assert!(s8.blur > s4.blur);
        assert_eq!(
            elevation_shadow(40).map(|s| s.blur),
            elevation_shadow(MAX_ELEVATION).map(|s| s.blur)
        );
    }

    #[test]
    fn container_dense_forces_dense_on_every_panel() {
        let container = MudExpansionPanels::new().dense(true);
        let panel = MudExpansionPanel::new("p").dense(false);
        let dense = panel.dense || container.dense;
        assert!(dense);
        assert_eq!(header_height(dense), header_height(true));
    }

    #[test]
    fn container_gutters_off_forces_no_padding() {
        let container = MudExpansionPanels::new().gutters(false);
        let panel = MudExpansionPanel::new("p").gutters(true);
        let gutters = panel.gutters && container.gutters;
        assert!(!gutters);
    }

    #[test]
    fn panel_can_opt_into_dense_and_out_of_gutters() {
        let container = MudExpansionPanels::new();
        let dense_panel = MudExpansionPanel::new("p").dense(true);
        assert!(dense_panel.dense || container.dense);
        let flush_panel = MudExpansionPanel::new("p").gutters(false);
        assert!(!(flush_panel.gutters && container.gutters));
    }

    #[test]
    fn chrome_marks_last_panel() {
        let chrome = PanelChrome {
            force_dense: false,
            allow_gutters: true,
            in_container: true,
            last: true,
        };
        assert!(chrome.in_container && chrome.last);
        assert!(!PanelChrome::standalone().in_container);
    }
}