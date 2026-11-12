//! `MudTreeView<T>` — a recursive, selectable, expandable tree.
//!
//! Port of `MudTreeView<T>` + `MudTreeViewItem<T>` + `MudTreeViewItemToggleButton`
//! (MudBlazor 8.13.0).
//!
//! # Parity
//!
//! | MudBlazor `MudTreeView<T>` | Rust method | Default |
//! |----------------------------|-------------|---------|
//! | `AutoExpand`               | [`MudTreeView::auto_expand`] | `false` |
//! | `AutoSelectParent`         | [`MudTreeView::auto_select_parent`] | `true` |
//! | `CheckBoxColor`            | [`MudTreeView::check_box_color`] | `Color::Default` |
//! | `CheckedIcon`              | [`MudTreeView::checked_icon`] | `CHECK_BOX` |
//! | `ChildContent`             | closure to [`MudTreeView::show`] | required |
//! | `Color`                    | [`MudTreeView::color`] | `Primary` |
//! | `Comparer`                 | *(Rust `T: PartialEq`)* | — |
//! | `Dense`                    | [`MudTreeView::dense`] | `false` |
//! | `Disabled`                 | [`MudTreeView::disabled`] | `false` |
//! | `ExpandOnClick`            | [`MudTreeView::expand_on_click`] | `false` |
//! | `ExpandOnDoubleClick`      | [`MudTreeView::expand_on_double_click`] | `false` |
//! | `FilterFunc`               | [`MudTreeView::filter`] | `None` |
//! | `Height` / `MaxHeight`     | [`MudTreeView::height`] / [`MudTreeView::max_height`] | `None` |
//! | `Hover`                    | [`MudTreeView::hover`] | `false` |
//! | `IndeterminateIcon`        | [`MudTreeView::indeterminate_icon`] | `INDETERMINATE_CHECK_BOX` |
//! | `Items` / `ItemTemplate`   | *(builder closure)* | — |
//! | `ReadOnly`                 | [`MudTreeView::read_only`] | `false` |
//! | `Ripple`                   | [`MudTreeView::ripple`] | `true` |
//! | `SelectedValue`            | [`TreeState::selected_value`] | `None` |
//! | `SelectedValues`           | [`TreeState::selected_values`] | `[]` |
//! | `SelectionMode`            | [`MudTreeView::selection_mode`] | `Single` |
//! | `ServerData`               | **not ported** (see below) | — |
//! | `TriState`                 | [`MudTreeView::tri_state`] | `true` |
//! | `UncheckedIcon`            | [`MudTreeView::unchecked_icon`] | `CHECK_BOX_OUTLINE_BLANK` |
//! | `Width`                    | [`MudTreeView::width`] | `None` |
//!
//! | MudBlazor `MudTreeViewItem<T>` | Rust method | Default |
//! |-------------------------------|-------------|---------|
//! | `BodyContent`                 | [`TreeItem::body`] | `None` |
//! | `CanExpand`                   | [`TreeItem::can_expand`] | `true` |
//! | `ChildContent`                | [`TreeItem::child`] / [`TreeItem::children`] | none |
//! | `Content`                     | **not ported** (use `body`) | — |
//! | `Disabled`                    | [`TreeItem::disabled`] | `false` |
//! | `EndIcon`                     | [`TreeItem::end_icon`] | `None` |
//! | `EndIconColor`                | [`TreeItem::end_icon_color`] | `Inherit` |
//! | `EndText`                     | [`TreeItem::end_text`] | `None` |
//! | `EndTextTypo`                 | [`TreeItem::end_text_typo`] | `Body1` |
//! | `ExpandButtonIcon`            | [`TreeItem::expand_button_icon`] | `CHEVRON_RIGHT` |
//! | `ExpandButtonIconColor`       | [`TreeItem::expand_button_icon_color`] | `Default` |
//! | `Expanded`                    | [`TreeItem::expanded`] | `None` |
//! | `Icon`                        | [`TreeItem::icon`] | `None` |
//! | `IconColor`                   | [`TreeItem::icon_color`] | `Inherit` |
//! | `IconExpanded`                | [`TreeItem::icon_expanded`] | `None` |
//! | `Items`                       | [`TreeItem::child`] | none |
//! | `LoadingIcon` / `LoadingIconColor` | **not ported** (no ServerData) | — |
//! | `OnClick` / `OnDoubleClick`   | [`TreeItem::on_click`] / [`TreeItem::on_double_click`] | `None` |
//! | `ReadOnly`                    | [`TreeItem::read_only`] | `false` |
//! | `Selected`                    | [`TreeState`] | `None` |
//! | `Text`                        | [`TreeItem::with_text`] | `None` |
//! | `TextTypo`                    | *(internal; `Body1`)* | — |
//! | `Value`                       | [`TreeItem::new`] | required |
//! | `Visible`                     | [`TreeItem::visible`] | `true` |
//!
//! # Not ported
//!
//! * **`Items` + `ItemTemplate`** — MudBlazor's data-model mode. The Rust
//!   port uses the `ChildContent`/builder mode only; the closure-driven tree
//!   is the idiomatic egui equivalent and avoids materialising a separate
//!   `TreeItemData<T>` hierarchy. Build the tree with `TreeItem::child(...)`.
//! * **`ServerData`** — async lazy loading. egui is immediate-mode with no
//!   built-in async runtime per frame; the caller can implement lazy loading
//!   themselves by swapping the `TreeUi` contents on a click handler.
//! * **`Content`** (full row replacement) — `body(..)` replaces the icon +
//!   text + end area, which covers the common case. Use `body` when you need
//!   custom row content.
//! * **`FilterFunc` is synchronous** — `filter(Fn(&T) -> bool)`. Async filters
//!   are not supported.
//! * **`Style` / `UserAttributes`** — no CSS strings or HTML tags in egui.
//!
//! # Auto behaviour
//!
//! * `auto_expand(true)` expands every ancestor of a selected value so the
//!   selection is visible without manual clicks.
//! * `auto_select_parent(true)` (Multi mode only) maintains the invariant
//!   "a parent is checked iff every child is checked". The pass runs once per
//!   frame **before** rendering; a click therefore reaches the parent with a
//!   one-frame delay, exactly like an event handler.
//! * `tri_state(true)` (Multi mode only) paints a parent with an
//!   indeterminate glyph when its descendants are mixed.
//!
//! # Click targets
//!
//! The expand button, the checkbox, and the row body are three separate hit
//! regions:
//!
//! * **Expand button** — toggles expansion only.
//! * **Checkbox** — in Multi mode, toggles the whole subtree (cascade). In
//!   Single/Toggle, toggles the node.
//! * **Row body** — selects the node; also toggles expansion when
//!   `expand_on_click(true)`.
//! * **Double-click** — toggles expansion when `expand_on_double_click(true)`.

