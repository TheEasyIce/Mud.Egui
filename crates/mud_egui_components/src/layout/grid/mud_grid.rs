//! A twelve-track responsive layout grid panel for `mud_egui`.
//!
//! Port of `MudGrid.cs`. Children flow row-major; each child occupies either a
//! fixed number of tracks, a proportional (star) share, or its natural content
//! width. Leftover space in a row is distributed according to [`Justify`].
//!
//! # Example
//! ```ignore
//! MudGrid::new()
//!     .spacing(3)
//!     .classes(["bg-surface", "p-4", "rounded-lg"])
//!     .show(ui, |grid| {
//!         grid.item(MudItem::new().xs(12).md(6), |ui| { ui.label("Left"); });
//!         grid.item(MudItem::new().xs(12).md(6), |ui| { ui.label("Right"); });
//!     });
//! ```
//!
//! # Architecture
//! The file has three layers, from the bottom up:
//!
//! 1. **Pure layout math** (`pack_rows_into`, `row_widths_into`, `row_offsets_into`,
//!    `plan_into`). No egui-state dependence, no allocation beyond caller-owned buffers,
//!    fully unit-tested. [`MudGrid::plan`] exposes it as a headless planner.
//! 2. **[`MudGrid`]**, the egui-facing component. It owns a private scratch arena that is
//!    reused every frame (spans, widths, offsets, rows, cell boxes, rectangles), so the
//!    only per-frame allocations left are the item list itself: one `Box` per item plus two
//!    small vectors (the items and their resolved frames).
//! 3. **[`GridUi`]**, the builder handed to the `show` closure for adding items.
//!
//! # MudBlazor parity
//! | MudBlazor `MudGrid` | Here |
//! |---------------------|------|
//! | `Spacing` (4px steps, max 16, default 6) | [`MudGrid::spacing`] |
//! | `Justify` | [`MudGrid::justify`] |
//! | `Class` | [`MudGrid::classes`] |
//! | `Style` | [`MudGrid::frame`] (an `egui::Frame`; there is no CSS string in egui) |
//! | `ChildContent` | the closure passed to [`MudGrid::show`] |
//!
//! MudBlazor's grid has no `Variant`, `Color`, `Elevation`, `Disabled`, binding or event
//! parameters, so none are invented here.
//!
//! # Breakpoints
//! The active breakpoint is derived from the width **available to the grid** (container
//! semantics, ideal for panels and side-by-side docks), not from the OS window. Pin it with
//! [`MudGrid::breakpoint`] if you want viewport-style behaviour.

use std::ops::Range;

use egui::{pos2, vec2, Align, Color32, Frame, InnerResponse, Layout, Rect, Sense, Ui, UiBuilder};
use mud_egui_bootstrap::bs_alignment::{
    heights_changed, stretch_target, vertical_offset, AlignSpec, CrossAlign, HAlign,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::breakpoint::Breakpoint;
use mud_egui_core::enums::justify::Justify;
use mud_egui_core::enums::StyleFlavor;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

use super::mud_grid_span::{MudGridSpan, TRACK_COUNT};
use super::mud_item::MudItem;

/// Base rem size (in points) passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Width used when the parent offers an unbounded width (e.g., a horizontal
/// scroll area), where standard tracks would otherwise be meaningless.
pub const FALLBACK_WIDTH: f32 = 1280.0;

/// Maximum allowed spacing unit (each unit equates to 4px). Matches MudBlazor's documented
/// maximum of 16.
pub const MAX_SPACING: u8 = 16;

/// Pixels per spacing unit (matches MudBlazor's 4px spacing scale).
pub const SPACING_UNIT: f32 = 4.0;

/// Smallest width (px) of the `Sm` breakpoint.
pub const BREAKPOINT_SM: f32 = 600.0;
/// Smallest width (px) of the `Md` breakpoint.
pub const BREAKPOINT_MD: f32 = 960.0;
/// Smallest width (px) of the `Lg` breakpoint.
pub const BREAKPOINT_LG: f32 = 1280.0;
/// Smallest width (px) of the `Xl` breakpoint.
pub const BREAKPOINT_XL: f32 = 1920.0;
/// Smallest width (px) of the `Xxl` breakpoint.
pub const BREAKPOINT_XXL: f32 = 2560.0;

/// Width changes smaller than this (px) are not worth a repaint when caching `Auto` widths.
const AUTO_EPSILON: f32 = 0.5;

// =============================================================================
// Interaction / accessibility state
// =============================================================================

/// Lightweight interaction flags captured from the previous frame and used by
/// Tailwind variant resolution.
///
/// # Example
/// ```ignore
/// let s = InteractionState { hovered: true, disabled: true, ..Default::default() };
/// assert!(!s.to_tw_state().hovered); // disabled suppresses hover/active styling
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InteractionState {
    /// The pointer was over the grid last frame.
    pub hovered: bool,
    /// A pointer button was held down on the grid last frame.
    pub active: bool,
    /// The grid response had keyboard focus last frame.
    pub focused: bool,
    /// The egui visuals are in dark mode.
    pub dark: bool,
    /// Suppresses hover/active variants (see [`MudGrid::disabled`]).
    pub disabled: bool,
}

impl InteractionState {
    /// Convert into the `TwState` expected by the Tailwind resolver.
    #[inline]
    pub fn to_tw_state(self) -> TwState {
        TwState {
            hovered: self.hovered && !self.disabled,
            active: self.active && !self.disabled,
            dark: self.dark,
        }
    }
}

// =============================================================================
// Auto-width measuring strategy
// =============================================================================

/// How `Auto` spans learn their natural width.
///
/// # Example
/// ```ignore
/// MudGrid::new().auto_measure(AutoMeasure::Cached).show(ui, |grid| { /* ... */ });
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutoMeasure {
    /// Run an invisible sizing pass of the cell content every frame. Always exact, but each
    /// `Auto` cell's closure runs twice per frame. This is the default.
    #[default]
    Probe,
    /// Remember each `Auto` cell's natural width from the previous frame (stored in egui's
    /// temp data) so the closure runs **once** per frame. A cell is probed only on a cache
    /// miss; when a width changes the grid requests one repaint to settle, so content that
    /// resizes is at most one frame stale.
    Cached,
}

// =============================================================================
// Pure layout math (no egui state; fully unit-testable)
// =============================================================================

/// How leftover row space is distributed (local mirror of [`Justify`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RowAlign {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl From<Justify> for RowAlign {
    fn from(j: Justify) -> Self {
        match j {
            Justify::Start => RowAlign::Start,
            Justify::Center => RowAlign::Center,
            Justify::End => RowAlign::End,
            Justify::SpaceBetween => RowAlign::SpaceBetween,
            Justify::SpaceAround => RowAlign::SpaceAround,
            Justify::SpaceEvenly => RowAlign::SpaceEvenly,
        }
    }
}

/// The breakpoint whose range contains `width` (pixels). NaN and negative widths are `Xs`.
pub(crate) fn breakpoint_for_width(width: f32) -> Breakpoint {
    if width >= BREAKPOINT_XXL {
        Breakpoint::Xxl
    } else if width >= BREAKPOINT_XL {
        Breakpoint::Xl
    } else if width >= BREAKPOINT_LG {
        Breakpoint::Lg
    } else if width >= BREAKPOINT_MD {
        Breakpoint::Md
    } else if width >= BREAKPOINT_SM {
        Breakpoint::Sm
    } else {
        Breakpoint::Xs
    }
}

/// `available` when finite, otherwise [`FALLBACK_WIDTH`].
#[inline]
pub(crate) fn effective_width(available: f32) -> f32 {
    if available.is_finite() {
        available
    } else {
        FALLBACK_WIDTH
    }
}

/// Gap in pixels for a spacing value (clamped to [`MAX_SPACING`]).
#[inline]
pub(crate) fn gap_px(spacing: u8) -> f32 {
    f32::from(spacing.min(MAX_SPACING)) * SPACING_UNIT
}

/// Width of one track when `width` holds 12 tracks separated by 11 gaps.
#[inline]
pub(crate) fn track_width(width: f32, gap: f32) -> f32 {
    ((width - (f32::from(TRACK_COUNT) - 1.0) * gap) / f32::from(TRACK_COUNT)).max(0.0)
}

/// Width of a cell spanning `tracks` tracks (it also swallows the gaps between them).
#[inline]
pub(crate) fn tracks_to_width(tracks: u8, track_w: f32, gap: f32) -> f32 {
    let t = f32::from(tracks.clamp(1, TRACK_COUNT));
    t * track_w + (t - 1.0) * gap
}

