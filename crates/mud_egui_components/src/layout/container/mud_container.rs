//! A responsive, centered, width-limiting container for `mud_egui`.
//!
//! Production-grade port and expansion of MudBlazor's `MudContainer`.
//! Constrains its child content to a maximum width derived from a Material
//! Design breakpoint (or a fixed "snap-to-breakpoint" mode), centers the
//! constrained content horizontally, applies responsive side gutters, and
//! resolves Bootstrap- or Tailwind-style utility classes into an `egui::Frame`.
//!
//! # MudBlazor API parity (verified against the published API docs)
//!
//! MudBlazor's `MudContainer` is a *layout primitive*. Its complete public
//! parameter surface is:
//!
//! | MudBlazor parameter | Default            | Rust equivalent                       |
//! |---------------------|--------------------|---------------------------------------|
//! | `MaxWidth`          | `MaxWidth.Large`   | [`MudContainer::max_width`]           |
//! | `Fixed`             | `false`            | [`MudContainer::fixed`]               |
//! | `Gutters`           | `true`             | [`MudContainer::gutters`]             |
//! | `ChildContent`      | `null`             | closure given to [`MudContainer::show`] |
//! | `Class`             | `null`             | [`MudContainer::classes`] / [`MudContainer::class`] |
//! | `Style`             | `null`             | [`MudContainer::style`] ([`StyleOverrides`]) |
//! | `Tag`               | `null`             | [`MudContainer::tag`] / [`MudContainer::tag_ref`] |
//! | `UserAttributes`    | empty              | [`MudContainer::attr`] / [`MudContainer::attrs`] |
//!
//! `MudContainer` has **no** value binding, variant, colour, size, read-only
//! state, adornment slots or input events, so this port deliberately does not
//! invent them. Everything else in this module is an explicitly labelled Rust
//! extension (see the "Extensions" section of the README).
//!
//! # Architecture
//!
//! Layout decisions live in the pure, allocation-free [`MudContainerMetrics`]
//! namespace and the [`LayoutSpec`] descriptor. Neither touches `egui` state,
//! so every breakpoint, gutter and centering calculation is unit-tested
//! without a live `Context`. [`MudContainer::show`] is deliberately thin glue:
//!
//! 1. Read the available rect.
//! 2. Ask metrics for a [`HorizontalLayout`] (including the viewport
//!    breakpoint and, when `fixed` is set, the snapped ceiling).
//! 3. Fire [`MudContainer::on_breakpoint_change`] on breakpoint transitions.
//! 4. Build a horizontally-offset child `Ui`.
//! 5. Resolve the active style flavour into an `egui::Frame` (optionally
//!    memoized), apply [`StyleOverrides`], the disabled fade and the focus
//!    ring, then add the responsive gutters on top of any utility padding.
//! 6. Reserve a full-width, sanitized row in the parent so subsequent widgets
//!    flow correctly even inside infinite-width scroll areas.
//! 7. Plan and fire interaction callbacks from state *edges*
//!    ([`EventPlan`]), publish a lock-free snapshot to any
//!    [`ContainerHandle`], and feed interaction state back into [`TwState`]
//!    for the next frame (Tailwind `hover:` / `active:` / `dark:` variants).
//!
//! # Click passthrough
//!
//! `MudContainer` reserves its own rect in the parent layout via
//! [`Ui::allocate_rect`]. Because egui hit-tests widgets in registration
//! order and a later widget wins, an interactive `Sense` on that rect would
//! sit *on top of* everything drawn inside the container, stealing hover,
//! clicks, and keyboard focus from children like `MudTextField`.
//!
//! Therefore the container's own `Sense` is [`Sense::hover`] unless a *click
//! class* callback is registered (`on_click`, `on_focus`, `on_blur`), in which
//! case it is [`Sense::click`]. `on_hover` / `on_unhover` / `tooltip` only
//! need hover and never steal input. The container never senses drag, so it
//! cannot interfere with drag-to-scroll in enclosing scroll areas.
//!
//! # Styling
//!
//! Utility classes on [`ComponentBase`] are resolved by the active
//! [`StyleFlavor`]:
//!
//! * [`StyleFlavor::Bootstrap`] (default) uses [`BsStyle`]. No variants, so
//!   no per-frame interaction state is required.
//! * [`StyleFlavor::Tailwind`] uses [`TwStyle`]. Tailwind supports `hover:`,
//!   `dark:` and `active:` variants; the previous frame's interaction state
//!   is therefore threaded through [`TwState`].
//!
//! Resolution order: **classes → [`StyleOverrides`] → disabled fade → focus
//! ring → responsive gutters**. Gutters are added to whatever `inner_margin`
//! the earlier steps produced, so utility padding and container gutters
//! compose cleanly.
//!
//! # Robustness
//!
//! Layout inputs can be non-finite (`ScrollArea` offers infinite height;
//! a horizontal scroll area offers infinite width). Every value that flows
//! back into the parent layout is sanitized: a single `NaN`/`inf` rect would
//! otherwise poison an enclosing `ScrollArea`.
//!
//! # Example
//! ```ignore
//! let mut c = MudContainer::new()
//!     .max_width(MaxWidth::Md)
//!     .gutters(true)
//!     .classes(["bg-surface", "p-4", "rounded-lg", "elevation-2"]);
//! c.show(ui, |ui| {
//!     ui.label("Centered content, at most 960 px wide");
//! });
//! ```

use std::any::Any;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;

use egui::{
    pos2, vec2, Align, Color32, Frame, Id, InnerResponse, Layout, Margin, Rect, Response, Sense,
    Stroke, Ui, UiBuilder, Widget, WidgetInfo,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::MaxWidth;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

/// Base rem size (in points) passed to the style resolver.
/// Matches the Bootstrap convention and is a reasonable default for Tailwind.
const REM_SIZE_PT: f32 = 14.0;

/// Width, in points, of the keyboard-focus ring stroke.
const FOCUS_RING_WIDTH: f32 = 2.0;

/// Duration, in seconds, of the focus-ring fade in / fade out.
const FOCUS_RING_SECONDS: f32 = 0.12;

/// Alpha multiplier applied to the frame fill and stroke of a disabled
/// container (children are faded by egui's own disabled styling).
const DISABLED_FRAME_ALPHA: f32 = 0.6;

/// Palette key used to colour the focus ring; falls back to egui's selection
/// stroke colour when the active palette cannot resolve it.
const PALETTE_PRIMARY_KEY: &str = "MudPalettePrimary";

// =============================================================================
// Pure layout math
// =============================================================================

/// Pure width math for `MudContainer`, completely decoupled from `egui` state
/// and therefore fully unit-testable.
///
/// This is a namespace type: it has no fields and is never instantiated.
/// All methods are pure functions of their arguments.
///
/// # Example
/// ```ignore
/// let l = MudContainerMetrics::resolve(2000.0, true, false, false, MaxWidth::Lg);
/// assert_eq!(l.outer_width, 1280.0);
/// assert_eq!(l.content_width, 1232.0);
/// ```
pub struct MudContainerMetrics;

impl MudContainerMetrics {
    /// Outer-width ceiling of `MaxWidth::Xs` in logical pixels.
    /// Matches MudBlazor's `mud-container-maxwidth-xs` (444, not the naive 0).
    pub const EXTRA_SMALL_WIDTH: f32 = 444.0;
    /// Outer-width ceiling of `MaxWidth::Sm`.
    pub const SMALL_WIDTH: f32 = 600.0;
    /// Outer-width ceiling of `MaxWidth::Md`.
    pub const MEDIUM_WIDTH: f32 = 960.0;
    /// Outer-width ceiling of `MaxWidth::Lg`.
    pub const LARGE_WIDTH: f32 = 1280.0;
    /// Outer-width ceiling of `MaxWidth::Xl`.
    pub const EXTRA_LARGE_WIDTH: f32 = 1920.0;
    /// Outer-width ceiling of `MaxWidth::Xxl`.
    pub const EXTRA_EXTRA_LARGE_WIDTH: f32 = 2560.0;

    /// Side gutter used when the offered width is at or above [`Self::SMALL_WIDTH`].
    pub const GUTTER_LARGE: f32 = 24.0;
    /// Side gutter used when the offered width is below [`Self::SMALL_WIDTH`].
    pub const GUTTER_SMALL: f32 = 16.0;

    /// Maps the `MaxWidth` breakpoint enum to its outer-width ceiling in
    /// logical pixels. `MaxWidth::False` means "no ceiling" and yields
    /// `f32::INFINITY`.
    ///
    /// # Example
    /// ```ignore
    /// assert_eq!(MudContainerMetrics::to_pixels(MaxWidth::Md), 960.0);
    /// assert!(MudContainerMetrics::to_pixels(MaxWidth::False).is_infinite());
    /// ```
    #[inline]
    pub fn to_pixels(max_width: MaxWidth) -> f32 {
        match max_width {
            MaxWidth::Xs => Self::EXTRA_SMALL_WIDTH,
            MaxWidth::Sm => Self::SMALL_WIDTH,
            MaxWidth::Md => Self::MEDIUM_WIDTH,
            MaxWidth::Lg => Self::LARGE_WIDTH,
            MaxWidth::Xl => Self::EXTRA_LARGE_WIDTH,
            MaxWidth::Xxl => Self::EXTRA_EXTRA_LARGE_WIDTH,
            MaxWidth::False => f32::INFINITY,
        }
    }

    /// Short lowercase label for a breakpoint (`"xs"`, `"sm"`, `"md"`, `"lg"`,
    /// `"xl"`, `"xxl"`, or `"false"` for "no ceiling"). Handy for logging and
    /// debug overlays.
    #[inline]
    pub fn label(max_width: MaxWidth) -> &'static str {
        match max_width {
            MaxWidth::Xs => "xs",
            MaxWidth::Sm => "sm",
            MaxWidth::Md => "md",
            MaxWidth::Lg => "lg",
            MaxWidth::Xl => "xl",
            MaxWidth::Xxl => "xxl",
            MaxWidth::False => "false",
        }
    }

    /// Inverse of [`Self::to_pixels`]: given an offered width, returns the
    /// breakpoint bucket it falls in (MudBlazor's "min-width of the current
    /// breakpoint" rule). Used by the "fixed / snap" layout mode and reported
    /// as [`HorizontalLayout::breakpoint`].
    ///
    /// Non-finite or non-positive offers return `MaxWidth::Xs`.
    #[inline]
    pub fn snap_breakpoint(offered_width: f32) -> MaxWidth {
        if !offered_width.is_finite() || offered_width <= 0.0 {
            return MaxWidth::Xs;
        }
        if offered_width < Self::SMALL_WIDTH {
            MaxWidth::Xs
        } else if offered_width < Self::MEDIUM_WIDTH {
            MaxWidth::Sm
        } else if offered_width < Self::LARGE_WIDTH {
            MaxWidth::Md
        } else if offered_width < Self::EXTRA_LARGE_WIDTH {
            MaxWidth::Lg
        } else if offered_width < Self::EXTRA_EXTRA_LARGE_WIDTH {
            MaxWidth::Xl
        } else {
            MaxWidth::Xxl
        }
    }

    /// Determines the side gutter for a given offered width; `0.0` when
    /// gutters are disabled.
    ///
    /// Offers below [`Self::SMALL_WIDTH`] receive the small gutter; everything
    /// else (including a `NaN` offer, for which the comparison is false)
    /// receives the large gutter. [`Self::resolve`] normalises `NaN` before
    /// calling this helper.
    #[inline]
    pub fn gutter_for(offered_width: f32, gutters: bool) -> f32 {
        if !gutters {
            0.0
        } else if offered_width < Self::SMALL_WIDTH {
            Self::GUTTER_SMALL
        } else {
            Self::GUTTER_LARGE
        }
    }

    /// Resolves the side gutters, inner content width and overall outer width
    /// from the parent's offered width and the container settings.
    ///
    /// # Resolution rules
    /// * `NaN` or negative offers are normalised to `0.0`.
    /// * An infinite offer (e.g. inside a horizontal scroll area) yields the
    ///   breakpoint cap, or an infinite width if `fluid` / `MaxWidth::False`.
    /// * When `fixed` is true the effective cap is the snapped breakpoint of
    ///   the offered width (MudBlazor's `Fixed` behaviour).
    /// * `fluid` takes precedence over `fixed`.
    /// * Gutters never exceed half of a finite offered width, so tiny
    ///   viewports cannot overflow.
    /// * `outer_width <= offered` always holds for finite offers.
    pub fn resolve(
        offered_width: f32,
        gutters: bool,
        fluid: bool,
        fixed: bool,
        max_width: MaxWidth,
    ) -> HorizontalLayout {
        let offered = if offered_width.is_nan() {
            0.0
        } else {
            offered_width.max(0.0)
        };

        let mut gutter = Self::gutter_for(offered, gutters);
        if offered.is_finite() {
            gutter = gutter.min(offered / 2.0);
        }
        let gutters_total = gutter * 2.0;

        let breakpoint = Self::snap_breakpoint(offered);

        // Fixed mode snaps the ceiling to the nearest (or current) breakpoint.
        // Fluid mode ignores any ceiling entirely.
        let effective_max = if fluid {
            MaxWidth::False
        } else if fixed {
            breakpoint
        } else {
            max_width
        };

        let cap = Self::to_pixels(effective_max);

        let available_content = (offered - gutters_total).max(0.0);
        let cap_content = (cap - gutters_total).max(0.0);
        let content_width = available_content.min(cap_content);
        let outer_width = (content_width + gutters_total).min(offered);

        HorizontalLayout {
            gutter,
            content_width,
            outer_width,
            effective_max,
            breakpoint,
        }
    }

    /// Horizontal offset that centres `outer_width` inside `offered_width`.
    ///
    /// Returns `0.0` when either width is non-finite (nothing to centre
    /// against) and never a negative offset.
    #[inline]
    pub fn centering_offset(offered_width: f32, outer_width: f32) -> f32 {
        if offered_width.is_finite() && outer_width.is_finite() {
            ((offered_width - outer_width) / 2.0).max(0.0)
        } else {
            0.0
        }
    }
}