use std::any::Any;
use std::sync::Arc;

use egui::{
    vec2, Color32, CornerRadius, Rect, Response, Sense, Ui, UiBuilder,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, SelectionMode, StyleFlavor, Typo};
use mud_egui_icons_material::filled;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::typography::mud_text::MudText;

use super::tree_item::TreeItem;
use super::tree_state::TreeState;
use super::tree_ui::TreeUi;
use super::tri_state_kind::TriStateKind;

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Row height for a non-dense tree.
pub const ROW_HEIGHT: f32 = 40.0;
/// Row height for a dense tree.
pub const DENSE_ROW_HEIGHT: f32 = 32.0;
/// Indent added per nesting level.
pub const INDENT_PER_DEPTH: f32 = 24.0;
/// Side length of the expand-button hit-rect.
const EXPAND_BUTTON_SIZE: f32 = 24.0;
/// Side length of the selection checkbox.
const CHECKBOX_SIZE: f32 = 20.0;
/// Side length of item icons (leading and end).
const ITEM_ICON_SIZE: f32 = 20.0;
/// Horizontal padding inside a row.
const ROW_PADDING_X: f32 = 4.0;
/// Gap between inline widgets in a row.
const ROW_GAP: f32 = 6.0;
/// Duration of the ripple, in seconds.
const RIPPLE_DURATION: f32 = 0.35;

/// A recursive, selectable, expandable tree.
#[must_use = "MudTreeView does nothing until you call `.show(ui, state, |tree| ..)`"]
pub struct MudTreeView<'a, T> {
    // ── Data ───────────────────────────────────────────────────────────────
    /// Pre-filter hook applied to every node (`FilterFunc`). A node is shown
    /// when it or any descendant matches.
    pub filter: Option<Box<dyn Fn(&T) -> bool + 'a>>,

    // ── Selection ──────────────────────────────────────────────────────────
    /// Selection behaviour (`SelectionMode`, default `Single`).
    pub selection_mode: SelectionMode,
    /// Accent colour for selected rows (`Color`, default `Primary`).
    pub color: Color,
    /// Checkbox colour (`CheckBoxColor`, default `Default`).
    pub check_box_color: Color,
    /// Checked checkbox glyph (`CheckedIcon`, default `CHECK_BOX`).
    pub checked_icon: String,
    /// Unchecked checkbox glyph (`UncheckedIcon`, default `CHECK_BOX_OUTLINE_BLANK`).
    pub unchecked_icon: String,
    /// Indeterminate checkbox glyph (`IndeterminateIcon`, default
    /// `INDETERMINATE_CHECK_BOX`).
    pub indeterminate_icon: String,
    /// `TriState(true)` paints an indeterminate glyph on partially checked
    /// parents. Only effective in [`SelectionMode::Multi`].
    pub tri_state: bool,
    /// Auto-expand ancestors of selected values (`AutoExpand`, default `false`).
    pub auto_expand: bool,
    /// Auto-select parent when every child is selected (`AutoSelectParent`,
    /// default `true`). Only effective in [`SelectionMode::Multi`].
    pub auto_select_parent: bool,
    /// Prevents selection changes (`ReadOnly`, default `false`).
    pub read_only: bool,

    // ── Click behaviour ────────────────────────────────────────────────────
    /// Clicking a row also toggles its expansion (`ExpandOnClick`, default
    /// `false`).
    pub expand_on_click: bool,
    /// Double-clicking a row toggles its expansion (`ExpandOnDoubleClick`,
    /// default `false`).
    pub expand_on_double_click: bool,

    // ── Appearance ─────────────────────────────────────────────────────────
    /// Compact rows (`Dense`, default `false`).
    pub dense: bool,
    /// Row hover fill (`Hover`, default `false`).
    pub hover: bool,
    /// `Ripple(true)` shows a ripple on click.
    pub ripple: bool,
    /// Disables every node (`Disabled`, default `false`).
    pub disabled: bool,
    /// Fixed tree height (`Height`); enables a scroll area.
    pub height: Option<f32>,
    /// Maximum tree height (`MaxHeight`); enables a scroll area.
    pub max_height: Option<f32>,
    /// Fixed tree width (`Width`).
    pub width: Option<f32>,

    // ── One-shot expand / collapse ─────────────────────────────────────────
    /// Forces every expandable node open for this frame (`ExpandAllAsync`
    /// equivalent).
    pub expand_all: bool,
    /// Forces every node collapsed for this frame (`CollapseAllAsync`
    /// equivalent).
    pub collapse_all: bool,

    // ── Utilities ──────────────────────────────────────────────────────────
    /// Utility classes for the wrapper.
    pub base: ComponentBase,
    /// Bootstrap or Tailwind resolver.
    pub flavor: StyleFlavor,
    /// Bootstrap resolver state.
    pub bs_style: BsStyle,
    /// Tailwind resolver state.
    pub tw_style: TwStyle,
    /// Tailwind interaction state.
    pub tw_state: TwState,
    /// Accessible label published on the tree node.
    pub aria_label: Option<String>,
    tag_value: Option<Arc<dyn Any + Send + Sync>>,
}

impl<'a, T> Default for MudTreeView<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> MudTreeView<'a, T> {
    /// Creates a tree with MudBlazor defaults.
    pub fn new() -> Self {
        Self {
            filter: None,
            selection_mode: SelectionMode::Single,
            color: Color::Primary,
            check_box_color: Color::Default,
            checked_icon: filled::CHECK_BOX.to_owned(),
            unchecked_icon: filled::CHECK_BOX_OUTLINE_BLANK.to_owned(),
            indeterminate_icon: filled::INDETERMINATE_CHECK_BOX.to_owned(),
            tri_state: true,
            auto_expand: false,
            auto_select_parent: true,
            read_only: false,
            expand_on_click: false,
            expand_on_double_click: false,
            dense: false,
            hover: false,
            ripple: true,
            disabled: false,
            height: None,
            max_height: None,
            width: None,
            expand_all: false,
            collapse_all: false,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
            aria_label: None,
            tag_value: None,
        }
    }

