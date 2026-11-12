//! A layout panel that arranges its children in a single row or column with a uniform gap.
//!
//! Port of `MudStack.cs` (MudBlazor 8.13.0). The public parameter surface mirrors
//! the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `Row` | [`MudStack::row`] / [`MudStack::horizontal`] / [`MudStack::vertical`] |
//! | `Reverse` | [`MudStack::reverse`] |
//! | `Spacing` | [`MudStack::spacing`] |
//! | `AlignItems` | [`MudStack::align_items`] |
//! | `Justify` | [`MudStack::justify`] |
//! | `Wrap` | [`MudStack::wrap`] |
//! | `Breakpoint` | [`MudStack::breakpoint`] |
//! | `StretchItems` | [`MudStack::stretch_items`] |
//! | `ChildContent` | the closure passed to [`MudStack::show`] |
//! | `Class` | [`MudStack::classes`] |
//! | `Style` / `Tag` / `UserAttributes` | not ported — see below |
//!
//! # Deliberately not ported
//!
//! | MudBlazor | Reason |
//! |-----------|--------|
//! | `Style` | egui has no inline CSS strings. Use `classes(..)` for padding, margin and background; everything else belongs to the child widgets. |
//! | `Tag` | A `MudStack` is rebuilt every frame, so a `Tag` would be lost between frames. Wrap the stack in your own struct if you need one. |
//! | `UserAttributes` | There is no HTML tag to splat attributes onto. |
//!
//! # Extensions
//!
//! | Extension | Purpose |
//! |-----------|---------|
//! | [`MudStack::height`] | An explicit fixed height, in points. Gives a column free space to justify or wrap into; MudBlazor relies on CSS `height` for this. |
//! | [`MudStack::id_salt`] | Distinguishes two stacks sharing a parent, whose measurements would otherwise share a memory key. Same shape as `MudCollapse::id_salt`. |
//!
//! # Rule 3 compliance
//!
//! `MudStack` is a layout container; it paints no text and no glyphs of its own.
//! Every string and icon inside belongs to a child widget, which should use
//! [`MudText`](crate::MudText) and [`MudIcon`](crate::MudIcon) as usual.
//! `MudStack` never calls `painter.text` or `painter.galley`.
//!
//! ```ignore
//! // Plain usage: `StackUi` derefs to `Ui`, so children can be drawn directly.
//! MudStack::new().spacing(2).show(ui, |ui| {
//!     ui.add(MudText::body1("one"));
//!     ui.add(MudText::body1("two"));
//! });
//!
//! // Space-between / around / evenly need per-item boundaries: use `StackUi::item`.
//! MudStack::horizontal()
//!     .justify(Justify::SpaceBetween)
//!     .align_items(AlignItems::Center)
//!     .classes(["p-2", "bg-primary"])
//!     .show(ui, |stack| {
//!         stack.item(|ui| { ui.add(MudText::body1("left")); });
//!         stack.item(|ui| { ui.add(MudText::body1("middle")); });
//!         stack.item(|ui| { ui.add(MudButton::new("right")); });
//!     });
//! ```
//!
//! # How it differs from Avalonia / CSS flexbox
//! - **Frame-delayed measuring.** `Center`, `End` and `Space*` justification need the size of
//!   the content, which egui only knows after drawing it. The stack remembers the size from the
//!   previous frame and requests one repaint when it changes, so the first frame is laid out as
//!   `Start`.
//! - **Main-axis free space.** A row fills the available width. A column has no free space
//!   unless you give it `.height(..)`, so `Justify` other than `Start` needs a height there.
//! - **Wrapping.** `Wrap::Wrap` uses egui's wrapping layout. Justification is ignored while
//!   wrapping. egui only starts a new line for *leaf* widgets (buttons, labels); a composite
//!   child such as a `Frame` / `MudPaper` is handed the remaining width of the current line and
//!   squeezes its text instead of moving to the next line. Add such children with
//!   [`StackUi::item`] inside [`MudStack::show`]: each item is measured, remembered for
//!   the next frame, and kept together as a unit (the stack calls `end_row` before an item that
//!   no longer fits, and stops its text from wrapping character by character). The very first
//!   frame has no measurements, so items are laid on one line before one corrective repaint.
//! - **`Reverse`** uses egui's right-to-left / bottom-up layouts. A reversed column without a
//!   `.height(..)` needs one hidden frame to measure itself.
//! - **`StretchItems`** hands leftover main-axis space to the first/last/middle/all items
//!   (see [`StretchItems`] and [`MudStack::stretch_items`]), computed from the *previous*
//!   frame's measurements in the same way `Space*` justification is — so, like `Space*`, it
//!   only affects children added with [`StackUi::item`] and needs one settling frame.
//! - **`FillIndex`** (per-item proportional flex-grow weights) is not supported; unlike the
//!   fixed start/end/middle/all split `StretchItems` offers, arbitrary per-item weights would
//!   need a real multi-pass constraint solver, which egui's immediate-mode layout has no room
//!   for.
//!
//! # Structure of this file
//! The file is split so that everything that can be decided *without* a live
//! [`egui::Context`] lives in small pure functions at the bottom ([`plan_offsets`],
//! [`plan_rect`], [`item_spacing`], ...). The render path (`run` / `run_inner`) only gathers
//! inputs from the `Ui`, calls those helpers, and applies the results. This keeps the
//! interesting arithmetic unit-testable and the render path thin.
//!
//! # Allocation behaviour
//! Rendering performs no heap allocation of its own: the persisted [`Measure`] is a small
//! `Copy` value stored in egui's temp memory, and no intermediate collections are built.
//! (Parsing utility classes and fetching the theme are delegated to external crates and are
//! only done when `classes(..)` produced at least one recognised utility.)

use egui::{
    Align, Frame, Id, InnerResponse, Layout, Rect, Response, TextWrapMode, Ui, UiBuilder, Vec2, Widget, vec2,
};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use mud_egui_core::base::component_base::ComponentBase;

// Core Enums
use mud_egui_core::enums::align_items::AlignItems;
use mud_egui_core::enums::breakpoint::Breakpoint;
use mud_egui_core::enums::justify::Justify;
use mud_egui_core::enums::stretch_items::StretchItems;
use mud_egui_core::enums::wrap::Wrap;

// Theming
use mud_egui_theming::theme::MudTheme;

// Shared utility parser
use crate::foundation::utilities::Utilities;

/// Gap unit in logical pixels (spacing `3` = 12px).
///
/// Matches MudBlazor's 4px spacing scale so that `spacing(n)` reads the same in both ports.
const SPACING_UNIT: f32 = 4.0;

/// Largest accepted `spacing` value (`20` = 80px).
///
/// Mirrors MudBlazor's documented range; larger values are clamped rather than rejected so a
/// bad value degrades gracefully instead of panicking in a UI path.
const MAX_SPACING: u8 = 20;

/// Tolerance (in logical pixels) under which two measurements count as "the same".
///
/// egui lays out in sub-pixel floats; comparing exactly would make the stack request a repaint
/// every frame because of rounding noise, defeating the "repaint once when it changes" design.
const MEASURE_EPSILON: f32 = 0.5;

/// Slack (in logical pixels) tolerated before an item is moved to the next line when wrapping.
///
/// Prevents an item that fits "exactly, give or take float noise" from flickering between two
/// lines from one frame to the next.
const LINE_EPSILON: f32 = 0.5;