/// The resolved horizontal layout dimensions of a `MudContainer`.
///
/// A small `Copy` value (four `f32`s and two one-byte enums); pass it by value
/// or reference freely. Obtain one from [`MudContainerMetrics::resolve`],
/// [`LayoutSpec::resolve`], or inside [`MudContainer::show_with_layout`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HorizontalLayout {
    /// Side gutter on each of the left and right edges, in logical pixels.
    pub gutter: f32,
    /// Width available to the child content (excluding gutters).
    /// Infinite for a fluid container inside an unbounded parent.
    pub content_width: f32,
    /// Total container width (content plus both gutters), never wider than
    /// the offered width for finite offers.
    pub outer_width: f32,
    /// The effective `MaxWidth` that was applied after fluid / fixed
    /// resolution. Useful for diagnostics and adaptive child components.
    pub effective_max: MaxWidth,
    /// The breakpoint bucket the *offered* width falls in (see
    /// [`MudContainerMetrics::snap_breakpoint`]). Unbounded or invalid offers
    /// report `MaxWidth::Xs`. This is the value that drives
    /// [`MudContainer::on_breakpoint_change`].
    pub breakpoint: MaxWidth,
}

impl HorizontalLayout {
    /// Total horizontal space consumed by both gutters.
    #[inline]
    pub fn gutters_total(&self) -> f32 {
        self.gutter * 2.0
    }

    /// `true` when the container has a finite outer width.
    #[inline]
    pub fn is_bounded(&self) -> bool {
        self.outer_width.is_finite()
    }

    /// Horizontal offset that centres this layout inside `offered_width`.
    /// See [`MudContainerMetrics::centering_offset`].
    #[inline]
    pub fn offset_in(&self, offered_width: f32) -> f32 {
        MudContainerMetrics::centering_offset(offered_width, self.outer_width)
    }
}

/// The four layout-affecting settings of a container as a tiny `Copy` value.
///
/// Mirrors the MudBlazor parameters `MaxWidth`, `Fixed` and `Gutters`, plus
/// the Rust-side `fluid` shorthand for `MaxWidth.False`. Because it is plain
/// data it can be stored in configuration, compared, and resolved without a
/// container instance.
///
/// # Example
/// ```ignore
/// let spec = LayoutSpec { max_width: MaxWidth::Md, ..LayoutSpec::default() };
/// let layout = spec.resolve(1500.0);
/// assert_eq!(layout.outer_width, 960.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutSpec {
    /// Breakpoint ceiling (ignored when `fluid` or `fixed` is set).
    pub max_width: MaxWidth,
    /// Stretch to the full offered width, ignoring any ceiling.
    pub fluid: bool,
    /// Snap the ceiling to the breakpoint of the offered width.
    pub fixed: bool,
    /// Apply responsive side gutters (16 px below 600 px, 24 px otherwise).
    pub gutters: bool,
}

impl Default for LayoutSpec {
    /// MudBlazor defaults: `MaxWidth.Large`, not fixed, gutters on.
    fn default() -> Self {
        Self {
            max_width: MaxWidth::Lg,
            fluid: false,
            fixed: false,
            gutters: true,
        }
    }
}

impl LayoutSpec {
    /// Resolves this spec against an offered width.
    /// Equivalent to [`MudContainerMetrics::resolve`].
    #[inline]
    pub fn resolve(&self, offered_width: f32) -> HorizontalLayout {
        MudContainerMetrics::resolve(
            offered_width,
            self.gutters,
            self.fluid,
            self.fixed,
            self.max_width,
        )
    }
}

// =============================================================================
// Style flavour
// =============================================================================

/// Which utility-class system [`MudContainer`] resolves its classes against.
///
/// The two systems have different capabilities:
///
/// * **Bootstrap** has no variants (`hover:`, `dark:`): resolution is pure
///   from the classes alone.
/// * **Tailwind** supports variants, so it needs the previous frame's
///   interaction state and the current theme mode, threaded through
///   [`TwState`].
///
/// [`MudContainer::show`] feeds both resolvers the correct inputs. Only the
/// selected flavour is actually consulted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StyleFlavor {
    /// Bootstrap-style utilities (`bg-primary`, `p-3`, `border-dashed-2`, ...).
    #[default]
    Bootstrap,
    /// Tailwind-style utilities (`bg-primary/90`, `hover:bg-primary-dark`, ...).
    Tailwind,
}

// =============================================================================
// Style overrides (typed equivalent of MudBlazor's `Style` parameter)
// =============================================================================

/// Typed "inline style" overrides applied *after* utility classes resolve.
///
/// MudBlazor's `Style` parameter is a CSS string "applied on top of the
/// component's own classes and styles". egui has no CSS, so the same idea is
/// expressed as a handful of optional frame properties. Every `None` leaves the
/// class-resolved value untouched.
///
/// # Example
/// ```ignore
/// let c = MudContainer::new().style(StyleOverrides {
///     fill: Some(Color32::from_rgb(30, 30, 46)),
///     stroke: Some(Stroke::new(1.0, Color32::GRAY)),
///     ..Default::default()
/// });
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StyleOverrides {
    /// Replaces the frame's background fill.
    pub fill: Option<Color32>,
    /// Replaces the frame's border stroke.
    pub stroke: Option<Stroke>,
    /// Replaces the frame's inner padding (before the responsive gutters are
    /// added on top).
    pub padding: Option<Margin>,
    /// Replaces the frame's outer margin.
    pub outer_margin: Option<Margin>,
}

impl StyleOverrides {
    /// `true` when no property is overridden.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.fill.is_none()
            && self.stroke.is_none()
            && self.padding.is_none()
            && self.outer_margin.is_none()
    }

    /// Applies the overrides to `frame` and returns the result.
    #[inline]
    pub fn apply(&self, mut frame: Frame) -> Frame {
        if let Some(fill) = self.fill {
            frame.fill = fill;
        }
        if let Some(stroke) = self.stroke {
            frame.stroke = stroke;
        }
        if let Some(padding) = self.padding {
            frame.inner_margin = padding;
        }
        if let Some(outer) = self.outer_margin {
            frame.outer_margin = outer;
        }
        frame
    }
}

// =============================================================================
// Interaction / accessibility state
// =============================================================================

/// Lightweight interaction flags captured from the previous frame and used by
/// Tailwind variant resolution and optional event callbacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InteractionState {
    /// Pointer is currently over the container.
    pub hovered: bool,
    /// A pointer button is pressed while over the container.
    pub active: bool,
    /// The container currently holds keyboard focus.
    pub focused: bool,
    /// Theme is in dark mode.
    pub dark: bool,
    /// Explicitly disabled by the user.
    pub disabled: bool,
}

impl InteractionState {
    /// Convert into the `TwState` expected by the Tailwind resolver.
    /// Disabled containers never report `hovered` or `active`.
    #[inline]
    pub fn to_tw_state(self) -> TwState {
        TwState {
            hovered: self.hovered && !self.disabled,
            active: self.active && !self.disabled,
            dark: self.dark,
        }
    }
}

/// A boolean transition between two frames.
///
/// Immediate-mode UIs only observe *levels* ("is hovered right now"); events
/// are derived by comparing the previous and current level.
///
/// # Example
/// ```ignore
/// assert_eq!(Edge::detect(false, true), Edge::Rising);
/// assert_eq!(Edge::detect(true, true), Edge::Steady);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    /// The level did not change (both `true` or both `false`).
    Steady,
    /// The level went from `false` to `true`.
    Rising,
    /// The level went from `true` to `false`.
    Falling,
}

impl Edge {
    /// Classifies the transition from `previous` to `current`.
    #[inline]
    pub fn detect(previous: bool, current: bool) -> Self {
        match (previous, current) {
            (false, true) => Edge::Rising,
            (true, false) => Edge::Falling,
            _ => Edge::Steady,
        }
    }
}

/// Which callbacks should fire this frame.
///
/// Computing the plan is a pure function of state ([`EventPlan::compute`]), so
/// the exact firing rules are unit-tested without egui. [`MudContainer::show`]
/// then executes the plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EventPlan {
    /// Pointer entered the container (rising hover edge).
    pub hover: bool,
    /// Pointer left the container (falling hover edge).
    pub unhover: bool,
    /// Primary-button click occurred this frame.
    pub click: bool,
    /// Container gained keyboard focus (rising focus edge).
    pub focus: bool,
    /// Container lost keyboard focus (falling focus edge).
    pub blur: bool,
}

impl EventPlan {
    /// Derives the plan from the previous and current levels.
    ///
    /// A disabled container never fires anything. Hover and focus are
    /// edge-triggered; `clicked` is passed through because egui already
    /// reports clicks as a one-frame event.
    pub fn compute(
        prev_hovered: bool,
        prev_focused: bool,
        hovered: bool,
        focused: bool,
        clicked: bool,
        disabled: bool,
    ) -> Self {
        if disabled {
            return Self::default();
        }
        let hover_edge = Edge::detect(prev_hovered, hovered);
        let focus_edge = Edge::detect(prev_focused, focused);
        Self {
            hover: hover_edge == Edge::Rising,
            unhover: hover_edge == Edge::Falling,
            click: clicked,
            focus: focus_edge == Edge::Rising,
            blur: focus_edge == Edge::Falling,
        }
    }

    /// `true` when nothing should fire.
    #[inline]
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

// =============================================================================
// Thread-safe shared handle
// =============================================================================

/// Lock-free storage behind a [`ContainerHandle`].
///
/// `f32` values are stored as their IEEE-754 bit patterns in `AtomicU32`.
/// All value stores are `Relaxed`; the frame counter uses `Release` / `Acquire`
/// so a reader that observes frame *N* also observes at least frame *N*'s data.
#[derive(Debug)]
struct SharedState {
    outer_width: AtomicU32,
    content_width: AtomicU32,
    gutter: AtomicU32,
    viewport_breakpoint: AtomicU8,
    effective_max: AtomicU8,
    hovered: AtomicBool,
    active: AtomicBool,
    focused: AtomicBool,
    frames: AtomicU64,
}

impl SharedState {
    fn new() -> Self {
        Self {
            outer_width: AtomicU32::new(0.0_f32.to_bits()),
            content_width: AtomicU32::new(0.0_f32.to_bits()),
            gutter: AtomicU32::new(0.0_f32.to_bits()),
            viewport_breakpoint: AtomicU8::new(max_width_to_u8(MaxWidth::Xs)),
            effective_max: AtomicU8::new(max_width_to_u8(MaxWidth::Xs)),
            hovered: AtomicBool::new(false),
            active: AtomicBool::new(false),
            focused: AtomicBool::new(false),
            frames: AtomicU64::new(0),
        }
    }
}

/// A point-in-time copy of a container's published state.
///
/// Each field is individually atomic; the snapshot as a whole is *eventually
/// consistent* if a different thread is rendering the container at the same
/// moment (fields may come from adjacent frames). For UI code on the render
/// thread it is exact.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContainerSnapshot {
    /// Number of frames the container has published (0 before the first).
    pub frame: u64,
    /// Last outer width (content plus gutters).
    pub outer_width: f32,
    /// Last content width.
    pub content_width: f32,
    /// Last side gutter.
    pub gutter: f32,
    /// Breakpoint bucket of the last offered width.
    pub viewport_breakpoint: MaxWidth,
    /// Effective ceiling applied on the last frame.
    pub effective_max: MaxWidth,
    /// Pointer was over the container on the last frame.
    pub hovered: bool,
    /// A pointer button was held down on the container on the last frame.
    pub active: bool,
    /// The container had keyboard focus on the last frame.
    pub focused: bool,
}

/// A cheap, cloneable, `Send + Sync` window into a container's live state.
///
/// Obtain one with [`MudContainer::handle`]. Any thread (a background
/// worker, another panel, a test) can then read the container's most recent
/// layout without borrowing the container itself. The container only does the
/// atomic stores when a handle exists, so it is zero-cost otherwise.
///
/// # Example
/// ```ignore
/// let mut page = MudContainer::new().max_width(MaxWidth::Md);
/// let handle = page.handle();
///
/// // elsewhere, e.g. a sidebar that adapts to the page container:
/// if handle.snapshot().outer_width < 700.0 {
///     /* collapse the sidebar */
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ContainerHandle {
    inner: Arc<SharedState>,
}