    // ── Builders ───────────────────────────────────────────────────────────

    /// Sets the node visibility predicate (`FilterFunc`).
    #[inline] pub fn filter(mut self, f: impl Fn(&T) -> bool + 'a) -> Self { self.filter = Some(Box::new(f)); self }
    /// Sets the selection mode.
    #[inline] pub fn selection_mode(mut self, m: SelectionMode) -> Self { self.selection_mode = m; self }
    /// Sets the selected-row accent colour (`Color`).
    #[inline] pub fn color(mut self, c: Color) -> Self { self.color = c; self }
    /// Sets the checkbox colour (`CheckBoxColor`).
    #[inline] pub fn check_box_color(mut self, c: Color) -> Self { self.check_box_color = c; self }
    /// Sets the checked glyph (`CheckedIcon`).
    #[inline] pub fn checked_icon(mut self, s: impl Into<String>) -> Self { self.checked_icon = s.into(); self }
    /// Sets the unchecked glyph (`UncheckedIcon`).
    #[inline] pub fn unchecked_icon(mut self, s: impl Into<String>) -> Self { self.unchecked_icon = s.into(); self }
    /// Sets the indeterminate glyph (`IndeterminateIcon`).
    #[inline] pub fn indeterminate_icon(mut self, s: impl Into<String>) -> Self { self.indeterminate_icon = s.into(); self }
    /// Enables or disables tri-state checkbox painting (`TriState`).
    #[inline] pub fn tri_state(mut self, b: bool) -> Self { self.tri_state = b; self }
    /// Enables or disables ancestor auto-expansion (`AutoExpand`).
    #[inline] pub fn auto_expand(mut self, b: bool) -> Self { self.auto_expand = b; self }
    /// Enables or disables derived parent state (`AutoSelectParent`).
    #[inline] pub fn auto_select_parent(mut self, b: bool) -> Self { self.auto_select_parent = b; self }
    /// Prevents selection changes (`ReadOnly`).
    #[inline] pub fn read_only(mut self, b: bool) -> Self { self.read_only = b; self }
    /// Row clicks also toggle expansion (`ExpandOnClick`).
    #[inline] pub fn expand_on_click(mut self, b: bool) -> Self { self.expand_on_click = b; self }
    /// Double-clicks toggle expansion (`ExpandOnDoubleClick`).
    #[inline] pub fn expand_on_double_click(mut self, b: bool) -> Self { self.expand_on_double_click = b; self }
    /// Uses compact row heights (`Dense`).
    #[inline] pub fn dense(mut self, b: bool) -> Self { self.dense = b; self }
    /// Enables or disables row hover fill (`Hover`).
    #[inline] pub fn hover(mut self, b: bool) -> Self { self.hover = b; self }
    /// Enables or disables the click ripple (`Ripple`).
    #[inline] pub fn ripple(mut self, b: bool) -> Self { self.ripple = b; self }
    /// Disables every node (`Disabled`).
    #[inline] pub fn disabled(mut self, b: bool) -> Self { self.disabled = b; self }
    /// Sets a fixed tree height; enables a scroll area (`Height`).
    #[inline] pub fn height(mut self, h: f32) -> Self { self.height = Some(h.max(0.0)); self }
    /// Sets a maximum tree height; enables a scroll area (`MaxHeight`).
    #[inline] pub fn max_height(mut self, h: f32) -> Self { self.max_height = Some(h.max(0.0)); self }
    /// Sets a fixed tree width (`Width`).
    #[inline] pub fn width(mut self, w: f32) -> Self { self.width = Some(w.max(0.0)); self }
    /// Force-expands every node for this frame.
    #[inline] pub fn expand_all(mut self, b: bool) -> Self { self.expand_all = b; self }
    /// Force-collapses every node for this frame.
    #[inline] pub fn collapse_all(mut self, b: bool) -> Self { self.collapse_all = b; self }
    /// Sets the accessible label published on the tree node.
    #[inline] pub fn aria_label(mut self, s: impl Into<String>) -> Self { self.aria_label = Some(s.into()); self }
    /// Adds utility classes to the wrapper.
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }
    /// Selects the utility-class resolver.
    #[inline] pub fn flavor(mut self, f: StyleFlavor) -> Self { self.flavor = f; self }
    /// Attaches arbitrary user data (`Tag`).
    pub fn tag<U: Any + Send + Sync>(mut self, value: U) -> Self { self.tag_value = Some(Arc::new(value)); self }
    /// Reads the tag back if it is of type `U`.
    pub fn tag_ref<U: Any + Send + Sync>(&self) -> Option<&U> { self.tag_value.as_ref()?.downcast_ref::<U>() }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the tree. Returns `(response, user_result)`.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        state: &mut TreeState<T>,
        add_items: impl FnOnce(&mut TreeUi<'a, T>) -> R,
    ) -> (Response, R)
    where
        T: Clone + PartialEq,
    {
        // 1. Collect items.
        let mut collector = TreeUi { roots: Vec::new() };
        let user_result = add_items(&mut collector);
        let mut roots = collector.roots;

        // 2. Theme + wrapper frame.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);

        self.tw_state.dark = ui.visuals().dark_mode;
        let frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => self
                .tw_style
                .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
        };

        // 3. Pre-pass: auto-select-parent.
        if self.auto_select_parent && self.selection_mode == SelectionMode::Multi {
            for root in roots.iter_mut() {
                auto_select_pass(root, state, true);
            }
        }

        // 4. Pre-pass: auto-expand ancestors of selected values.
        let mut force_expanded: Vec<T> = Vec::new();
        if self.auto_expand {
            for root in &roots {
                collect_ancestors_of_selected(root, state, self.selection_mode, &mut force_expanded);
            }
        }

        // 5. Resolve palette-dependent colours.
        let accent: Color32 = self.color.resolve(palette).unwrap_or_else(|| palette.primary.into());
        let checkbox_color: Color32 = match self.check_box_color {
            Color::Default | Color::Inherit => accent,
            c => c.resolve(palette).map(Color32::from).unwrap_or(accent),
        };
        let text_primary: Color32 = palette.text_primary.into();
        let text_weak: Color32 = palette.text_secondary.into();
        let hover_fill = ui.visuals().widgets.hovered.weak_bg_fill;
        let selected_fill = with_alpha(accent, 28);
        let body_border = ui.visuals().widgets.noninteractive.bg_stroke.color;

        // 6. Build the draw configuration. Kept local so `cfg` borrows do
        //    not conflict with `state`.
        let cfg = DrawCfg {
            row_h: if self.dense { DENSE_ROW_HEIGHT } else { ROW_HEIGHT },
            selection_mode: self.selection_mode,
            tri_state: self.tri_state,
            filter: self.filter.as_deref(),
            accent,
            checkbox_color,
            checked_icon: &self.checked_icon,
            unchecked_icon: &self.unchecked_icon,
            indeterminate_icon: &self.indeterminate_icon,
            text_primary,
            text_weak,
            hover_fill,
            selected_fill,
            body_border,
            hover: self.hover,
            expand_on_click: self.expand_on_click,
            expand_on_double_click: self.expand_on_double_click,
            disabled: self.disabled,
            read_only: self.read_only,
            expand_all: self.expand_all,
            collapse_all: self.collapse_all,
            force_expanded: &force_expanded,
            palette,
        };

        // 7. Scroll wrapper + render.
        let outer = frame.show(ui, |ui| {
            // `body` must be `mut` because it mutates `state` and `roots`
            // through its captures, and is called from `FnMut` closures
            // inside `ScrollArea::show` / `frame.show`.
            let mut body = |ui: &mut Ui| {
                state.last_node_rects.clear();
                for (i, root) in roots.iter_mut().enumerate() {
                    let path = [i];
                    render_node(ui, root, 0, &path, state, &cfg);
                }
            };

            let cap = self.max_height.or(self.height);
            if let Some(h) = cap {
                egui::ScrollArea::vertical()
                    .max_height(h)
                    .auto_shrink([false, false])
                    .show(ui, |ui| body(ui));
            } else {
                body(ui);
            }
        });

        self.tw_state.hovered = outer.response.hovered();
        self.tw_state.active = outer.response.is_pointer_button_down_on();

        if let Some(label) = &self.aria_label {
            outer.response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::Tree, true, label)
            });
        }

        (outer.response, user_result)
    }
}