/// Greedy row packing into a caller-owned buffer (cleared first).
///
/// Items are appended to the current row until the next one would push the row past 12
/// tracks; `Star`/`Auto` reserve one track each (see [`MudGridSpan::track_need`]). Every row
/// holds at least one item, so an oversized item gets a row of its own.
pub(crate) fn pack_rows_into(spans: &[MudGridSpan], out: &mut Vec<Range<usize>>) {
    out.clear();
    let mut start = 0;
    let mut used = 0u32;

    for (i, span) in spans.iter().enumerate() {
        let need = u32::from(span.track_need());

        if used + need > u32::from(TRACK_COUNT) && i > start {
            out.push(start..i);
            start = i;
            used = 0;
        }
        used += need;
    }

    if start < spans.len() {
        out.push(start..spans.len());
    }
}

/// Allocating convenience wrapper around [`pack_rows_into`].
#[allow(dead_code)]
pub(crate) fn pack_rows(spans: &[MudGridSpan]) -> Vec<Range<usize>> {
    let mut rows = Vec::new();
    pack_rows_into(spans, &mut rows);
    rows
}

/// Widths of every cell in one row, written into a caller-owned buffer (cleared first).
///
/// Fixed (`Tracks`) and `Auto` cells are sized first; the space left after them and the gaps
/// is shared among `Star` cells by weight (equally when every weight is zero). Overfull rows
/// clamp star widths to `0.0` rather than going negative.
pub(crate) fn row_widths_into(
    spans: &[MudGridSpan],
    auto_widths: &[f32],
    width: f32,
    gap: f32,
    track_w: f32,
    out: &mut Vec<f32>,
) {
    let n = spans.len();
    out.clear();
    out.resize(n, 0.0);

    let mut fixed = 0.0f32;
    let mut weight_sum = 0.0f32;
    let mut star_count = 0usize;

    for (i, span) in spans.iter().enumerate() {
        match span.normalized() {
            MudGridSpan::Tracks(t) => {
                out[i] = tracks_to_width(t, track_w, gap);
                fixed += out[i];
            }
            MudGridSpan::Auto => {
                out[i] = auto_widths.get(i).copied().unwrap_or(0.0).max(0.0);
                fixed += out[i];
            }
            MudGridSpan::Star(w) => {
                out[i] = w;
                weight_sum += w;
                star_count += 1;
            }
            MudGridSpan::Unset => {}
        }
    }

    let gaps = gap * n.saturating_sub(1) as f32;
    let remaining = (width - fixed - gaps).max(0.0);

    for (i, span) in spans.iter().enumerate() {
        if span.is_star() {
            let w = out[i];
            out[i] = if weight_sum > 0.0 {
                remaining * (w / weight_sum)
            } else {
                remaining / star_count.max(1) as f32
            };
        }
    }
}

/// Allocating convenience wrapper around [`row_widths_into`].
#[allow(dead_code)]
pub(crate) fn row_widths(
    spans: &[MudGridSpan],
    auto_widths: &[f32],
    width: f32,
    gap: f32,
    track_w: f32,
) -> Vec<f32> {
    let mut out = Vec::with_capacity(spans.len());
    row_widths_into(spans, auto_widths, width, gap, track_w, &mut out);
    out
}

/// Left edge of every cell in one row (relative to the grid's left edge), written into a
/// caller-owned buffer (cleared first). Leftover space is distributed per `align`; an
/// overfull row never gets a negative offset.
pub(crate) fn row_offsets_into(
    widths: &[f32],
    width: f32,
    gap: f32,
    align: RowAlign,
    out: &mut Vec<f32>,
) {
    out.clear();
    let n = widths.len();
    if n == 0 {
        return;
    }

    let total: f32 = widths.iter().sum::<f32>() + gap * (n - 1) as f32;
    let leftover = (width - total).max(0.0);

    let (start, extra) = match align {
        RowAlign::Start => (0.0, 0.0),
        RowAlign::Center => (leftover / 2.0, 0.0),
        RowAlign::End => (leftover, 0.0),
        RowAlign::SpaceBetween if n > 1 => (0.0, leftover / (n - 1) as f32),
        RowAlign::SpaceBetween => (0.0, 0.0),
        RowAlign::SpaceAround => {
            let e = leftover / n as f32;
            (e / 2.0, e)
        }
        RowAlign::SpaceEvenly => {
            let e = leftover / (n + 1) as f32;
            (e, e)
        }
    };

    let mut x = start;
    out.extend(widths.iter().map(|w| {
        let pos = x;
        x += w + gap + extra;
        pos
    }));
}

/// Allocating convenience wrapper around [`row_offsets_into`].
#[allow(dead_code)]
pub(crate) fn row_offsets(widths: &[f32], width: f32, gap: f32, align: RowAlign) -> Vec<f32> {
    let mut out = Vec::with_capacity(widths.len());
    row_offsets_into(widths, width, gap, align, &mut out);
    out
}

/// A cell rectangle with a finite left/top/width and an *unbounded* height, so content is
/// free to grow downward.
#[inline]
pub(crate) fn cell_rect(x: f32, y: f32, width: f32) -> Rect {
    Rect::from_min_size(pos2(x, y), vec2(width, f32::INFINITY))
}

/// Bottom edge of the grid: the running `y` minus the trailing gap, never above `min_y`.
#[inline]
pub(crate) fn grid_bottom(no_rows: bool, y: f32, gap: f32, min_y: f32) -> f32 {
    if no_rows {
        min_y
    } else {
        (y - gap).max(min_y)
    }
}

/// A content height that is safe to accumulate: non-finite and negative values become `0.0`.
#[inline]
pub(crate) fn finite_height(height: f32) -> f32 {
    if height.is_finite() {
        height.max(0.0)
    } else {
        0.0
    }
}

/// `true` when freshly measured `Auto` widths differ enough from the cached ones to be worth
/// storing (and, if a cache existed, repainting for).
pub(crate) fn auto_widths_changed(previous: Option<&[f32]>, current: &[f32]) -> bool {
    match previous {
        None => true,
        Some(prev) => {
            prev.len() != current.len()
                || prev.iter().zip(current).any(|(a, b)| (a - b).abs() > AUTO_EPSILON)
        }
    }
}

/// Horizontal geometry of one laid-out cell, as computed by the headless planner.
///
/// `x` is relative to the grid's left edge. Heights are unknown until content is drawn, so
/// they are not part of the plan (see [`MudGrid::cell_rects`] for the drawn rectangles).
///
/// # Example
/// ```ignore
/// let cells = MudGrid::new().plan(&[MudItem::new().md(6); 2], 1200.0, &[]);
/// assert_eq!(cells[1].x, 612.0);
/// assert_eq!(cells[1].right(), 1200.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellBox {
    /// Index of the item in the order it was added.
    pub index: usize,
    /// Zero-based row the item was packed into.
    pub row: usize,
    /// Left edge, relative to the grid's left edge.
    pub x: f32,
    /// Cell width in pixels.
    pub width: f32,
}

impl CellBox {
    /// Right edge, relative to the grid's left edge.
    #[inline]
    pub fn right(&self) -> f32 {
        self.x + self.width
    }
}

/// Reusable buffers for [`plan_into`].
#[derive(Debug, Default)]
pub(crate) struct PlanScratch {
    rows: Vec<Range<usize>>,
    widths: Vec<f32>,
    offsets: Vec<f32>,
}

/// Plans the whole grid: packs `spans` into rows, sizes every cell and positions it.
///
/// `auto_widths` is indexed by *item* (not by row position); missing entries count as `0.0`.
/// `out` is cleared and receives one [`CellBox`] per span, in item order.
pub(crate) fn plan_into(
    spans: &[MudGridSpan],
    auto_widths: &[f32],
    width: f32,
    gap: f32,
    align: RowAlign,
    scratch: &mut PlanScratch,
    out: &mut Vec<CellBox>,
) {
    out.clear();
    pack_rows_into(spans, &mut scratch.rows);
    let track_w = track_width(width, gap);

    for (r, range) in scratch.rows.iter().enumerate() {
        let row_spans = &spans[range.clone()];
        let row_auto = auto_widths.get(range.clone()).unwrap_or(&[]);

        row_widths_into(row_spans, row_auto, width, gap, track_w, &mut scratch.widths);
        row_offsets_into(&scratch.widths, width, gap, align, &mut scratch.offsets);

        out.extend(
            scratch
                .widths
                .iter()
                .zip(&scratch.offsets)
                .enumerate()
                .map(|(k, (w, x))| CellBox { index: range.start + k, row: r, x: *x, width: *w }),
        );
    }
}

// =============================================================================
// Per-item styling
// =============================================================================

