//! A resizable split-view container for `mud_egui`.
//!
//! Port of `MudSplitPanel.cs`. Provides two panels separated by a draggable
//! splitter, with support for horizontal/vertical layouts, custom ratios,
//! min sizes, themable surfaces, and CSS utility classes via `ComponentBase`.
//!
//! # Important: use `.id_salt()`
//! `egui` persists the split ratio in memory across frames. To survive widget
//! reordering, layout changes, or conditional rendering, **you must** provide a
//! stable salt via `.id_salt("my_split")`. Without it, the panel falls back to
//! `next_auto_id()`, which can collide or reset if surrounding widgets change.
//!
//! # MudBlazor API parity
//!
//! MudBlazor's `MudSplitPanel` exposes `FirstPanel`, `SecondPanel`,
//! `FirstPanelInitialSize` (`int?`, pixels), `Horizontal`, `MinPanelSize`
//! (default 50), `ResetOnDoubleClick` (default `true`), `UseAsOverlay`,
//! `ClassDivider`, `ClassFirstPanel`, `ClassSecondPanel`, the base members
//! (`Class`, `Style`, `Tag`, `UserAttributes`) and the methods
//! `GetDividerPositionAsync`, `SetDividerPositionAsync`,
//! `ResetDividerPositionAsync`. The parameter table I could retrieve lists no
//! event callbacks, so none are invented here.
//!
//! | MudBlazor                | Rust                                              | Default |
//! |--------------------------|---------------------------------------------------|---------|
//! | `FirstPanel`/`SecondPanel` | the two closures of [`MudSplitPanel::show`]     | —       |
//! | `FirstPanelInitialSize`  | [`MudSplitPanel::first_panel_initial_size`]       | `None`  |
//! | `Horizontal`             | [`MudSplitPanel::horizontal`]                     | `false` |
//! | `MinPanelSize`           | [`MudSplitPanel::min_panel_size`]                 | `50`    |
//! | `ResetOnDoubleClick`     | [`MudSplitPanel::reset_on_double_click`]          | `true`  |
//! | `UseAsOverlay`           | [`MudSplitPanel::use_as_overlay`]                 | `false` |
//! | `ClassDivider`           | [`MudSplitPanel::class_divider`]                  | none    |
//! | `ClassFirstPanel`        | [`MudSplitPanel::class_first_panel`]              | none    |
//! | `ClassSecondPanel`       | [`MudSplitPanel::class_second_panel`]             | none    |
//! | `Class`                  | [`MudSplitPanel::classes`]                        | none    |
//! | `Tag`                    | [`MudSplitPanel::tag`]                            | none    |
//! | `GetDividerPositionAsync`| [`MudSplitPanel::divider_ratio`]                  | —       |
//! | `SetDividerPositionAsync`| [`MudSplitPanel::set_divider_ratio`]              | —       |
//! | `ResetDividerPositionAsync` | [`MudSplitPanel::reset_divider`]               | —       |
//! | `Style`, `UserAttributes`| — (no CSS strings / HTML attributes in egui)      |         |
//!
//! Extensions already in this port and kept: `min_first_size`,
//! `min_second_size`, `initial_ratio`, `rounded`, `transparent`, `elevation`,
//! `first_panel_is_nested`, `second_panel_is_nested`, `locked`,
//! `splitter_thickness`, `flavor`. (`rounded`, `transparent` and `elevation` are
//! forwarded to the `MudPaper` that wraps each panel.)
//!
//! # Architecture
//!
//! Layout and interaction decisions are pure functions ([`plan_layout`],
//! [`effective_ratio`], [`drag_to_first_size`], ...), unit tested without a
//! context. `show` fetches the theme, resolves a stable state id **before**
//! entering any frame, then runs: load state → plan layout → splitter
//! interaction → update state → re-plan if changed → draw panels → draw splitter.
//! Because the layout is re-planned after the interaction, panels follow the
//! pointer in the same frame (the previous version lagged one frame).
//!
//! # Example
//! ```ignore
//! MudSplitPanel::new()
//!     .id_salt("editor")
//!     .min_first_size(150.0)
//!     .min_second_size(100.0)
//!     .classes(["p-2", "bg-surface-container-low", "rounded-lg"])
//!     .show(ui, |ui| ui.label("Left"), |ui| ui.label("Right"));
//!
//! // Programmatic control (MudBlazor: Get/Set/ResetDividerPositionAsync):
//! let id = MudSplitPanel::divider_id(ui, "editor");
//! MudSplitPanel::set_divider_ratio(ui.ctx(), id, 0.3);
//! MudSplitPanel::reset_divider(ui.ctx(), id);
//! ```

use std::any::Any;
use std::hash::Hash;
use std::sync::Arc;