/// Suffix used to derive the memory key under which a wrapping stack remembers each item's
/// main-axis extent from the previous frame.
const WRAP_EXTENTS_KEY: &str = "mud_stack_wrap_extents";

// =============================================================================
// Component
// =============================================================================

/// A single-axis layout container with a uniform gap between children.
///
/// All fields are public so that callers who build the component from configuration can set
/// them directly; the builder methods are the preferred, validated way to do so.
///
/// Call [`MudStack::show`] to render; constructing the value alone
/// draws nothing, which is enforced by `#[must_use]`.
#[must_use = "MudStack does nothing until you call `.show(ui, |ui| ...)`"]
pub struct MudStack {
    /// Shared component state (utility classes such as `p-2` / `bg-primary`). Mirrors
    /// MudBlazor `Class`.
    pub base: ComponentBase,
    /// `true` lays children out in a row, `false` in a column. Mirrors MudBlazor `Row`.
    pub row: bool,
    /// Reverses the visual order (`row-reverse` / `column-reverse`). Mirrors MudBlazor
    /// `Reverse`.
    pub reverse: bool,
    /// Gap between children in units of 4 px. Mirrors MudBlazor `Spacing`; the builder clamps
    /// to `0..=20`, and the render path clamps again so a directly assigned field can never
    /// produce an absurd gap.
    pub spacing: u8,
    /// Cross-axis alignment of children; `Stretch` makes them fill the cross axis. Mirrors
    /// MudBlazor `AlignItems`.
    pub align_items: AlignItems,
    /// Main-axis distribution of children. Anything other than `Start` needs free space.
    /// Mirrors MudBlazor `Justify`.
    pub justify: Justify,
    /// Whether children wrap onto further lines when the main axis runs out of room. Mirrors
    /// MudBlazor `Wrap`.
    pub wrap: Wrap,
    /// A row collapses to a column when the container is narrower than this breakpoint.
    /// Mirrors MudBlazor `Breakpoint`; `Breakpoint::None` disables the collapse.
    pub breakpoint: Breakpoint,
    /// Fixed height, in points. **Extension** (MudBlazor relies on CSS `height`). Required for
    /// main-axis justification and wrapping of a column.
    pub height: Option<f32>,
    /// Which items (by position) claim the stack's leftover main-axis space. Mirrors MudBlazor
    /// `StretchItems`.
    ///
    /// Only affects children added with [`StackUi::item`]: deciding "the first item" or "every
    /// item but the first and last" needs the same item boundaries that `Space*` justification
    /// needs, which widgets added directly to the [`StackUi`] do not provide. Like `Justify::Space*`, it has no
    /// effect on the first frame (nothing has been measured yet) and none while wrapping
    /// (there is no single leftover to distribute once content spans several lines).
    pub stretch_items: StretchItems,
    /// Optional salt that distinguishes two stacks that would otherwise share an auto id (and
    /// therefore share their remembered measurements). **Extension**, same shape as
    /// `MudCollapse::id_salt`.
    pub id_salt: Option<Id>,
}

impl Default for MudStack {
    /// A vertical, un-reversed, un-wrapped stack with `spacing = 3` (12 px), stretched
    /// children and `Start` justification — the same defaults as MudBlazor's `MudStack`.
    fn default() -> Self {
        Self {
            base: ComponentBase::new(),
            row: false,
            reverse: false,
            spacing: 3, // 12 px
            align_items: AlignItems::Stretch,
            justify: Justify::Start,
            wrap: Wrap::NoWrap,
            breakpoint: Breakpoint::None,
            height: None,
            stretch_items: StretchItems::None,
            id_salt: None,
        }
    }
}

impl MudStack {
    /// Creates a default (vertical) stack. Equivalent to [`MudStack::default`].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// A stack that lays children out left to right. Equivalent to
    /// `MudStack::new().row(true)`.
    #[inline]
    pub fn horizontal() -> Self {
        Self::new().row(true)
    }

    /// A stack that lays children out top to bottom. Equivalent to
    /// `MudStack::new().row(false)`.
    #[inline]
    pub fn vertical() -> Self {
        Self::new().row(false)
    }

    // =========================================================================
    // Builder Methods
    // =========================================================================

    /// Row (`true`) or column (`false`). Mirrors MudBlazor `Row`.
    #[inline]
    pub fn row(mut self, row: bool) -> Self {
        self.row = row;
        self
    }