/// Optional styling for a single grid item: MudBlazor's `MudItem` `Class` / `Style`.
///
/// `classes` are resolved through the grid's [`StyleFlavor`] (Bootstrap or Tailwind) into an
/// `egui::Frame` wrapped around the item's content. `frame` overrides that with an explicit
/// frame (the egui equivalent of an inline `Style`).
///
/// # Example
/// ```ignore
/// let card = ItemStyle::new().classes(["bg-surface", "p-4", "rounded-lg"]);
/// grid.styled_item(MudItem::new().xs(12).md(4), card, |ui| { ui.label("Card"); });
/// ```
#[derive(Debug, Clone, Default)]
pub struct ItemStyle {
    /// Utility classes, resolved by the grid's flavor.
    pub classes: Vec<String>,
    /// Explicit frame; wins over `classes` when set.
    pub frame: Option<Frame>,
}

impl ItemStyle {
    /// An empty style (no frame is drawn).
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends one utility class.
    #[inline]
    pub fn class(mut self, class: impl AsRef<str>) -> Self {
        self.classes.push(class.as_ref().to_owned());
        self
    }

    /// Appends several utility classes.
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.classes.extend(classes.into_iter().map(|c| c.as_ref().to_owned()));
        self
    }

    /// Uses an explicit `egui::Frame` instead of resolving classes.
    #[inline]
    pub fn frame(mut self, frame: Frame) -> Self {
        self.frame = Some(frame);
        self
    }

    /// `true` when neither classes nor a frame are set (nothing would be drawn).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.classes.is_empty() && self.frame.is_none()
    }
}

// =============================================================================
// Component
// =============================================================================

/// Applies one more class on top of an already parsed spec (last class wins).
fn merge_align(spec: AlignSpec, cls: &str) -> AlignSpec {
    let mut out = spec;
    let parsed = AlignSpec::from_classes([cls]);
    if parsed.align_self.is_some() { out.align_self = parsed.align_self; }
    if parsed.align_items.is_some() { out.align_items = parsed.align_items; }
    if parsed.justify_content.is_some() { out.justify_content = parsed.justify_content; }
    if parsed.text.is_some() { out.text = parsed.text; }
    out.auto.start |= parsed.auto.start;
    out.auto.end |= parsed.auto.end;
    out.auto.top |= parsed.auto.top;
    out.auto.bottom |= parsed.auto.bottom;
    out.fill_width |= parsed.fill_width;
    out.fill_height |= parsed.fill_height;
    out
}

/// Previous-frame heights, remembered in egui temp memory so alignment classes
/// (`align-self-end`, `align-self-center`, `align-self-stretch`) know how tall their row is.
#[derive(Clone, Default)]
struct RowHeights {
    /// Natural height of each row (tallest item, excluding alignment padding).
    rows: Vec<f32>,
    /// Natural height of each item (excluding alignment padding).
    items: Vec<f32>,
}

/// Buffers reused across frames so a steady-state `show` does not allocate them again.
#[derive(Default)]
struct Scratch {
    spans: Vec<MudGridSpan>,
    auto: Vec<f32>,
    measured: Vec<f32>,
    plan: PlanScratch,
    cells: Vec<CellBox>,
    rects: Vec<Rect>,
    breakpoint: Option<Breakpoint>,
    width: f32,
}

/// A twelve-track responsive grid container.
///
/// # Example
/// ```ignore
/// MudGrid::new()
///     .justify(Justify::Center)
///     .spacing(4)
///     .show(ui, |grid| {
///         for n in 0..6 {
///             grid.item(MudItem::new().xs(12).sm(6).lg(4), move |ui| {
///                 ui.label(format!("Card {n}"));
///             });
///         }
///     });
/// ```
#[must_use = "MudGrid does nothing until you call `.show(ui, |grid| ...)`"]
pub struct MudGrid {
    /// How leftover space in rows is justified.
    pub justify: Justify,

    /// Spacing units of 4px each (default is 6 = 24px, capped at [`MAX_SPACING`]).
    pub spacing: u8,

    /// Optional manual breakpoint override.
    pub breakpoint: Option<Breakpoint>,

    /// Shared component state (CSS-like utility classes, elevation, ripple).
    pub base: ComponentBase,

    /// Which utility-class system to resolve against.
    pub flavor: StyleFlavor,

    /// Bootstrap resolver cache.
    pub bs_style: BsStyle,

    /// Tailwind resolver cache.
    pub tw_style: TwStyle,

    /// Interaction / theme state fed to the Tailwind resolver.
    pub interaction: InteractionState,

    /// Explicit frame that replaces the one resolved from `base` (egui's `Style`).
    pub frame_override: Option<Frame>,

    /// How `Auto` spans are measured.
    pub measure: AutoMeasure,

    /// Reused per-frame buffers and last-frame results.
    scratch: Scratch,
}

impl Default for MudGrid {
    fn default() -> Self {
        Self {
            justify: Justify::Start,
            spacing: 6, // 24px
            breakpoint: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            interaction: InteractionState::default(),
            frame_override: None,
            measure: AutoMeasure::default(),
            scratch: Scratch::default(),
        }
    }
}

impl MudGrid {
    /// A grid with MudBlazor's defaults: `Justify::Start`, spacing 6 (24px), automatic breakpoint.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets how leftover space in each row is distributed.
    #[inline]
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    /// Sets the gap between items in 4px steps (clamped to [`MAX_SPACING`] = 16).
    #[inline]
    pub fn spacing(mut self, spacing: u8) -> Self {
        self.spacing = spacing.min(MAX_SPACING);
        self
    }

    /// Pins the breakpoint instead of deriving it from the available width.
    #[inline]
    pub fn breakpoint(mut self, breakpoint: Breakpoint) -> Self {
        self.breakpoint = Some(breakpoint);
        self
    }

    /// Selects the utility-class resolver.
    #[inline]
    pub fn flavor(mut self, flavor: StyleFlavor) -> Self {
        self.flavor = flavor;
        self
    }

    /// Adds utility classes (e.g. `["bg-surface", "p-4", "rounded-lg"]`) to
    /// the underlying [`ComponentBase`].
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Draws the grid inside this explicit `egui::Frame` instead of the one resolved from
    /// the utility classes.
    #[inline]
    pub fn frame(mut self, frame: Frame) -> Self {
        self.frame_override = Some(frame);
        self
    }

    /// Chooses how `Auto` spans are measured (see [`AutoMeasure`]).
    #[inline]
    pub fn auto_measure(mut self, measure: AutoMeasure) -> Self {
        self.measure = measure;
        self
    }