// ---------------------------------------------------------------------------
// Draw configuration
// ---------------------------------------------------------------------------

/// Immutable per-frame drawing configuration.
struct DrawCfg<'c, T> {
    row_h: f32,
    selection_mode: SelectionMode,
    tri_state: bool,
    filter: Option<&'c dyn Fn(&T) -> bool>,
    accent: Color32,
    checkbox_color: Color32,
    checked_icon: &'c str,
    unchecked_icon: &'c str,
    indeterminate_icon: &'c str,
    text_primary: Color32,
    text_weak: Color32,
    hover_fill: Color32,
    selected_fill: Color32,
    body_border: Color32,
    hover: bool,
    expand_on_click: bool,
    expand_on_double_click: bool,
    disabled: bool,
    read_only: bool,
    expand_all: bool,
    collapse_all: bool,
    force_expanded: &'c [T],
    palette: &'c Palette,
}

#[inline]
fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

// ---------------------------------------------------------------------------
// Pre-passes
// ---------------------------------------------------------------------------

/// Auto-select-parent pass: walks the tree bottom-up and maintains the
/// invariant "a parent is checked iff every direct child is checked".
/// Returns the effective checked state of `node` for its own parent.
fn auto_select_pass<'a, T: Clone + PartialEq>(
    node: &mut TreeItem<'a, T>,
    state: &mut TreeState<T>,
    enabled: bool,
) -> bool {
    let mut all_children_checked = true;
    let has_children = !node.children.is_empty();

    for child in node.children.iter_mut() {
        let c = auto_select_pass(child, state, enabled);
        if !c {
            all_children_checked = false;
        }
    }

    let self_selected = state.selected_values.iter().any(|v| v == &node.value);

    if !has_children {
        return self_selected;
    }

    if enabled {
        if all_children_checked {
            if !self_selected {
                state.selected_values.push(node.value.clone());
            }
            true
        } else {
            if self_selected {
                state.selected_values.retain(|v| v != &node.value);
            }
            false
        }
    } else {
        self_selected
    }
}

/// Auto-expand pass: collects the value of every node that has a selected
/// descendant.
fn collect_ancestors_of_selected<'a, T: Clone + PartialEq>(
    node: &TreeItem<'a, T>,
    state: &TreeState<T>,
    mode: SelectionMode,
    out: &mut Vec<T>,
) -> bool {
    let self_sel = state.is_selected(&node.value, mode);
    let mut any_desc = false;
    for child in &node.children {
        if collect_ancestors_of_selected(child, state, mode, out) {
            any_desc = true;
        }
    }
    if any_desc {
        out.push(node.value.clone());
    }
    self_sel || any_desc
}

/// Computes the tri-state kind of a node from its descendants (leaves use
/// their own state).
fn compute_tri_state<'a, T: PartialEq + Clone>(
    node: &TreeItem<'a, T>,
    state: &TreeState<T>,
) -> TriStateKind {
    if node.children.is_empty() {
        return if state.selected_values.iter().any(|v| v == &node.value) {
            TriStateKind::Checked
        } else {
            TriStateKind::Unchecked
        };
    }
    let mut any = false;
    let mut all = true;
    for child in &node.children {
        match compute_tri_state(child, state) {
            TriStateKind::Checked => any = true,
            TriStateKind::Unchecked => all = false,
            TriStateKind::Indeterminate => {
                any = true;
                all = false;
            }
        }
    }
    if all {
        TriStateKind::Checked
    } else if any {
        TriStateKind::Indeterminate
    } else {
        TriStateKind::Unchecked
    }
}