use egui::{
    pos2, vec2, Area, Color32, Context, CursorIcon, Frame, Id, InnerResponse, Key, Order, Rect,
    Response, Sense, Ui, UiBuilder, Vec2, WidgetInfo,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::StyleFlavor;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

use crate::surfaces::paper::mud_paper::MudPaper;

/// Default visual thickness of the divider splitter in logical pixels.
const DEFAULT_SPLITTER_THICKNESS: f32 = 4.0;
/// Extra pixels added to each side of the splitter to widen the interactive grab area.
const SPLITTER_HIT_PADDING: f32 = 4.0;
/// Default minimum size restriction for panels (MudBlazor: `MinPanelSize = 50`).
const DEFAULT_MIN_SIZE: f32 = 50.0;
/// Multiplier for keyboard-based arrow key nudging, scaled by delta-time.
const KEYBOARD_NUDGE_SPEED: f32 = 0.5;
/// Base rem size (in points) passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;
/// Extent used for an axis whose available space is unbounded (for example the
/// scroll direction of a `ScrollArea`).
const DEFAULT_UNBOUNDED_EXTENT: f32 = 300.0;
/// Smallest splitter thickness in points.
const MIN_SPLITTER_THICKNESS: f32 = 1.0;
/// Long side of the grip pill painted on the splitter.
const GRIP_LONG: f32 = 16.0;
/// Short side of the grip pill painted on the splitter.
const GRIP_SHORT: f32 = 2.0;

// =============================================================================
// Pure helpers
// =============================================================================

#[inline]
fn sane_length(v: f32) -> f32 {
    if v.is_finite() {
        v.max(0.0)
    } else {
        0.0
    }
}

#[inline]
fn sane_thickness(t: f32) -> f32 {
    if t.is_finite() {
        t.max(MIN_SPLITTER_THICKNESS)
    } else {
        DEFAULT_SPLITTER_THICKNESS
    }
}

#[inline]
fn sane_ratio(r: f32) -> f32 {
    if r.is_finite() {
        r.clamp(0.0, 1.0)
    } else {
        0.5
    }
}

/// Replaces non-finite or negative extents of `avail` with a default so that an
/// unbounded parent never produces infinite panels.
pub(crate) fn normalize_avail(avail: Rect) -> Rect {
    let fix_pos = |v: f32| if v.is_finite() { v } else { 0.0 };
    let fix_len = |v: f32| {
        if v.is_finite() {
            v.max(0.0)
        } else {
            DEFAULT_UNBOUNDED_EXTENT
        }
    };
    Rect::from_min_size(
        pos2(fix_pos(avail.min.x), fix_pos(avail.min.y)),
        vec2(fix_len(avail.width()), fix_len(avail.height())),
    )
}

/// Resolves the minimum sizes against the usable length. If both minimums do not
/// fit, they are scaled down proportionally (preserving their ratio) instead of
/// being capped at half each.
pub(crate) fn resolve_min_sizes(min_first: f32, min_second: f32, usable: f32) -> (f32, f32) {
    let a = sane_length(min_first);
    let b = sane_length(min_second);
    let usable = sane_length(usable);
    let sum = a + b;
    if sum <= usable || sum <= 0.0 {
        (a, b)
    } else {
        let k = usable / sum;
        (a * k, b * k)
    }
}

/// Ratio of the first panel before the user has moved the splitter.
///
/// A pixel size (MudBlazor: `FirstPanelInitialSize`) wins over a ratio; with
/// neither, the panels split evenly.
pub(crate) fn initial_ratio_value(
    initial_ratio: Option<f32>,
    initial_size: Option<f32>,
    usable: f32,
) -> f32 {
    match initial_size {
        Some(size) if size.is_finite() && usable > 0.0 => (size.max(0.0) / usable).clamp(0.0, 1.0),
        _ => initial_ratio.map(sane_ratio).unwrap_or(0.5),
    }
}

/// The ratio to lay out with: the stored one if the user has set it, otherwise
/// the initial one (recomputed each frame, so a pixel size stays a pixel size
/// while the window is resized).
pub(crate) fn effective_ratio(
    stored: Option<f32>,
    initial_ratio: Option<f32>,
    initial_size: Option<f32>,
    usable: f32,
) -> f32 {
    match stored {
        Some(r) if r.is_finite() => r.clamp(0.0, 1.0),
        _ => initial_ratio_value(initial_ratio, initial_size, usable),
    }
}

/// Size the first panel should take when the splitter's center follows the pointer.
///
/// `grab_offset` is the distance between the pointer and the splitter center at
/// the moment the drag started, so the handle does not jump.
#[inline]
pub(crate) fn drag_to_first_size(
    pointer_axis: f32,
    grab_offset: f32,
    origin_axis: f32,
    thickness: f32,
) -> f32 {
    pointer_axis - grab_offset - origin_axis - thickness * 0.5
}

/// Converts a desired first-panel size into a stored ratio, respecting the minimums.
pub(crate) fn ratio_for_first_size(
    first_px: f32,
    usable: f32,
    min_first: f32,
    min_second: f32,
) -> f32 {
    if !(usable > 0.0) || !first_px.is_finite() {
        return 0.5;
    }
    let (lo, hi_cap) = resolve_min_sizes(min_first, min_second, usable);
    let hi = (usable - hi_cap).max(lo);
    (first_px.clamp(lo, hi) / usable).clamp(0.0, 1.0)
}

/// The geometry of one frame of the split.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SplitLayout {
    pub first: Rect,
    pub splitter: Rect,
    pub second: Rect,
    pub hit: Rect,
    pub first_size: f32,
    pub second_size: f32,
    pub usable: f32,
    pub min_first: f32,
    pub min_second: f32,
}

/// Computes all rectangles of the split inside `avail`.
///
/// `usable` is the length left after the splitter; the first panel takes
/// `ratio * usable`, clamped to the (resolved) minimum sizes. A collapsed first
/// panel has size `0` and the second takes everything.
#[allow(clippy::too_many_arguments)]
pub(crate) fn plan_layout(
    avail: Rect,
    horizontal: bool,
    thickness: f32,
    ratio: f32,
    collapsed: bool,
    min_first: f32,
    min_second: f32,
    hit_padding: f32,
) -> SplitLayout {
    let thickness = sane_thickness(thickness);
    let total = if horizontal { avail.height() } else { avail.width() };
    let cross = if horizontal { avail.width() } else { avail.height() };
    let usable = sane_length(total - thickness);
    let (min_first, min_second) = resolve_min_sizes(min_first, min_second, usable);

    let (first_size, second_size) = if collapsed {
        (0.0, usable)
    } else if usable <= 0.0 {
        (0.0, 0.0)
    } else {
        let hi = (usable - min_second).max(min_first);
        let first = (sane_ratio(ratio) * usable).clamp(min_first, hi);
        (first, usable - first)
    };

    let pad = sane_length(hit_padding);
    let (first, splitter, second, hit) = if horizontal {
        let first = Rect::from_min_size(avail.min, vec2(cross, first_size));
        let splitter = Rect::from_min_size(pos2(avail.min.x, first.max.y), vec2(cross, thickness));
        let second =
            Rect::from_min_size(pos2(avail.min.x, splitter.max.y), vec2(cross, second_size));
        let hit = Rect::from_min_max(
            pos2(splitter.min.x, splitter.min.y - pad),
            pos2(splitter.max.x, splitter.max.y + pad),
        );
        (first, splitter, second, hit)
    } else {
        let first = Rect::from_min_size(avail.min, vec2(first_size, cross));
        let splitter = Rect::from_min_size(pos2(first.max.x, avail.min.y), vec2(thickness, cross));
        let second =
            Rect::from_min_size(pos2(splitter.max.x, avail.min.y), vec2(second_size, cross));
        let hit = Rect::from_min_max(
            pos2(splitter.min.x - pad, splitter.min.y),
            pos2(splitter.max.x + pad, splitter.max.y),
        );
        (first, splitter, second, hit)
    };

    SplitLayout {
        first,
        splitter,
        second,
        hit,
        first_size,
        second_size,
        usable,
        min_first,
        min_second,
    }
}

/// Visual state of the splitter handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HandleState {
    Locked,
    Idle,
    Hovered,
    Focused,
    Dragging,
}