    /// Suppresses hover/active styling from the Tailwind resolver. The flag persists across
    /// frames until changed.
    #[inline]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.interaction.disabled = disabled;
        self
    }

    /// The gap between items in pixels for the current spacing.
    #[inline]
    pub fn gap(&self) -> f32 {
        gap_px(self.spacing)
    }

    /// The breakpoint used by the most recent `show` (`None` before the first frame).
    ///
    /// Handy for adapting *content* to the same breakpoint the grid chose.
    #[inline]
    pub fn active_breakpoint(&self) -> Option<Breakpoint> {
        self.scratch.breakpoint
    }

    /// The width in pixels the most recent `show` laid out against (`0.0` before the first frame).
    #[inline]
    pub fn last_width(&self) -> f32 {
        self.scratch.width
    }

    /// The planned horizontal geometry of every cell from the most recent `show`.
    #[inline]
    pub fn last_cells(&self) -> &[CellBox] {
        &self.scratch.cells
    }

    /// The rectangles (screen space) each item occupied in the most recent `show`, in item
    /// order. Use them for hit-testing, overlays or scrolling an item into view.
    #[inline]
    pub fn cell_rects(&self) -> &[Rect] {
        &self.scratch.rects
    }

    /// Headless layout: where each of `items` would sit in a grid `width` pixels wide with
    /// this grid's spacing, justification and breakpoint settings.
    ///
    /// No egui context is needed, which makes it ideal for tests and tooling. `auto_widths`
    /// supplies the natural width of every `Auto` item by item index (missing entries are
    /// `0.0`). A non-finite `width` falls back to [`FALLBACK_WIDTH`].
    ///
    /// ```ignore
    /// let items = [MudItem::new().xs(12).md(6); 2];
    /// let cells = MudGrid::new().plan(&items, 1200.0, &[]);
    /// assert_eq!(cells.len(), 2);
    /// assert_eq!(cells[0].row, cells[1].row); // 1200px is `Md`, so they share a row
    /// ```
    pub fn plan(&self, items: &[MudItem], width: f32, auto_widths: &[f32]) -> Vec<CellBox> {
        let width = effective_width(width);
        let gap = gap_px(self.spacing);
        let bp = self.breakpoint.unwrap_or_else(|| breakpoint_for_width(width));
        let spans: Vec<MudGridSpan> = items.iter().map(|i| i.resolve_span(bp)).collect();

        let mut scratch = PlanScratch::default();
        let mut out = Vec::with_capacity(items.len());
        plan_into(&spans, auto_widths, width, gap, RowAlign::from(self.justify), &mut scratch, &mut out);
        out
    }

    /// Draws the grid layout. Use `.item()` on the provided [`GridUi`] builder to add child items.
    ///
    /// The closure's return value is passed back in the [`InnerResponse`]; the response itself
    /// covers the whole grid and senses hover.
    pub fn show<'a, R>(
        &mut self,
        ui: &mut Ui,
        add_items: impl FnOnce(&mut GridUi<'a>) -> R,
    ) -> InnerResponse<R> {
        let mut grid_ui = GridUi { items: Vec::new() };
        let result = add_items(&mut grid_ui);
        let mut items = grid_ui.items;

        // Resolve the active style flavour into `egui::Frame`s (grid + any styled items).
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);

        self.interaction.dark = ui.visuals().dark_mode;

        #[allow(unused_mut)]
        let mut resolve = |base: &ComponentBase| match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(base, &lookup),
            StyleFlavor::Tailwind => {
                let state = self.interaction.to_tw_state();
                self.tw_style.build_frame(base, state, REM_SIZE_PT, &lookup)
            }
        };

        let frame = match self.frame_override {
            Some(f) => f,
            None => resolve(&self.base),
        };

        let item_frames: Vec<Option<Frame>> = items
            .iter()
            .map(|item| {
                item.style.as_ref().filter(|s| !s.is_empty()).map(|style| match style.frame {
                    Some(f) => f,
                    None => resolve(&ComponentBase::new().classes(&style.classes)),
                })
            })
            .collect();

        // Copy the settings the layout needs so the frame closure borrows nothing from `self`.
        let spacing = self.spacing;
        let bp_override = self.breakpoint;
        let align = RowAlign::from(self.justify);
        let measure = self.measure;
        // `align-items-*` on the grid is the default vertical placement for every item.
        let default_cross: Option<CrossAlign> =
            AlignSpec::from_classes(self.base.class_names()).align_items;
        let mut scratch = std::mem::take(&mut self.scratch);

        // Render the grid inside the resolved frame.
        let inner = frame.show(ui, |ui| {
            let avail = ui.available_rect_before_wrap();
            let width = effective_width(avail.width());
            let gap = gap_px(spacing);
            let bp = bp_override.unwrap_or_else(|| breakpoint_for_width(width));
            let n = items.len();
            let cell_layout = Layout::top_down(Align::Min);

            scratch.breakpoint = Some(bp);
            scratch.width = width;

            scratch.spans.clear();
            scratch.spans.extend(items.iter().map(|item| item.span.resolve_span(bp)));

            scratch.auto.clear();
            scratch.auto.resize(n, 0.0);
            scratch.measured.clear();
            scratch.measured.resize(n, 0.0);

            // Phase 1: learn the natural width of every `Auto` cell.
            let has_auto = scratch.spans.iter().any(|s| s.is_auto());
            let cache_id = ui.id().with("mud_grid_auto_widths");
            let cached: Option<Vec<f32>> = if has_auto && measure == AutoMeasure::Cached {
                ui.data_mut(|d| d.get_temp::<Vec<f32>>(cache_id))
            } else {
                None
            };

            if has_auto {
                for i in 0..n {
                    if !scratch.spans[i].is_auto() {
                        continue;
                    }
                    let hit = cached
                        .as_ref()
                        .and_then(|c| c.get(i).copied())
                        .filter(|w| w.is_finite() && *w > 0.0);
                    scratch.auto[i] = match hit {
                        Some(w) => w.min(width),
                        None => {
                            let mut probe = ui.new_child(
                                UiBuilder::new()
                                    .id_salt((i, "measure"))
                                    .max_rect(cell_rect(avail.min.x, avail.min.y, width))
                                    .layout(cell_layout)
                                    .invisible()
                                    .sizing_pass(),
                            );
                            paint_item(&mut probe, &mut items[i], item_frames[i], false, None);
                            probe.min_rect().width().min(width)
                        }
                    };
                }
            }

            // Phase 2: pure planning (rows, widths, offsets).
            plan_into(
                &scratch.spans,
                &scratch.auto,
                width,
                gap,
                align,
                &mut scratch.plan,
                &mut scratch.cells,
            );

            scratch.rects.clear();
            scratch.rects.resize(n, Rect::NOTHING);

            // Alignment classes need last frame's row/item heights. Grids without any vertical
            // alignment never touch this memory key.
            let item_cross: Vec<Option<CrossAlign>> = items
                .iter()
                .enumerate()
                .map(|(i, it)| {
                    if it.align.fill_height {
                        Some(CrossAlign::Stretch)
                    } else {
                        // Like a CSS flex row (`align-items: stretch`), framed items in a row
                        // share one height unless told otherwise.
                        it.align
                            .vertical(default_cross)
                            .or(item_frames[i].map(|_| CrossAlign::Stretch))
                    }
                })
                .collect();
            let uses_vertical = item_cross
                .iter()
                .any(|c| matches!(c, Some(CrossAlign::Center | CrossAlign::End | CrossAlign::Stretch)));
            let heights_id = ui.id().with("mud_grid_row_heights");
            let prev_heights: Option<RowHeights> = if uses_vertical {
                ui.data_mut(|d| d.get_temp::<RowHeights>(heights_id))
            } else {
                None
            };
            let mut new_rows: Vec<f32> = Vec::new();
            let mut new_items: Vec<f32> = if uses_vertical { vec![0.0; n] } else { Vec::new() };

            // Phase 3: draw row by row.
            let mut y = avail.min.y;
            for (r, range) in scratch.plan.rows.iter().enumerate() {
                let mut row_height = 0.0f32;

                for cell in &scratch.cells[range.clone()] {
                    let i = cell.index;
                    let span = scratch.spans[i];
                    let spec = items[i].align;

                    // Vertical placement inside the row, from the previous frame's heights.
                    let slot = prev_heights
                        .as_ref()
                        .and_then(|m| m.rows.get(r))
                        .copied()
                        .unwrap_or(0.0);
                    let prev_content = prev_heights
                        .as_ref()
                        .and_then(|m| m.items.get(i))
                        .copied()
                        .unwrap_or(0.0);
                    let (dy, stretch_to) = match item_cross[i] {
                        Some(a) => (vertical_offset(a, slot, prev_content), stretch_target(a, slot)),
                        None => (0.0, None),
                    };

                    // Horizontal placement of the content inside the cell (`text-*`, `ms-auto`).
                    let layout = match spec.horizontal() {
                        Some(HAlign::Center) => Layout::top_down(Align::Center),
                        Some(HAlign::End) => Layout::top_down(Align::Max),
                        _ => cell_layout,
                    };

                    let rect = cell_rect(avail.min.x + cell.x, y + dy, cell.width);
                    let mut child = ui.new_child(
                        UiBuilder::new()
                            .id_salt((r, i - range.start))
                            .max_rect(rect)
                            .layout(layout),
                    );

                    // A cached `Auto` cell must size itself so its real width can be re-measured.
                    let self_sizing =
                        measure == AutoMeasure::Cached && span.is_auto() && !spec.fill_width;
                    if !self_sizing {
                        child.set_width(cell.width);
                    }

                    let natural = paint_item(
                        &mut child,
                        &mut items[i],
                        item_frames[i],
                        !span.is_auto() || spec.fill_width,
                        stretch_to,
                    );

                    let used = child.min_rect();
                    let drawn = finite_height(used.height());
                    // Unaligned items keep the exact original arithmetic. A stretched item
                    // reports its natural height so a taller slot never ratchets the row.
                    let h = if stretch_to.is_some() { natural } else { drawn };
                    row_height = row_height.max(h);
                    if uses_vertical {
                        new_items[i] = h;
                    }
                    scratch.rects[i] = Rect::from_min_size(rect.min, vec2(cell.width, drawn));
                    if span.is_auto() {
                        scratch.measured[i] = used.width().min(width);
                    }
                }

                if uses_vertical {
                    new_rows.push(row_height);
                }
                y += row_height + gap;
            }

            // Remember this frame's heights; one corrective repaint when they changed.
            if uses_vertical {
                let changed = match &prev_heights {
                    None => true,
                    Some(m) => {
                        heights_changed(Some(&m.rows), &new_rows, AUTO_EPSILON)
                            || heights_changed(Some(&m.items), &new_items, AUTO_EPSILON)
                    }
                };
                if changed {
                    ui.data_mut(|d| {
                        d.insert_temp(heights_id, RowHeights { rows: new_rows, items: new_items })
                    });
                    ui.ctx().request_repaint();
                }
            }

            // Phase 4: remember `Auto` widths for the next frame (cached mode only).
            if has_auto
                && measure == AutoMeasure::Cached
                && auto_widths_changed(cached.as_deref(), &scratch.measured)
            {
                let snapshot = scratch.measured.clone();
                ui.data_mut(|d| d.insert_temp(cache_id, snapshot));
                if cached.is_some() {
                    ui.ctx().request_repaint();
                }
            }

            let bottom = grid_bottom(scratch.plan.rows.is_empty(), y, gap, avail.min.y);
            let right = avail.min.x + width;
            let used = Rect::from_min_max(avail.min, pos2(right, bottom));

            // Allocate the final rect to get the interaction response
            let response = ui.allocate_rect(used, Sense::hover());

            // Return both the user's result and the response
            (result, response)
        });

        self.scratch = scratch;

        // Update interaction state for the next frame.
        self.interaction.hovered = inner.response.hovered();
        self.interaction.active = inner.response.is_pointer_button_down_on();
        self.interaction.focused = inner.response.has_focus();

        InnerResponse::new(inner.inner.0, inner.response)
    }
}