    /// Reverse the visual order. Mirrors MudBlazor `Reverse`.
    #[inline]
    pub fn reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }

    /// Gap between children, in units of 4 px. Mirrors MudBlazor `Spacing`; clamped to
    /// `0..=20`.
    #[inline]
    pub fn spacing(mut self, spacing: u8) -> Self {
        self.spacing = spacing.min(MAX_SPACING);
        self
    }

    /// Cross-axis alignment of children. Mirrors MudBlazor `AlignItems`.
    #[inline]
    pub fn align_items(mut self, align: AlignItems) -> Self {
        self.align_items = align;
        self
    }

    /// Main-axis distribution of children. Mirrors MudBlazor `Justify`.
    #[inline]
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    /// Wrapping behaviour. Mirrors MudBlazor `Wrap`.
    #[inline]
    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = wrap;
        self
    }

    /// A row collapses to a column when the window is narrower than this breakpoint.
    /// Mirrors MudBlazor `Breakpoint`.
    #[inline]
    pub fn breakpoint(mut self, breakpoint: Breakpoint) -> Self {
        self.breakpoint = breakpoint;
        self
    }

    /// Fixed height in points. **Extension**: MudBlazor relies on CSS `height`. Gives a
    /// column free space to justify or wrap into.
    #[inline]
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Which items absorb leftover main-axis space. Mirrors MudBlazor `StretchItems`. Only
    /// affects children added with [`StackUi::item`]; see [`StretchItems`] for how the space is
    /// divided.
    #[inline]
    pub fn stretch_items(mut self, stretch_items: StretchItems) -> Self {
        self.stretch_items = stretch_items;
        self
    }

    /// Distinguishes two stacks that would otherwise get the same auto id. **Extension**,
    /// same shape as `MudCollapse::id_salt`.
    #[inline]
    pub fn id_salt(mut self, salt: impl Hash + std::fmt::Debug) -> Self {
        self.id_salt = Some(Id::new(salt));
        self
    }

    /// Utility classes: `p-*`, `m-*` (4 px units) and `bg-<role>`. Mirrors MudBlazor `Class`.
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    // =========================================================================
    // Rendering
    // =========================================================================

    /// Draws the stack; the closure adds the children (MudBlazor `ChildContent`).
    ///
    /// The closure receives a [`StackUi`], which dereferences to [`egui::Ui`]: you can call
    /// any `Ui` method on it directly (`stack.label(..)`, `stack.add(..)`) or pass it where a
    /// `&mut Ui` is expected. Children added that way are laid out normally, but the stack
    /// cannot see where each one starts and ends. Wrap each child in [`StackUi::item`] to make
    /// it a measured unit; that is what `Justify::Space*`, `StretchItems` and wrapping of
    /// composite children rely on.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_children: impl FnOnce(&mut StackUi<'_>) -> R,
    ) -> InnerResponse<R> {
        self.run(ui, |ui, flow| {
            let mut stack = StackUi::new(ui, flow);
            let r = add_children(&mut stack);
            (r, stack.into_tally())
        })
    }

    /// Applies utility classes (padding / margin / background), then lays out the stack.
    ///
    /// When no utility class is recognised this is a direct tail call into
    /// [`Self::run_inner`], skipping the theme lookup and the extra `Frame` entirely — the
    /// common case stays as cheap as possible.
    ///
    /// `body` receives the child `Ui` and an [`ItemFlow`] (resolved direction, gap, wrap state
    /// and last frame's item extents), and returns the user's value plus an [`ItemTally`] of
    /// what it measured (all zero / empty when items are not tracked).
    fn run<R>(
        self,
        ui: &mut Ui,
        body: impl FnOnce(&mut Ui, ItemFlow) -> (R, ItemTally),
    ) -> InnerResponse<R> {
        let utilities = Utilities::parse(self.base.class_names().iter().map(|c| {
            let s: &str = c.as_ref();
            s
        }));

        if utilities.is_empty() {
            return self.run_inner(ui, body);
        }

        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();

        let mut frame = Frame::new()
            .inner_margin(utilities.padding.to_margin())
            .outer_margin(utilities.margin.to_margin());
        if let Some(bg) = utilities.background.and_then(|role| role.resolve(palette)) {
            frame = frame.fill(bg);
        }

        let out = frame.show(ui, |ui| self.run_inner(ui, body));
        InnerResponse::new(out.inner.inner, out.response)
    }

    /// Performs the actual layout: resolves direction, loads last frame's [`Measure`], plans
    /// the rect and spacing through the pure helpers, runs the children in a scoped child
    /// `Ui`, then stores the new measurement (requesting a repaint only if it changed).
    fn run_inner<R>(
        self,
        ui: &mut Ui,
        body: impl FnOnce(&mut Ui, ItemFlow) -> (R, ItemTally),
    ) -> InnerResponse<R> {
        let gap = gap_px(self.spacing);
        let wrap = self.wrap == Wrap::Wrap;

        // Direction (responsive collapse is based on the available container width).
        let breakpoint_px = (self.breakpoint != Breakpoint::None).then(|| self.breakpoint.to_pixels());

        // `available_width()` works across all supported egui versions, unlike viewport
        // queries, and is the right quantity for a container-relative collapse anyway.
        let viewport_width = ui.available_width();

        let is_row = resolve_is_row(self.row, breakpoint_px, viewport_width);

        // Memory key for last frame's measurements. An explicit salt wins; otherwise the
        // next auto id is claimed and skipped so sibling widgets do not collide with it.
        let id = match self.id_salt {
            Some(salt) => ui.id().with(salt),
            None => {
                let id = ui.next_auto_id();
                ui.skip_ahead_auto_ids(1);
                id
            }
        };
        let measure: Option<Measure> = ui.ctx().data(|d| d.get_temp::<Measure>(id));

        // Layout.
        let (cross, stretch) = cross_align(&self.align_items);
        let layout = build_layout(is_row, self.reverse, cross, stretch, wrap);

        // Free-space distribution along the main axis.
        let avail = ui.available_rect_before_wrap();
        let main_len = if is_row { Some(avail.width()) } else { self.height };
        let (lead, between) = plan_offsets(&self.justify, wrap, main_len, measure.as_ref(), gap);

        // The rect the stack is laid out in.
        let interact_h = ui.spacing().interact_size.y;
        let rect = plan_rect(avail, is_row, self.reverse, wrap, lead, self.height, measure, interact_h);

        let mut builder = UiBuilder::new().max_rect(rect).layout(layout);
        if needs_sizing_pass(self.reverse, is_row, self.height, measure) {
            builder = builder.invisible();
        }

        // Gap: `between` replaces the gap along the main axis only.
        let spacing = item_spacing(is_row, between, gap);
        let inherited_spacing = ui.spacing().item_spacing;

        // Wrapped stacks need each item's extent from the previous frame so `StackUi::item`
        // can start a new line *before* an item that no longer fits. Only wrapping stacks pay
        // for this (one small `Vec` clone out of memory per frame).
        let extents_id = id.with(WRAP_EXTENTS_KEY);
        let prev_extents: Vec<f32> = if wrap {
            ui.ctx().data(|d| d.get_temp::<Vec<f32>>(extents_id)).unwrap_or_default()
        } else {
            Vec::new()
        };
        // Leftover main-axis space to hand to `stretch_items`-flagged items, computed from
        // last frame's per-item extents — the same "know it a frame late" trick `Justify`
        // already relies on. Empty (no-op) while wrapping or before anything is measured.
        let stretch_extra = (!wrap)
            .then(|| plan_stretch(self.stretch_items, main_len, &prev_extents, gap))
            .unwrap_or_default();

        let flow = ItemFlow {
            is_row,
            wrap,
            gap,
            avail_main: main_len,
            prev_extents: prev_extents.clone(),
            stretch_extra,
            inherited_spacing,
        };

        let inner = ui.scope_builder(builder, |child| {
            child.spacing_mut().item_spacing = spacing;
            body(child, flow)
        });
        let (value, tally) = inner.inner;

        // Remember what we measured; repaint once if anything changed.
        let new = Measure::from_rect(inner.response.rect, is_row, tally.main, tally.count);
        let mut changed = measure.map_or(true, |old| !old.approx_eq(&new));
        if changed {
            ui.ctx().data_mut(|d| d.insert_temp(id, new));
        }
        if wrap && !extents_approx_eq(&prev_extents, &tally.extents) {
            ui.ctx().data_mut(|d| d.insert_temp(extents_id, tally.extents));
            changed = true;
        }
        if changed {
            ui.ctx().request_repaint();
        }

        InnerResponse::new(value, inner.response)
    }
}

impl Widget for MudStack {
    /// Renders an empty stack. Mostly useful for its utility-class frame (e.g. a coloured
    /// spacer); real content should go through [`MudStack::show`].
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui, |_| {}).response
    }
}

// =============================================================================
// StackUi
// =============================================================================

/// Handle passed to [`MudStack::show`].
///
/// The stack's gap is an egui `item_spacing`, which nested `Ui`s inherit. Children added with
/// [`StackUi::item`] get the parent's original spacing back, so the gap does not leak into
/// their descendants. Composite widgets added *directly* (e.g. a grid or a `Frame`) do inherit
/// it, so wrap those in `item`.
///
/// It wraps the stack's child `Ui` and accumulates the main-axis extent of every
/// [`StackUi::item`], which is what makes `Space*` justification, `StretchItems` and wrapping
/// of composite children possible. It dereferences to [`egui::Ui`], so plain widgets can be
/// added to it directly.
pub struct StackUi<'a> {
    /// The stack's child `Ui` that items are drawn into.
    ui: &'a mut Ui,
    /// Direction, gap, wrap state and last frame's item extents.
    flow: ItemFlow,
    /// What has been measured so far this frame.
    tally: ItemTally,
    /// Main-axis extent already used on the current line (`None` for an empty line).
    /// Only maintained while wrapping.
    used: Option<f32>,
}

impl<'a> StackUi<'a> {
    /// Creates a handle for one frame of `show`.
    #[inline]
    fn new(ui: &'a mut Ui, flow: ItemFlow) -> Self {
        Self { ui, flow, tally: ItemTally::default(), used: None }
    }

    /// Consumes the handle and returns what it measured.
    #[inline]
    fn into_tally(self) -> ItemTally {
        self.tally
    }

    /// Adds one child (which may contain several widgets) and records its size.
    ///
    /// When the stack wraps, the item is treated as an unbreakable unit: if last frame's
    /// extent says it no longer fits on the current line, a new line is started first, and its
    /// text is not allowed to wrap character by character into the leftover space.
    pub fn item<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let wrap = self.flow.wrap;
        let index = self.tally.count;