impl Default for ContainerHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainerHandle {
    /// Creates a detached handle with zeroed state. [`MudContainer::handle`]
    /// is the usual way to get one that is wired to a container.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SharedState::new()),
        }
    }

    /// Reads everything at once. See [`ContainerSnapshot`] for the
    /// consistency guarantee.
    pub fn snapshot(&self) -> ContainerSnapshot {
        let s = &*self.inner;
        let frame = s.frames.load(Ordering::Acquire);
        ContainerSnapshot {
            frame,
            outer_width: f32::from_bits(s.outer_width.load(Ordering::Relaxed)),
            content_width: f32::from_bits(s.content_width.load(Ordering::Relaxed)),
            gutter: f32::from_bits(s.gutter.load(Ordering::Relaxed)),
            viewport_breakpoint: u8_to_max_width(s.viewport_breakpoint.load(Ordering::Relaxed)),
            effective_max: u8_to_max_width(s.effective_max.load(Ordering::Relaxed)),
            hovered: s.hovered.load(Ordering::Relaxed),
            active: s.active.load(Ordering::Relaxed),
            focused: s.focused.load(Ordering::Relaxed),
        }
    }

    /// Last outer width in logical pixels (`0.0` before the first frame).
    #[inline]
    pub fn outer_width(&self) -> f32 {
        f32::from_bits(self.inner.outer_width.load(Ordering::Relaxed))
    }

    /// Last content width in logical pixels (`0.0` before the first frame).
    #[inline]
    pub fn content_width(&self) -> f32 {
        f32::from_bits(self.inner.content_width.load(Ordering::Relaxed))
    }

    /// Breakpoint bucket of the last offered width.
    #[inline]
    pub fn viewport_breakpoint(&self) -> MaxWidth {
        u8_to_max_width(self.inner.viewport_breakpoint.load(Ordering::Relaxed))
    }

    /// `true` if the pointer was over the container on the last frame.
    #[inline]
    pub fn is_hovered(&self) -> bool {
        self.inner.hovered.load(Ordering::Relaxed)
    }

    /// `true` if the container had keyboard focus on the last frame.
    #[inline]
    pub fn is_focused(&self) -> bool {
        self.inner.focused.load(Ordering::Relaxed)
    }

    /// Number of frames published so far.
    #[inline]
    pub fn frame_count(&self) -> u64 {
        self.inner.frames.load(Ordering::Acquire)
    }

    /// `true` once the container has rendered at least once.
    #[inline]
    pub fn has_rendered(&self) -> bool {
        self.frame_count() > 0
    }

    /// Stores one frame's worth of state. Called by [`MudContainer::show`].
    fn publish(&self, layout: &HorizontalLayout, interaction: &InteractionState) {
        let s = &*self.inner;
        s.outer_width
            .store(layout.outer_width.to_bits(), Ordering::Relaxed);
        s.content_width
            .store(layout.content_width.to_bits(), Ordering::Relaxed);
        s.gutter.store(layout.gutter.to_bits(), Ordering::Relaxed);
        s.viewport_breakpoint
            .store(max_width_to_u8(layout.breakpoint), Ordering::Relaxed);
        s.effective_max
            .store(max_width_to_u8(layout.effective_max), Ordering::Relaxed);
        s.hovered.store(interaction.hovered, Ordering::Relaxed);
        s.active.store(interaction.active, Ordering::Relaxed);
        s.focused.store(interaction.focused, Ordering::Relaxed);
        s.frames.fetch_add(1, Ordering::Release);
    }
}

// =============================================================================
// Private helpers
// =============================================================================

/// Stable one-byte encoding of a breakpoint, used for the atomic handle.
#[inline]
fn max_width_to_u8(max_width: MaxWidth) -> u8 {
    match max_width {
        MaxWidth::Xs => 0,
        MaxWidth::Sm => 1,
        MaxWidth::Md => 2,
        MaxWidth::Lg => 3,
        MaxWidth::Xl => 4,
        MaxWidth::Xxl => 5,
        MaxWidth::False => 6,
    }
}

/// Inverse of [`max_width_to_u8`]; unknown values decode to `MaxWidth::False`.
#[inline]
fn u8_to_max_width(value: u8) -> MaxWidth {
    match value {
        0 => MaxWidth::Xs,
        1 => MaxWidth::Sm,
        2 => MaxWidth::Md,
        3 => MaxWidth::Lg,
        4 => MaxWidth::Xl,
        5 => MaxWidth::Xxl,
        _ => MaxWidth::False,
    }
}

/// Returns `v` if it is finite, otherwise `fallback`.
#[inline]
fn finite_or(v: f32, fallback: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
}

/// Converts a resolved gutter into the integer margin `egui::Margin` expects.
///
/// Uses `floor` (not `round`) so the margins can never add up to more than the
/// float gutters were budgeted for: with fractional gutters (tiny viewports)
/// rounding up could overflow the offered width by a pixel. Standard gutters
/// (16 / 24) are exact either way. Non-finite or non-positive gutters map to
/// `0`.
#[inline]
fn gutter_margin_px(gutter: f32) -> i8 {
    if gutter.is_finite() && gutter > 0.0 {
        gutter.floor().min(f32::from(i8::MAX)) as i8
    } else {
        0
    }
}

/// Builds the rect the container reserves in the parent layout: the full
/// available width, from the top of `avail` down to the content's bottom edge.
///
/// Every edge is sanitized. An infinite available width (horizontal scroll
/// area) falls back to the child's right edge, and a non-finite bottom falls
/// back to the top. Reserving an infinite or `NaN` rect would poison the
/// parent's measured size.
#[inline]
fn used_rect(avail: Rect, child_right: f32, child_bottom: f32) -> Rect {
    let right = finite_or(avail.max.x, finite_or(child_right, avail.min.x)).max(avail.min.x);
    let bottom = finite_or(child_bottom, avail.min.y).max(avail.min.y);
    Rect::from_min_max(avail.min, pos2(right, bottom))
}

/// Adds `g` to the left and right sides of `m`, leaving top/bottom unchanged.
#[inline]
fn add_side_gutter(m: Margin, g: i8) -> Margin {
    Margin {
        left: m.left.saturating_add(g),
        right: m.right.saturating_add(g),
        top: m.top,
        bottom: m.bottom,
    }
}

/// Moves `current` toward `target` by at most `step`, without overshooting.
/// Non-finite inputs are repaired: a bad `current` snaps to `target` and a bad
/// or negative `step` means "do not move".
#[inline]
fn approach(current: f32, target: f32, step: f32) -> f32 {
    let current = finite_or(current, target);
    let step = if step.is_finite() { step.max(0.0) } else { 0.0 };
    if current < target {
        (current + step).min(target)
    } else {
        (current - step).max(target)
    }
}

/// Multiplies the alpha of a frame's fill and stroke by `alpha`.
#[inline]
fn fade_frame(mut frame: Frame, alpha: f32) -> Frame {
    frame.fill = frame.fill.gamma_multiply(alpha);
    frame.stroke.color = frame.stroke.color.gamma_multiply(alpha);
    frame
}

/// `true` when the viewport breakpoint differs from the previous frame's (or
/// there was no previous frame).
#[inline]
fn breakpoint_changed(previous: Option<MaxWidth>, current: MaxWidth) -> bool {
    previous != Some(current)
}

/// Tiny non-cryptographic FNV-1a hasher used for style-cache keys.
///
/// Deterministic across runs and platforms (unlike the default `SipHash`
/// random keys), allocation-free, and a few instructions per byte.
struct Fnv1a(u64);

impl Fnv1a {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    #[inline]
    fn new() -> Self {
        Self(Self::OFFSET)
    }
}

impl Hasher for Fnv1a {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }
}

/// Fingerprints the handful of egui visuals that realistically change when a
/// theme is switched, so a memoized frame is invalidated alongside them.
fn visuals_fingerprint(v: &egui::Visuals) -> u64 {
    let mut h = Fnv1a::new();
    v.dark_mode.hash(&mut h);
    v.panel_fill.to_array().hash(&mut h);
    v.extreme_bg_color.to_array().hash(&mut h);
    v.selection.bg_fill.to_array().hash(&mut h);
    v.hyperlink_color.to_array().hash(&mut h);
    h.finish()
}

// =============================================================================
// Component
// =============================================================================

/// A responsive, centered, width-limited container wrapper.
///
/// Ports (and expands) MudBlazor's `MudContainer`. The hot path is designed to
/// do as little work as possible: layout math is pure `Copy` arithmetic, the
/// shared [`ContainerHandle`] is created lazily, the resolved `Frame` can be
/// memoized, and the container never senses more input than it needs.
///
/// # Click passthrough
///
/// By default the container's own reserved rect uses [`Sense::hover`], so
/// clicks and keyboard focus flow through to children. An interactive
/// `Sense::click` rect registered *after* the children (which is when
/// [`Ui::allocate_rect`] runs in `show`) would sit on top of everything
/// inside the container and block it. If you register `on_click`, `on_focus`
/// or `on_blur`, the container upgrades to `click` sense, and by design
/// becomes the click target. If you need a clickable container with typable
/// children, wrap the clickable part in its own `MudContainer` instead.
///
/// # MudBlazor API parity
///
/// See the module-level table: `MaxWidth`, `Fixed`, `Gutters`, `ChildContent`,
/// `Class`, `Style`, `Tag` and `UserAttributes` are all covered.
///
/// # Rust extensions beyond MudBlazor
///
/// * `fluid` (shorthand for `MaxWidth.False`), `disabled`, `elevation`
/// * Bootstrap + Tailwind style flavours with variant support
/// * Callbacks: `on_hover`, `on_unhover`, `on_click`, `on_focus`, `on_blur`,
///   `on_breakpoint_change`
/// * Accessibility: `id`, `aria_label`, `tooltip`, keyboard focus ring
/// * [`ContainerHandle`] thread-safe live state, [`LayoutSpec`], opt-in frame
///   memoization
#[must_use = "MudContainer does nothing until you call `.show(ui, |ui| ...)` or add it with `ui.add(..)`"]
pub struct MudContainer {
    /// The maximum width breakpoint ceiling (ignored if `fluid` is true or
    /// when `fixed` is true: fixed mode snaps to the current breakpoint).
    /// MudBlazor: `MaxWidth`.
    pub max_width: MaxWidth,

    /// If `true`, the container stretches to fill 100 % of the available
    /// width, ignoring the `max_width` breakpoint cap. Equivalent to
    /// MudBlazor's `MaxWidth.False`.
    pub fluid: bool,

    /// If `true`, the container "snaps" its max-width to the breakpoint that
    /// matches the offered width (MudBlazor `Fixed` semantics). Takes
    /// precedence over the explicit `max_width` when enabled.
    pub fixed: bool,

    /// If `true`, applies standard responsive side gutters (16 px or 24 px).
    /// MudBlazor: `Gutters`.
    pub gutters: bool,

    /// When `true` the container ignores pointer interaction, disables its
    /// children (egui's disabled styling) and fades its own frame.
    pub disabled: bool,

    /// Optional stable identifier used for accessibility and egui memory.
    pub id: Option<Id>,

    /// Optional ARIA label exposed to assistive technology through egui's
    /// accessibility tree. Falls back to the `"aria-label"` entry of
    /// [`Self::user_attributes`].
    pub aria_label: Option<String>,

    /// Optional visible hover tooltip (separate from the ARIA label).
    pub tooltip: Option<String>,

    /// Shared component state (CSS-like utility classes, elevation, ripple).
    /// MudBlazor: `Class`.
    ///
    /// Utility classes such as `["bg-surface", "p-4", "rounded-lg"]` are
    /// stored here. They are resolved by [`Self::flavor`] into the
    /// `egui::Frame` used to draw the container's content.
    pub base: ComponentBase,

    /// Which utility-class system to resolve [`Self::base`] against.
    pub flavor: StyleFlavor,

    /// Typed inline-style overrides applied after class resolution.
    /// MudBlazor: `Style`.
    pub style: StyleOverrides,

    /// Paint an animated keyboard-focus ring when the container is focusable
    /// (that is, when it has a click-class callback) and focused. Interactive
    /// containers reserve a constant 2-point stroke so the ring never shifts
    /// layout. Default `true`.
    pub focus_ring: bool,

    /// Memoize the class-resolved frame across frames. Default `false`.
    /// See [`Self::memoize_style`] for when this is safe.
    pub memoize_style: bool,

    /// Arbitrary user data attached to the container. MudBlazor: `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,

    /// Extra key/value attributes. MudBlazor: `UserAttributes`.
    pub user_attributes: BTreeMap<String, String>,

    /// Bootstrap resolver cache. Consulted when `flavor == Bootstrap`.
    pub bs_style: BsStyle,

    /// Tailwind resolver cache. Consulted when `flavor == Tailwind`.
    pub tw_style: TwStyle,

    /// Interaction / theme state fed to the Tailwind resolver and optional
    /// callbacks. Updated each frame from the previous frame's response.
    pub interaction: InteractionState,

    // ------------------------------------------------------------------
    // Optional event callbacks (zero-cost when None)
    // ------------------------------------------------------------------
    /// Called once when the pointer enters the container.
    pub on_hover: Option<Box<dyn FnMut() + Send + Sync>>,
    /// Called once when the pointer leaves the container.
    pub on_unhover: Option<Box<dyn FnMut() + Send + Sync>>,
    /// Called when the primary button is clicked on the container.
    pub on_click: Option<Box<dyn FnMut() + Send + Sync>>,
    /// Called when the container gains keyboard focus.
    pub on_focus: Option<Box<dyn FnMut() + Send + Sync>>,
    /// Called when the container loses keyboard focus.
    pub on_blur: Option<Box<dyn FnMut() + Send + Sync>>,
    /// Called with the new breakpoint whenever the offered width crosses a
    /// breakpoint boundary, and once on the first frame.
    pub on_breakpoint_change: Option<Box<dyn FnMut(MaxWidth) + Send + Sync>>,

    // Private, derived state.
    last_layout: Option<HorizontalLayout>,
    shared: Option<ContainerHandle>,
    frame_cache: Option<(u64, Frame)>,
    cache_epoch: u64,
    focus_anim: f32,
}