/// Visibility test: a node is visible when it is not `visible(false)` and it
/// or a descendant satisfies the filter.
fn is_visible<'a, T>(node: &TreeItem<'a, T>, filter: Option<&dyn Fn(&T) -> bool>) -> bool {
    if !node.visible {
        return false;
    }
    if let Some(f) = filter {
        if f(&node.value) {
            return true;
        }
        return node.children.iter().any(|c| is_visible(c, filter));
    }
    true
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn render_node<'a, T: Clone + PartialEq>(
    ui: &mut Ui,
    node: &mut TreeItem<'a, T>,
    depth: usize,
    path: &[usize],
    state: &mut TreeState<T>,
    cfg: &DrawCfg<'_, T>,
) {
    // 1. Filter / visibility.
    if !is_visible(node, cfg.filter) {
        return;
    }

    let disabled = cfg.disabled || node.disabled;
    let read_only = cfg.read_only || node.read_only;
    let has_children = !node.children.is_empty();
    let can_expand = node.can_expand && has_children;

    // 2. Seed + resolve expansion.
    state.ensure_expanded(&node.value, node.expanded_init.unwrap_or(false));
    let expanded = if cfg.collapse_all || !can_expand {
        false
    } else if cfg.expand_all {
        true
    } else {
        state.is_expanded(&node.value).unwrap_or(false)
            || cfg.force_expanded.iter().any(|v| v == &node.value)
    };

    // 3. Resolve selection / tri-state.
    let mode = cfg.selection_mode;
    let is_selected = state.is_selected(&node.value, mode);
    let tri = if mode == SelectionMode::Multi && cfg.tri_state {
        compute_tri_state(node, state)
    } else {
        TriStateKind::Unchecked
    };
    let effective_selected = match mode {
        SelectionMode::Multi if cfg.tri_state => matches!(tri, TriStateKind::Checked),
        _ => is_selected,
    };

    // 4. Allocate the row.
    let row_h = cfg.row_h;
    let row_w = ui.available_width();
    let (rect, row_resp) = ui.allocate_exact_size(vec2(row_w, row_h), Sense::click());

    state.last_node_rects.push(rect);

    // 5. Row background.
    let bg = if disabled {
        Color32::TRANSPARENT
    } else if effective_selected {
        cfg.selected_fill
    } else if cfg.hover && row_resp.hovered() {
        cfg.hover_fill
    } else {
        Color32::TRANSPARENT
    };
    if bg != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(3), bg);
    }

    // 6. Layout.
    let indent_x = ROW_PADDING_X + depth as f32 * INDENT_PER_DEPTH;
    let mut cursor_x = rect.left() + indent_x;
    let content_right = rect.right() - ROW_PADDING_X;

    // 6a. Expand button.
    let expand_rect = if can_expand {
        let r = Rect::from_min_size(
            egui::pos2(cursor_x, rect.center().y - EXPAND_BUTTON_SIZE * 0.5),
            vec2(EXPAND_BUTTON_SIZE, EXPAND_BUTTON_SIZE),
        );
        cursor_x += EXPAND_BUTTON_SIZE + ROW_GAP;
        Some(r)
    } else {
        cursor_x += EXPAND_BUTTON_SIZE + ROW_GAP;
        None
    };

    // 6b. Checkbox (Multi only).
    let checkbox_rect = if mode == SelectionMode::Multi {
        let r = Rect::from_min_size(
            egui::pos2(cursor_x, rect.center().y - CHECKBOX_SIZE * 0.5),
            vec2(CHECKBOX_SIZE, CHECKBOX_SIZE),
        );
        cursor_x += CHECKBOX_SIZE + ROW_GAP;
        Some(r)
    } else {
        None
    };

    // 6c. Icon slot.
    let icon_rect = if node.icon.is_some() || node.icon_expanded.is_some() {
        let r = Rect::from_min_size(
            egui::pos2(cursor_x, rect.center().y - ITEM_ICON_SIZE * 0.5),
            vec2(ITEM_ICON_SIZE, ITEM_ICON_SIZE),
        );
        cursor_x += ITEM_ICON_SIZE + ROW_GAP;
        Some(r)
    } else {
        None
    };

    // 6d. End slots (right-aligned).
    let mut end_x = content_right;
    let mut end_rects: Vec<(&str, Rect)> = Vec::new();
    if let Some(svg) = node.end_icon.as_deref() {
        end_x -= ITEM_ICON_SIZE;
        end_rects.push((
            svg,
            Rect::from_min_size(
                egui::pos2(end_x, rect.center().y - ITEM_ICON_SIZE * 0.5),
                vec2(ITEM_ICON_SIZE, ITEM_ICON_SIZE),
            ),
        ));
        end_x -= ROW_GAP;
    }
    let end_text_rect = node.end_text.as_ref().map(|_| {
        let w = 60.0_f32;
        let r = Rect::from_min_size(
            egui::pos2((end_x - w).max(cursor_x), rect.top()),
            vec2(w, row_h),
        );
        end_x -= w + ROW_GAP;
        r
    });

    // 6e. Body.
    let body_rect = Rect::from_min_max(
        egui::pos2(cursor_x, rect.top()),
        egui::pos2(end_x.max(cursor_x), rect.bottom()),
    );

    // 7. Paint expand button.
    let expand_resp = expand_rect.map(|r| {
        let id = ui.scope_id().with(("tree_expand", path));
        let resp = ui.interact(r, id, Sense::click());
        let use_glyph = if node.expand_button_icon == filled::CHEVRON_RIGHT {
            if expanded {
                filled::EXPAND_MORE
            } else {
                filled::CHEVRON_RIGHT
            }
        } else {
            node.expand_button_icon.as_str()
        };
        let color = if disabled {
            cfg.text_weak
        } else {
            node.expand_button_icon_color
                .resolve(cfg.palette)
                .map(Color32::from)
                .unwrap_or(cfg.text_weak)
        };
        ui.put(
            r,
            MudIcon::new(use_glyph.to_owned())
                .size_px(EXPAND_BUTTON_SIZE * 0.75)
                .custom_color(color),
        );
        resp
    });

    // 8. Paint checkbox.
    let checkbox_resp = checkbox_rect.map(|r| {
        let id = ui.scope_id().with(("tree_checkbox", path));
        let resp = ui.interact(r, id, Sense::click());
        let (glyph, color) = match tri {
            TriStateKind::Checked => (cfg.checked_icon, cfg.checkbox_color),
            TriStateKind::Indeterminate => (cfg.indeterminate_icon, cfg.checkbox_color),
            TriStateKind::Unchecked => {
                let c = if disabled { cfg.text_weak } else { cfg.checkbox_color };
                (cfg.unchecked_icon, c)
            }
        };
        ui.put(
            r,
            MudIcon::new(glyph.to_owned())
                .size_px(CHECKBOX_SIZE)
                .custom_color(color),
        );
        resp
    });

    // 9. Paint icon.
    if let (Some(r), Some(icon)) = (icon_rect, node.icon.as_deref()) {
        let glyph = if expanded {
            node.icon_expanded.as_deref().unwrap_or(icon)
        } else {
            icon
        };
        let color = if disabled {
            cfg.text_weak
        } else {
            node.icon_color
                .resolve(cfg.palette)
                .map(Color32::from)
                .unwrap_or(cfg.text_primary)
        };
        ui.put(
            r,
            MudIcon::new(glyph.to_owned())
                .size_px(ITEM_ICON_SIZE)
                .custom_color(color),
        );
    }

    // 10. Paint end icon(s).
    for (svg, r) in end_rects {
        let color = if disabled {
            cfg.text_weak
        } else {
            node.end_icon_color
                .resolve(cfg.palette)
                .map(Color32::from)
                .unwrap_or(cfg.text_weak)
        };
        ui.put(
            r,
            MudIcon::new(svg.to_owned())
                .size_px(ITEM_ICON_SIZE)
                .custom_color(color),
        );
    }

    // 11. Paint body (custom or default).
    if let Some(body) = node.body.as_mut() {
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(body_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child.set_clip_rect(body_rect);
        body(&mut child, effective_selected);
    } else {
        // MudBlazor requires `Text` to be supplied; a node without a caption
        // renders no text. `T` has no `Debug` bound, so no fallback format.
        let text = node.text.clone().unwrap_or_default();
        let color = if disabled {
            cfg.text_weak
        } else if effective_selected {
            cfg.accent
        } else {
            cfg.text_primary
        };
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(body_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child.set_clip_rect(body_rect);
        child.add(
            MudText::new(text)
                .typo(Typo::Body1)
                .custom_color(color)
                .no_wrap(true),
        );
    }

    // 12. Paint end text.
    if let (Some(r), Some(t)) = (end_text_rect, node.end_text.as_deref()) {
        let color = cfg.text_weak;
        ui.put(
            r,
            MudText::new(t.to_owned())
                .typo(node.end_text_typo)
                .custom_color(color)
                .no_wrap(true),
        );
    }

    // 13. Ripple.
    let ripple_id = ui.scope_id().with(("tree_ripple", path));
    let clicked_any = expand_resp.as_ref().map_or(false, |r| r.clicked())
        || checkbox_resp.as_ref().map_or(false, |r| r.clicked())
        || row_resp.clicked();
    if clicked_any {
        start_ripple(ui, ripple_id);
    }
    paint_ripple(ui, rect, ripple_id, cfg.accent);

    // 14. Handle clicks.
    // 14a. Expand button.
    if let Some(resp) = expand_resp {
        if resp.clicked() && !disabled {
            state.toggle_expanded(&node.value);
            if let Some(cb) = node.on_click.as_mut() {
                cb(&node.value);
            }
            return; // re-render next frame; skip double-click / row-click.
        }
    }

    // 14b. Checkbox.
    if let Some(resp) = checkbox_resp {
        if resp.clicked() && !disabled && !read_only {
            handle_checkbox_click(node, state, cfg);
            if let Some(cb) = node.on_click.as_mut() {
                cb(&node.value);
            }
        }
    }

    // 14c. Row body.
    if !disabled && !read_only {
        if row_resp.double_clicked() && cfg.expand_on_double_click && can_expand {
            state.toggle_expanded(&node.value);
            if let Some(cb) = node.on_double_click.as_mut() {
                cb(&node.value);
            }
        } else if row_resp.clicked() {
            handle_row_click(node, state, cfg);
            if cfg.expand_on_click && can_expand {
                state.toggle_expanded(&node.value);
            }
            if let Some(cb) = node.on_click.as_mut() {
                cb(&node.value);
            }
        }
    }

    // 15. Recurse into visible children.
    if expanded {
        for (i, child) in node.children.iter_mut().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(i);
            render_node(ui, child, depth + 1, &child_path, state, cfg);
        }
    }
}