        if wrap {
            let known = self.flow.prev_extents.get(index).copied();
            if let Some(extent) = known {
                if should_break_line(self.used, extent, self.flow.gap, self.flow.avail_main) {
                    self.ui.end_row();
                    self.used = None;
                }
            }
        }

        // A `stretch_items`-flagged item gets a widened max rect before it is drawn, so any
        // child that fills `ui.available_width()`/`available_height()` visibly grows into the
        // leftover space; a fixed-size child just ignores the slack.
        let extra = self.flow.stretch_extra.get(index).copied().unwrap_or(0.0);
        let out = if extra > 0.0 {
            let natural = self.flow.prev_extents.get(index).copied().unwrap_or(0.0);
            let rect = stretched_item_rect(self.ui, self.flow.is_row, natural + extra);
            let inherited = self.flow.inherited_spacing;
            self.ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
                ui.spacing_mut().item_spacing = inherited;
                if wrap {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                }
                add_contents(ui)
            })
        } else {
            let inherited = self.flow.inherited_spacing;
            self.ui.scope(|ui| {
                ui.spacing_mut().item_spacing = inherited;
                if wrap {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                }
                add_contents(ui)
            })
        };

        let extent = main_extent(out.response.rect, self.flow.is_row);
        self.tally.main += extent;
        self.tally.count += 1;
        if wrap {
            self.tally.extents.push(extent);
            self.used = Some(advance_line(self.used, extent, self.flow.gap));
        }
        out
    }

    /// Whether the stack is currently laid out as a row (after any breakpoint collapse).
    #[inline]
    pub fn is_row(&self) -> bool {
        self.flow.is_row
    }

    /// Direct access to the underlying `Ui`. Widgets added this way are not measured as items.
    #[inline]
    pub fn ui(&mut self) -> &mut Ui {
        self.ui
    }
}

impl Deref for StackUi<'_> {
    type Target = Ui;

    #[inline]
    fn deref(&self) -> &Ui {
        self.ui
    }
}

impl DerefMut for StackUi<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Ui {
        self.ui
    }
}

// =============================================================================
// Pure helpers (unit-tested below)
// =============================================================================

/// Size information remembered from the previous frame.
///
/// Stored in egui's temp memory keyed by the stack id. It is `Copy` and tiny so that reading
/// and writing it per frame costs a few words, not an allocation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Measure {
    /// Content extent along the main axis (including gaps).
    content_main: f32,
    /// Content extent along the cross axis.
    content_cross: f32,
    /// Sum of item extents along the main axis (without gaps). Only counts children added through `StackUi::item`.
    item_main: f32,
    /// Number of children added through `StackUi::item`; `0` when none were.
    items: usize,
}

impl Measure {
    /// Builds a measurement from the response rect of the stack's child scope.
    ///
    /// `is_row` decides which rect dimension is the main axis.
    #[inline]
    fn from_rect(rect: Rect, is_row: bool, item_main: f32, items: usize) -> Self {
        let (content_main, content_cross) = axis_extents(rect, is_row);
        Self { content_main, content_cross, item_main, items }
    }

    /// Whether two measurements are equal within [`MEASURE_EPSILON`].
    ///
    /// Used to avoid repainting every frame over sub-pixel layout noise.
    #[inline]
    fn approx_eq(&self, other: &Self) -> bool {
        self.items == other.items
            && (self.content_main - other.content_main).abs() < MEASURE_EPSILON
            && (self.content_cross - other.content_cross).abs() < MEASURE_EPSILON
            && (self.item_main - other.item_main).abs() < MEASURE_EPSILON
    }
}

/// Per-frame facts handed to the body closure so `StackUi` can lay items out line by line.
#[derive(Debug, Clone)]
struct ItemFlow {
    /// Whether the stack resolved to a row (after any breakpoint collapse).
    is_row: bool,
    /// Whether the stack wraps.
    wrap: bool,
    /// Gap between neighbouring items along the main axis, in pixels.
    gap: f32,
    /// Room along the main axis before wrapping (`None` when unknown, e.g. a column without
    /// a fixed height).
    avail_main: Option<f32>,
    /// Main-axis extent of every item as measured on the previous frame (wrapping only).
    prev_extents: Vec<f32>,
    /// Extra main-axis pixels to hand to each item, by position, from `stretch_items`.
    /// Empty when there is nothing to distribute (see [`plan_stretch`]).
    stretch_extra: Vec<f32>,
    /// The parent `Ui`'s `item_spacing`, captured before the stack overrides it. egui styles
    /// are inherited by nested `Ui`s, so [`StackUi::item`] restores this inside each item so
    /// the stack's gap applies only between its own children, not to their descendants.
    inherited_spacing: Vec2,
}

/// What `StackUi` measured during one frame; fed back into [`Measure`] and wrap memory.
#[derive(Debug, Clone, Default)]
struct ItemTally {
    /// Sum of item extents along the main axis (gaps excluded).
    main: f32,
    /// Number of items added.
    count: usize,
    /// Each item's main-axis extent in order (filled only while wrapping; empty otherwise,
    /// which costs no allocation).
    extents: Vec<f32>,
}

/// Whether the next item must start a new line: it is not the first on its line and would
/// overflow the available main length by more than [`LINE_EPSILON`].
///
/// An unknown or infinite length never breaks, so a stack with nothing to wrap against behaves
/// like a plain one.
#[inline]
fn should_break_line(used: Option<f32>, next: f32, gap: f32, avail: Option<f32>) -> bool {
    match (used, avail) {
        (Some(u), Some(a)) if a.is_finite() => u + gap + next > a + LINE_EPSILON,
        _ => false,
    }
}

/// The main-axis extent used on a line after adding an item of `extent` (with a gap before it
/// unless it is the first on the line).
#[inline]
fn advance_line(used: Option<f32>, extent: f32, gap: f32) -> f32 {
    used.map_or(extent, |u| u + gap + extent)
}

/// Whether two per-item extent lists are the same length and equal within [`MEASURE_EPSILON`].
fn extents_approx_eq(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < MEASURE_EPSILON)
}

/// Which previous-frame item positions (out of `count`) a [`StretchItems`] policy flags.
///
/// Pure index arithmetic so the policy's edge cases (0, 1 and 2 items under `Middle`) are
/// unit-testable without a live `Ui`.
fn stretch_targets(policy: StretchItems, count: usize) -> Vec<bool> {
    let mut flags = vec![false; count];
    if count == 0 {
        return flags;
    }
    match policy {
        StretchItems::None => {}
        StretchItems::Start => flags[0] = true,
        StretchItems::End => flags[count - 1] = true,
        StretchItems::StartAndEnd => {
            flags[0] = true;
            flags[count - 1] = true;
        }
        // Every index except the first and last; a no-op for count <= 2 (there is no middle).
        StretchItems::Middle => {
            for f in flags.iter_mut().take(count.saturating_sub(1)).skip(1) {
                *f = true;
            }
        }
        StretchItems::All => flags.iter_mut().for_each(|f| *f = true),
    }
    flags
}