impl Default for MudContainer {
    /// Large breakpoint cap (`Lg`), fixed mode off, gutters enabled, empty
    /// base, Bootstrap flavour, focus ring on, memoization off, no callbacks.
    fn default() -> Self {
        Self {
            max_width: MaxWidth::Lg,
            fluid: false,
            fixed: false,
            gutters: true,
            disabled: false,
            id: None,
            aria_label: None,
            tooltip: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            style: StyleOverrides::default(),
            focus_ring: true,
            memoize_style: false,
            tag: None,
            user_attributes: BTreeMap::new(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            interaction: InteractionState::default(),
            on_hover: None,
            on_unhover: None,
            on_click: None,
            on_focus: None,
            on_blur: None,
            on_breakpoint_change: None,
            last_layout: None,
            shared: None,
            frame_cache: None,
            cache_epoch: 0,
            focus_anim: 0.0,
        }
    }
}

impl std::fmt::Debug for MudContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MudContainer")
            .field("max_width", &self.max_width)
            .field("fluid", &self.fluid)
            .field("fixed", &self.fixed)
            .field("gutters", &self.gutters)
            .field("disabled", &self.disabled)
            .field("flavor", &self.flavor)
            .field("style", &self.style)
            .field("focus_ring", &self.focus_ring)
            .field("memoize_style", &self.memoize_style)
            .field("interaction", &self.interaction)
            .field("last_layout", &self.last_layout)
            .finish_non_exhaustive()
    }
}

impl MudContainer {
    /// Creates a new `MudContainer` with default settings.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a container capped at the given breakpoint.
    ///
    /// # Example
    /// ```ignore
    /// let page = MudContainer::for_breakpoint(MaxWidth::Md);
    /// ```
    #[inline]
    pub fn for_breakpoint(max_width: MaxWidth) -> Self {
        Self::new().max_width(max_width)
    }

    /// Creates a container with no width ceiling (`MaxWidth.False`); gutters
    /// stay on, like MudBlazor.
    #[inline]
    pub fn full_width() -> Self {
        Self::new().max_width(MaxWidth::False)
    }

    /// Creates a container from a [`LayoutSpec`].
    #[inline]
    pub fn from_spec(spec: LayoutSpec) -> Self {
        let mut c = Self::new();
        c.max_width = spec.max_width;
        c.fluid = spec.fluid;
        c.fixed = spec.fixed;
        c.gutters = spec.gutters;
        c
    }

    // ------------------------------------------------------------------
    // Builder API: layout behaviour
    // ------------------------------------------------------------------

    /// Sets the breakpoint ceiling constraint (MudBlazor `MaxWidth`).
    ///
    /// Ignored when [`Self::fluid`] is true or when [`Self::fixed`] is true
    /// (fixed mode computes its own ceiling from the offered width).
    #[inline]
    pub fn max_width(mut self, max_width: MaxWidth) -> Self {
        self.max_width = max_width;
        self
    }

    /// Configures the container to be fluid (stretching to fill available
    /// width, ignoring any max-width ceiling).
    #[inline]
    pub fn fluid(mut self, fluid: bool) -> Self {
        self.fluid = fluid;
        self
    }

    /// Enables "fixed / snap-to-breakpoint" mode (MudBlazor `Fixed`).
    ///
    /// When true the container's max-width is set to the breakpoint that
    /// matches the offered width. Takes precedence over the explicit
    /// `max_width` value, but not over `fluid`.
    #[inline]
    pub fn fixed(mut self, fixed: bool) -> Self {
        self.fixed = fixed;
        self
    }

    /// Enables or disables responsive side gutters (MudBlazor `Gutters`).
    #[inline]
    pub fn gutters(mut self, gutters: bool) -> Self {
        self.gutters = gutters;
        self
    }