// ---------------------------------------------------------------------------
// Click handlers
// ---------------------------------------------------------------------------

fn handle_row_click<'a, T: Clone + PartialEq>(
    node: &mut TreeItem<'a, T>,
    state: &mut TreeState<T>,
    cfg: &DrawCfg<'_, T>,
) {
    match cfg.selection_mode {
        SelectionMode::Single => {
            if state.selected_value.as_ref() == Some(&node.value) {
                // no-op
            } else {
                state.selected_value = Some(node.value.clone());
                state.selected_values.clear();
            }
        }
        SelectionMode::Toggle => {
            if state.selected_value.as_ref() == Some(&node.value) {
                state.selected_value = None;
            } else {
                state.selected_value = Some(node.value.clone());
            }
            state.selected_values.clear();
        }
        SelectionMode::Multi => {
            // Row click toggles the node (without cascading).
            if let Some(pos) = state.selected_values.iter().position(|v| v == &node.value) {
                state.selected_values.remove(pos);
            } else {
                state.selected_values.push(node.value.clone());
            }
            state.selected_value = None;
        }
    }
}

fn handle_checkbox_click<'a, T: Clone + PartialEq>(
    node: &mut TreeItem<'a, T>,
    state: &mut TreeState<T>,
    cfg: &DrawCfg<'_, T>,
) {
    // Multi mode: the checkbox cascades the whole subtree.
    let checked = match cfg.selection_mode {
        SelectionMode::Multi => {
            let tri = compute_tri_state(node, state);
            !matches!(tri, TriStateKind::Checked)
        }
        SelectionMode::Single | SelectionMode::Toggle => {
            !state.is_selected(&node.value, cfg.selection_mode)
        }
    };
    cascade_set(node, state, cfg.selection_mode, checked);
    if cfg.selection_mode != SelectionMode::Multi {
        handle_row_click(node, state, cfg);
    }
}

fn cascade_set<'a, T: Clone + PartialEq>(
    node: &mut TreeItem<'a, T>,
    state: &mut TreeState<T>,
    mode: SelectionMode,
    checked: bool,
) {
    if mode == SelectionMode::Multi {
        if checked {
            if !state.selected_values.iter().any(|v| v == &node.value) {
                state.selected_values.push(node.value.clone());
            }
        } else {
            state.selected_values.retain(|v| v != &node.value);
        }
    }
    for child in node.children.iter_mut() {
        cascade_set(child, state, mode, checked);
    }
}

// ---------------------------------------------------------------------------
// Ripple
// ---------------------------------------------------------------------------

fn start_ripple(ui: &Ui, id: egui::Id) {
    let now = ui.input(|i| i.time);
    ui.ctx().memory_mut(|m| m.data.insert_temp(id, now));
}