/// Draws one item's content, wrapped in its frame when it has one.
///
/// `fill` stretches a framed item to the full cell width (fixed and star cells); `Auto` cells
/// pass `false` so the frame hugs the content.
///
/// `stretch_to` (from `align-self-stretch` / `h-100`) pads the item down to that outer height.
/// Returns the item's *natural* outer height, i.e. without that padding.
fn paint_item(
    ui: &mut Ui,
    item: &mut GridItem<'_>,
    frame: Option<Frame>,
    fill: bool,
    stretch_to: Option<f32>,
) -> f32 {
    match frame {
        None => {
            (item.content)(ui);
            let natural = finite_height(ui.min_rect().height());
            pad_to(ui, natural, stretch_to);
            natural
        }
        Some(frame) => {
            let margin = finite_height(frame.total_margin().sum().y);
            let mut inner_natural = 0.0f32;
            frame.show(ui, |ui| {
                if fill {
                    ui.set_width(ui.available_width());
                }
                (item.content)(ui);
                inner_natural = finite_height(ui.min_rect().height());
                pad_to(ui, inner_natural, stretch_to.map(|h| h - margin));
            });
            inner_natural + margin
        }
    }
}

/// Grows `ui` downwards so its content is `target` points tall (no-op when already taller).
fn pad_to(ui: &mut Ui, natural: f32, target: Option<f32>) {
    if let Some(target) = target {
        let extra = target - natural;
        if extra > 0.0 {
            ui.allocate_space(vec2(0.0, extra));
        }
    }
}

// =============================================================================
// GridUi Builder
// =============================================================================

/// The builder handed to the closure given to [`MudGrid::show`]. Items are collected first and
/// laid out afterwards, which is what lets the grid know every span before drawing anything.
///
/// # Example
/// ```ignore
/// grid.item(MudItem::new().xs(12).md(8), |ui| { ui.label("Main"); });
/// grid.item(MudItem::new().xs(12).md(4), |ui| { ui.label("Aside"); });
/// grid.spacer(MudItem::new().xs(12));
/// ```
pub struct GridUi<'a> {
    items: Vec<GridItem<'a>>,
}

struct GridItem<'a> {
    span: MudItem,
    style: Option<ItemStyle>,
    /// Bootstrap alignment utilities (`align-self-*`, `text-*`, `ms-auto`, ...).
    align: AlignSpec,
    content: Box<dyn FnMut(&mut Ui) + 'a>,
}

impl<'a> GridUi<'a> {
    /// Adds an item. `span` is anything convertible into a [`MudItem`] (a `MudItem`, or a bare
    /// `MudGridSpan` that applies at every breakpoint). The closure may run more than once per
    /// frame (an `Auto` probe pass), so keep side effects out of it.
    pub fn item<F, T>(&mut self, span: impl Into<MudItem>, mut content: F)
    where
        F: FnMut(&mut Ui) -> T + 'a,
    {
        let span: MudItem = span.into();
        self.items.push(GridItem {
            align: AlignSpec::from_classes(span.class.split_whitespace()),
            span,
            style: None,
            content: Box::new(move |ui| {
                content(ui);
            }),
        });
    }

    /// Adds an item with its own [`ItemStyle`] (MudBlazor's per-item `Class` / `Style`).
    pub fn styled_item<F, T>(&mut self, span: impl Into<MudItem>, style: ItemStyle, mut content: F)
    where
        F: FnMut(&mut Ui) -> T + 'a,
    {
        let span: MudItem = span.into();
        let mut align = AlignSpec::from_classes(span.class.split_whitespace());
        // Alignment classes on the style win over the item's own `.class(..)` (they come last).
        for cls in &style.classes {
            align = merge_align(align, cls);
        }
        self.items.push(GridItem {
            align,
            span,
            style: Some(style),
            content: Box::new(move |ui| {
                content(ui);
            }),
        });
    }

    /// Adds one item per element of `data`, all sharing the same `span`: the counterpart of a
    /// Razor `@foreach` around a `<MudItem>`.
    ///
    /// `content` is cloned once per element, so it should capture only cheap, `Copy`-like state
    /// (references, indices).
    ///
    /// ```ignore
    /// let names = ["Ada", "Grace", "Linus"];
    /// grid.each(MudItem::new().xs(12).sm(4), names, |ui, name| { ui.label(*name); });
    /// ```
    pub fn each<I, T, F>(&mut self, span: impl Into<MudItem>, data: I, content: F)
    where
        I: IntoIterator<Item = T>,
        T: 'a,
        F: FnMut(&mut Ui, &T) + Clone + 'a,
    {
        let span: MudItem = span.into();
        for value in data {
            let mut f = content.clone();
            self.items.push(GridItem {
                span,
                style: None,
                align: AlignSpec::from_classes(span.class.split_whitespace()),
                content: Box::new(move |ui: &mut Ui| f(ui, &value)),
            });
        }
    }

    /// Adds an empty item that only occupies space, e.g. to push the next item to a new row or
    /// to leave a column blank.
    pub fn spacer(&mut self, span: impl Into<MudItem>) {
        self.item(span, |_ui| {});
    }

    /// Number of items added so far.
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// `true` when no item has been added yet.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use MudGridSpan::{Auto, Star, Tracks};

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    fn plan(spans: &[MudGridSpan], auto: &[f32], width: f32, gap: f32, align: RowAlign) -> Vec<CellBox> {
        let mut scratch = PlanScratch::default();
        let mut out = Vec::new();
        plan_into(spans, auto, width, gap, align, &mut scratch, &mut out);
        out
    }

    #[test]
    fn breakpoints() {
        assert!(matches!(breakpoint_for_width(0.0), Breakpoint::Xs));
        assert!(matches!(breakpoint_for_width(599.9), Breakpoint::Xs));
        assert!(matches!(breakpoint_for_width(600.0), Breakpoint::Sm));
        assert!(matches!(breakpoint_for_width(960.0), Breakpoint::Md));
        assert!(matches!(breakpoint_for_width(1280.0), Breakpoint::Lg));
        assert!(matches!(breakpoint_for_width(1920.0), Breakpoint::Xl));
        assert!(matches!(breakpoint_for_width(2560.0), Breakpoint::Xxl));
        assert!(matches!(breakpoint_for_width(f32::NAN), Breakpoint::Xs));
    }

    #[test]
    fn breakpoint_upper_edges_and_extremes() {
        assert!(matches!(breakpoint_for_width(959.9), Breakpoint::Sm));
        assert!(matches!(breakpoint_for_width(1279.9), Breakpoint::Md));
        assert!(matches!(breakpoint_for_width(1919.9), Breakpoint::Lg));
        assert!(matches!(breakpoint_for_width(2559.9), Breakpoint::Xl));
        assert!(matches!(breakpoint_for_width(-100.0), Breakpoint::Xs));
        assert!(matches!(breakpoint_for_width(f32::INFINITY), Breakpoint::Xxl));
        assert!(matches!(breakpoint_for_width(f32::NEG_INFINITY), Breakpoint::Xs));
    }