/// Priority: locked > dragging > focused > hovered > idle.
pub(crate) fn handle_state(locked: bool, hovered: bool, dragged: bool, focused: bool) -> HandleState {
    if locked {
        HandleState::Locked
    } else if dragged {
        HandleState::Dragging
    } else if focused {
        HandleState::Focused
    } else if hovered {
        HandleState::Hovered
    } else {
        HandleState::Idle
    }
}

/// Persisted per-split state (in egui's temp memory).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct SplitState {
    /// `None` until the user moves the splitter: the initial size/ratio applies.
    ratio: Option<f32>,
    collapsed: bool,
    /// Orientation of the previous frame; a change resets the ratio.
    was_horizontal: Option<bool>,
    /// Pointer-to-center distance captured when a drag starts.
    grab_offset: f32,
}

/// Copies a `StyleFlavor` without requiring it to implement `Copy`/`Clone`.
fn copy_flavor(flavor: &StyleFlavor) -> StyleFlavor {
    match flavor {
        StyleFlavor::Bootstrap => StyleFlavor::Bootstrap,
        StyleFlavor::Tailwind => StyleFlavor::Tailwind,
    }
}

// =============================================================================
// Component
// =============================================================================

/// A resizable split panel container component.
#[must_use = "MudSplitPanel does nothing until you call `.show(ui, ...)`"]
pub struct MudSplitPanel {
    /// If `true`, splits top/bottom (horizontal split bar).
    /// If `false`, splits left/right (vertical split bar).
    /// MudBlazor: `Horizontal`.
    pub horizontal: bool,

    /// If `true`, double-clicking the splitter handle resets the layout to the initial ratio.
    /// MudBlazor: `ResetOnDoubleClick`. Default: `true`.
    pub reset_on_double_click: bool,

    /// If `true`, the panel wrappers have rounded corners. *Extension (MudPaper).*
    pub rounded: bool,

    /// If `true`, the panels are rendered completely transparent. *Extension (MudPaper).*
    pub transparent: bool,

    /// Elevation shadow depth level for the underlying panels. *Extension (MudPaper).*
    pub elevation: u8,

    /// Minimum allowable size for the first panel in points.
    /// [`MudSplitPanel::min_panel_size`] sets both minimums at once (MudBlazor: `MinPanelSize`).
    pub min_first_size: f32,

    /// Minimum allowable size for the second panel in points.
    pub min_second_size: f32,

    /// Initial ratio allocation of the first panel (`0.0` to `1.0`). *Extension.*
    pub initial_ratio: Option<f32>,

    /// Initial size of the first panel in points. MudBlazor: `FirstPanelInitialSize`.
    /// Wins over `initial_ratio` when both are set.
    pub first_panel_initial_size: Option<f32>,

    /// If `true`, the split is drawn in an overlay above the parent's content,
    /// covering the parent's rectangle and taking no layout space.
    /// MudBlazor: `UseAsOverlay`. Default: `false`.
    pub use_as_overlay: bool,

    /// If `true`, the first panel skips background surface painting.
    pub first_panel_is_nested: bool,

    /// If `true`, the second panel skips background surface painting.
    pub second_panel_is_nested: bool,

    /// Stable salt used for persisting state across frames. **Strongly recommended.**
    pub id_salt: Option<Id>,

    /// If `true`, locks the splitter handle so the user cannot drag or resize it. *Extension.*
    pub locked: bool,

    /// Visual thickness of the divider splitter in points. *Extension.*
    pub splitter_thickness: f32,

    /// Utility classes of the divider (MudBlazor: `ClassDivider`). A `bg-*` class
    /// sets the divider's idle color.
    pub class_divider: Vec<String>,

    /// Utility classes of the first panel (MudBlazor: `ClassFirstPanel`).
    pub class_first_panel: Vec<String>,

    /// Utility classes of the second panel (MudBlazor: `ClassSecondPanel`).
    pub class_second_panel: Vec<String>,

    /// Arbitrary user data. MudBlazor: `Tag`.
    tag: Option<Arc<dyn Any + Send + Sync>>,

    // --- CSS utilities ---
    /// Shared component state (CSS-like utility classes). MudBlazor: `Class`.
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

impl Default for MudSplitPanel {
    fn default() -> Self {
        Self {
            horizontal: false,
            reset_on_double_click: true,
            rounded: true,
            transparent: false,
            elevation: 0,
            min_first_size: DEFAULT_MIN_SIZE,
            min_second_size: DEFAULT_MIN_SIZE,
            initial_ratio: None,
            first_panel_initial_size: None,
            use_as_overlay: false,
            first_panel_is_nested: false,
            second_panel_is_nested: false,
            id_salt: None,
            locked: false,
            splitter_thickness: DEFAULT_SPLITTER_THICKNESS,
            class_divider: Vec::new(),
            class_first_panel: Vec::new(),
            class_second_panel: Vec::new(),
            tag: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }
}

/// Result output container returned after rendering a `MudSplitPanel`.
pub struct MudSplitPanelOutput<R1, R2> {
    /// Response and value of the first panel.
    pub first: InnerResponse<R1>,
    /// Response and value of the second panel.
    pub second: InnerResponse<R2>,
    /// Response of the splitter handle (use `.dragged()`, `.double_clicked()`, ...).
    pub splitter: Response,
    /// Ratio of the first panel in `0.0..=1.0` (`0.0` while collapsed).
    pub ratio: f32,
    /// Main-axis size of the first panel in points.
    pub first_size: f32,
    /// Main-axis size of the second panel in points.
    pub second_size: f32,
    /// `true` while the first panel is collapsed with [`MudSplitPanel::set_first_collapsed`].
    pub collapsed: bool,
    /// `true` on a frame where the user (or an orientation change) moved the splitter.
    pub changed: bool,
    /// The id under which this split keeps its state; pass it to the
    /// `divider_ratio` / `set_divider_ratio` / `reset_divider` functions.
    pub id: Id,
}

/// Everything the render path needs besides the two content closures.
struct Runtime<'a> {
    id: Id,
    horizontal: bool,
    reset_on_double_click: bool,
    rounded: bool,
    transparent: bool,
    elevation: u8,
    min_first: f32,
    min_second: f32,
    initial_ratio: Option<f32>,
    initial_size: Option<f32>,
    first_nested: bool,
    second_nested: bool,
    locked: bool,
    thickness: f32,
    flavor: StyleFlavor,
    class_first: &'a [String],
    class_second: &'a [String],
    divider_color: Color32,
    divider_fill: Option<Color32>,
}

impl MudSplitPanel {
    /// Creates a left/right split with two equal panels and 50 pt minimums.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    // =========================================================================
    // Builder Methods
    // =========================================================================