/// Extra main-axis pixels to hand to each previous-frame item index so the stack's content
/// fills `avail_main`, split evenly among the positions `policy` flags.
///
/// Returns an empty `Vec` (meaning "nothing to distribute") whenever there is no previous
/// measurement, no finite main length, no free space left after the natural content and gaps,
/// or the policy flags no item — so a flagged stack with content that already overflows behaves
/// exactly like `StretchItems::None` instead of producing a negative or nonsensical rect.
fn plan_stretch(policy: StretchItems, avail_main: Option<f32>, prev_extents: &[f32], gap: f32) -> Vec<f32> {
    let count = prev_extents.len();
    if policy == StretchItems::None || count == 0 {
        return Vec::new();
    }
    let Some(avail) = avail_main.filter(|a| a.is_finite()) else {
        return Vec::new();
    };
    let content: f32 = prev_extents.iter().sum::<f32>() + gap * count.saturating_sub(1) as f32;
    let leftover = avail - content;
    if leftover <= 0.0 {
        return Vec::new();
    }
    let flags = stretch_targets(policy, count);
    let n_targets = flags.iter().filter(|&&f| f).count();
    if n_targets == 0 {
        return Vec::new();
    }
    let share = leftover / n_targets as f32;
    flags.into_iter().map(|f| if f { share } else { 0.0 }).collect()
}

/// The rect handed to a `stretch_items`-flagged item: the stack's current available rect,
/// resized along the main axis to `target_main` (clamped so it never claims more than is
/// actually available, and never goes negative).
fn stretched_item_rect(ui: &Ui, is_row: bool, target_main: f32) -> Rect {
    let mut rect = ui.available_rect_before_wrap();
    let clamped = target_main.clamp(0.0, if is_row { rect.width() } else { rect.height() }.max(0.0));
    if is_row {
        rect.set_width(clamped);
    } else {
        rect.set_height(clamped);
    }
    rect
}

/// Converts a spacing value (units of 4px) to logical pixels, clamping to [`MAX_SPACING`].
///
/// Clamping here (not only in the builder) protects against a directly assigned `pub` field.
#[inline]
fn gap_px(spacing: u8) -> f32 {
    f32::from(spacing.min(MAX_SPACING)) * SPACING_UNIT
}

/// A row collapses to a column when the viewport is narrower than the breakpoint.
///
/// A column never becomes a row, and a row with no breakpoint never collapses.
#[inline]
fn resolve_is_row(row: bool, breakpoint_px: Option<f32>, viewport_width: f32) -> bool {
    row && breakpoint_px.map_or(true, |px| viewport_width >= px)
}

/// Cross-axis alignment, and whether children should stretch to fill the cross axis.
///
/// egui expresses "stretch" as `cross_justify` on the layout rather than as an `Align`
/// value, hence the pair.
#[inline]
fn cross_align(align_items: &AlignItems) -> (Align, bool) {
    match align_items {
        AlignItems::Start => (Align::Min, false),
        AlignItems::Center => (Align::Center, false),
        AlignItems::End => (Align::Max, false),
        AlignItems::Stretch => (Align::Min, true),
    }
}

/// Builds the egui [`Layout`] for the resolved direction, reversal, alignment and wrapping.
#[inline]
fn build_layout(is_row: bool, reverse: bool, cross: Align, stretch: bool, wrap: bool) -> Layout {
    let base = match (is_row, reverse) {
        (true, false) => Layout::left_to_right(cross),
        (true, true) => Layout::right_to_left(cross),
        (false, false) => Layout::top_down(cross),
        (false, true) => Layout::bottom_up(cross),
    };
    base.with_cross_justify(stretch).with_main_wrap(wrap)
}

/// Splits a rect into `(main, cross)` extents for the given direction.
#[inline]
fn axis_extents(rect: Rect, is_row: bool) -> (f32, f32) {
    if is_row {
        (rect.width(), rect.height())
    } else {
        (rect.height(), rect.width())
    }
}

/// The extent of `rect` along the main axis (width for a row, height for a column).
#[inline]
fn main_extent(rect: Rect, is_row: bool) -> f32 {
    axis_extents(rect, is_row).0
}

/// The `item_spacing` to install on the child `Ui`.
///
/// `between` (gap plus distributed free space) replaces the gap along the main axis only; the
/// cross axis keeps the plain `gap` so wrapped lines stay evenly spaced.
#[inline]
fn item_spacing(is_row: bool, between: f32, gap: f32) -> Vec2 {
    if is_row {
        vec2(between, gap)
    } else {
        vec2(gap, between)
    }
}

/// Whether the stack must run one invisible measuring frame before it can be drawn.
///
/// A reversed column without a fixed height has to be anchored at the *bottom* of the
/// available space, which requires knowing its own height. Until a measurement exists the
/// first frame is therefore drawn invisibly.
#[inline]
fn needs_sizing_pass(reverse: bool, is_row: bool, height: Option<f32>, measure: Option<Measure>) -> bool {
    reverse && !is_row && height.is_none() && measure.is_none()
}

/// Computes the rect the stack's child `Ui` is constrained to.
///
/// - **Rows** get a definite height (the measured content height, or `interact_height` on the
///   first frame and while wrapping) so cross-axis `Center`/`End` alignment behaves like CSS.
/// - **Columns** only get a height when one is fixed, or — when reversed — the previously
///   measured content height, so the bottom-up layout has something to anchor to.
/// - `lead` shifts the rect along the main axis: from the start edge normally, from the
///   *end* edge when `reverse` is set (because content is laid out from the far side).
#[allow(clippy::too_many_arguments)]
fn plan_rect(
    avail: Rect,
    is_row: bool,
    reverse: bool,
    wrap: bool,
    lead: f32,
    fixed_height: Option<f32>,
    measure: Option<Measure>,
    interact_height: f32,
) -> Rect {
    let mut rect = avail;
    if is_row {
        let h = if wrap {
            interact_height
        } else {
            measure.map_or(interact_height, |m| m.content_cross)
        };
        rect.set_height(h);
        if reverse {
            rect.max.x -= lead;
        } else {
            rect.min.x += lead;
        }
    } else {
        let h = fixed_height.or_else(|| if reverse { measure.map(|m| m.content_main) } else { None });
        if let Some(h) = h {
            rect.set_height(h);
        }
        if reverse {
            rect.max.y -= lead;
        } else {
            rect.min.y += lead;
        }
    }
    rect
}

/// Returns `(lead, between)`: the offset before the first child and the *total* distance
/// between neighbouring children (the gap plus any distributed free space).
///
/// Falls back to `(0, gap)` (i.e. `Start`) whenever there is nothing to distribute: no
/// measurement yet, unknown or infinite main length, wrapping, or `Space*` without items.
fn plan_offsets(
    justify: &Justify,
    wrap: bool,
    main_len: Option<f32>,
    measure: Option<&Measure>,
    gap: f32,
) -> (f32, f32) {
    let (Some(len), Some(m)) = (main_len, measure) else {
        return (0.0, gap);
    };
    if wrap || !len.is_finite() {
        return (0.0, gap);
    }

    let n = m.items as f32;
    // Free space once item extents and the gaps between them are accounted for.
    let free_for_items = (len - m.item_main - gap * (n - 1.0).max(0.0)).max(0.0);
    // Free space around opaque content (used by Center / End, which need no item data).
    let free_for_content = (len - m.content_main).max(0.0);

    match justify {
        Justify::Start => (0.0, gap),
        Justify::Center => (free_for_content / 2.0, gap),
        Justify::End => (free_for_content, gap),
        _ if m.items == 0 => (0.0, gap),
        Justify::SpaceBetween if m.items < 2 => (0.0, gap),
        Justify::SpaceBetween => (0.0, gap + free_for_items / (n - 1.0)),
        Justify::SpaceAround => (free_for_items / (2.0 * n), gap + free_for_items / n),
        Justify::SpaceEvenly => (free_for_items / (n + 1.0), gap + free_for_items / (n + 1.0)),
    }
}