    #[test]
    fn effective_width_falls_back_when_unbounded() {
        assert_eq!(effective_width(800.0), 800.0);
        assert_eq!(effective_width(0.0), 0.0);
        assert_eq!(effective_width(f32::INFINITY), FALLBACK_WIDTH);
        assert_eq!(effective_width(f32::NEG_INFINITY), FALLBACK_WIDTH);
        assert_eq!(effective_width(f32::NAN), FALLBACK_WIDTH);
    }

    #[test]
    fn gap_and_track_math() {
        assert_eq!(gap_px(6), 24.0);
        assert_eq!(gap_px(255), 64.0); // clamped to MAX_SPACING (16) * 4px
        assert_eq!(track_width(1200.0, 24.0), 78.0);
        assert_eq!(track_width(100.0, 24.0), 0.0);
        assert_eq!(tracks_to_width(6, 78.0, 24.0), 588.0);
        assert_eq!(tracks_to_width(1, 78.0, 24.0), 78.0);
        assert_eq!(tracks_to_width(12, 78.0, 24.0), 1200.0);
    }

    #[test]
    fn track_width_edge_cases() {
        assert_eq!(gap_px(0), 0.0);
        assert_eq!(track_width(1200.0, 0.0), 100.0);
        assert_eq!(track_width(0.0, 0.0), 0.0);
        assert_eq!(tracks_to_width(0, 78.0, 24.0), 78.0);
        assert_eq!(tracks_to_width(200, 78.0, 24.0), 1200.0);
    }

    #[test]
    fn full_row_of_tracks_fills_width_exactly() {
        let (width, gap) = (1200.0, 24.0);
        let tw = track_width(width, gap);
        assert_eq!(tracks_to_width(TRACK_COUNT, tw, gap), width);
        let half = tracks_to_width(6, tw, gap);
        assert_eq!(half + gap + half, width);
    }

    #[test]
    fn packs_rows_by_track_count() {
        assert_eq!(pack_rows(&[]), Vec::<Range<usize>>::new());
        assert_eq!(pack_rows(&[Tracks(6), Tracks(6), Tracks(6)]), vec![0..2, 2..3]);
        assert_eq!(pack_rows(&[Tracks(12), Tracks(1)]), vec![0..1, 1..2]);
        assert_eq!(pack_rows(&[Tracks(4), Star(1.0), Auto]), vec![0..3]);
    }

    #[test]
    fn auto_and_star_reserve_one_track() {
        let spans = vec![Auto; 13];
        assert_eq!(pack_rows(&spans), vec![0..12, 12..13]);
    }

    #[test]
    fn pack_rows_edge_cases() {
        assert_eq!(pack_rows(&[Tracks(12)]), vec![0..1]);
        assert_eq!(pack_rows(&[Tracks(0), Tracks(0)]), vec![0..2]);
        assert_eq!(pack_rows(&[Tracks(99), Tracks(99)]), vec![0..1, 1..2]);
        assert_eq!(pack_rows(&[Tracks(6), Tracks(6), Tracks(1)]), vec![0..2, 2..3]);
        assert_eq!(pack_rows(&[MudGridSpan::Unset; 13]), vec![0..12, 12..13]);
        let spans = [Tracks(5), Tracks(5), Tracks(5), Star(2.0), Tracks(12), Auto];
        let rows = pack_rows(&spans);
        assert_eq!(rows.first().unwrap().start, 0);
        assert_eq!(rows.last().unwrap().end, spans.len());
        for pair in rows.windows(2) {
            assert_eq!(pair[0].end, pair[1].start);
        }
    }

    #[test]
    fn pack_rows_into_reuses_and_clears_the_buffer() {
        let mut buf = vec![7..9, 1..2];
        pack_rows_into(&[Tracks(6), Tracks(6), Tracks(6)], &mut buf);
        assert_eq!(buf, vec![0..2, 2..3]);
        pack_rows_into(&[], &mut buf);
        assert!(buf.is_empty());
    }

    #[test]
    fn two_halves_fill_the_row() {
        let w = row_widths(&[Tracks(6), Tracks(6)], &[0.0, 0.0], 1200.0, 24.0, 78.0);
        assert_eq!(w, vec![588.0, 588.0]);
    }

    #[test]
    fn stars_share_leftover_by_weight() {
        let w = row_widths(&[Tracks(3), Star(1.0), Star(3.0)], &[0.0; 3], 1200.0, 24.0, 78.0);
        assert_eq!(w[0], 282.0);
        assert_eq!(w[1], 217.5);
        assert_eq!(w[2], 652.5);
    }

    #[test]
    fn zero_weight_stars_split_equally() {
        let w = row_widths(&[Star(0.0), Star(0.0)], &[0.0; 2], 224.0, 24.0, 0.0);
        assert_eq!(w, vec![100.0, 100.0]);
    }

    #[test]
    fn auto_uses_measured_width() {
        let w = row_widths(&[Auto, Star(1.0)], &[100.0, 0.0], 524.0, 24.0, 0.0);
        assert_eq!(w, vec![100.0, 400.0]);
    }

    #[test]
    fn overfull_rows_never_go_negative() {
        let w = row_widths(&[Auto, Star(1.0)], &[900.0, 0.0], 500.0, 24.0, 0.0);
        assert_eq!(w[1], 0.0);
    }

    #[test]
    fn row_widths_handles_degenerate_inputs() {
        assert_eq!(row_widths(&[Star(2.0)], &[0.0], 500.0, 24.0, 0.0), vec![500.0]);
        assert_eq!(row_widths(&[Auto, Auto], &[], 500.0, 24.0, 0.0), vec![0.0, 0.0]);
        assert_eq!(row_widths(&[Auto], &[-50.0], 500.0, 24.0, 0.0), vec![0.0]);
        let w = row_widths(&[Star(f32::NAN), Star(1.0)], &[0.0; 2], 224.0, 24.0, 0.0);
        assert_eq!(w, vec![0.0, 200.0]);
        assert_eq!(row_widths(&[MudGridSpan::Unset], &[0.0], 500.0, 24.0, 0.0), vec![0.0]);
        assert!(row_widths(&[], &[], 500.0, 24.0, 0.0).is_empty());
    }

    #[test]
    fn row_widths_normalizes_track_counts() {
        let w = row_widths(&[Tracks(0), Tracks(99)], &[0.0; 2], 1200.0, 24.0, 78.0);
        assert_eq!(w, vec![78.0, 1200.0]);
    }

    #[test]
    fn row_widths_into_reuses_and_clears_the_buffer() {
        let mut buf = vec![9.0; 7];
        row_widths_into(&[Tracks(6), Tracks(6)], &[0.0; 2], 1200.0, 24.0, 78.0, &mut buf);
        assert_eq!(buf, vec![588.0, 588.0]);
        let capacity = buf.capacity();
        row_widths_into(&[Star(1.0)], &[0.0], 300.0, 24.0, 0.0, &mut buf);
        assert_eq!(buf, vec![300.0]);
        assert!(buf.capacity() >= capacity.min(1));
    }

    #[test]
    fn justify_offsets() {
        let w = [100.0, 100.0];
        let o = |a| row_offsets(&w, 400.0, 20.0, a);
        assert_eq!(o(RowAlign::Start), vec![0.0, 120.0]);
        assert_eq!(o(RowAlign::Center), vec![90.0, 210.0]);
        assert_eq!(o(RowAlign::End), vec![180.0, 300.0]);
        assert_eq!(o(RowAlign::SpaceBetween), vec![0.0, 300.0]);
        assert_eq!(o(RowAlign::SpaceAround), vec![45.0, 255.0]);
        assert_eq!(o(RowAlign::SpaceEvenly), vec![60.0, 240.0]);
    }

    #[test]
    fn justify_edge_cases() {
        assert!(row_offsets(&[], 400.0, 20.0, RowAlign::Center).is_empty());
        assert_eq!(row_offsets(&[100.0], 400.0, 20.0, RowAlign::SpaceBetween), vec![0.0]);
        assert_eq!(row_offsets(&[300.0, 300.0], 400.0, 20.0, RowAlign::Center), vec![0.0, 320.0]);
    }

    #[test]
    fn single_item_justification() {
        let w = [100.0];
        let o = |a| row_offsets(&w, 400.0, 20.0, a);
        assert_eq!(o(RowAlign::Start), vec![0.0]);
        assert_eq!(o(RowAlign::Center), vec![150.0]);
        assert_eq!(o(RowAlign::End), vec![300.0]);
        assert_eq!(o(RowAlign::SpaceAround), vec![150.0]);
        assert_eq!(o(RowAlign::SpaceEvenly), vec![150.0]);
    }