    /// Splits top/bottom instead of left/right.
    #[inline]
    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }

    /// Double-clicking the splitter resets it to its initial position.
    #[inline]
    pub fn reset_on_double_click(mut self, reset: bool) -> Self {
        self.reset_on_double_click = reset;
        self
    }

    /// Rounds the panel corners.
    #[inline]
    pub fn rounded(mut self, rounded: bool) -> Self {
        self.rounded = rounded;
        self
    }

    /// Renders both panels transparent.
    #[inline]
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    /// Elevation of the panel surfaces.
    #[inline]
    pub fn elevation(mut self, elevation: u8) -> Self {
        self.elevation = elevation;
        self
    }

    /// Minimum size of the first panel in points (negative / non-finite → `0`).
    #[inline]
    pub fn min_first_size(mut self, size: f32) -> Self {
        self.min_first_size = sane_length(size);
        self
    }

    /// Minimum size of the second panel in points (negative / non-finite → `0`).
    #[inline]
    pub fn min_second_size(mut self, size: f32) -> Self {
        self.min_second_size = sane_length(size);
        self
    }

    /// Sets both minimum sizes (MudBlazor: `MinPanelSize`).
    #[inline]
    pub fn min_panel_size(mut self, size: f32) -> Self {
        let size = sane_length(size);
        self.min_first_size = size;
        self.min_second_size = size;
        self
    }

    /// Initial ratio of the first panel; non-finite values become `0.5`.
    #[inline]
    pub fn initial_ratio(mut self, ratio: f32) -> Self {
        self.initial_ratio = Some(sane_ratio(ratio));
        self
    }

    /// Initial size of the first panel in points (MudBlazor: `FirstPanelInitialSize`).
    /// It stays a fixed size while the window is resized until the user moves
    /// the splitter, after which the split is proportional.
    #[inline]
    pub fn first_panel_initial_size(mut self, size: f32) -> Self {
        self.first_panel_initial_size = Some(sane_length(size));
        self
    }

    /// Draws the split in an overlay above the parent's content (MudBlazor: `UseAsOverlay`).
    #[inline]
    pub fn use_as_overlay(mut self, overlay: bool) -> Self {
        self.use_as_overlay = overlay;
        self
    }

    /// The first panel skips its surface painting.
    #[inline]
    pub fn first_panel_is_nested(mut self, nested: bool) -> Self {
        self.first_panel_is_nested = nested;
        self
    }

    /// The second panel skips its surface painting.
    #[inline]
    pub fn second_panel_is_nested(mut self, nested: bool) -> Self {
        self.second_panel_is_nested = nested;
        self
    }