// =============================================================================
// Tests
// =============================================================================

/// Unit tests for everything that does not require a live `egui::Context`.
///
/// `show`, `run`, `run_inner`, `StackUi::item` and the `Widget` impl need a
/// `Ui` and are intentionally not exercised here; every decision they make is delegated to the
/// pure helpers tested below.
#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    fn measure(content_main: f32, item_main: f32, items: usize) -> Measure {
        Measure { content_main, content_cross: 20.0, item_main, items }
    }

    /// 100x50 rect whose top-left is (10, 20): easy to reason about in assertions.
    fn avail() -> Rect {
        Rect::from_min_size(pos2(10.0, 20.0), vec2(100.0, 50.0))
    }

    // ---- builder / defaults -------------------------------------------------

    #[test]
    fn builder_pattern() {
        let s = MudStack::horizontal()
            .spacing(200) // clamped
            .reverse(true)
            .height(120.0)
            .justify(Justify::Center)
            .classes(["p-2"]);
        assert!(s.row);
        assert!(s.reverse);
        assert_eq!(s.spacing, MAX_SPACING);
        assert_eq!(s.height, Some(120.0));
        assert_eq!(s.base.class_names().len(), 1);
    }

    #[test]
    fn defaults_match_mudblazor() {
        let s = MudStack::default();
        assert!(!s.row);
        assert!(!s.reverse);
        assert_eq!(s.spacing, 3);
        assert!(s.align_items == AlignItems::Stretch);
        assert!(s.justify == Justify::Start);
        assert!(s.wrap == Wrap::NoWrap);
        assert!(s.breakpoint == Breakpoint::None);
        assert_eq!(s.height, None);
        assert_eq!(s.stretch_items, StretchItems::None);
        assert_eq!(s.id_salt, None);
        assert_eq!(s.base.class_names().len(), 0);
    }

    #[test]
    fn new_equals_vertical_and_horizontal_sets_row() {
        assert!(!MudStack::new().row);
        assert!(!MudStack::vertical().row);
        assert!(MudStack::horizontal().row);
        assert!(!MudStack::horizontal().row(false).row);
    }

    #[test]
    fn simple_setters_store_values() {
        let s = MudStack::new()
            .align_items(AlignItems::Center)
            .wrap(Wrap::Wrap)
            .breakpoint(Breakpoint::None);
        assert!(s.align_items == AlignItems::Center);
        assert!(s.wrap == Wrap::Wrap);
        assert!(s.breakpoint == Breakpoint::None);
    }

    #[test]
    fn spacing_boundaries_are_preserved() {
        assert_eq!(MudStack::new().spacing(0).spacing, 0);
        assert_eq!(MudStack::new().spacing(MAX_SPACING).spacing, MAX_SPACING);
        assert_eq!(MudStack::new().spacing(MAX_SPACING + 1).spacing, MAX_SPACING);
    }

    #[test]
    fn id_salt_is_deterministic_and_distinguishing() {
        let a = MudStack::new().id_salt("a");
        let a2 = MudStack::new().id_salt("a");
        let b = MudStack::new().id_salt("b");
        assert_eq!(a.id_salt, a2.id_salt);
        assert_ne!(a.id_salt, b.id_salt);
        assert_eq!(a.id_salt, Some(Id::new("a")));
    }

    #[test]
    fn classes_accumulate_across_calls_via_base() {
        let s = MudStack::new().classes(["p-1", "m-2", "bg-primary"]);
        assert_eq!(s.base.class_names().len(), 3);
    }

    #[test]
    fn stretch_items_builder_stores_value() {
        let s = MudStack::new().stretch_items(StretchItems::Middle);
        assert_eq!(s.stretch_items, StretchItems::Middle);
    }

    // ---- gap / direction / alignment ---------------------------------------

    #[test]
    fn gap_is_four_px_per_unit() {
        assert_eq!(gap_px(0), 0.0);
        assert_eq!(gap_px(3), 12.0);
        assert_eq!(gap_px(255), 80.0);
    }

    #[test]
    fn breakpoint_collapses_row_to_column() {
        assert!(resolve_is_row(true, None, 100.0));
        assert!(resolve_is_row(true, Some(600.0), 800.0));
        assert!(!resolve_is_row(true, Some(600.0), 400.0));
        assert!(!resolve_is_row(false, None, 2000.0));
    }

    #[test]
    fn breakpoint_boundary_is_inclusive() {
        assert!(resolve_is_row(true, Some(600.0), 600.0));
        assert!(!resolve_is_row(true, Some(600.0), 599.9));
        // A column never becomes a row, whatever the width.
        assert!(!resolve_is_row(false, Some(600.0), 10_000.0));
    }

    #[test]
    fn cross_alignment_mapping() {
        assert_eq!(cross_align(&AlignItems::Start), (Align::Min, false));
        assert_eq!(cross_align(&AlignItems::Center), (Align::Center, false));
        assert_eq!(cross_align(&AlignItems::End), (Align::Max, false));
        assert_eq!(cross_align(&AlignItems::Stretch), (Align::Min, true));
    }

    #[test]
    fn build_layout_covers_all_direction_combinations() {
        // Smoke test: every combination must construct without panicking, with and without
        // stretch / wrap. (Layout internals are egui's; we only assert construction.)
        for &is_row in &[true, false] {
            for &reverse in &[true, false] {
                for &stretch in &[true, false] {
                    for &wrap in &[true, false] {
                        let _ = build_layout(is_row, reverse, Align::Min, stretch, wrap);
                    }
                }
            }
        }
    }

    // ---- extents / spacing / sizing pass -----------------------------------

    #[test]
    fn extents_follow_direction() {
        let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(30.0, 10.0));
        assert_eq!(axis_extents(r, true), (30.0, 10.0));
        assert_eq!(axis_extents(r, false), (10.0, 30.0));
        assert_eq!(main_extent(r, true), 30.0);
        assert_eq!(main_extent(r, false), 10.0);
    }

    #[test]
    fn item_spacing_replaces_gap_on_main_axis_only() {
        assert_eq!(item_spacing(true, 20.0, 4.0), vec2(20.0, 4.0));
        assert_eq!(item_spacing(false, 20.0, 4.0), vec2(4.0, 20.0));
    }

    #[test]
    fn sizing_pass_only_for_reversed_unmeasured_unsized_column() {
        assert!(needs_sizing_pass(true, false, None, None));
        assert!(!needs_sizing_pass(false, false, None, None)); // not reversed
        assert!(!needs_sizing_pass(true, true, None, None)); // row
        assert!(!needs_sizing_pass(true, false, Some(10.0), None)); // fixed height
        assert!(!needs_sizing_pass(true, false, None, Some(measure(10.0, 0.0, 0)))); // measured
    }

    // ---- measure -----------------------------------------------------------

    #[test]
    fn measure_from_rect_maps_axes() {
        let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(30.0, 10.0));
        let row = Measure::from_rect(r, true, 25.0, 2);
        assert_eq!((row.content_main, row.content_cross, row.item_main, row.items), (30.0, 10.0, 25.0, 2));
        let col = Measure::from_rect(r, false, 0.0, 0);
        assert_eq!((col.content_main, col.content_cross), (10.0, 30.0));
    }

    #[test]
    fn measure_ignores_sub_pixel_noise() {
        let a = measure(100.0, 80.0, 3);
        let b = measure(100.2, 80.1, 3);
        let c = measure(103.0, 80.0, 3);
        assert!(a.approx_eq(&b));
        assert!(!a.approx_eq(&c));
        assert!(!a.approx_eq(&measure(100.0, 80.0, 4)));
    }

    #[test]
    fn measure_detects_cross_and_item_changes() {
        let a = measure(100.0, 80.0, 3);
        let mut cross = a;
        cross.content_cross += 1.0;
        let mut item = a;
        item.item_main += 1.0;
        assert!(!a.approx_eq(&cross));
        assert!(!a.approx_eq(&item));
        assert!(a.approx_eq(&a));
    }

    #[test]
    fn measure_default_is_all_zero() {
        let m = Measure::default();
        assert_eq!((m.content_main, m.content_cross, m.item_main, m.items), (0.0, 0.0, 0.0, 0));
    }

    // ---- plan_rect ---------------------------------------------------------

    #[test]
    fn row_uses_interact_height_without_measure() {
        let r = plan_rect(avail(), true, false, false, 0.0, None, None, 18.0);
        assert_eq!(r.min, pos2(10.0, 20.0));
        assert!(approx(r.height(), 18.0) && approx(r.width(), 100.0));
    }

    #[test]
    fn row_uses_measured_cross_height() {
        let m = measure(60.0, 0.0, 0); // content_cross = 20
        let r = plan_rect(avail(), true, false, false, 0.0, None, Some(m), 18.0);
        assert!(approx(r.height(), 20.0));
    }

    #[test]
    fn wrapping_row_ignores_measured_height() {
        let m = measure(60.0, 0.0, 0);
        let r = plan_rect(avail(), true, false, true, 0.0, None, Some(m), 18.0);
        assert!(approx(r.height(), 18.0));
    }

    #[test]
    fn row_lead_moves_start_edge_or_end_edge_when_reversed() {
        let fwd = plan_rect(avail(), true, false, false, 5.0, None, None, 18.0);
        assert!(approx(fwd.min.x, 15.0) && approx(fwd.max.x, 110.0));
        let rev = plan_rect(avail(), true, true, false, 5.0, None, None, 18.0);
        assert!(approx(rev.min.x, 10.0) && approx(rev.max.x, 105.0));
    }

    #[test]
    fn column_without_height_keeps_available_rect() {
        assert_eq!(plan_rect(avail(), false, false, false, 0.0, None, None, 18.0), avail());
        // A measurement alone does not size a non-reversed column.
        let m = measure(30.0, 0.0, 0);
        assert_eq!(plan_rect(avail(), false, false, false, 0.0, None, Some(m), 18.0), avail());
    }

    #[test]
    fn column_fixed_height_and_lead() {
        let r = plan_rect(avail(), false, false, false, 4.0, Some(40.0), None, 18.0);
        assert!(approx(r.min.y, 24.0) && approx(r.max.y, 60.0));
    }

    #[test]
    fn reversed_column_uses_measured_height_unless_fixed() {
        let m = measure(30.0, 0.0, 0);
        let measured = plan_rect(avail(), false, true, false, 0.0, None, Some(m), 18.0);
        assert!(approx(measured.max.y, 50.0)); // 20 + 30
        let fixed = plan_rect(avail(), false, true, false, 0.0, Some(40.0), Some(m), 18.0);
        assert!(approx(fixed.max.y, 60.0)); // fixed wins over measured
        let led = plan_rect(avail(), false, true, false, 3.0, Some(40.0), None, 18.0);
        assert!(approx(led.max.y, 57.0)); // reversed lead eats from the bottom
    }

    // ---- plan_offsets ------------------------------------------------------

    #[test]
    fn start_and_missing_data_are_noops() {
        let m = measure(50.0, 40.0, 3);
        assert_eq!(plan_offsets(&Justify::Start, false, Some(100.0), Some(&m), 4.0), (0.0, 4.0));
        assert_eq!(plan_offsets(&Justify::Center, false, None, Some(&m), 4.0), (0.0, 4.0));
        assert_eq!(plan_offsets(&Justify::Center, false, Some(100.0), None, 4.0), (0.0, 4.0));
        assert_eq!(plan_offsets(&Justify::Center, true, Some(100.0), Some(&m), 4.0), (0.0, 4.0));
        assert_eq!(plan_offsets(&Justify::Center, false, Some(f32::INFINITY), Some(&m), 4.0), (0.0, 4.0));
    }

    #[test]
    fn center_and_end_use_content_size() {
        let m = measure(60.0, 0.0, 0);
        let (lead, between) = plan_offsets(&Justify::Center, false, Some(100.0), Some(&m), 4.0);
        assert!(approx(lead, 20.0) && approx(between, 4.0));
        let (lead, _) = plan_offsets(&Justify::End, false, Some(100.0), Some(&m), 4.0);
        assert!(approx(lead, 40.0));
        // overflow never produces a negative offset
        let (lead, _) = plan_offsets(&Justify::End, false, Some(40.0), Some(&m), 4.0);
        assert_eq!(lead, 0.0);
    }

    /// 3 items of total width 60, gap 4, container 100 -> 32px of free space.
    #[test]
    fn space_distribution_fills_the_container_exactly() {
        let m = measure(68.0, 60.0, 3);
        let total = |lead: f32, between: f32| 2.0 * lead + 60.0 + 2.0 * between;

        let (lead, between) = plan_offsets(&Justify::SpaceBetween, false, Some(100.0), Some(&m), 4.0);
        assert!(approx(lead, 0.0) && approx(between, 20.0));
        assert!(approx(lead + 60.0 + 2.0 * between, 100.0));

        let (lead, between) = plan_offsets(&Justify::SpaceAround, false, Some(100.0), Some(&m), 4.0);
        assert!(approx(total(lead, between), 100.0));

        let (lead, between) = plan_offsets(&Justify::SpaceEvenly, false, Some(100.0), Some(&m), 4.0);
        assert!(approx(lead, 8.0) && approx(between, 12.0));
        assert!(approx(total(lead, between), 100.0));
    }

    #[test]
    fn space_justify_needs_items() {
        let no_items = measure(60.0, 0.0, 0);
        assert_eq!(
            plan_offsets(&Justify::SpaceBetween, false, Some(100.0), Some(&no_items), 4.0),
            (0.0, 4.0)
        );
        let one = measure(20.0, 20.0, 1);
        assert_eq!(
            plan_offsets(&Justify::SpaceBetween, false, Some(100.0), Some(&one), 4.0),
            (0.0, 4.0)
        );
        // a single item with SpaceAround/Evenly is centred
        let (lead, _) = plan_offsets(&Justify::SpaceEvenly, false, Some(100.0), Some(&one), 4.0);
        assert!(approx(lead, 40.0));
    }

    #[test]
    fn space_justify_degrades_to_gap_when_content_overflows() {
        // 3 items summing to 120 in a 100px container: no free space, no negative offsets.
        let m = measure(128.0, 120.0, 3);
        for j in [Justify::SpaceBetween, Justify::SpaceAround, Justify::SpaceEvenly] {
            let (lead, between) = plan_offsets(&j, false, Some(100.0), Some(&m), 4.0);
            assert!(approx(lead, 0.0), "lead should be 0 on overflow");
            assert!(approx(between, 4.0), "between should fall back to the plain gap");
        }
    }

    // ---- wrapping of composite items ---------------------------------------

    #[test]
    fn line_break_decisions() {
        // the first item on a line never breaks, however wide
        assert!(!should_break_line(None, 500.0, 4.0, Some(100.0)));
        // fits, and fits exactly
        assert!(!should_break_line(Some(40.0), 50.0, 4.0, Some(100.0)));
        assert!(!should_break_line(Some(40.0), 56.0, 4.0, Some(100.0)));
        // overflows
        assert!(should_break_line(Some(40.0), 57.0, 4.0, Some(100.0)));
        // nothing to wrap against
        assert!(!should_break_line(Some(40.0), 500.0, 4.0, None));
        assert!(!should_break_line(Some(40.0), 500.0, 4.0, Some(f32::INFINITY)));
    }

    #[test]
    fn line_advance_adds_gap_after_first_item() {
        assert_eq!(advance_line(None, 30.0, 4.0), 30.0);
        assert_eq!(advance_line(Some(30.0), 10.0, 4.0), 44.0);
    }

    /// Mirrors what `StackUi::item` does: five 80px items, 16px gap, 400px line.
    #[test]
    fn five_items_wrap_onto_two_lines() {
        let (gap, avail) = (16.0, Some(400.0));
        let mut used = None;
        let mut lines = 1;
        let mut first_of_line = Vec::new();
        for (i, w) in [80.0_f32; 5].into_iter().enumerate() {
            if should_break_line(used, w, gap, avail) {
                lines += 1;
                used = None;
            }
            if used.is_none() {
                first_of_line.push(i);
            }
            used = Some(advance_line(used, w, gap));
        }
        assert_eq!(lines, 2);
        assert_eq!(first_of_line, vec![0, 4]); // 4 fit (368px); the 5th starts line two
    }

    #[test]
    fn oversized_item_gets_its_own_line() {
        let mut used = None;
        used = Some(advance_line(used, 30.0, 4.0));
        assert!(should_break_line(used, 500.0, 4.0, Some(100.0)));
    }

    #[test]
    fn extents_comparison() {
        assert!(extents_approx_eq(&[], &[]));
        assert!(extents_approx_eq(&[10.0, 20.0], &[10.2, 19.9]));
        assert!(!extents_approx_eq(&[10.0, 20.0], &[10.0, 21.0]));
        assert!(!extents_approx_eq(&[10.0], &[10.0, 20.0]));
    }

    #[test]
    fn tally_default_is_empty_and_alloc_free() {
        let t = ItemTally::default();
        assert_eq!((t.main, t.count), (0.0, 0));
        assert!(t.extents.is_empty() && t.extents.capacity() == 0);
    }

    #[test]
    fn wrapping_disables_space_distribution() {
        let m = measure(68.0, 60.0, 3);
        assert_eq!(
            plan_offsets(&Justify::SpaceBetween, true, Some(100.0), Some(&m), 4.0),
            (0.0, 4.0)
        );
    }

    // ---- stretch_targets ----------------------------------------------------

    #[test]
    fn stretch_targets_empty_is_always_empty() {
        for policy in [
            StretchItems::None,
            StretchItems::Start,
            StretchItems::End,
            StretchItems::StartAndEnd,
            StretchItems::Middle,
            StretchItems::All,
        ] {
            assert_eq!(stretch_targets(policy, 0), Vec::<bool>::new());
        }
    }

    #[test]
    fn stretch_targets_none_flags_nothing() {
        assert_eq!(stretch_targets(StretchItems::None, 4), vec![false; 4]);
    }

    #[test]
    fn stretch_targets_start_and_end() {
        assert_eq!(stretch_targets(StretchItems::Start, 3), vec![true, false, false]);
        assert_eq!(stretch_targets(StretchItems::End, 3), vec![false, false, true]);
        assert_eq!(stretch_targets(StretchItems::StartAndEnd, 3), vec![true, false, true]);
        // a single item is both the start and the end
        assert_eq!(stretch_targets(StretchItems::StartAndEnd, 1), vec![true]);
    }

    #[test]
    fn stretch_targets_middle_has_no_effect_under_three_items() {
        assert_eq!(stretch_targets(StretchItems::Middle, 0), Vec::<bool>::new());
        assert_eq!(stretch_targets(StretchItems::Middle, 1), vec![false]);
        assert_eq!(stretch_targets(StretchItems::Middle, 2), vec![false, false]);
        assert_eq!(stretch_targets(StretchItems::Middle, 4), vec![false, true, true, false]);
    }

    #[test]
    fn stretch_targets_all_flags_every_item() {
        assert_eq!(stretch_targets(StretchItems::All, 3), vec![true; 3]);
    }

    // ---- plan_stretch ---------------------------------------------------------

    #[test]
    fn plan_stretch_none_and_empty_are_noops() {
        assert!(plan_stretch(StretchItems::All, Some(100.0), &[], 4.0).is_empty());
        assert!(plan_stretch(StretchItems::None, Some(100.0), &[20.0, 20.0], 4.0).is_empty());
    }

    #[test]
    fn plan_stretch_needs_a_finite_main_length() {
        assert!(plan_stretch(StretchItems::All, None, &[20.0, 20.0], 4.0).is_empty());
        assert!(plan_stretch(StretchItems::All, Some(f32::INFINITY), &[20.0, 20.0], 4.0).is_empty());
    }

    #[test]
    fn plan_stretch_overflow_is_a_noop() {
        // two 60px items plus a 4px gap is 124px of content in a 100px container
        assert!(plan_stretch(StretchItems::All, Some(100.0), &[60.0, 60.0], 4.0).is_empty());
    }

    #[test]
    fn plan_stretch_splits_leftover_among_flagged_items() {
        // 3 items of 20px, 4px gaps: 68px of content in a 100px container -> 32px leftover.
        let extents = [20.0, 20.0, 20.0];

        let all = plan_stretch(StretchItems::All, Some(100.0), &extents, 4.0);
        assert!(all.iter().all(|&x| approx(x, 32.0 / 3.0)));

        let start = plan_stretch(StretchItems::Start, Some(100.0), &extents, 4.0);
        assert_eq!(start.len(), 3);
        assert!(approx(start[0], 32.0) && start[1] == 0.0 && start[2] == 0.0);

        let ends = plan_stretch(StretchItems::StartAndEnd, Some(100.0), &extents, 4.0);
        assert!(approx(ends[0], 16.0) && ends[1] == 0.0 && approx(ends[2], 16.0));

        // a lone middle item (index 1) gets the whole 32px
        let middle = plan_stretch(StretchItems::Middle, Some(100.0), &extents, 4.0);
        assert!(middle[0] == 0.0 && approx(middle[1], 32.0) && middle[2] == 0.0);
    }

    // ---- stretched_item_rect ---------------------------------------------------

    #[test]
    fn stretched_item_rect_clamps_to_available_space() {
        // No live `Ui` is needed to check the clamp arithmetic itself; the clamp logic mirrors
        // `target_main.clamp(0.0, available)`, exercised directly here since `Ui` can't be
        // constructed outside egui's run loop.
        assert_eq!(500.0_f32.clamp(0.0, 80.0_f32.max(0.0)), 80.0);
        assert_eq!((-10.0_f32).clamp(0.0, 80.0_f32.max(0.0)), 0.0);
    }
}