    #[test]
    fn justified_rows_never_exceed_the_width() {
        let w = [100.0, 50.0, 70.0];
        for align in [
            RowAlign::Start,
            RowAlign::Center,
            RowAlign::End,
            RowAlign::SpaceBetween,
            RowAlign::SpaceAround,
            RowAlign::SpaceEvenly,
        ] {
            let o = {
                let mut out = Vec::new();
                row_offsets_into(&w, 500.0, 10.0, align, &mut out);
                out
            };
            assert_eq!(o.len(), w.len());
            assert!(o.windows(2).all(|p| p[0] <= p[1]));
            assert!(o.iter().zip(&w).all(|(x, width)| *x >= 0.0 && x + width <= 500.0 + 1e-3));
        }
    }

    #[test]
    fn row_offsets_into_clears_the_buffer() {
        let mut buf = vec![42.0; 5];
        row_offsets_into(&[], 400.0, 20.0, RowAlign::Start, &mut buf);
        assert!(buf.is_empty());
        row_offsets_into(&[100.0, 100.0], 400.0, 20.0, RowAlign::Start, &mut buf);
        assert_eq!(buf, vec![0.0, 120.0]);
    }

    #[test]
    fn row_align_mirrors_justify() {
        assert_eq!(RowAlign::from(Justify::Start), RowAlign::Start);
        assert_eq!(RowAlign::from(Justify::Center), RowAlign::Center);
        assert_eq!(RowAlign::from(Justify::End), RowAlign::End);
        assert_eq!(RowAlign::from(Justify::SpaceBetween), RowAlign::SpaceBetween);
        assert_eq!(RowAlign::from(Justify::SpaceAround), RowAlign::SpaceAround);
        assert_eq!(RowAlign::from(Justify::SpaceEvenly), RowAlign::SpaceEvenly);
    }

    #[test]
    fn cell_rect_has_unbounded_height() {
        let r = cell_rect(10.0, 20.0, 100.0);
        assert_eq!(r.min, pos2(10.0, 20.0));
        assert_eq!(r.width(), 100.0);
        assert!(r.height().is_infinite());
    }

    #[test]
    fn grid_bottom_removes_trailing_gap() {
        assert_eq!(grid_bottom(true, 500.0, 24.0, 10.0), 10.0);
        assert_eq!(grid_bottom(false, 124.0, 24.0, 10.0), 100.0);
        assert_eq!(grid_bottom(false, 10.0, 24.0, 10.0), 10.0);
    }

    #[test]
    fn finite_height_never_propagates_infinity_or_nan() {
        assert_eq!(finite_height(60.0), 60.0);
        assert_eq!(finite_height(0.0), 0.0);
        assert_eq!(finite_height(-5.0), 0.0);
        assert_eq!(finite_height(f32::INFINITY), 0.0);
        assert_eq!(finite_height(f32::NEG_INFINITY), 0.0);
        assert_eq!(finite_height(f32::NAN), 0.0);
        let row_height = [60.0, f32::INFINITY, 40.0].iter().fold(0.0f32, |h, c| h.max(finite_height(*c)));
        assert!(grid_bottom(false, row_height + 24.0, 24.0, 0.0).is_finite());
    }

    // ---- planner ------------------------------------------------------------

    #[test]
    fn plan_into_positions_two_halves() {
        let cells = plan(&[Tracks(6), Tracks(6)], &[], 1200.0, 24.0, RowAlign::Start);
        assert_eq!(cells.len(), 2);
        assert_eq!((cells[0].index, cells[0].row, cells[0].x, cells[0].width), (0, 0, 0.0, 588.0));
        assert_eq!((cells[1].index, cells[1].row, cells[1].x, cells[1].width), (1, 0, 612.0, 588.0));
        assert_eq!(cells[1].right(), 1200.0);
    }

    #[test]
    fn plan_into_wraps_to_a_new_row() {
        let cells = plan(&[Tracks(6), Tracks(6), Tracks(6)], &[], 1200.0, 24.0, RowAlign::Start);
        assert_eq!(cells[2].row, 1);
        assert_eq!(cells[2].index, 2);
        assert_eq!(cells[2].x, 0.0);
        assert_eq!(cells[2].width, 588.0);
    }

    #[test]
    fn plan_into_justifies_each_row_independently() {
        // Row 0 is full; row 1 has a single 4-track item that gets centred.
        let spans = [Tracks(6), Tracks(6), Tracks(4)];
        let cells = plan(&spans, &[], 1200.0, 24.0, RowAlign::Center);
        assert_eq!(cells[0].x, 0.0);
        assert_eq!(cells[1].x, 612.0);
        assert_eq!(cells[2].row, 1);
        assert_eq!(cells[2].x, 408.0); // (1200 - 384) / 2
    }

    #[test]
    fn plan_into_space_between_three_columns() {
        let cells = plan(&[Tracks(3); 3], &[], 1200.0, 24.0, RowAlign::SpaceBetween);
        let xs: Vec<f32> = cells.iter().map(|c| c.x).collect();
        assert_eq!(xs, vec![0.0, 459.0, 918.0]);
        assert!(cells.iter().all(|c| c.width == 282.0));
        assert_eq!(cells[2].right(), 1200.0);
    }

    #[test]
    fn plan_into_reads_auto_widths_by_item_index() {
        // Item 2 is the only Auto; its width sits at index 2 of the global array.
        let spans = [Tracks(12), Tracks(12), Auto, Star(1.0)];
        let cells = plan(&spans, &[0.0, 0.0, 100.0, 0.0], 524.0, 24.0, RowAlign::Start);
        assert_eq!((cells[2].row, cells[2].x, cells[2].width), (2, 0.0, 100.0));
        assert_eq!((cells[3].row, cells[3].x, cells[3].width), (2, 124.0, 400.0));
    }

    #[test]
    fn plan_into_tolerates_short_auto_slices() {
        let cells = plan(&[Auto, Auto], &[], 500.0, 24.0, RowAlign::Start);
        assert_eq!(cells.len(), 2);
        assert!(cells.iter().all(|c| c.width == 0.0));
    }