fn paint_ripple(ui: &Ui, rect: Rect, id: egui::Id, color: Color32) {
    let start: Option<f64> = ui.ctx().memory(|m| m.data.get_temp(id));
    let Some(start) = start else {
        return;
    };
    let now = ui.input(|i| i.time);
    let elapsed = (now - start) as f32;
    if elapsed >= RIPPLE_DURATION {
        ui.ctx().memory_mut(|m| m.data.remove_temp::<f64>(id));
        return;
    }
    let t = (elapsed / RIPPLE_DURATION).clamp(0.0, 1.0);
    let max_r = rect.width().max(rect.height()) * 0.75;
    let r = max_r * t;
    let a = ((1.0 - t) * 48.0) as u8;
    let c = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a);
    ui.painter()
        .with_clip_rect(rect)
        .circle_filled(rect.center(), r, c);
    ui.ctx().request_repaint();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Node {
        id: u32,
        label: String,
    }

    fn leaf(id: u32, label: &str) -> Node {
        Node { id, label: label.into() }
    }

    // --- TreeState ---------------------------------------------------------

    #[test]
    fn state_starts_empty() {
        let s: TreeState<Node> = TreeState::new();
        assert!(s.selected_value.is_none());
        assert!(s.selected_values.is_empty());
        assert!(s.expanded.is_empty());
    }

    #[test]
    fn ensure_expanded_seeds_once() {
        let mut s: TreeState<Node> = TreeState::new();
        let n = leaf(1, "a");
        assert_eq!(s.is_expanded(&n), None);
        s.ensure_expanded(&n, true);
        assert_eq!(s.is_expanded(&n), Some(true));
        s.ensure_expanded(&n, false);
        assert_eq!(s.is_expanded(&n), Some(true), "seed must not overwrite");
    }

    #[test]
    fn toggle_expanded_flips_and_creates() {
        let mut s: TreeState<Node> = TreeState::new();
        let n = leaf(1, "a");
        s.toggle_expanded(&n);
        assert_eq!(s.is_expanded(&n), Some(true));
        s.toggle_expanded(&n);
        assert_eq!(s.is_expanded(&n), Some(false));
    }

    #[test]
    fn is_selected_mode_aware() {
        let mut s: TreeState<Node> = TreeState::new();
        let a = leaf(1, "a");
        s.selected_value = Some(a.clone());
        assert!(s.is_selected(&a, SelectionMode::Single));
        assert!(!s.is_selected(&a, SelectionMode::Multi));

        s.selected_value = None;
        s.selected_values.push(a.clone());
        assert!(s.is_selected(&a, SelectionMode::Multi));
        assert!(!s.is_selected(&a, SelectionMode::Single));
    }

    // --- TreeItem ----------------------------------------------------------

    #[test]
    fn item_defaults() {
        let it: TreeItem<'static, Node> = TreeItem::new(leaf(1, "a"));
        assert!(it.text.is_none());
        assert!(it.children.is_empty());
        assert!(it.icon.is_none());
        assert!(it.end_icon.is_none());
        assert!(it.end_text.is_none());
        assert_eq!(it.expanded_init, None);
        assert!(it.can_expand);
        assert!(!it.disabled);
        assert!(!it.read_only);
        assert!(it.visible);
    }

    #[test]
    fn child_appends_and_preserves_order() {
        let it = TreeItem::new(leaf(1, "a"))
            .child(TreeItem::new(leaf(2, "b")))
            .children([TreeItem::new(leaf(3, "c")), TreeItem::new(leaf(4, "d"))]);
        assert_eq!(it.children.len(), 3);
        assert_eq!(it.children[0].value.id, 2);
        assert_eq!(it.children[1].value.id, 3);
        assert_eq!(it.children[2].value.id, 4);
    }

    // --- visibility --------------------------------------------------------

    #[test]
    fn visible_false_hides_whole_subtree() {
        let n = TreeItem::new(leaf(1, "a"))
            .visible(false)
            .child(TreeItem::new(leaf(2, "b")));
        assert!(!is_visible(&n, None));
    }

    #[test]
    fn filter_matches_self_or_descendant() {
        let n = TreeItem::new(leaf(1, "root"))
            .child(TreeItem::new(leaf(2, "keep")))
            .child(TreeItem::new(leaf(3, "drop")));
        let f = |x: &Node| x.label == "keep";
        assert!(is_visible(&n, Some(&f)));

        let f = |x: &Node| x.label == "nope";
        assert!(!is_visible(&n, Some(&f)));
    }

    #[test]
    fn filter_matches_leaf_directly() {
        let n = TreeItem::new(leaf(1, "keep"));
        let f = |x: &Node| x.label == "keep";
        assert!(is_visible(&n, Some(&f)));
    }

    // --- tri-state ---------------------------------------------------------

    #[test]
    fn tri_state_leaf_uses_own_state() {
        let mut s: TreeState<Node> = TreeState::new();
        let n = TreeItem::new(leaf(1, "a"));
        s.selected_values.push(n.value.clone());
        assert_eq!(compute_tri_state(&n, &s), TriStateKind::Checked);

        let empty: TreeState<Node> = TreeState::new();
        assert_eq!(compute_tri_state(&n, &empty), TriStateKind::Unchecked);
    }

    #[test]
    fn tri_state_parent_mixed_is_indeterminate() {
        let mut s: TreeState<Node> = TreeState::new();
        let n = TreeItem::new(leaf(1, "p"))
            .child(TreeItem::new(leaf(2, "a")))
            .child(TreeItem::new(leaf(3, "b")));
        s.selected_values.push(leaf(2, "a"));
        assert_eq!(compute_tri_state(&n, &s), TriStateKind::Indeterminate);
    }

    #[test]
    fn tri_state_parent_all_is_checked() {
        let mut s: TreeState<Node> = TreeState::new();
        let n = TreeItem::new(leaf(1, "p"))
            .child(TreeItem::new(leaf(2, "a")))
            .child(TreeItem::new(leaf(3, "b")));
        s.selected_values.push(leaf(2, "a"));
        s.selected_values.push(leaf(3, "b"));
        assert_eq!(compute_tri_state(&n, &s), TriStateKind::Checked);
    }

    #[test]
    fn tri_state_parent_none_is_unchecked() {
        let s: TreeState<Node> = TreeState::new();
        let n = TreeItem::new(leaf(1, "p"))
            .child(TreeItem::new(leaf(2, "a")));
        assert_eq!(compute_tri_state(&n, &s), TriStateKind::Unchecked);
    }

    // --- auto-select-parent ------------------------------------------------

    #[test]
    fn auto_select_adds_parent_when_all_children_checked() {
        let mut s: TreeState<Node> = TreeState::new();
        let mut n = TreeItem::new(leaf(1, "p"))
            .child(TreeItem::new(leaf(2, "a")))
            .child(TreeItem::new(leaf(3, "b")));
        s.selected_values.push(leaf(2, "a"));
        s.selected_values.push(leaf(3, "b"));
        auto_select_pass(&mut n, &mut s, true);
        assert!(s.selected_values.iter().any(|v| v.id == 1), "parent must be added");
    }

    #[test]
    fn auto_select_removes_parent_when_some_child_unchecked() {
        let mut s: TreeState<Node> = TreeState::new();
        let mut n = TreeItem::new(leaf(1, "p"))
            .child(TreeItem::new(leaf(2, "a")))
            .child(TreeItem::new(leaf(3, "b")));
        s.selected_values.push(leaf(1, "p"));
        s.selected_values.push(leaf(2, "a"));
        auto_select_pass(&mut n, &mut s, true);
        assert!(!s.selected_values.iter().any(|v| v.id == 1), "parent must be removed");
    }

    #[test]
    fn auto_select_disabled_keeps_user_choice() {
        let mut s: TreeState<Node> = TreeState::new();
        let mut n = TreeItem::new(leaf(1, "p"))
            .child(TreeItem::new(leaf(2, "a")));
        s.selected_values.push(leaf(1, "p"));
        auto_select_pass(&mut n, &mut s, false);
        assert!(s.selected_values.iter().any(|v| v.id == 1), "parent must be kept");
    }

    // --- auto-expand -------------------------------------------------------

    #[test]
    fn ancestors_of_selected_collected() {
        let mut st: TreeState<Node> = TreeState::new();
        st.selected_value = Some(leaf(2, "a"));
        let n = TreeItem::new(leaf(1, "p"))
            .child(TreeItem::new(leaf(2, "a")));
        let mut out = Vec::new();
        collect_ancestors_of_selected(&n, &st, SelectionMode::Single, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, 1);
    }

    #[test]
    fn no_ancestors_when_nothing_selected() {
        let st: TreeState<Node> = TreeState::new();
        let n = TreeItem::new(leaf(1, "p")).child(TreeItem::new(leaf(2, "a")));
        let mut out = Vec::new();
        collect_ancestors_of_selected(&n, &st, SelectionMode::Single, &mut out);
        assert!(out.is_empty());
    }

    // --- cascade -----------------------------------------------------------

    #[test]
    fn cascade_set_checks_every_descendant() {
        let mut s: TreeState<Node> = TreeState::new();
        let mut n = TreeItem::new(leaf(1, "p"))
            .child(TreeItem::new(leaf(2, "a")))
            .child(TreeItem::new(leaf(3, "b")));
        cascade_set(&mut n, &mut s, SelectionMode::Multi, true);
        assert!(s.selected_values.iter().any(|v| v.id == 1));
        assert!(s.selected_values.iter().any(|v| v.id == 2));
        assert!(s.selected_values.iter().any(|v| v.id == 3));
    }

    #[test]
    fn cascade_uncheck_removes_every_descendant() {
        let mut s: TreeState<Node> = TreeState::new();
        s.selected_values = vec![leaf(1, "p"), leaf(2, "a"), leaf(3, "b")];
        let mut n = TreeItem::new(leaf(1, "p"))
            .child(TreeItem::new(leaf(2, "a")))
            .child(TreeItem::new(leaf(3, "b")));
        cascade_set(&mut n, &mut s, SelectionMode::Multi, false);
        assert!(s.selected_values.is_empty());
    }

    // --- builders ----------------------------------------------------------

    #[test]
    fn tree_defaults_match_mudblazor() {
        let t: MudTreeView<'static, Node> = MudTreeView::new();
        assert!(matches!(t.selection_mode, SelectionMode::Single));
        assert!(matches!(t.color, Color::Primary));
        assert!(matches!(t.check_box_color, Color::Default));
        assert!(t.tri_state);
        assert!(t.auto_select_parent);
        assert!(!t.auto_expand);
        assert!(!t.read_only);
        assert!(!t.expand_on_click);
        assert!(!t.expand_on_double_click);
        assert!(!t.dense);
        assert!(!t.hover);
        assert!(t.ripple);
        assert!(!t.disabled);
        assert!(t.height.is_none());
        assert!(t.max_height.is_none());
        assert!(t.width.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let t = MudTreeView::<Node>::new()
            .filter(|n| n.id > 0)
            .selection_mode(SelectionMode::Multi)
            .color(Color::Secondary)
            .check_box_color(Color::Success)
            .checked_icon("C")
            .unchecked_icon("U")
            .indeterminate_icon("I")
            .tri_state(false)
            .auto_expand(true)
            .auto_select_parent(false)
            .read_only(true)
            .expand_on_click(true)
            .expand_on_double_click(true)
            .dense(true)
            .hover(true)
            .ripple(false)
            .disabled(true)
            .height(300.0)
            .max_height(500.0)
            .width(200.0)
            .expand_all(true)
            .collapse_all(true)
            .aria_label("tree")
            .classes(["a"]);
        assert!(t.filter.is_some());
        assert!(matches!(t.selection_mode, SelectionMode::Multi));
        assert_eq!(t.checked_icon, "C");
        assert_eq!(t.unchecked_icon, "U");
        assert_eq!(t.indeterminate_icon, "I");
        assert!(!t.tri_state);
        assert!(t.auto_expand);
        assert!(!t.auto_select_parent);
        assert!(t.read_only);
        assert!(t.expand_on_click && t.expand_on_double_click);
        assert!(t.dense && t.hover && !t.ripple && t.disabled);
        assert_eq!(t.height, Some(300.0));
        assert_eq!(t.max_height, Some(500.0));
        assert_eq!(t.width, Some(200.0));
        assert!(t.expand_all && t.collapse_all);
    }

    #[test]
    fn tag_round_trips() {
        let t = MudTreeView::<Node>::new().tag("tree-tag");
        assert_eq!(t.tag_ref::<&str>(), Some(&"tree-tag"));
        assert!(t.tag_ref::<u32>().is_none());
    }

    // --- collector ---------------------------------------------------------

    #[test]
    fn collector_preserves_root_order() {
        let mut c: TreeUi<'static, Node> = TreeUi { roots: Vec::new() };
        c.item(TreeItem::new(leaf(1, "a")));
        c.item(TreeItem::new(leaf(2, "b")));
        assert_eq!(c.len(), 2);
        assert_eq!(c.roots[0].value.id, 1);
        assert_eq!(c.roots[1].value.id, 2);
    }
}