    /// Marks the container as disabled. Disabled containers sense hover only,
    /// fire no callbacks, disable their children and fade their own frame.
    #[inline]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self.interaction.disabled = disabled;
        self
    }

    // ------------------------------------------------------------------
    // Builder API: identity, accessibility and metadata
    // ------------------------------------------------------------------

    /// Assigns a stable `egui::Id` used for memory and accessibility.
    #[inline]
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets an ARIA label for assistive technologies. The label is published
    /// through egui's accessibility tree; it is *not* shown as a tooltip (use
    /// [`Self::tooltip`] for that).
    #[inline]
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// Sets a visible hover tooltip.
    #[inline]
    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    /// Attaches arbitrary user data (MudBlazor `Tag`). Replaces any previous
    /// tag. Read it back with [`Self::tag_ref`].
    #[inline]
    pub fn tag<T: Any + Send + Sync>(mut self, value: T) -> Self {
        self.tag = Some(Box::new(value));
        self
    }

    /// Returns the attached tag if it is of type `T`.
    ///
    /// # Example
    /// ```ignore
    /// let c = MudContainer::new().tag(42_u32);
    /// assert_eq!(c.tag_ref::<u32>(), Some(&42));
    /// assert_eq!(c.tag_ref::<String>(), None);
    /// ```
    #[inline]
    pub fn tag_ref<T: Any>(&self) -> Option<&T> {
        self.tag.as_deref().and_then(|t| t.downcast_ref::<T>())
    }

    /// Adds one extra attribute (MudBlazor `UserAttributes`). Existing keys
    /// are overwritten. The key `"aria-label"` doubles as an ARIA label
    /// fallback.
    #[inline]
    pub fn attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.user_attributes.insert(key.into(), value.into());
        self
    }

    /// Adds several attributes at once.
    pub fn attrs<K, V>(mut self, pairs: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        for (k, v) in pairs {
            self.user_attributes.insert(k.into(), v.into());
        }
        self
    }

    /// Looks up an extra attribute by key.
    #[inline]
    pub fn user_attribute(&self, key: &str) -> Option<&str> {
        self.user_attributes.get(key).map(String::as_str)
    }

    /// The ARIA label that will be published: the explicit
    /// [`Self::aria_label`] if set, else the `"aria-label"` attribute.
    #[inline]
    pub fn effective_aria_label(&self) -> Option<&str> {
        self.aria_label
            .as_deref()
            .or_else(|| self.user_attribute("aria-label"))
    }

    // ------------------------------------------------------------------
    // Builder API: styling
    // ------------------------------------------------------------------

    /// Selects the utility-class resolver.
    ///
    /// Defaults to [`StyleFlavor::Bootstrap`]. Switching to
    /// [`StyleFlavor::Tailwind`] enables `hover:`, `dark:` and `active:`
    /// classes to be resolved.
    #[inline]
    pub fn flavor(mut self, flavor: StyleFlavor) -> Self {
        self.flavor = flavor;
        self
    }

    /// Adds utility classes (e.g. `["bg-surface", "p-4", "rounded-lg"]`) to
    /// the underlying [`ComponentBase`] (MudBlazor `Class`).
    ///
    /// Each entry may itself contain several space-separated classes, and
    /// duplicates are ignored (see [`ComponentBase::classes`]).
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Adds one space-separated class string, exactly like MudBlazor's
    /// `Class="a b c"`.
    #[inline]
    pub fn class(self, classes: impl AsRef<str>) -> Self {
        self.classes([classes])
    }

    /// Convenience setter for elevation (0 to 24; larger values are clamped).
    #[inline]
    pub fn elevation(mut self, level: u8) -> Self {
        self.base.set_elevation(level.min(24));
        self
    }

    /// Sets all typed inline-style overrides at once (MudBlazor `Style`).
    #[inline]
    pub fn style(mut self, style: StyleOverrides) -> Self {
        self.style = style;
        self
    }

    /// Overrides the frame fill colour.
    #[inline]
    pub fn fill(mut self, fill: Color32) -> Self {
        self.style.fill = Some(fill);
        self
    }

    /// Overrides the frame border stroke.
    #[inline]
    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.style.stroke = Some(stroke);
        self
    }

    /// Overrides the frame inner padding (responsive gutters are still added
    /// on top).
    #[inline]
    pub fn padding(mut self, padding: Margin) -> Self {
        self.style.padding = Some(padding);
        self
    }

    /// Enables or disables the animated keyboard-focus ring.
    #[inline]
    pub fn focus_ring(mut self, enabled: bool) -> Self {
        self.focus_ring = enabled;
        self
    }

    /// Opts in to memoizing the class-resolved `Frame`.
    ///
    /// The cache key covers: flavour, classes, elevation, ripple flag, dark
    /// mode, a fingerprint of key egui visuals (panel fill, extreme
    /// background, selection fill, hyperlink colour), and, for Tailwind, the
    /// hover/active/dark variant bits. It does **not** observe `MudTheme`
    /// values that live outside egui's visuals; if you mutate the theme
    /// palette independently, call [`Self::invalidate_style_cache`].
    #[inline]
    pub fn memoize_style(mut self, enabled: bool) -> Self {
        self.memoize_style = enabled;
        self
    }

    // ------------------------------------------------------------------
    // Builder API: callbacks
    // ------------------------------------------------------------------

    /// Registers a callback invoked when the pointer enters the container.
    #[inline]
    pub fn on_hover(mut self, f: impl FnMut() + Send + Sync + 'static) -> Self {
        self.on_hover = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when the pointer leaves the container.
    #[inline]
    pub fn on_unhover(mut self, f: impl FnMut() + Send + Sync + 'static) -> Self {
        self.on_unhover = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked on primary-button click. Upgrades the
    /// container's sense to `click` (see the type-level docs).
    #[inline]
    pub fn on_click(mut self, f: impl FnMut() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when the container gains keyboard focus.
    #[inline]
    pub fn on_focus(mut self, f: impl FnMut() + Send + Sync + 'static) -> Self {
        self.on_focus = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when the container loses keyboard focus.
    #[inline]
    pub fn on_blur(mut self, f: impl FnMut() + Send + Sync + 'static) -> Self {
        self.on_blur = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked with the new [`MaxWidth`] bucket whenever
    /// the offered width crosses a breakpoint boundary (and once on the first
    /// frame). Does not change the container's sense.
    ///
    /// # Example
    /// ```ignore
    /// let c = MudContainer::new().on_breakpoint_change(|bp| {
    ///     println!("now at {}", MudContainerMetrics::label(bp));
    /// });
    /// ```
    #[inline]
    pub fn on_breakpoint_change(
        mut self,
        f: impl FnMut(MaxWidth) + Send + Sync + 'static,
    ) -> Self {
        self.on_breakpoint_change = Some(Box::new(f));
        self
    }

    // ------------------------------------------------------------------
    // State & introspection
    // ------------------------------------------------------------------

    /// The layout-affecting settings as a `Copy` [`LayoutSpec`].
    #[inline]
    pub fn layout_spec(&self) -> LayoutSpec {
        LayoutSpec {
            max_width: self.max_width,
            fluid: self.fluid,
            fixed: self.fixed,
            gutters: self.gutters,
        }
    }

    /// Computes the layout this container *would* produce for an offered
    /// width, without any egui state. Ideal for previews, tests and
    /// responsive-design tooling.
    #[inline]
    pub fn preview_layout(&self, offered_width: f32) -> HorizontalLayout {
        self.layout_spec().resolve(offered_width)
    }

    /// The layout computed on the most recent frame, if any.
    #[inline]
    pub fn last_layout(&self) -> Option<HorizontalLayout> {
        self.last_layout
    }

    /// Returns a thread-safe live-state handle, creating it on first use.
    /// Every call returns a clone of the same underlying state.
    pub fn handle(&mut self) -> ContainerHandle {
        self.shared.get_or_insert_with(ContainerHandle::new).clone()
    }

    /// Discards the memoized frame and bumps the cache epoch so the next
    /// frame re-resolves classes against the current theme.
    pub fn invalidate_style_cache(&mut self) {
        self.frame_cache = None;
        self.cache_epoch = self.cache_epoch.wrapping_add(1);
    }

    /// Resets interaction flags and the focus-ring animation to their idle
    /// state (keeps `disabled` in sync with [`Self::disabled`]).
    pub fn reset_interaction(&mut self) {
        self.interaction = InteractionState {
            disabled: self.disabled,
            ..InteractionState::default()
        };
        self.focus_anim = 0.0;
    }

    // ------------------------------------------------------------------
    // Core rendering
    // ------------------------------------------------------------------

    /// Draws the container, centres its content horizontally, and runs the
    /// supplied closure (MudBlazor `ChildContent`).
    ///
    /// See [`Self::show_with_layout`] for a variant whose closure also
    /// receives the resolved [`HorizontalLayout`].
    ///
    /// # Layout behaviour
    ///
    /// The container reserves the full available width in the parent layout
    /// (so subsequent widgets flow below it), while the child content is
    /// centred inside the max-width bounds.
    ///
    /// # Example
    /// ```ignore
    /// MudContainer::new().max_width(MaxWidth::Sm).show(ui, |ui| {
    ///     ui.heading("Hello");
    /// });
    /// ```
    pub fn show<R>(
        &mut self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.show_with_layout(ui, |ui, _| add_contents(ui))
    }

    /// Like [`Self::show`], but the closure also receives the resolved
    /// [`HorizontalLayout`], so children can adapt to the real content width
    /// or breakpoint without a second measurement.
    ///
    /// # Styling
    ///
    /// Classes on [`Self::base`] are resolved into an [`Frame`] via the
    /// active flavour, then [`StyleOverrides`], the disabled fade and the
    /// focus ring are applied, and finally the responsive gutters are added on
    /// top of the resulting `inner_margin`.
    ///
    /// For [`StyleFlavor::Tailwind`], [`Self::interaction`] supplies the
    /// interaction / theme flags from the **previous** frame (egui's
    /// immediate model cannot know this frame's hover state before drawing).
    /// This method refreshes the interaction state from the current frame's
    /// response before returning.
    ///
    /// Centering is performed by offsetting the child rect's X coordinate
    /// rather than with `ui.add_space`, which would insert *vertical* space
    /// in a top-down parent.
    ///
    /// # Example
    /// ```ignore
    /// let mut c = MudContainer::new().fixed(true);
    /// c.show_with_layout(ui, |ui, layout| {
    ///     ui.label(format!("content is {:.0}px wide", layout.content_width));
    /// });
    /// ```
    pub fn show_with_layout<R>(
        &mut self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui, &HorizontalLayout) -> R,
    ) -> InnerResponse<R> {
        self.interaction.disabled = self.disabled;

        let avail = ui.available_rect_before_wrap();
        let offered = avail.width();

        // 1. Pure math: widths, gutters and centering offset.
        let layout = self.layout_spec().resolve(offered);
        let offset = layout.offset_in(offered);

        // 2. Breakpoint transition callback (also fires on the first frame).
        if breakpoint_changed(self.last_layout.map(|l| l.breakpoint), layout.breakpoint) {
            if let Some(ref mut f) = self.on_breakpoint_change {
                f(layout.breakpoint);
            }
        }
        self.last_layout = Some(layout);

        // 3. Child region, horizontally offset to centre it.
        let child_rect = Rect::from_min_size(
            pos2(avail.min.x + offset, avail.min.y),
            vec2(layout.outer_width, avail.height()),
        );

        let mut child_builder = UiBuilder::new()
            .max_rect(child_rect)
            .layout(Layout::top_down(Align::Min));

        if let Some(id) = self.id {
            child_builder = child_builder.id_salt(id);
        }

        let mut child = ui.new_child(child_builder);

        // 4. Decide how much pointer input the container itself needs.
        //    * `disabled`            -> hover only
        //    * no click-class cbs    -> hover only (children keep the pointer)
        //    * click-class cbs       -> click (container wants the pointer)
        let needs_click =
            self.on_click.is_some() || self.on_focus.is_some() || self.on_blur.is_some();
        let interactive = !self.disabled && needs_click;
        let sense = if interactive {
            Sense::click()
        } else {
            Sense::hover()
        };

        // 5. Resolve the frame: classes -> overrides -> fade -> ring -> gutters.
        let mut frame = self.resolve_frame(ui);
        frame = self.style.apply(frame);
        if self.disabled {
            frame = fade_frame(frame, DISABLED_FRAME_ALPHA);
        }
        if self.focus_ring && interactive {
            let ring_target = self.interaction.focused;
            let ring_t = self.step_focus_anim(ui, ring_target);
            let base_stroke = frame.stroke;
            let idle = if base_stroke.width > 0.0 {
                base_stroke.color
            } else {
                Color32::TRANSPARENT
            };
            let color = if ring_t > 0.0 {
                self.focus_ring_color(ui).gamma_multiply(ring_t)
            } else {
                idle
            };
            frame.stroke = Stroke::new(FOCUS_RING_WIDTH.max(base_stroke.width), color);
        } else {
            self.focus_anim = 0.0;
        }
        let g = gutter_margin_px(layout.gutter);
        frame.inner_margin = add_side_gutter(frame.inner_margin, g);

        let content_width = layout.content_width;
        let disabled = self.disabled;
        let inner = frame.show(&mut child, |ui| {
            // Pin the content column; skipped when unbounded (fluid inside an
            // infinite-width parent) because an infinite width is meaningless.
            if content_width.is_finite() {
                ui.set_width(content_width);
            }
            if disabled {
                ui.disable();
            }
            add_contents(ui, &layout)
        });

        // 6. Advance the parent's cursor past the full-width row block, with
        //    every edge sanitized (see `used_rect`).
        let used = used_rect(avail, inner.response.rect.max.x, inner.response.rect.max.y);
        let response = ui.allocate_rect(used, sense);

        // 7. Accessibility: publish the ARIA label through egui's a11y tree.
        let enabled = !self.disabled;
        if let Some(label) = self.effective_aria_label() {
            response.widget_info(|| {
                WidgetInfo::labeled(egui::accesskit::Role::GenericContainer, enabled, label)
            });
        }

        // 8. Optional visible tooltip.
        let response = match self.tooltip.as_deref() {
            Some(text) => response.on_hover_text(text),
            None => response,
        };

        // 9. Fire callbacks from state edges, then record this frame's state.
        self.dispatch_events(&response);
        self.interaction.hovered = response.hovered();
        self.interaction.active = response.is_pointer_button_down_on();
        self.interaction.focused = response.has_focus();
        self.interaction.dark = ui.visuals().dark_mode;
        self.interaction.disabled = self.disabled;

        // 10. Publish to any live handle (lock-free; skipped when none).
        if let Some(handle) = self.shared.as_ref() {
            handle.publish(&layout, &self.interaction);
        }

        InnerResponse::new(inner.inner, response)
    }

    /// Resolves the active flavour's classes into an `egui::Frame`, using the
    /// memoized copy when [`Self::memoize_style`] is on and the key matches.
    fn resolve_frame(&mut self, ui: &Ui) -> Frame {
        if !self.memoize_style {
            return self.resolve_frame_uncached(ui);
        }

        let key = self.style_key(ui.visuals().dark_mode, visuals_fingerprint(ui.visuals()));
        if let Some((cached_key, cached_frame)) = &self.frame_cache {
            if *cached_key == key {
                return cached_frame.clone();
            }
        }
        let frame = self.resolve_frame_uncached(ui);
        self.frame_cache = Some((key, frame.clone()));
        frame
    }

    /// Resolves the frame from scratch.
    ///
    /// Uses [`MudTheme::fetch`] to read the current palette. The lookup
    /// closure maps keys like `"MudPalettePrimary"` (emitted by the BS and TW
    /// parsers) to actual palette colours via the theme's resolve path.
    fn resolve_frame_uncached(&mut self, ui: &Ui) -> Frame {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();

        // Shared colour lookup used by both resolvers.
        let lookup = |key: &str| -> Option<Color32> { palette.resolve(key).map(Color32::from) };

        match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => {
                let state = self.interaction.to_tw_state();
                self.tw_style
                    .build_frame(&self.base, state, REM_SIZE_PT, &lookup)
            }
        }
    }

    /// Computes the memoization key for the class-resolved frame.
    ///
    /// Bootstrap has no variants, so interaction state is deliberately
    /// excluded; Tailwind includes the hover/active/dark bits it consumes.
    fn style_key(&self, dark: bool, visuals_fp: u64) -> u64 {
        let mut h = Fnv1a::new();
        (self.flavor as u8).hash(&mut h);
        dark.hash(&mut h);
        visuals_fp.hash(&mut h);
        self.cache_epoch.hash(&mut h);
        self.base.elevation_value().hash(&mut h);
        self.base.ripple_enabled().hash(&mut h);
        if self.flavor == StyleFlavor::Tailwind {
            let state = self.interaction.to_tw_state();
            state.hovered.hash(&mut h);
            state.active.hash(&mut h);
            state.dark.hash(&mut h);
        }
        for class in self.base.class_names().iter() {
            class.hash(&mut h);
        }
        h.finish()
    }

    /// Advances the focus-ring fade toward `target` and requests a repaint
    /// while it is still animating. Returns the ring's current opacity (0..=1).
    fn step_focus_anim(&mut self, ui: &Ui, target: bool) -> f32 {
        let dt = ui.input(|i| i.stable_dt).clamp(0.0, 0.1);
        let goal = if target { 1.0 } else { 0.0 };
        self.focus_anim = approach(self.focus_anim, goal, dt / FOCUS_RING_SECONDS);
        if (self.focus_anim - goal).abs() > f32::EPSILON {
            ui.ctx().request_repaint();
        }
        self.focus_anim
    }

    /// Colour of the focus ring: the palette's primary colour, falling back to
    /// egui's selection stroke colour.
    fn focus_ring_color(&self, ui: &Ui) -> Color32 {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let resolved = palette.resolve(PALETTE_PRIMARY_KEY).map(Color32::from);
        resolved.unwrap_or(ui.visuals().selection.stroke.color)
    }

    /// Plans and fires callbacks from the latest response.
    fn dispatch_events(&mut self, response: &Response) {
        let plan = EventPlan::compute(
            self.interaction.hovered,
            self.interaction.focused,
            response.hovered(),
            response.has_focus(),
            response.clicked(),
            self.disabled,
        );
        self.fire(plan);
    }

    /// Executes an [`EventPlan`], invoking each registered callback at most
    /// once. Exposed to the module's tests so firing is verified without egui.
    fn fire(&mut self, plan: EventPlan) {
        if plan.hover {
            if let Some(ref mut f) = self.on_hover {
                f();
            }
        }
        if plan.unhover {
            if let Some(ref mut f) = self.on_unhover {
                f();
            }
        }
        if plan.click {
            if let Some(ref mut f) = self.on_click {
                f();
            }
        }
        if plan.focus {
            if let Some(ref mut f) = self.on_focus {
                f();
            }
        }
        if plan.blur {
            if let Some(ref mut f) = self.on_blur {
                f();
            }
        }
    }
}

// Allows `MudContainer` to be used with standard `egui` `ui.add()` syntax.
impl Widget for MudContainer {
    /// Adds an *empty* container (useful only as a spacer-like block);
    /// use [`MudContainer::show`] to place content inside.
    ///
    /// The style caches are per-instance, so `ui.add(MudContainer::new())`
    /// re-resolves classes each frame. Store the container on a component
    /// field to benefit from caching and the focus-ring animation across
    /// frames.
    fn ui(mut self, ui: &mut Ui) -> Response {
        self.show(ui, |_| {}).response
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Mutex;

    type M = MudContainerMetrics;

    /// Every `MaxWidth` variant, freshly constructed so no `Copy`/`Clone`
    /// assumption is needed.
    fn all_max_widths() -> [MaxWidth; 7] {
        [
            MaxWidth::Xs,
            MaxWidth::Sm,
            MaxWidth::Md,
            MaxWidth::Lg,
            MaxWidth::Xl,
            MaxWidth::Xxl,
            MaxWidth::False,
        ]
    }

    /// A callback that increments a shared counter.
    fn counter() -> (Arc<AtomicUsize>, impl FnMut() + Send + Sync + 'static) {
        let n = Arc::new(AtomicUsize::new(0));
        let n2 = Arc::clone(&n);
        (n, move || {
            n2.fetch_add(1, Ordering::SeqCst);
        })
    }

    /// Runs `f` inside a real egui pass whose parent `Ui` is limited to
    /// `width` points, so `show` is exercised without a window or backend.
    fn with_ui<R>(width: f32, f: impl FnOnce(&mut Ui) -> R) -> R {
        let ctx = egui::Context::default();
        let mut f = Some(f);
        let mut out: Option<R> = None;

        // Use `run_ui` which yields a `&mut Ui` for the top-level viewport
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {

            // Pass the `ui` instead of `&ctx`, and use `show_inside`
            egui::CentralPanel::default().show(ui, |ui| {
                ui.set_max_width(width);
                if let Some(f) = f.take() {
                    out = Some(f(ui));
                }
            });

        });

        out.expect("egui ran the test closure")
    }

    // ---- breakpoints & gutters ---------------------------------------------

    #[test]
    fn to_pixels_table() {
        assert_eq!(M::to_pixels(MaxWidth::Xs), 444.0);
        assert_eq!(M::to_pixels(MaxWidth::Sm), 600.0);
        assert_eq!(M::to_pixels(MaxWidth::Md), 960.0);
        assert_eq!(M::to_pixels(MaxWidth::Lg), 1280.0);
        assert_eq!(M::to_pixels(MaxWidth::Xl), 1920.0);
        assert_eq!(M::to_pixels(MaxWidth::Xxl), 2560.0);
        assert!(M::to_pixels(MaxWidth::False).is_infinite());
    }

    #[test]
    fn to_pixels_increases_with_breakpoint() {
        let px: Vec<f32> = all_max_widths().into_iter().map(M::to_pixels).collect();
        for pair in px.windows(2) {
            assert!(pair[0] < pair[1]);
        }
    }

    #[test]
    fn labels_are_distinct_and_lowercase() {
        let labels: Vec<&str> = all_max_widths().into_iter().map(M::label).collect();
        assert_eq!(labels, ["xs", "sm", "md", "lg", "xl", "xxl", "false"]);
    }

    #[test]
    fn gutter_for_thresholds() {
        assert_eq!(M::gutter_for(599.9, true), 16.0);
        assert_eq!(M::gutter_for(600.0, true), 24.0);
        assert_eq!(M::gutter_for(5000.0, true), 24.0);
        assert_eq!(M::gutter_for(0.0, true), 16.0);
        assert_eq!(M::gutter_for(100.0, false), 0.0);
        assert_eq!(M::gutter_for(f32::INFINITY, true), 24.0);
    }

    #[test]
    fn snap_breakpoint_table() {
        assert_eq!(M::snap_breakpoint(100.0), MaxWidth::Xs);
        assert_eq!(M::snap_breakpoint(599.0), MaxWidth::Xs);
        assert_eq!(M::snap_breakpoint(600.0), MaxWidth::Sm);
        assert_eq!(M::snap_breakpoint(959.0), MaxWidth::Sm);
        assert_eq!(M::snap_breakpoint(960.0), MaxWidth::Md);
        assert_eq!(M::snap_breakpoint(1279.0), MaxWidth::Md);
        assert_eq!(M::snap_breakpoint(1280.0), MaxWidth::Lg);
        assert_eq!(M::snap_breakpoint(1919.0), MaxWidth::Lg);
        assert_eq!(M::snap_breakpoint(1920.0), MaxWidth::Xl);
        assert_eq!(M::snap_breakpoint(2559.0), MaxWidth::Xl);
        assert_eq!(M::snap_breakpoint(2560.0), MaxWidth::Xxl);
        assert_eq!(M::snap_breakpoint(10_000.0), MaxWidth::Xxl);
        assert_eq!(M::snap_breakpoint(f32::INFINITY), MaxWidth::Xs); // non-finite -> Xs
        assert_eq!(M::snap_breakpoint(f32::NAN), MaxWidth::Xs);
        assert_eq!(M::snap_breakpoint(-5.0), MaxWidth::Xs);
    }

    // ---- resolve -------------------------------------------------------------

    #[test]
    fn gutters_scale_with_width() {
        let l = M::resolve(500.0, true, false, false, MaxWidth::Lg);
        assert_eq!((l.gutter, l.content_width), (16.0, 468.0));

        let l = M::resolve(800.0, true, false, false, MaxWidth::Lg);
        assert_eq!((l.gutter, l.content_width), (24.0, 752.0));
    }

    #[test]
    fn no_gutters() {
        let l = M::resolve(800.0, false, false, false, MaxWidth::Lg);
        assert_eq!(
            (l.gutter, l.content_width, l.outer_width),
            (0.0, 800.0, 800.0)
        );
    }

    #[test]
    fn max_width_caps_outer_width() {
        let l = M::resolve(2000.0, true, false, false, MaxWidth::Lg);
        assert_eq!(
            (l.gutter, l.content_width, l.outer_width),
            (24.0, 1232.0, 1280.0)
        );
        assert_eq!(l.effective_max, MaxWidth::Lg);
    }

    #[test]
    fn fluid_removes_cap() {
        let l = M::resolve(2000.0, true, true, false, MaxWidth::Lg);
        assert_eq!((l.content_width, l.outer_width), (1952.0, 2000.0));
        assert_eq!(l.effective_max, MaxWidth::False);
    }

    #[test]
    fn max_width_false_removes_cap() {
        let l = M::resolve(5000.0, true, false, false, MaxWidth::False);
        assert_eq!(l.outer_width, 5000.0);
        assert_eq!(l.effective_max, MaxWidth::False);
    }

    #[test]
    fn xs_is_not_zero() {
        let l = M::resolve(1000.0, true, false, false, MaxWidth::Xs);
        assert_eq!(l.outer_width, 444.0);
        assert_eq!(l.content_width, 396.0);
    }

    #[test]
    fn offer_exactly_at_cap() {
        let l = M::resolve(1280.0, true, false, false, MaxWidth::Lg);
        assert_eq!((l.content_width, l.outer_width), (1232.0, 1280.0));
    }

    #[test]
    fn cap_smaller_than_gutters_yields_empty_content() {
        let l = M::resolve(1000.0, true, false, false, MaxWidth::Xs);
        assert!(l.content_width >= 0.0);
        let l = M::resolve(30.0, true, false, false, MaxWidth::Xs);
        assert_eq!(l.content_width, 0.0);
        assert_eq!(l.outer_width, 30.0);
    }

    #[test]
    fn infinite_offer_uses_cap() {
        let l = M::resolve(f32::INFINITY, true, false, false, MaxWidth::Lg);
        assert_eq!((l.content_width, l.outer_width), (1232.0, 1280.0));
    }

    #[test]
    fn infinite_offer_fluid_stays_unbounded() {
        let l = M::resolve(f32::INFINITY, true, true, false, MaxWidth::Lg);
        assert!(l.content_width.is_infinite());
        assert!(l.outer_width.is_infinite());
        assert!(!l.is_bounded());
    }

    #[test]
    fn infinite_offer_with_max_width_false_is_unbounded() {
        let l = M::resolve(f32::INFINITY, false, false, false, MaxWidth::False);
        assert!(l.content_width.is_infinite());
    }

    #[test]
    fn tiny_or_invalid_widths_do_not_overflow() {
        let l = M::resolve(10.0, true, false, false, MaxWidth::Lg);
        assert_eq!((l.gutter, l.content_width, l.outer_width), (5.0, 0.0, 10.0));

        let l = M::resolve(f32::NAN, true, false, false, MaxWidth::Lg);
        assert_eq!((l.content_width, l.outer_width), (0.0, 0.0));

        let l = M::resolve(-50.0, true, false, false, MaxWidth::Lg);
        assert_eq!(l.outer_width, 0.0);

        let l = M::resolve(f32::NEG_INFINITY, true, false, false, MaxWidth::Lg);
        assert_eq!(l.outer_width, 0.0);
    }

    #[test]
    fn fixed_mode_snaps_to_breakpoint() {
        // Offered 900 -> snaps to Sm (600)
        let l = M::resolve(900.0, true, false, true, MaxWidth::Lg);
        assert_eq!(l.effective_max, MaxWidth::Sm);
        assert_eq!(l.outer_width, 600.0);

        // Offered 1500 -> snaps to Lg (1280)
        let l = M::resolve(1500.0, true, false, true, MaxWidth::Xs);
        assert_eq!(l.effective_max, MaxWidth::Lg);
        assert_eq!(l.outer_width, 1280.0);

        // Fluid takes precedence over fixed
        let l = M::resolve(1500.0, true, true, true, MaxWidth::Lg);
        assert_eq!(l.effective_max, MaxWidth::False);
        assert_eq!(l.outer_width, 1500.0);
    }

    #[test]
    fn layout_reports_viewport_breakpoint() {
        assert_eq!(
            M::resolve(500.0, true, false, false, MaxWidth::Lg).breakpoint,
            MaxWidth::Xs
        );
        assert_eq!(
            M::resolve(900.0, true, false, false, MaxWidth::Lg).breakpoint,
            MaxWidth::Sm
        );
        assert_eq!(
            M::resolve(1500.0, true, true, false, MaxWidth::Lg).breakpoint,
            MaxWidth::Lg
        );
        // In fixed mode the effective ceiling *is* the breakpoint.
        let l = M::resolve(1500.0, true, false, true, MaxWidth::Xs);
        assert_eq!(l.breakpoint, l.effective_max);
    }

    #[test]
    fn resolve_invariants_hold_across_a_sweep() {
        let offers = [
            0.0, 1.0, 10.0, 11.0, 31.0, 32.0, 33.0, 443.0, 444.0, 445.0, 599.0, 600.0, 601.0,
            959.0, 960.0, 1279.0, 1280.0, 1281.0, 1920.0, 2560.0, 5000.0, 1e6,
        ];
        for off in offers {
            for gutters in [true, false] {
                for fluid in [true, false] {
                    for fixed in [true, false] {
                        for mw in all_max_widths() {
                            let l = M::resolve(off, gutters, fluid, fixed, mw);
                            assert!(l.gutter >= 0.0 && l.gutter.is_finite());
                            assert!(l.content_width >= 0.0);
                            if l.content_width.is_finite() {
                                assert!(l.outer_width <= off + 1e-3);
                            }
                            assert!(
                                l.gutter * 2.0 <= l.outer_width + 1e-3
                                    || !l.outer_width.is_finite()
                            );
                            if fluid {
                                assert!(
                                    (l.outer_width - off).abs() < 1e-3 || !off.is_finite()
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // ---- layout spec ---------------------------------------------------------

    #[test]
    fn layout_spec_default_matches_mudblazor_defaults() {
        let s = LayoutSpec::default();
        assert_eq!(s.max_width, MaxWidth::Lg);
        assert!(!s.fluid && !s.fixed && s.gutters);
    }

    #[test]
    fn layout_spec_resolve_equals_metrics_resolve() {
        let s = LayoutSpec {
            max_width: MaxWidth::Md,
            fluid: false,
            fixed: false,
            gutters: true,
        };
        assert_eq!(
            s.resolve(1500.0),
            M::resolve(1500.0, true, false, false, MaxWidth::Md)
        );
        assert_eq!(s.resolve(1500.0).outer_width, 960.0);
    }

    #[test]
    fn horizontal_layout_helpers() {
        let l = M::resolve(2000.0, true, false, false, MaxWidth::Lg);
        assert_eq!(l.gutters_total(), 48.0);
        assert!(l.is_bounded());
        assert_eq!(l.offset_in(2000.0), 360.0);
    }

    // ---- centering -----------------------------------------------------------

    #[test]
    fn centering_offset_math() {
        assert_eq!(M::centering_offset(2000.0, 1280.0), 360.0);
        assert_eq!(M::centering_offset(500.0, 500.0), 0.0);
        assert_eq!(M::centering_offset(f32::INFINITY, 1280.0), 0.0);
        assert_eq!(M::centering_offset(1280.0, f32::INFINITY), 0.0);
        assert_eq!(M::centering_offset(f32::NAN, 100.0), 0.0);
    }

    #[test]
    fn centering_offset_is_never_negative() {
        assert_eq!(M::centering_offset(100.0, 500.0), 0.0);
    }

    // ---- private helpers -----------------------------------------------------

    #[test]
    fn finite_or_falls_back() {
        assert_eq!(finite_or(3.0, 9.0), 3.0);
        assert_eq!(finite_or(f32::NAN, 9.0), 9.0);
        assert_eq!(finite_or(f32::INFINITY, 9.0), 9.0);
        assert_eq!(finite_or(f32::NEG_INFINITY, 9.0), 9.0);
    }

    #[test]
    fn gutter_margin_px_standard_values_are_exact() {
        assert_eq!(gutter_margin_px(16.0), 16);
        assert_eq!(gutter_margin_px(24.0), 24);
        assert_eq!(gutter_margin_px(0.0), 0);
    }

    #[test]
    fn gutter_margin_px_floors_fractions_so_it_cannot_overflow() {
        assert_eq!(gutter_margin_px(5.5), 5);
        assert_eq!(gutter_margin_px(5.99), 5);
    }

    #[test]
    fn gutter_margin_px_repairs_bad_values() {
        assert_eq!(gutter_margin_px(-3.0), 0);
        assert_eq!(gutter_margin_px(f32::NAN), 0);
        assert_eq!(gutter_margin_px(f32::INFINITY), 0);
        assert_eq!(gutter_margin_px(1000.0), i8::MAX);
    }

    #[test]
    fn add_side_gutter_only_touches_left_right() {
        let m = Margin {
            left: 4,
            right: 6,
            top: 8,
            bottom: 10,
        };
        let m = add_side_gutter(m, 3);
        assert_eq!(
            m,
            Margin {
                left: 7,
                right: 9,
                top: 8,
                bottom: 10
            }
        );
    }

    #[test]
    fn add_side_gutter_saturates_at_i8_bounds() {
        let m = Margin {
            left: i8::MAX,
            right: i8::MAX,
            top: 0,
            bottom: 0,
        };
        let m = add_side_gutter(m, 5);
        assert_eq!(m.left, i8::MAX);
        assert_eq!(m.right, i8::MAX);
    }

    #[test]
    fn used_rect_spans_full_width_down_to_content() {
        let avail = Rect::from_min_max(pos2(10.0, 20.0), pos2(510.0, 920.0));
        let r = used_rect(avail, 400.0, 300.0);
        assert_eq!(r.min, pos2(10.0, 20.0));
        assert_eq!(r.max, pos2(510.0, 300.0));
    }

    #[test]
    fn used_rect_with_infinite_width_uses_child_right() {
        let avail = Rect::from_min_max(pos2(0.0, 0.0), pos2(f32::INFINITY, 100.0));
        let r = used_rect(avail, 1280.0, 50.0);
        assert_eq!(r.max.x, 1280.0);
    }

    #[test]
    fn used_rect_never_yields_non_finite_or_inverted_rects() {
        let avail = Rect::from_min_max(pos2(5.0, 5.0), pos2(f32::INFINITY, f32::INFINITY));
        let r = used_rect(avail, f32::INFINITY, f32::INFINITY);
        assert!(r.max.x.is_finite() && r.max.y.is_finite());
        assert!(r.max.x >= r.min.x && r.max.y >= r.min.y);

        let r = used_rect(avail, f32::NAN, f32::NAN);
        assert_eq!(r.max, pos2(5.0, 5.0));

        let avail = Rect::from_min_max(pos2(0.0, 100.0), pos2(200.0, 900.0));
        let r = used_rect(avail, 200.0, 40.0);
        assert_eq!(r.max.y, 100.0);
    }

    #[test]
    fn approach_moves_without_overshooting() {
        assert_eq!(approach(0.0, 1.0, 0.25), 0.25);
        assert_eq!(approach(0.9, 1.0, 0.25), 1.0);
        assert_eq!(approach(1.0, 0.0, 0.25), 0.75);
        assert_eq!(approach(0.1, 0.0, 0.25), 0.0);
        assert_eq!(approach(0.5, 0.5, 0.25), 0.5);
    }

    #[test]
    fn approach_repairs_non_finite_inputs() {
        assert_eq!(approach(f32::NAN, 1.0, 0.1), 1.0);
        assert_eq!(approach(0.3, 1.0, f32::NAN), 0.3);
        assert_eq!(approach(0.3, 1.0, -1.0), 0.3);
        assert_eq!(approach(0.3, 1.0, f32::INFINITY), 0.3);
    }

    #[test]
    fn fade_frame_reduces_alpha_and_keeps_stroke_width() {
        let mut frame = Frame::default();
        frame.fill = Color32::from_rgb(200, 100, 50);
        frame.stroke = Stroke::new(2.0, Color32::WHITE);
        let faded = fade_frame(frame, 0.5);
        assert!(faded.fill.a() < 255);
        assert!(faded.stroke.color.a() < 255);
        assert_eq!(faded.stroke.width, 2.0);
    }

    #[test]
    fn breakpoint_changed_detects_first_frame_and_transitions() {
        assert!(breakpoint_changed(None, MaxWidth::Xs));
        assert!(!breakpoint_changed(Some(MaxWidth::Md), MaxWidth::Md));
        assert!(breakpoint_changed(Some(MaxWidth::Md), MaxWidth::Lg));
    }

    #[test]
    fn max_width_u8_round_trips() {
        for mw in all_max_widths() {
            assert_eq!(u8_to_max_width(max_width_to_u8(mw)), mw);
        }
        assert_eq!(u8_to_max_width(200), MaxWidth::False);
    }

    #[test]
    fn fnv1a_is_deterministic_and_discriminating() {
        let hash = |s: &str| {
            let mut h = Fnv1a::new();
            s.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash("p-4"), hash("p-4"));
        assert_ne!(hash("p-4"), hash("p-5"));
        assert_ne!(Fnv1a::new().finish(), hash("p-4"));
    }

    #[test]
    fn visuals_fingerprint_distinguishes_light_and_dark() {
        let dark = visuals_fingerprint(&egui::Visuals::dark());
        let light = visuals_fingerprint(&egui::Visuals::light());
        assert_ne!(dark, light);
        assert_eq!(dark, visuals_fingerprint(&egui::Visuals::dark()));
    }

    // ---- edges & event planning ---------------------------------------------

    #[test]
    fn edge_detect_table() {
        assert_eq!(Edge::detect(false, false), Edge::Steady);
        assert_eq!(Edge::detect(true, true), Edge::Steady);
        assert_eq!(Edge::detect(false, true), Edge::Rising);
        assert_eq!(Edge::detect(true, false), Edge::Falling);
    }

    #[test]
    fn plan_fires_hover_and_unhover_on_edges_only() {
        let p = EventPlan::compute(false, false, true, false, false, false);
        assert_eq!(
            p,
            EventPlan {
                hover: true,
                ..Default::default()
            }
        );
        let p = EventPlan::compute(true, false, true, false, false, false);
        assert!(p.is_empty());
        let p = EventPlan::compute(true, false, false, false, false, false);
        assert_eq!(
            p,
            EventPlan {
                unhover: true,
                ..Default::default()
            }
        );
    }

    #[test]
    fn plan_fires_focus_and_blur_on_edges_only() {
        let p = EventPlan::compute(false, false, false, true, false, false);
        assert!(p.focus && !p.blur);
        let p = EventPlan::compute(false, true, false, true, false, false);
        assert!(p.is_empty());
        let p = EventPlan::compute(false, true, false, false, false, false);
        assert!(p.blur && !p.focus);
    }

    #[test]
    fn plan_passes_clicks_through() {
        let p = EventPlan::compute(false, false, true, false, true, false);
        assert!(p.click && p.hover);
    }

    #[test]
    fn disabled_plan_is_always_empty() {
        let p = EventPlan::compute(false, false, true, true, true, true);
        assert!(p.is_empty());
        let p = EventPlan::compute(true, true, false, false, true, true);
        assert!(p.is_empty());
    }

    // ---- callbacks -----------------------------------------------------------

    #[test]
    fn fire_invokes_only_the_planned_callbacks_once() {
        let (hover, f1) = counter();
        let (unhover, f2) = counter();
        let (click, f3) = counter();
        let (focus, f4) = counter();
        let (blur, f5) = counter();
        let mut c = MudContainer::new()
            .on_hover(f1)
            .on_unhover(f2)
            .on_click(f3)
            .on_focus(f4)
            .on_blur(f5);

        c.fire(EventPlan {
            hover: true,
            click: true,
            ..Default::default()
        });
        assert_eq!(hover.load(Ordering::SeqCst), 1);
        assert_eq!(click.load(Ordering::SeqCst), 1);
        assert_eq!(unhover.load(Ordering::SeqCst), 0);
        assert_eq!(focus.load(Ordering::SeqCst), 0);
        assert_eq!(blur.load(Ordering::SeqCst), 0);

        c.fire(EventPlan {
            unhover: true,
            focus: true,
            blur: true,
            ..Default::default()
        });
        assert_eq!(unhover.load(Ordering::SeqCst), 1);
        assert_eq!(focus.load(Ordering::SeqCst), 1);
        assert_eq!(blur.load(Ordering::SeqCst), 1);
        assert_eq!(hover.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn fire_with_no_callbacks_is_a_no_op() {
        let mut c = MudContainer::new();
        c.fire(EventPlan {
            hover: true,
            unhover: true,
            click: true,
            focus: true,
            blur: true,
        });
    }

    #[test]
    fn empty_plan_fires_nothing() {
        let (hover, f) = counter();
        let mut c = MudContainer::new().on_hover(f);
        c.fire(EventPlan::default());
        assert_eq!(hover.load(Ordering::SeqCst), 0);
    }

    // ---- component state -----------------------------------------------------

    #[test]
    fn defaults() {
        let c = MudContainer::default();
        assert!(matches!(c.max_width, MaxWidth::Lg));
        assert!(!c.fluid);
        assert!(!c.fixed);
        assert!(c.gutters);
        assert!(!c.disabled);
        assert_eq!(c.flavor, StyleFlavor::Bootstrap);
        assert!(c.id.is_none());
        assert!(c.aria_label.is_none());
        assert!(c.tooltip.is_none());
        assert!(c.tag.is_none());
        assert!(c.user_attributes.is_empty());
        assert!(c.style.is_empty());
        assert!(c.focus_ring);
        assert!(!c.memoize_style);
        assert!(c.on_hover.is_none());
        assert!(c.on_click.is_none());
        assert!(c.on_breakpoint_change.is_none());
        assert!(c.last_layout().is_none());
    }

    #[test]
    fn builder_pattern() {
        let c = MudContainer::new()
            .max_width(MaxWidth::Md)
            .fluid(true)
            .fixed(false)
            .gutters(false)
            .disabled(true);
        assert!(c.fluid && !c.gutters && c.disabled && !c.fixed);
        assert!(matches!(c.max_width, MaxWidth::Md));
    }

    #[test]
    fn presets() {
        assert_eq!(
            MudContainer::for_breakpoint(MaxWidth::Sm).max_width,
            MaxWidth::Sm
        );
        let f = MudContainer::full_width();
        assert_eq!(f.max_width, MaxWidth::False);
        assert!(f.gutters);
    }

    #[test]
    fn spec_round_trips_through_container() {
        let spec = LayoutSpec {
            max_width: MaxWidth::Xl,
            fluid: false,
            fixed: true,
            gutters: false,
        };
        let c = MudContainer::from_spec(spec);
        assert_eq!(c.layout_spec(), spec);
        assert_eq!(c.preview_layout(1000.0), spec.resolve(1000.0));
    }

    #[test]
    fn preview_layout_needs_no_egui_state() {
        let c = MudContainer::new().max_width(MaxWidth::Md);
        let l = c.preview_layout(1500.0);
        assert_eq!(l.outer_width, 960.0);
        assert_eq!(l.content_width, 912.0);
    }

    #[test]
    fn flavor_builder_selects_tailwind() {
        let c = MudContainer::new().flavor(StyleFlavor::Tailwind);
        assert_eq!(c.flavor, StyleFlavor::Tailwind);
    }

    #[test]
    fn horizontal_layout_is_copy_and_comparable() {
        let a = M::resolve(800.0, true, false, false, MaxWidth::Lg);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn debug_output_names_the_type() {
        let s = format!("{:?}", MudContainer::new());
        assert!(s.contains("MudContainer"));
    }

    // ---- tag & user attributes ----------------------------------------------

    #[test]
    fn tag_round_trips_by_type() {
        let c = MudContainer::new().tag(42_u32);
        assert_eq!(c.tag_ref::<u32>(), Some(&42));
        assert_eq!(c.tag_ref::<String>(), None);
        assert!(MudContainer::new().tag_ref::<u32>().is_none());
    }

    #[test]
    fn tag_is_replaced_by_a_later_tag() {
        let c = MudContainer::new().tag(1_u8).tag("two".to_string());
        assert_eq!(c.tag_ref::<u8>(), None);
        assert_eq!(c.tag_ref::<String>().map(String::as_str), Some("two"));
    }

    #[test]
    fn user_attributes_store_and_overwrite() {
        let c = MudContainer::new()
            .attr("data-x", "1")
            .attrs([("data-y", "2"), ("data-x", "3")]);
        assert_eq!(c.user_attribute("data-x"), Some("3"));
        assert_eq!(c.user_attribute("data-y"), Some("2"));
        assert_eq!(c.user_attribute("missing"), None);
    }

    #[test]
    fn aria_label_prefers_explicit_field_over_attribute() {
        let c = MudContainer::new().attr("aria-label", "from attr");
        assert_eq!(c.effective_aria_label(), Some("from attr"));
        let c = c.aria_label("explicit");
        assert_eq!(c.effective_aria_label(), Some("explicit"));
        assert_eq!(MudContainer::new().effective_aria_label(), None);
    }

    #[test]
    fn tooltip_is_independent_of_aria_label() {
        let c = MudContainer::new().tooltip("hi");
        assert_eq!(c.tooltip.as_deref(), Some("hi"));
        assert!(c.aria_label.is_none());
    }

    // ---- component base ------------------------------------------------------

    #[test]
    fn default_base_is_empty() {
        let c = MudContainer::default();
        assert!(c.base.class_names().is_empty());
        assert_eq!(c.base.elevation_value(), 0);
        assert!(!c.base.ripple_enabled());
    }

    #[test]
    fn classes_builder_populates_base() {
        let c = MudContainer::new().classes(["bg-surface", "p-4"]);
        assert_eq!(c.base.class_names(), ["bg-surface", "p-4"]);
        assert!(c.base.has_class("bg-surface"));
        assert!(c.base.has_class("p-4"));
    }

    #[test]
    fn classes_builder_accepts_space_separated_and_dedups() {
        let c = MudContainer::new()
            .classes(["bg-surface p-4"])
            .classes(["p-4", "rounded"]);
        assert_eq!(c.base.class_names(), ["bg-surface", "p-4", "rounded"]);
    }

    #[test]
    fn class_builder_matches_mudblazor_class_string() {
        let c = MudContainer::new().class("bg-surface p-4 rounded");
        assert_eq!(c.base.class_names(), ["bg-surface", "p-4", "rounded"]);
    }

    #[test]
    fn classes_builder_composes_with_other_builders() {
        let c = MudContainer::new()
            .max_width(MaxWidth::Md)
            .fluid(true)
            .gutters(false)
            .flavor(StyleFlavor::Tailwind)
            .classes(["card"]);
        assert!(matches!(c.max_width, MaxWidth::Md));
        assert!(c.fluid && !c.gutters);
        assert_eq!(c.flavor, StyleFlavor::Tailwind);
        assert_eq!(c.base.class_names(), ["card"]);
    }

    #[test]
    fn elevation_builder() {
        let c = MudContainer::new().elevation(8);
        assert_eq!(c.base.elevation_value(), 8);
    }

    #[test]
    fn elevation_builder_clamps_to_24() {
        let c = MudContainer::new().elevation(200);
        assert_eq!(c.base.elevation_value(), 24);
    }

    #[test]
    fn base_field_is_publicly_accessible_for_extra_mutations() {
        let mut c = MudContainer::new();
        c.base.set_elevation(4);
        assert_eq!(c.base.elevation_value(), 4);
        c.base.add_class("elevated");
        assert!(c.base.has_class("elevated"));
    }

    // ---- style overrides -----------------------------------------------------

    #[test]
    fn empty_overrides_leave_the_frame_untouched() {
        let overrides = StyleOverrides::default();
        assert!(overrides.is_empty());
        let mut frame = Frame::default();
        frame.fill = Color32::from_rgb(1, 2, 3);
        let out = overrides.apply(frame);
        assert_eq!(out.fill, Color32::from_rgb(1, 2, 3));
    }

    #[test]
    fn overrides_replace_only_what_is_set() {
        let mut frame = Frame::default();
        frame.fill = Color32::from_rgb(1, 2, 3);
        frame.stroke = Stroke::new(1.0, Color32::BLACK);

        let pad = Margin {
            left: 1,
            right: 2,
            top: 3,
            bottom: 4,
        };
        let overrides = StyleOverrides {
            fill: Some(Color32::from_rgb(9, 9, 9)),
            padding: Some(pad),
            ..Default::default()
        };
        assert!(!overrides.is_empty());

        let out = overrides.apply(frame);
        assert_eq!(out.fill, Color32::from_rgb(9, 9, 9));
        assert_eq!(out.inner_margin, pad);
        // Untouched properties survive.
        assert_eq!(out.stroke, Stroke::new(1.0, Color32::BLACK));
    }

    #[test]
    fn override_outer_margin_and_stroke_apply() {
        let outer = Margin {
            left: 5,
            right: 5,
            top: 5,
            bottom: 5,
        };
        let overrides = StyleOverrides {
            stroke: Some(Stroke::new(3.0, Color32::RED)),
            outer_margin: Some(outer),
            ..Default::default()
        };
        let out = overrides.apply(Frame::default());
        assert_eq!(out.stroke, Stroke::new(3.0, Color32::RED));
        assert_eq!(out.outer_margin, outer);
    }

    #[test]
    fn style_builders_write_into_overrides() {
        let pad = Margin {
            left: 2,
            right: 2,
            top: 2,
            bottom: 2,
        };
        let c = MudContainer::new()
            .fill(Color32::BLUE)
            .stroke(Stroke::new(1.0, Color32::GREEN))
            .padding(pad);
        assert_eq!(c.style.fill, Some(Color32::BLUE));
        assert_eq!(c.style.stroke, Some(Stroke::new(1.0, Color32::GREEN)));
        assert_eq!(c.style.padding, Some(pad));

        let whole = MudContainer::new().style(StyleOverrides {
            fill: Some(Color32::RED),
            ..Default::default()
        });
        assert_eq!(whole.style.fill, Some(Color32::RED));
    }

    #[test]
    fn focus_ring_and_memoize_builders() {
        let c = MudContainer::new().focus_ring(false).memoize_style(true);
        assert!(!c.focus_ring);
        assert!(c.memoize_style);
    }

    // ---- flavour / interaction state ----------------------------------------

    #[test]
    fn flavor_default_is_bootstrap() {
        assert_eq!(StyleFlavor::default(), StyleFlavor::Bootstrap);
    }

    #[test]
    fn interaction_state_starts_defaulted() {
        let c = MudContainer::default();
        let s = c.interaction;
        assert!(!s.hovered && !s.active && !s.focused && !s.dark && !s.disabled);
    }

    #[test]
    fn interaction_state_is_publicly_mutable() {
        let mut c = MudContainer::new().flavor(StyleFlavor::Tailwind);
        c.interaction.dark = true;
        c.interaction.hovered = true;
        assert!(c.interaction.dark && c.interaction.hovered);
    }

    #[test]
    fn interaction_to_tw_state_respects_disabled() {
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

    #[test]
    fn rem_size_pt_is_bootstrap_default() {
        assert_eq!(REM_SIZE_PT, 14.0);
    }

    #[test]
    fn disabled_propagates_to_interaction() {
        let c = MudContainer::new().disabled(true);
        assert!(c.disabled);
        assert!(c.interaction.disabled);
    }

    #[test]
    fn reset_interaction_clears_flags_but_keeps_disabled() {
        let mut c = MudContainer::new().disabled(true);
        c.interaction.hovered = true;
        c.interaction.focused = true;
        c.focus_anim = 0.7;
        c.reset_interaction();
        assert!(!c.interaction.hovered && !c.interaction.focused);
        assert!(c.interaction.disabled);
        assert_eq!(c.focus_anim, 0.0);
    }

    // ---- style cache keys ----------------------------------------------------

    #[test]
    fn style_key_is_stable_for_identical_inputs() {
        let a = MudContainer::new().classes(["p-4"]);
        let b = MudContainer::new().classes(["p-4"]);
        assert_eq!(a.style_key(true, 7), b.style_key(true, 7));
    }

    #[test]
    fn style_key_changes_with_classes_elevation_dark_and_visuals() {
        let base = MudContainer::new().classes(["p-4"]);
        let k = base.style_key(true, 7);
        assert_ne!(k, MudContainer::new().classes(["p-5"]).style_key(true, 7));
        assert_ne!(
            k,
            MudContainer::new()
                .classes(["p-4"])
                .elevation(4)
                .style_key(true, 7)
        );
        assert_ne!(k, base.style_key(false, 7));
        assert_ne!(k, base.style_key(true, 8));
    }

    #[test]
    fn style_key_changes_with_flavor_and_epoch() {
        let mut c = MudContainer::new().classes(["p-4"]);
        let bootstrap = c.style_key(true, 0);
        let tailwind = MudContainer::new()
            .classes(["p-4"])
            .flavor(StyleFlavor::Tailwind)
            .style_key(true, 0);
        assert_ne!(bootstrap, tailwind);

        c.invalidate_style_cache();
        assert_ne!(bootstrap, c.style_key(true, 0));
    }

    #[test]
    fn tailwind_key_tracks_hover_but_bootstrap_key_does_not() {
        let mut tw = MudContainer::new().flavor(StyleFlavor::Tailwind);
        let idle = tw.style_key(true, 0);
        tw.interaction.hovered = true;
        assert_ne!(idle, tw.style_key(true, 0));

        let mut bs = MudContainer::new();
        let idle = bs.style_key(true, 0);
        bs.interaction.hovered = true;
        assert_eq!(idle, bs.style_key(true, 0));
    }

    #[test]
    fn disabled_tailwind_key_ignores_hover() {
        let mut tw = MudContainer::new()
            .flavor(StyleFlavor::Tailwind)
            .disabled(true);
        let idle = tw.style_key(true, 0);
        tw.interaction.hovered = true;
        assert_eq!(idle, tw.style_key(true, 0));
    }

    #[test]
    fn invalidate_style_cache_clears_and_bumps_epoch() {
        let mut c = MudContainer::new();
        c.frame_cache = Some((1, Frame::default()));
        c.invalidate_style_cache();
        assert!(c.frame_cache.is_none());
        assert_eq!(c.cache_epoch, 1);
    }

    // ---- shared handle -------------------------------------------------------

    #[test]
    fn handle_types_are_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ContainerHandle>();
        assert_send_sync::<ContainerSnapshot>();
    }

    #[test]
    fn fresh_handle_is_zeroed() {
        let h = ContainerHandle::new();
        let s = h.snapshot();
        assert_eq!(s.frame, 0);
        assert_eq!(s.outer_width, 0.0);
        assert_eq!(s.content_width, 0.0);
        assert!(!s.hovered && !s.active && !s.focused);
        assert!(!h.has_rendered());
    }

    #[test]
    fn publish_then_snapshot_round_trips() {
        let h = ContainerHandle::new();
        let layout = M::resolve(1500.0, true, false, false, MaxWidth::Md);
        let state = InteractionState {
            hovered: true,
            focused: true,
            ..Default::default()
        };
        h.publish(&layout, &state);

        let s = h.snapshot();
        assert_eq!(s.frame, 1);
        assert_eq!(s.outer_width, 960.0);
        assert_eq!(s.content_width, 912.0);
        assert_eq!(s.gutter, 24.0);
        assert_eq!(s.viewport_breakpoint, MaxWidth::Lg);
        assert_eq!(s.effective_max, MaxWidth::Md);
        assert!(s.hovered && s.focused && !s.active);
        assert!(h.is_hovered() && h.is_focused());
        assert_eq!(h.outer_width(), 960.0);
        assert_eq!(h.content_width(), 912.0);
        assert_eq!(h.viewport_breakpoint(), MaxWidth::Lg);
        assert!(h.has_rendered());
    }

    #[test]
    fn handle_clones_share_state() {
        let a = ContainerHandle::new();
        let b = a.clone();
        a.publish(
            &M::resolve(800.0, true, false, false, MaxWidth::Lg),
            &InteractionState::default(),
        );
        assert_eq!(b.frame_count(), 1);
    }

    #[test]
    fn container_handle_is_created_lazily_and_shared() {
        let mut c = MudContainer::new();
        assert!(c.shared.is_none());
        let h1 = c.handle();
        let h2 = c.handle();
        assert!(c.shared.is_some());
        h1.publish(
            &M::resolve(800.0, true, false, false, MaxWidth::Lg),
            &InteractionState::default(),
        );
        assert_eq!(h2.frame_count(), 1);
    }

    #[test]
    fn handle_can_be_read_from_another_thread() {
        let h = ContainerHandle::new();
        h.publish(
            &M::resolve(1000.0, true, false, false, MaxWidth::Lg),
            &InteractionState::default(),
        );
        let h2 = h.clone();
        let width = std::thread::spawn(move || h2.outer_width())
            .join()
            .expect("reader thread");
        assert_eq!(width, 1000.0);
    }

    // ---- egui-driven rendering ----------------------------------------------

    #[test]
    fn show_returns_the_closure_value() {
        let mut c = MudContainer::new();
        let v = with_ui(800.0, |ui| c.show(ui, |_| 7_u32).inner);
        assert_eq!(v, 7);
    }

    #[test]
    fn show_reports_resolved_layout_to_the_closure() {
        let mut c = MudContainer::new().max_width(MaxWidth::Lg);
        let layout = with_ui(2000.0, |ui| {
            let mut seen = None;
            c.show_with_layout(ui, |_, l| {
                seen = Some(*l);
            });
            seen.expect("closure ran")
        });
        assert_eq!(layout.outer_width, 1280.0);
        assert_eq!(layout.content_width, 1232.0);
        assert_eq!(layout.gutter, 24.0);
        assert_eq!(layout.effective_max, MaxWidth::Lg);
    }

    #[test]
    fn show_reserves_the_full_available_width() {
        let (avail, rect) = with_ui(800.0, |ui| {
            let avail = ui.available_rect_before_wrap();
            let rect = MudContainer::new().show(ui, |_| {}).response.rect;
            (avail, rect)
        });
        assert_eq!(rect.min, avail.min);
        assert!((rect.width() - avail.width()).abs() < 0.01);
    }

    #[test]
    fn show_records_last_layout() {
        let mut c = MudContainer::new();
        with_ui(2000.0, |ui| {
            c.show(ui, |_| {});
        });
        let l = c.last_layout().expect("layout recorded");
        assert_eq!(l.outer_width, 1280.0);
    }

    #[test]
    fn fixed_mode_applies_the_snapped_breakpoint_in_show() {
        let mut c = MudContainer::new().fixed(true);
        let layout = with_ui(900.0, |ui| {
            let mut seen = None;
            c.show_with_layout(ui, |_, l| {
                seen = Some(*l);
            });
            seen.expect("closure ran")
        });
        assert_eq!(layout.effective_max, MaxWidth::Sm);
        assert!((layout.outer_width - 600.0).abs() < 0.01);
    }

    #[test]
    fn fluid_mode_fills_the_offered_width_in_show() {
        let mut c = MudContainer::new().fluid(true);
        let (layout, avail_w) = with_ui(1500.0, |ui| {
            let avail_w = ui.available_rect_before_wrap().width();
            let mut seen = None;
            c.show_with_layout(ui, |_, l| {
                seen = Some(*l);
            });
            (seen.expect("closure ran"), avail_w)
        });
        assert!((layout.outer_width - avail_w).abs() < 0.01);
        assert_eq!(layout.effective_max, MaxWidth::False);
    }

    #[test]
    fn handle_tracks_frames_rendered_through_show() {
        let mut c = MudContainer::new();
        let h = c.handle();
        assert_eq!(h.snapshot().frame, 0);
        with_ui(1500.0, |ui| {
            c.show(ui, |_| {});
        });
        with_ui(1500.0, |ui| {
            c.show(ui, |_| {});
        });
        let s = h.snapshot();
        assert_eq!(s.frame, 2);
        assert_eq!(s.outer_width, 1280.0);
        assert_eq!(s.viewport_breakpoint, MaxWidth::Lg);
        assert_eq!(s.effective_max, MaxWidth::Lg);
    }

    #[test]
    fn breakpoint_callback_fires_on_first_frame_and_transitions_only() {
        let log = Arc::new(Mutex::new(Vec::<u8>::new()));
        let log2 = Arc::clone(&log);
        let mut c = MudContainer::new().on_breakpoint_change(move |bp| {
            log2.lock().expect("log lock").push(max_width_to_u8(bp));
        });

        for width in [500.0, 520.0, 900.0, 1500.0, 1510.0] {
            with_ui(width, |ui| {
                c.show(ui, |_| {});
            });
        }
        // Xs (first frame), Sm, Lg; same-bucket frames do not fire.
        assert_eq!(*log.lock().expect("log lock"), vec![0_u8, 1, 3]);
    }

    #[test]
    fn disabled_container_disables_its_children() {
        let mut c = MudContainer::new().disabled(true);
        let enabled = with_ui(800.0, |ui| {
            let mut e = true;
            c.show(ui, |ui| {
                e = ui.is_enabled();
            });
            e
        });
        assert!(!enabled);
    }

    #[test]
    fn enabled_container_leaves_children_enabled() {
        let mut c = MudContainer::new();
        let enabled = with_ui(800.0, |ui| {
            let mut e = false;
            c.show(ui, |ui| {
                e = ui.is_enabled();
            });
            e
        });
        assert!(enabled);
    }

    #[test]
    fn show_syncs_public_disabled_field_into_interaction() {
        let mut c = MudContainer::new();
        c.disabled = true; // mutate the public field directly
        with_ui(800.0, |ui| {
            c.show(ui, |_| {});
        });
        assert!(c.interaction.disabled);
    }

    #[test]
    fn memoized_frame_is_reused_until_invalidated() {
        let mut c = MudContainer::new().memoize_style(true).classes(["p-4"]);
        with_ui(800.0, |ui| {
            c.show(ui, |_| {});
        });
        let first = c.frame_cache.as_ref().map(|(k, _)| *k);
        assert!(first.is_some());

        with_ui(800.0, |ui| {
            c.show(ui, |_| {});
        });
        assert_eq!(c.frame_cache.as_ref().map(|(k, _)| *k), first);

        c.invalidate_style_cache();
        assert!(c.frame_cache.is_none());
        with_ui(800.0, |ui| {
            c.show(ui, |_| {});
        });
        let second = c.frame_cache.as_ref().map(|(k, _)| *k);
        assert!(second.is_some());
        assert_ne!(second, first);
    }

    #[test]
    fn non_memoized_container_never_populates_the_cache() {
        let mut c = MudContainer::new();
        with_ui(800.0, |ui| {
            c.show(ui, |_| {});
        });
        assert!(c.frame_cache.is_none());
    }

    #[test]
    fn widget_impl_adds_an_empty_container() {
        let rect = with_ui(800.0, |ui| ui.add(MudContainer::new()).rect);
        assert!(rect.width() > 0.0);
    }

    #[test]
    fn idle_interactive_container_keeps_focus_animation_at_zero() {
        let (_clicks, f) = counter();
        let mut c = MudContainer::new().on_click(f);
        with_ui(800.0, |ui| {
            c.show(ui, |_| {});
        });
        assert_eq!(c.focus_anim, 0.0);
    }
}