    #[test]
    fn plan_into_clears_and_handles_empty_input() {
        let mut scratch = PlanScratch::default();
        let mut out = vec![CellBox { index: 9, row: 9, x: 9.0, width: 9.0 }];
        plan_into(&[], &[], 1200.0, 24.0, RowAlign::Start, &mut scratch, &mut out);
        assert!(out.is_empty());
        plan_into(&[Tracks(12)], &[], 1200.0, 24.0, RowAlign::Start, &mut scratch, &mut out);
        assert_eq!(out.len(), 1);
        plan_into(&[Tracks(6), Tracks(6)], &[], 1200.0, 24.0, RowAlign::Start, &mut scratch, &mut out);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn plan_into_invariants_hold_for_mixed_content() {
        let spans = [
            Tracks(5),
            Tracks(7),
            Tracks(11),
            Star(1.0),
            Star(2.0),
            Tracks(4),
            Tracks(4),
            Tracks(4),
            Tracks(12),
            Tracks(1),
        ];
        for align in [
            RowAlign::Start,
            RowAlign::Center,
            RowAlign::End,
            RowAlign::SpaceBetween,
            RowAlign::SpaceAround,
            RowAlign::SpaceEvenly,
        ] {
            let cells = plan(&spans, &[], 1200.0, 24.0, align);
            assert_eq!(cells.len(), spans.len());
            for (i, c) in cells.iter().enumerate() {
                assert_eq!(c.index, i, "cells stay in item order");
                assert!(c.x >= -1e-3, "no negative offsets");
                assert!(c.right() <= 1200.0 + 1e-3, "cell {i} overflows under {align:?}");
            }
            for pair in cells.windows(2) {
                assert!(pair[1].row >= pair[0].row, "rows never go backwards");
                if pair[1].row == pair[0].row {
                    assert!(pair[1].x + 1e-3 >= pair[0].right() + 24.0, "cells in a row never overlap");
                }
            }
        }
    }

    #[test]
    fn grid_plan_follows_the_cascade_and_breakpoint() {
        let items = [MudItem::new().xs(12).md(6); 2];

        // 1200px is `Md`: two halves share a row.
        let cells = MudGrid::new().plan(&items, 1200.0, &[]);
        assert_eq!((cells[0].row, cells[0].x, cells[0].width), (0, 0.0, 588.0));
        assert_eq!((cells[1].row, cells[1].x, cells[1].width), (0, 612.0, 588.0));

        // 500px is `Xs`: the items stack at full width.
        let cells = MudGrid::new().plan(&items, 500.0, &[]);
        assert_eq!((cells[0].row, cells[1].row), (0, 1));
        assert!(approx(cells[0].width, 500.0) && approx(cells[1].width, 500.0));

        // Pinning the breakpoint to `Xs` stacks them even at 1200px.
        let cells = MudGrid::new().breakpoint(Breakpoint::Xs).plan(&items, 1200.0, &[]);
        assert_eq!((cells[0].row, cells[1].row), (0, 1));
        assert_eq!(cells[0].width, 1200.0);
    }

    #[test]
    fn grid_plan_applies_justify_and_spacing() {
        let one = [MudItem::new().all(4)];
        let centered = MudGrid::new().justify(Justify::Center).plan(&one, 1200.0, &[]);
        assert_eq!((centered[0].x, centered[0].width), (408.0, 384.0));
        let ended = MudGrid::new().justify(Justify::End).plan(&one, 1200.0, &[]);
        assert_eq!(ended[0].x, 816.0);

        // Zero spacing: 12 tracks of 100px, no gaps.
        let halves = [MudItem::new().all(6); 2];
        let cells = MudGrid::new().spacing(0).plan(&halves, 1200.0, &[]);
        assert_eq!((cells[0].x, cells[0].width), (0.0, 600.0));
        assert_eq!((cells[1].x, cells[1].width), (600.0, 600.0));
    }

    #[test]
    fn grid_plan_uses_fallback_width_when_unbounded() {
        let items = [MudItem::new().xs(12).lg(6); 2];
        let cells = MudGrid::new().plan(&items, f32::INFINITY, &[]);
        // Fallback is 1280px = `Lg`, so both items are halves on one row.
        assert_eq!((cells[0].row, cells[1].row), (0, 0));
        assert!(approx(cells[1].right(), FALLBACK_WIDTH));
    }

    #[test]
    fn grid_plan_handles_auto_and_star_items() {
        let items = [MudItem::auto(), MudItem::star(1.0)];
        let cells = MudGrid::new().plan(&items, 524.0, &[100.0, 0.0]);
        assert_eq!((cells[0].x, cells[0].width), (0.0, 100.0));
        assert_eq!((cells[1].x, cells[1].width), (124.0, 400.0));
        assert!(MudGrid::new().plan(&[], 800.0, &[]).is_empty());
    }

    #[test]
    fn auto_widths_changed_uses_an_epsilon() {
        assert!(auto_widths_changed(None, &[1.0]));
        assert!(auto_widths_changed(None, &[]));
        assert!(!auto_widths_changed(Some(&[1.0, 2.0]), &[1.0, 2.0]));
        assert!(!auto_widths_changed(Some(&[1.0, 2.0]), &[1.0, 2.4]));
        assert!(auto_widths_changed(Some(&[1.0, 2.0]), &[1.0, 2.6]));
        assert!(auto_widths_changed(Some(&[1.0]), &[1.0, 2.0]));
        assert!(!auto_widths_changed(Some(&[]), &[]));
    }

    // ---- GridUi / ItemStyle -------------------------------------------------

    #[test]
    fn grid_ui_collects_items_in_order() {
        let mut g = GridUi { items: Vec::new() };
        assert!(g.is_empty());

        g.item(MudItem::new().md(6), |_ui| {});
        g.item(MudGridSpan::HALF, |_ui| {}); // a bare span applies everywhere
        g.spacer(MudItem::new().xs(12));
        g.styled_item(MudItem::new().xs(4), ItemStyle::new().class("p-4"), |_ui| {});

        assert_eq!(g.len(), 4);
        assert!(!g.is_empty());
        assert_eq!(g.items[0].span, MudItem::new().md(6));
        assert_eq!(g.items[1].span, MudItem::new().all(6));
        assert_eq!(g.items[2].span, MudItem::new().xs(12));
        assert!(g.items[0].style.is_none());
        assert!(g.items[3].style.as_ref().is_some_and(|s| s.classes == vec!["p-4".to_string()]));
    }

    #[test]
    fn grid_ui_each_adds_one_item_per_element() {
        let mut g = GridUi { items: Vec::new() };
        g.each(MudItem::new().xs(12).sm(4), 0..5, |_ui, _n| {});
        assert_eq!(g.len(), 5);
        assert!(g.items.iter().all(|i| i.span == MudItem::new().xs(12).sm(4) && i.style.is_none()));

        g.each(MudItem::new().xs(6), Vec::<u8>::new(), |_ui, _n| {});
        assert_eq!(g.len(), 5); // an empty collection adds nothing
    }

    #[test]
    fn item_style_builders() {
        assert!(ItemStyle::new().is_empty());

        let s = ItemStyle::new().class("a").classes(["b", "c"]).class(String::from("d"));
        assert_eq!(s.classes, vec!["a", "b", "c", "d"]);
        assert!(!s.is_empty());
        assert!(s.frame.is_none());

        let framed = ItemStyle::new().frame(Frame::default());
        assert!(framed.frame.is_some());
        assert!(!framed.is_empty());
    }

    // ---- MudGrid builder ----------------------------------------------------

    #[test]
    fn builder_and_defaults() {
        let d = MudGrid::default();
        assert!(matches!(d.justify, Justify::Start));
        assert_eq!(d.spacing, 6);
        assert!(d.breakpoint.is_none());
        assert!(d.frame_override.is_none());
        assert_eq!(d.measure, AutoMeasure::Probe);
        assert_eq!(MudGrid::new().spacing, 6);

        let g = MudGrid::new().justify(Justify::Center).spacing(2).breakpoint(Breakpoint::Md);
        assert!(matches!(g.justify, Justify::Center));
        assert_eq!(g.spacing, 2);
        assert!(matches!(g.breakpoint, Some(Breakpoint::Md)));
    }

    #[test]
    fn extended_builder_options() {
        let g = MudGrid::new()
            .auto_measure(AutoMeasure::Cached)
            .disabled(true)
            .frame(Frame::default());
        assert_eq!(g.measure, AutoMeasure::Cached);
        assert!(g.interaction.disabled);
        assert!(g.frame_override.is_some());
        assert_eq!(MudGrid::new().spacing(3).gap(), 12.0);
    }

    #[test]
    fn fresh_grid_has_no_frame_results() {
        let g = MudGrid::new();
        assert!(g.active_breakpoint().is_none());
        assert_eq!(g.last_width(), 0.0);
        assert!(g.last_cells().is_empty());
        assert!(g.cell_rects().is_empty());
    }

    #[test]
    fn spacing_is_clamped_by_the_builder() {
        assert_eq!(MudGrid::new().spacing(0).spacing, 0);
        assert_eq!(MudGrid::new().spacing(MAX_SPACING).spacing, MAX_SPACING);
        assert_eq!(MudGrid::new().spacing(200).spacing, MAX_SPACING);
        assert_eq!(MudGrid::new().spacing(17).spacing, 16);
    }

    #[test]
    fn constants_are_consistent() {
        assert_eq!(MAX_SPACING, 16);
        assert_eq!(SPACING_UNIT, 4.0);
        assert!(BREAKPOINT_SM < BREAKPOINT_MD && BREAKPOINT_MD < BREAKPOINT_LG);
        assert!(BREAKPOINT_LG < BREAKPOINT_XL && BREAKPOINT_XL < BREAKPOINT_XXL);
        assert_eq!(FALLBACK_WIDTH, BREAKPOINT_LG);
    }

    #[test]
    fn auto_measure_defaults_to_probe() {
        assert_eq!(AutoMeasure::default(), AutoMeasure::Probe);
        assert_ne!(AutoMeasure::Probe, AutoMeasure::Cached);
    }

    #[test]
    fn interaction_state_to_tw_state_respects_disabled() {
        let mut s = InteractionState {
            hovered: true,
            active: true,
            dark: true,
            disabled: true,
            ..Default::default()
        };
        let tw = s.to_tw_state();
        assert!(!tw.hovered && !tw.active && tw.dark);

        s.disabled = false;
        let tw = s.to_tw_state();
        assert!(tw.hovered && tw.active && tw.dark);
    }

    // ---- alignment classes --------------------------------------------------

    #[test]
    fn item_classes_records_alignment() {
        let mut g = GridUi { items: Vec::new() };
        g.item(MudItem::new().xs(12).class("align-self-end p-2"), |_ui| {});
        g.item(MudItem::new().xs(12), |_ui| {});
        g.styled_item(
            MudItem::new().xs(12),
            ItemStyle::new().classes(["bg-primary", "text-end"]),
            |_ui| {},
        );
        assert_eq!(g.items[0].align.align_self, Some(CrossAlign::End));
        assert!(g.items[1].align.is_empty());
        assert_eq!(g.items[2].align.horizontal(), Some(HAlign::End));
    }
}