    /// Sets the stable identity of the split.
    #[allow(deprecated)]
    #[inline]
    pub fn id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_salt = Some(Id::new(salt));
        self
    }

    /// Locks the splitter so it cannot be dragged.
    #[inline]
    pub fn locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }

    /// Sets the visual thickness of the splitter handle in points (minimum `1`).
    #[inline]
    pub fn splitter_thickness(mut self, thickness: f32) -> Self {
        self.splitter_thickness = sane_thickness(thickness);
        self
    }

    /// Selects the utility-class resolver.
    #[inline]
    pub fn flavor(mut self, flavor: StyleFlavor) -> Self {
        self.flavor = flavor;
        self
    }

    /// Adds utility classes to the outer wrapper of the split region.
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Utility classes for the divider (MudBlazor: `ClassDivider`).
    #[inline]
    pub fn class_divider<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.class_divider = classes.into_iter().map(|c| c.as_ref().to_owned()).collect();
        self
    }

    /// Utility classes for the first panel (MudBlazor: `ClassFirstPanel`).
    #[inline]
    pub fn class_first_panel<S: AsRef<str>>(
        mut self,
        classes: impl IntoIterator<Item = S>,
    ) -> Self {
        self.class_first_panel = classes.into_iter().map(|c| c.as_ref().to_owned()).collect();
        self
    }

    /// Utility classes for the second panel (MudBlazor: `ClassSecondPanel`).
    #[inline]
    pub fn class_second_panel<S: AsRef<str>>(
        mut self,
        classes: impl IntoIterator<Item = S>,
    ) -> Self {
        self.class_second_panel = classes.into_iter().map(|c| c.as_ref().to_owned()).collect();
        self
    }

    /// Attaches arbitrary user data to the component (MudBlazor: `Tag`).
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
    // Programmatic control (MudBlazor: Get/Set/ResetDividerPositionAsync)
    // =========================================================================

    /// The id under which a split with this salt keeps its state.
    ///
    /// Call it with the **same `ui`** that you pass to [`MudSplitPanel::show`].
    #[allow(deprecated)]
    pub fn divider_id(ui: &Ui, salt: impl Hash + std::fmt::Debug) -> Id {
        ui.scope_id().with(Id::new(salt))
    }

    /// The ratio the user (or [`MudSplitPanel::set_divider_ratio`]) has set, or
    /// `None` while the split still uses its initial position.
    pub fn divider_ratio(ctx: &Context, id: Id) -> Option<f32> {
        ctx.data(|d| d.get_temp::<SplitState>(id)).and_then(|s| s.ratio)
    }

    /// Moves the divider to `ratio` (`0.0..=1.0`). The minimum panel sizes are
    /// still applied when drawing. It may be called before the first `show`.
    pub fn set_divider_ratio(ctx: &Context, id: Id, ratio: f32) {
        ctx.data_mut(|d| {
            let state = d.get_temp_mut_or_insert_with(id, SplitState::default);
            state.ratio = Some(sane_ratio(ratio));
            state.collapsed = false;
        });
        ctx.request_repaint();
    }

    /// Returns the divider to its initial position (and un-collapses the first panel).
    pub fn reset_divider(ctx: &Context, id: Id) {
        ctx.data_mut(|d| {
            let state = d.get_temp_mut_or_insert_with(id, SplitState::default);
            state.ratio = None;
            state.collapsed = false;
        });
        ctx.request_repaint();
    }

    /// Collapses (or restores) the first panel; the second then takes all space.
    pub fn set_first_collapsed(ctx: &Context, id: Id, collapsed: bool) {
        ctx.data_mut(|d| {
            d.get_temp_mut_or_insert_with(id, SplitState::default).collapsed = collapsed;
        });
        ctx.request_repaint();
    }

    // =========================================================================
    // Rendering
    // =========================================================================

    /// Draws the split and the two panels.
    pub fn show<R1, R2>(
        mut self,
        ui: &mut Ui,
        add_first: impl FnOnce(&mut Ui) -> R1,
        add_second: impl FnOnce(&mut Ui) -> R2,
    ) -> MudSplitPanelOutput<R1, R2> {
        // Resolve the state id on the OUTER ui, before any frame child is created,
        // so it stays stable and matches `divider_id`.
        let id = match self.id_salt {
            Some(salt) => ui.scope_id().with(salt),
            None => {
                let fallback = ui.next_auto_id();
                ui.skip_ahead_auto_ids(1);
                fallback
            }
        };

        // Resolve CSS utility frame.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);
        self.tw_state.dark = ui.visuals().dark_mode;

        let frame: Frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => self
                .tw_style
                .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
        };

        // Divider utility classes: a background class sets the idle divider color.
        // A fresh resolver is used so the main resolver cache is never polluted.
        let divider_fill = if self.class_divider.is_empty() {
            None
        } else {
            let divider_base = ComponentBase::new().classes(self.class_divider.iter());
            let divider_frame: Frame = match self.flavor {
                StyleFlavor::Bootstrap => BsStyle::default().build_frame(&divider_base, &lookup),
                StyleFlavor::Tailwind => {
                    TwStyle::default().build_frame(&divider_base, self.tw_state, REM_SIZE_PT, &lookup)
                }
            };
            (divider_frame.fill != Color32::TRANSPARENT).then_some(divider_frame.fill)
        };

        let runtime = Runtime {
            id,
            horizontal: self.horizontal,
            reset_on_double_click: self.reset_on_double_click,
            rounded: self.rounded,
            transparent: self.transparent,
            elevation: self.elevation,
            min_first: self.min_first_size,
            min_second: self.min_second_size,
            initial_ratio: self.initial_ratio,
            initial_size: self.first_panel_initial_size,
            first_nested: self.first_panel_is_nested,
            second_nested: self.second_panel_is_nested,
            locked: self.locked,
            thickness: self.splitter_thickness,
            flavor: copy_flavor(&self.flavor),
            class_first: &self.class_first_panel,
            class_second: &self.class_second_panel,
            divider_color: palette.divider.into(),
            divider_fill,
        };

        if self.use_as_overlay {
            let rect = normalize_avail(ui.max_rect());
            Area::new(id.with("overlay"))
                .order(Order::Foreground)
                .fixed_pos(rect.min)
                .show(ui.ctx(), |ui| {
                    ui.set_min_size(rect.size());
                    ui.set_max_size(rect.size());
                    frame
                        .show(ui, |ui| Self::run_split(ui, &runtime, add_first, add_second, false))
                        .inner
                })
                .inner
        } else {
            frame
                .show(ui, |ui| Self::run_split(ui, &runtime, add_first, add_second, true))
                .inner
        }
    }

    /// Draws one panel inside its rectangle, wrapped in a `MudPaper` surface.
    #[allow(clippy::too_many_arguments)]
    fn render_panel<R>(
        ui: &mut Ui,
        rect: Rect,
        visible: bool,
        transparent: bool,
        rt: &Runtime<'_>,
        classes: &[String],
        add: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let target = if visible {
            rect
        } else {
            Rect::from_min_size(rect.min, Vec2::ZERO)
        };

        ui.scope_builder(
            UiBuilder::new().max_rect(target).layout(*ui.layout()),
            |ui| {
                ui.set_clip_rect(target.intersect(ui.clip_rect()));
                ui.set_min_size(target.size());

                let mut paper = MudPaper::new()
                    .elevation(rt.elevation)
                    .transparent(transparent)
                    .flavor(copy_flavor(&rt.flavor));
                if !classes.is_empty() {
                    paper = paper.classes(classes.iter());
                }
                if visible {
                    // Make the surface fill the whole panel, not only its content.
                    paper = paper.width(target.width()).height(target.height());
                }
                paper.show(ui, add)
            },
        )
            .inner
    }

    fn run_split<R1, R2>(
        ui: &mut Ui,
        rt: &Runtime<'_>,
        add_first: impl FnOnce(&mut Ui) -> R1,
        add_second: impl FnOnce(&mut Ui) -> R2,
        allocate: bool,
    ) -> MudSplitPanelOutput<R1, R2> {
        let id = rt.id;
        let splitter_id = id.with("splitter");
        let horizontal = rt.horizontal;
        let locked = rt.locked;

        // --- Available space (never infinite) ---
        let avail = normalize_avail(ui.available_rect_before_wrap());

        // --- Fetch state; an orientation change resets the position ---
        let mut state: SplitState = ui.data_mut(|d| *d.get_temp_mut_or_insert_with(id, SplitState::default));
        let mut changed = false;
        if matches!(state.was_horizontal, Some(h) if h != horizontal) {
            state.ratio = None;
            changed = true;
        }
        state.was_horizontal = Some(horizontal);

        let plan = |state: &SplitState| -> SplitLayout {
            let total = if horizontal { avail.height() } else { avail.width() };
            let usable0 = sane_length(total - sane_thickness(rt.thickness));
            let ratio = effective_ratio(state.ratio, rt.initial_ratio, rt.initial_size, usable0);
            plan_layout(
                avail,
                horizontal,
                rt.thickness,
                ratio,
                state.collapsed,
                rt.min_first,
                rt.min_second,
                SPLITTER_HIT_PADDING,
            )
        };

        let mut layout = plan(&state);

        // --- Splitter interaction ---
        let sense = if locked { Sense::hover() } else { Sense::click_and_drag() };
        let splitter_resp = ui.interact(layout.hit, splitter_id, sense);
        let axis = |p: egui::Pos2| if horizontal { p.y } else { p.x };
        let center_axis = |r: Rect| axis(r.center());
        let origin_axis = if horizontal { avail.min.y } else { avail.min.x };

        if splitter_resp.clicked() && !locked {
            splitter_resp.request_focus();
        }

        if (splitter_resp.hovered() || splitter_resp.dragged()) && !locked {
            ui.ctx().set_cursor_icon(if horizontal {
                CursorIcon::ResizeVertical
            } else {
                CursorIcon::ResizeHorizontal
            });
        }

        if !locked && splitter_resp.drag_started() {
            if let Some(p) = splitter_resp.interact_pointer_pos() {
                state.grab_offset = axis(p) - center_axis(layout.splitter);
            }
        }

        if !locked && splitter_resp.dragged() && layout.usable > 0.0 {
            if let Some(p) = splitter_resp.interact_pointer_pos() {
                let first_px = drag_to_first_size(axis(p), state.grab_offset, origin_axis, rt.thickness);
                let new_ratio =
                    ratio_for_first_size(first_px, layout.usable, layout.min_first, layout.min_second);
                if state.ratio.map_or(true, |r| (r - new_ratio).abs() > 1e-5) || state.collapsed {
                    state.ratio = Some(new_ratio);
                    state.collapsed = false;
                    changed = true;
                }
            }
        }

        if rt.reset_on_double_click && splitter_resp.double_clicked() {
            state.ratio = None;
            state.collapsed = false;
            changed = true;
        }

        if !locked && splitter_resp.has_focus() && layout.usable > 0.0 {
            let current = if layout.usable > 0.0 { layout.first_size / layout.usable } else { 0.5 };
            let dt = ui.input(|i| i.stable_dt).min(0.1);
            let mut nudge = 0.0;
            if ui.input(|i| i.key_down(Key::ArrowLeft) || i.key_down(Key::ArrowUp)) {
                nudge -= KEYBOARD_NUDGE_SPEED * dt;
            }
            if ui.input(|i| i.key_down(Key::ArrowRight) || i.key_down(Key::ArrowDown)) {
                nudge += KEYBOARD_NUDGE_SPEED * dt;
            }
            if ui.input(|i| i.key_pressed(Key::Home)) {
                state.ratio = Some(layout.min_first / layout.usable);
                state.collapsed = false;
                changed = true;
            } else if ui.input(|i| i.key_pressed(Key::End)) {
                state.ratio = Some((layout.usable - layout.min_second) / layout.usable);
                state.collapsed = false;
                changed = true;
            } else if nudge.abs() > 0.0001 {
                state.ratio = Some((current + nudge).clamp(0.0, 1.0));
                state.collapsed = false;
                changed = true;
            }
        }

        ui.data_mut(|d| d.insert_temp(id, state));

        // Re-plan with the updated state so panels follow the pointer this frame.
        if changed {
            layout = plan(&state);
        }

        // --- Render panels ---
        let first_visible = layout.first_size > 0.0;
        let first_transparent = rt.transparent || rt.first_nested || !first_visible;
        let first = Self::render_panel(
            ui,
            layout.first,
            first_visible,
            first_transparent,
            rt,
            rt.class_first,
            add_first,
        );

        let second_visible = layout.second_size > 0.0;
        let second_transparent = rt.transparent || rt.second_nested || !second_visible;
        let second = Self::render_panel(
            ui,
            layout.second,
            second_visible,
            second_transparent,
            rt,
            rt.class_second,
            add_second,
        );

        // --- Splitter strip + grip ---
        let hstate = handle_state(
            locked,
            splitter_resp.hovered(),
            splitter_resp.dragged(),
            splitter_resp.has_focus(),
        );
        let visuals = ui.visuals();
        let idle_strip = rt.divider_fill.unwrap_or(rt.divider_color);
        let (strip_color, grip_color) = match hstate {
            HandleState::Locked => (idle_strip.gamma_multiply(0.5), visuals.widgets.noninteractive.bg_fill),
            HandleState::Idle => (idle_strip, visuals.widgets.inactive.fg_stroke.color.gamma_multiply(0.6)),
            HandleState::Hovered => (visuals.widgets.hovered.bg_fill, visuals.widgets.hovered.fg_stroke.color),
            HandleState::Focused | HandleState::Dragging => {
                (visuals.selection.bg_fill, visuals.selection.stroke.color)
            }
        };

        let grip_size = if horizontal {
            vec2(GRIP_LONG, GRIP_SHORT)
        } else {
            vec2(GRIP_SHORT, GRIP_LONG)
        };
        let grip_rect = Rect::from_center_size(layout.splitter.center(), grip_size);

        if ui.is_rect_visible(layout.splitter) {
            let painter = ui.painter();
            painter.rect_filled(layout.splitter, 0.0, strip_color);
            painter.rect_filled(grip_rect, 1.0, grip_color);
        }

        let effective = if state.collapsed {
            0.0
        } else if layout.usable > 0.0 {
            layout.first_size / layout.usable
        } else {
            0.0
        };
        splitter_resp.widget_info(|| WidgetInfo::slider(!locked, f64::from(effective), "Splitter"));

        // --- Advance Parent Cursor ---
        if allocate {
            ui.allocate_rect(avail, Sense::hover());
        }

        MudSplitPanelOutput {
            first,
            second,
            splitter: splitter_resp,
            ratio: effective,
            first_size: layout.first_size,
            second_size: layout.second_size,
            collapsed: state.collapsed,
            changed,
            id,
        }
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

    fn rect(w: f32, h: f32) -> Rect {
        Rect::from_min_size(pos2(10.0, 20.0), vec2(w, h))
    }

    /// Runs `f` inside a real egui frame on a headless 400x300 context.
    fn run_in_ui<R>(f: impl FnOnce(&mut Ui) -> R) -> R {
        let ctx = Context::default();
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

    // ---- sanitizers ---------------------------------------------------------

    #[test]
    fn sanitizers_handle_bad_input() {
        assert_eq!(sane_length(-1.0), 0.0);
        assert_eq!(sane_length(f32::NAN), 0.0);
        assert_eq!(sane_thickness(0.1), MIN_SPLITTER_THICKNESS);
        assert_eq!(sane_thickness(f32::NAN), DEFAULT_SPLITTER_THICKNESS);
        assert_eq!(sane_thickness(6.0), 6.0);
        assert_eq!(sane_ratio(f32::NAN), 0.5);
        assert_eq!(sane_ratio(2.0), 1.0);
        assert_eq!(sane_ratio(-1.0), 0.0);
    }

    #[test]
    fn normalize_avail_replaces_unbounded_extents() {
        let r = normalize_avail(Rect::from_min_size(pos2(5.0, 6.0), vec2(200.0, f32::INFINITY)));
        assert_eq!(r.width(), 200.0);
        assert_eq!(r.height(), DEFAULT_UNBOUNDED_EXTENT);
        assert_eq!(r.min, pos2(5.0, 6.0));
        let r = normalize_avail(Rect::from_min_size(pos2(f32::NAN, 0.0), vec2(10.0, 10.0)));
        assert_eq!(r.min.x, 0.0);
    }

    // ---- minimum sizes ------------------------------------------------------

    #[test]
    fn min_sizes_pass_through_when_they_fit() {
        assert_eq!(resolve_min_sizes(50.0, 30.0, 200.0), (50.0, 30.0));
    }

    #[test]
    fn min_sizes_scale_proportionally_when_they_do_not_fit() {
        let (a, b) = resolve_min_sizes(100.0, 50.0, 90.0);
        assert!((a + b - 90.0).abs() < EPS);
        assert!((a / b - 2.0).abs() < EPS);
        assert_eq!(resolve_min_sizes(10.0, 10.0, 0.0), (0.0, 0.0));
    }

    // ---- ratios -------------------------------------------------------------

    #[test]
    fn initial_size_wins_over_initial_ratio() {
        assert!((initial_ratio_value(Some(0.9), Some(100.0), 400.0) - 0.25).abs() < EPS);
        assert!((initial_ratio_value(Some(0.9), None, 400.0) - 0.9).abs() < EPS);
        assert!((initial_ratio_value(None, None, 400.0) - 0.5).abs() < EPS);
        // Unknown usable length falls back to the ratio.
        assert!((initial_ratio_value(Some(0.3), Some(100.0), 0.0) - 0.3).abs() < EPS);
    }

    #[test]
    fn effective_ratio_prefers_the_stored_value() {
        assert!((effective_ratio(Some(0.7), Some(0.2), None, 400.0) - 0.7).abs() < EPS);
        assert!((effective_ratio(None, Some(0.2), None, 400.0) - 0.2).abs() < EPS);
        assert!((effective_ratio(Some(f32::NAN), Some(0.2), None, 400.0) - 0.2).abs() < EPS);
    }

    #[test]
    fn pixel_initial_size_stays_a_pixel_size_across_resizes() {
        for usable in [300.0, 500.0, 800.0] {
            let r = effective_ratio(None, None, Some(120.0), usable);
            assert!((r * usable - 120.0).abs() < EPS);
        }
    }

    #[test]
    fn drag_math_keeps_the_grab_offset() {
        // Pointer at x=110, grabbed 2px right of the splitter center, origin 10, thickness 4.
        assert!((drag_to_first_size(110.0, 2.0, 10.0, 4.0) - 96.0).abs() < EPS);
    }

    #[test]
    fn ratio_for_first_size_respects_minimums() {
        assert!((ratio_for_first_size(100.0, 400.0, 50.0, 50.0) - 0.25).abs() < EPS);
        assert!((ratio_for_first_size(0.0, 400.0, 50.0, 50.0) - 0.125).abs() < EPS);
        assert!((ratio_for_first_size(1000.0, 400.0, 50.0, 50.0) - 0.875).abs() < EPS);
        assert_eq!(ratio_for_first_size(100.0, 0.0, 50.0, 50.0), 0.5);
        assert_eq!(ratio_for_first_size(f32::NAN, 400.0, 50.0, 50.0), 0.5);
    }

    // ---- layout -------------------------------------------------------------

    #[test]
    fn vertical_split_divides_the_width() {
        let l = plan_layout(rect(404.0, 200.0), false, 4.0, 0.5, false, 50.0, 50.0, 4.0);
        assert!((l.first_size - 200.0).abs() < EPS);
        assert!((l.second_size - 200.0).abs() < EPS);
        assert!((l.splitter.width() - 4.0).abs() < EPS);
        assert!((l.first.height() - 200.0).abs() < EPS);
        assert!((l.splitter.min.x - l.first.max.x).abs() < EPS);
        assert!((l.second.min.x - l.splitter.max.x).abs() < EPS);
        assert!((l.second.max.x - 414.0).abs() < EPS);
        assert!((l.hit.width() - 12.0).abs() < EPS);
    }

    #[test]
    fn horizontal_split_divides_the_height() {
        let l = plan_layout(rect(300.0, 204.0), true, 4.0, 0.25, false, 50.0, 50.0, 4.0);
        assert!((l.first_size - 50.0).abs() < EPS);
        assert!((l.second_size - 150.0).abs() < EPS);
        assert!((l.first.width() - 300.0).abs() < EPS);
        assert!((l.splitter.height() - 4.0).abs() < EPS);
        assert!((l.hit.height() - 12.0).abs() < EPS);
    }

    #[test]
    fn ratio_is_clamped_by_the_minimums() {
        let low = plan_layout(rect(404.0, 100.0), false, 4.0, 0.0, false, 80.0, 60.0, 4.0);
        assert!((low.first_size - 80.0).abs() < EPS);
        let high = plan_layout(rect(404.0, 100.0), false, 4.0, 1.0, false, 80.0, 60.0, 4.0);
        assert!((high.second_size - 60.0).abs() < EPS);
    }

    #[test]
    fn collapsed_first_panel_gives_everything_to_the_second() {
        let l = plan_layout(rect(404.0, 100.0), false, 4.0, 0.5, true, 50.0, 50.0, 4.0);
        assert_eq!(l.first_size, 0.0);
        assert!((l.second_size - 400.0).abs() < EPS);
    }

    #[test]
    fn tiny_space_never_produces_negative_sizes() {
        let l = plan_layout(rect(2.0, 100.0), false, 4.0, 0.5, false, 50.0, 50.0, 4.0);
        assert_eq!((l.first_size, l.second_size, l.usable), (0.0, 0.0, 0.0));
        assert!(l.first.width() >= 0.0 && l.second.width() >= 0.0);
    }

    #[test]
    fn panels_plus_splitter_fill_the_available_extent() {
        let l = plan_layout(rect(500.0, 100.0), false, 6.0, 0.3, false, 20.0, 20.0, 4.0);
        assert!((l.first_size + 6.0 + l.second_size - 500.0).abs() < EPS);
    }

    // ---- handle state -------------------------------------------------------

    #[test]
    fn handle_state_priority() {
        assert_eq!(handle_state(true, true, true, true), HandleState::Locked);
        assert_eq!(handle_state(false, true, true, true), HandleState::Dragging);
        assert_eq!(handle_state(false, true, false, true), HandleState::Focused);
        assert_eq!(handle_state(false, true, false, false), HandleState::Hovered);
        assert_eq!(handle_state(false, false, false, false), HandleState::Idle);
    }

    // ---- builder ------------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let p = MudSplitPanel::new();
        assert!(!p.horizontal && !p.use_as_overlay && !p.locked);
        assert!(p.reset_on_double_click);
        assert_eq!(p.min_first_size, 50.0);
        assert_eq!(p.min_second_size, 50.0);
        assert!(p.first_panel_initial_size.is_none());
        assert!(p.class_divider.is_empty());
    }

    #[test]
    fn min_panel_size_sets_both_minimums() {
        let p = MudSplitPanel::new().min_panel_size(80.0);
        assert_eq!((p.min_first_size, p.min_second_size), (80.0, 80.0));
        let p = MudSplitPanel::new().min_panel_size(f32::NAN);
        assert_eq!((p.min_first_size, p.min_second_size), (0.0, 0.0));
    }

    #[test]
    fn builder_sanitizes_numbers() {
        let p = MudSplitPanel::new()
            .initial_ratio(f32::NAN)
            .splitter_thickness(0.0)
            .first_panel_initial_size(-5.0);
        assert_eq!(p.initial_ratio, Some(0.5));
        assert_eq!(p.splitter_thickness, MIN_SPLITTER_THICKNESS);
        assert_eq!(p.first_panel_initial_size, Some(0.0));
    }

    #[test]
    fn class_lists_are_stored_per_part() {
        let p = MudSplitPanel::new()
            .class_divider(["bg-primary"])
            .class_first_panel(["p-2", "m-1"])
            .class_second_panel(vec![String::from("p-4")]);
        assert_eq!(p.class_divider, vec!["bg-primary"]);
        assert_eq!(p.class_first_panel, vec!["p-2", "m-1"]);
        assert_eq!(p.class_second_panel, vec!["p-4"]);
    }

    #[test]
    fn tag_round_trips_by_type() {
        let p = MudSplitPanel::new().tag(5_u8);
        assert_eq!(p.tag_ref::<u8>(), Some(&5));
        assert_eq!(p.tag_ref::<String>(), None);
    }

    // ---- programmatic control ----------------------------------------------

    #[test]
    fn set_get_and_reset_divider_ratio() {
        let ctx = Context::default();
        let id = Id::new("split-test");
        assert_eq!(MudSplitPanel::divider_ratio(&ctx, id), None);
        MudSplitPanel::set_divider_ratio(&ctx, id, 0.3);
        assert_eq!(MudSplitPanel::divider_ratio(&ctx, id), Some(0.3));
        MudSplitPanel::set_divider_ratio(&ctx, id, 7.0);
        assert_eq!(MudSplitPanel::divider_ratio(&ctx, id), Some(1.0));
        MudSplitPanel::reset_divider(&ctx, id);
        assert_eq!(MudSplitPanel::divider_ratio(&ctx, id), None);
    }

    #[test]
    fn setting_a_ratio_uncollapses() {
        let ctx = Context::default();
        let id = Id::new("collapse-test");
        MudSplitPanel::set_first_collapsed(&ctx, id, true);
        assert!(ctx.data(|d| d.get_temp::<SplitState>(id)).unwrap().collapsed);
        MudSplitPanel::set_divider_ratio(&ctx, id, 0.5);
        assert!(!ctx.data(|d| d.get_temp::<SplitState>(id)).unwrap().collapsed);
    }

    // ---- rendering (headless frames) ---------------------------------------

    #[test]
    fn default_split_is_even_and_fills_the_width() {
        let (w, out) = run_in_ui(|ui| {
            let w = ui.available_width();
            (w, MudSplitPanel::new().id_salt("s").show(ui, |_| 1, |_| 2))
        });
        assert_eq!(out.first.inner, 1);
        assert_eq!(out.second.inner, 2);
        assert!((out.first_size - out.second_size).abs() < 1.0);
        assert!((out.first_size + out.second_size + 4.0 - w).abs() < 1.0);
        assert!((out.ratio - 0.5).abs() < 0.01);
        assert!(!out.changed && !out.collapsed);
    }

    #[test]
    fn initial_size_and_ratio_are_applied() {
        let size = run_in_ui(|ui| {
            MudSplitPanel::new()
                .id_salt("a")
                .first_panel_initial_size(100.0)
                .show(ui, |_| {}, |_| {})
        });
        assert!((size.first_size - 100.0).abs() < 1.0);

        let ratio = run_in_ui(|ui| {
            let r = MudSplitPanel::new().id_salt("b").initial_ratio(0.25).show(ui, |_| {}, |_| {});
            let usable = r.first_size + r.second_size;
            (r.first_size / usable, r.ratio)
        });
        assert!((ratio.0 - 0.25).abs() < 0.01);
    }

    #[test]
    fn minimum_size_is_enforced() {
        let out = run_in_ui(|ui| {
            MudSplitPanel::new()
                .id_salt("m")
                .initial_ratio(0.0)
                .min_first_size(60.0)
                .show(ui, |_| {}, |_| {})
        });
        assert!((out.first_size - 60.0).abs() < 1.0);
    }

    #[test]
    fn horizontal_split_uses_the_height() {
        let out = run_in_ui(|ui| {
            MudSplitPanel::new()
                .id_salt("h")
                .horizontal(true)
                .show(ui, |_| {}, |_| {})
        });
        assert!(out.first_size > 20.0 && out.second_size > 20.0);
        assert!((out.first_size - out.second_size).abs() < 1.0);
    }

    #[test]
    fn programmatic_ratio_is_applied_on_the_first_frame() {
        let out = run_in_ui(|ui| {
            let id = MudSplitPanel::divider_id(ui, "p");
            MudSplitPanel::set_divider_ratio(ui.ctx(), id, 0.25);
            MudSplitPanel::new().id_salt("p").show(ui, |_| {}, |_| {})
        });
        let usable = out.first_size + out.second_size;
        assert!((out.first_size / usable - 0.25).abs() < 0.02);
        assert!((MudSplitPanel::divider_ratio(&Context::default(), out.id)).is_none());
    }

    #[test]
    fn collapsed_state_hides_the_first_panel() {
        let out = run_in_ui(|ui| {
            let id = MudSplitPanel::divider_id(ui, "c");
            MudSplitPanel::set_first_collapsed(ui.ctx(), id, true);
            MudSplitPanel::new().id_salt("c").show(ui, |_| {}, |_| {})
        });
        assert!(out.collapsed);
        assert_eq!(out.first_size, 0.0);
        assert_eq!(out.ratio, 0.0);
    }

    #[test]
    fn overlay_takes_no_layout_space() {
        let (before, after) = run_in_ui(|ui| {
            let before = ui.min_rect();
            let _ = MudSplitPanel::new()
                .id_salt("o")
                .use_as_overlay(true)
                .show(ui, |_| {}, |_| {});
            (before, ui.min_rect())
        });
        assert_eq!(before, after);
    }

    #[test]
    fn locked_splitter_does_not_drag() {
        let out = run_in_ui(|ui| {
            MudSplitPanel::new().id_salt("l").locked(true).show(ui, |_| {}, |_| {})
        });
        assert!(!out.splitter.dragged());
        assert!(!out.changed);
    }
}