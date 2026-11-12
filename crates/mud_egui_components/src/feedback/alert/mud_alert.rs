//! A statically embedded feedback banner.
//!
//! Port of `MudAlert.razor.cs` (MudBlazor 8.13.0). The public parameter surface
//! mirrors the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `ChildContent` | [`MudAlert::new`] / [`MudAlert::text`] / [`MudAlert::content`] |
//! | `CloseIcon` | [`MudAlert::close_icon`] |
//! | `CloseIconClicked` | [`AlertResponse::close_clicked`] + [`MudAlert::on_close`] |
//! | `ContentAlignment` | [`MudAlert::content_alignment`] |
//! | `Dense` | [`MudAlert::dense`] |
//! | `Elevation` | [`MudAlert::elevation`] |
//! | `Icon` | [`MudAlert::icon`] |
//! | `NoIcon` | [`MudAlert::no_icon`] |
//! | `OnClick` | [`AlertResponse::body_clicked`] + [`MudAlert::on_click`] |
//! | `RightToLeft` | [`MudAlert::right_to_left`] |
//! | `Severity` | [`MudAlert::severity`] |
//! | `ShowCloseIcon` | [`MudAlert::show_close_icon`] |
//! | `Square` | [`MudAlert::square`] |
//! | `Variant` | [`MudAlert::variant`] |
//! | `Class` | [`MudAlert::classes`] (stored, not resolved — see below) |
//! | `Style` | not ported: egui has no inline CSS strings |
//! | `Tag` | [`MudAlert::tag`] / [`MudAlert::tag_ref`] |
//! | `UserAttributes` | not ported: there is no HTML tag |
//!
//! # Variants
//!
//! | [`Variant`] | Fill | Border | Text / icon |
//! |-------------|------|--------|-------------|
//! | `Text` (default) | accent at 12 % alpha | none | accent |
//! | `Outlined` | transparent | 1 pt accent | accent |
//! | `Filled` | accent | none | black or white by luminance |
//!
//! # Severity
//!
//! The severity picks both the accent colour and the default glyph, exactly as
//! MudBlazor's `OnParametersSet` does:
//!
//! | [`Severity`] | [`Color`] role | Default glyph |
//! |--------------|----------------|---------------|
//! | `Normal` | `Color::Default` | `INFO` |
//! | `Info` | `Color::Info` | `INFO` |
//! | `Success` | `Color::Success` | `CHECK_CIRCLE` |
//! | `Warning` | `Color::Warning` | `WARNING` |
//! | `Error` | `Color::Error` | `ERROR` |
//!
//! `Normal` and `Info` deliberately share the `INFO` glyph — that is what
//! MudBlazor renders.
//!
//! # Per-frame model
//!
//! `MudAlert` is stateless between frames. The caller owns nothing: a close
//! click is reported through [`AlertResponse::close_clicked`], so whether the
//! alert disappears is the caller's decision (MudBlazor behaves the same way —
//! `CloseIconClicked` does not remove the alert).
//!
//! # Content measurement
//!
//! The alert must know its row height *before* it can allocate space, because
//! the whole banner is a single painted rectangle. Custom [`content`](MudAlert::content)
//! closures are therefore run **twice per frame**: once into an invisible probe
//! `Ui` to measure, once for real. Closures are `FnMut` and should be free of
//! side effects — the same contract as `MudToggleGroup`'s custom cells.
//!
//! # Performance
//!
//! - Theme palette is fetched **at most once per frame**, and only one
//!   `Color::resolve` call is made.
//! - The message text is measured and drawn from the same `&str`; no `String`
//!   clone on the hot path (`MudText::new` takes `impl Into<String>`, so it
//!   allocates once per pass — same as any `MudText`).
//! - Every geometry decision is a pure `#[inline]` function over `Copy` values.
//! - The close button is one `ui.interact` plus one optional `circle_filled`.
//! - No `egui::Memory`, so there is no re-entrant-lock hazard.

use std::any::Any;

use egui::epaint::Shadow;
use egui::{
    pos2, vec2, Align, Color32, CornerRadius, Layout, Margin, Rect, Response, Sense, Stroke, Ui,
    UiBuilder,
};

use crate::{MudIcon, MudText};
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, HorizontalAlignment, Severity, Typo, Variant};
use mud_egui_theming::theme::MudTheme;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Horizontal padding of a standard alert, in logical pixels.
const PADDING_X: f32 = 16.0;

/// Vertical padding of a standard alert, in logical pixels.
const PADDING_Y: f32 = 8.0;

/// Horizontal padding of a dense alert, in logical pixels.
const DENSE_PADDING_X: f32 = 12.0;

/// Vertical padding of a dense alert, in logical pixels.
const DENSE_PADDING_Y: f32 = 4.0;

/// Floor for the content row height of a standard alert, in logical pixels.
const MIN_ROW_HEIGHT: f32 = 24.0;

/// Floor for the content row height of a dense alert, in logical pixels.
const DENSE_MIN_ROW_HEIGHT: f32 = 18.0;

/// Square side of the leading severity icon, in logical pixels.
const ICON_SIZE: f32 = 22.0;

/// Gap between the leading icon and the message, in logical pixels.
const ICON_GAP: f32 = 12.0;

/// Gap between the message and the close button hit area, in logical pixels.
const CLOSE_GAP: f32 = 8.0;

/// Side of the close button's square hit area, in logical pixels.
const CLOSE_HIT: f32 = 28.0;

/// Square side of the close glyph drawn inside the hit area, in logical pixels.
const CLOSE_GLYPH: f32 = 18.0;

/// Corner radius of a non-square alert, in logical pixels.
const DEFAULT_CORNER_RADIUS: f32 = 4.0;

/// Width of the outlined variant's border stroke, in logical pixels.
const OUTLINE_STROKE_WIDTH: f32 = 1.0;

/// Alpha applied to the accent colour for the `Text` variant's fill.
const TEXT_VARIANT_FILL_ALPHA: u8 = 30;

/// Highest elevation MudBlazor accepts; values above this are clamped.
const MAX_ELEVATION: u8 = 24;

/// Height of the invisible probe `Ui` used to measure content.
///
/// It must be finite: an infinite probe height makes content that stretches to
/// the available height measure as infinite, which poisons parent scroll areas
/// with NaN offsets.
const MAX_PROBE_HEIGHT: f32 = 2048.0;

/// Upper bound on the corner radius, matching `CornerRadius`'s `u8` payload.
const MAX_CORNER_RADIUS: f32 = 255.0;

// ---------------------------------------------------------------------------
// MudAlert
// ---------------------------------------------------------------------------

/// A statically embedded feedback banner.
///
/// Configure it with the builder methods, then call [`MudAlert::show`].
///
/// Mirrors `MudAlert` in MudBlazor 8.13.0. The two Blazor event callbacks
/// `OnClick` and `CloseIconClicked` are reported as data on
/// [`AlertResponse`] (the idiomatic egui equivalent) and can additionally be
/// wired with [`on_click`](Self::on_click) / [`on_close`](Self::on_close).
#[must_use = "MudAlert does nothing until you call `.show(ui)`"]
pub struct MudAlert {
    /// Shared component state (utility classes). Stored for parity with other
    /// components; not read while rendering, because the variant already owns
    /// the alert's fill, border and radius.
    pub base: ComponentBase,
    /// Plain-text message. Ignored when [`content`](Self::content) is set.
    /// Mirrors MudBlazor `ChildContent` (text form).
    pub text: String,
    /// Custom content renderer. Replaces [`text`](Self::text) entirely.
    /// Runs twice per frame (measure, then draw). Mirrors MudBlazor
    /// `ChildContent` (render-fragment form).
    pub content: Option<Box<dyn FnMut(&mut Ui) + 'static>>,
    /// Colour and default-icon source. Mirrors MudBlazor `Severity`.
    pub severity: Severity,
    /// Visual variant. Mirrors MudBlazor `Variant` (default `Text`).
    pub variant: Variant,
    /// Compact padding. Mirrors MudBlazor `Dense`.
    pub dense: bool,
    /// Square corners. Mirrors MudBlazor `Square`.
    pub square: bool,
    /// Drop-shadow depth, clamped to [`MAX_ELEVATION`]. Mirrors MudBlazor
    /// `Elevation` (default `0`).
    pub elevation: u8,
    /// Suppress the leading icon. Mirrors MudBlazor `NoIcon`.
    pub no_icon: bool,
    /// Override the severity-derived glyph. Mirrors MudBlazor `Icon`.
    pub icon: Option<String>,
    /// Override the close-button glyph. Mirrors MudBlazor `CloseIcon`
    /// (default `filled::CLOSE`).
    pub close_icon: Option<String>,
    /// Show the close button. Mirrors MudBlazor `ShowCloseIcon`.
    pub show_close_icon: bool,
    /// Horizontal placement of the message. Mirrors MudBlazor
    /// `ContentAlignment` (default `Left`).
    pub content_alignment: HorizontalAlignment,
    /// Mirror the column order and resolve `Start` / `End` for RTL. Mirrors
    /// MudBlazor's cascading `RightToLeft`.
    pub right_to_left: bool,
    /// Fired when the alert body is clicked. Mirrors MudBlazor `OnClick`.
    pub on_click: Option<Box<dyn FnMut() + 'static>>,
    /// Fired when the close button is clicked. Mirrors MudBlazor
    /// `CloseIconClicked`.
    pub on_close: Option<Box<dyn FnMut() + 'static>>,
    /// Arbitrary user data. Mirrors MudBlazor `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl Default for MudAlert {
    /// Normal severity, `Text` variant, no icon suppression, no close button,
    /// left-aligned, elevation `0` — MudBlazor's defaults.
    fn default() -> Self {
        Self {
            base: ComponentBase::new(),
            text: String::new(),
            content: None,
            severity: Severity::Normal,
            variant: Variant::Text,
            dense: false,
            square: false,
            elevation: 0,
            no_icon: false,
            icon: None,
            close_icon: None,
            show_close_icon: false,
            content_alignment: HorizontalAlignment::Left,
            right_to_left: false,
            on_click: None,
            on_close: None,
            tag: None,
        }
    }
}

impl MudAlert {
    /// Creates an alert showing `text` with MudBlazor's default settings.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }

    /// Replaces the plain-text message.
    #[inline]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Replaces the message with custom content.
    ///
    /// The closure runs **twice per frame** (measure, then draw) and should be
    /// free of side effects. It takes precedence over [`text`](Self::text).
    #[inline]
    pub fn content(mut self, f: impl FnMut(&mut Ui) + 'static) -> Self {
        self.content = Some(Box::new(f));
        self
    }

    /// Sets the severity (accent colour and default icon).
    #[inline]
    pub fn severity(mut self, s: Severity) -> Self {
        self.severity = s;
        self
    }

    /// Sets the visual variant.
    #[inline]
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    /// Enables compact padding.
    #[inline]
    pub fn dense(mut self, b: bool) -> Self {
        self.dense = b;
        self
    }

    /// Removes the corner rounding.
    #[inline]
    pub fn square(mut self, b: bool) -> Self {
        self.square = b;
        self
    }

    /// Sets the drop-shadow depth (clamped to 24).
    #[inline]
    pub fn elevation(mut self, e: u8) -> Self {
        self.elevation = e;
        self
    }

    /// Suppresses the leading icon.
    #[inline]
    pub fn no_icon(mut self, b: bool) -> Self {
        self.no_icon = b;
        self
    }

    /// Overrides the severity-derived leading glyph.
    #[inline]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Overrides the close-button glyph.
    #[inline]
    pub fn close_icon(mut self, icon: impl Into<String>) -> Self {
        self.close_icon = Some(icon.into());
        self
    }

    /// Shows or hides the close button.
    #[inline]
    pub fn show_close_icon(mut self, b: bool) -> Self {
        self.show_close_icon = b;
        self
    }

    /// Sets the horizontal placement of the message.
    #[inline]
    pub fn content_alignment(mut self, a: HorizontalAlignment) -> Self {
        self.content_alignment = a;
        self
    }

    /// Mirrors the column order and resolves `Start` / `End` for RTL.
    #[inline]
    pub fn right_to_left(mut self, b: bool) -> Self {
        self.right_to_left = b;
        self
    }

    /// Registers a body-click handler.
    #[inline]
    pub fn on_click(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    /// Registers a close-click handler.
    #[inline]
    pub fn on_close(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_close = Some(Box::new(f));
        self
    }

    /// Adds utility classes to the alert's [`ComponentBase`].
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Attaches arbitrary user data.
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

    /// Renders the alert and reports what happened this frame.
    ///
    /// The returned [`AlertResponse`] dereferences to the content row's
    /// [`Response`], so `out.hovered()` works. The row excludes the padding.
    pub fn show(mut self, ui: &mut Ui) -> AlertResponse {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let accent = severity_color(self.severity)
            .resolve(theme.current_palette())
            .unwrap_or_else(|| ui.visuals().text_color());

        let style = resolve_style(accent, self.variant);
        let ink = style.ink;

        let has_icon = !self.no_icon;
        let has_close = self.show_close_icon;
        let d = density(self.dense);
        let align = resolve_alignment(self.content_alignment, self.right_to_left);

        let icon_glyph = has_icon.then(|| resolve_icon(self.icon.as_deref(), self.severity));
        let close_glyph = has_close.then(|| {
            self.close_icon
                .clone()
                .unwrap_or_else(|| mud_egui_icons_material::filled::CLOSE.to_owned())
        });

        let mut frame = egui::Frame::new()
            .fill(style.fill)
            .corner_radius(CornerRadius::same(corner_radius(self.square)))
            .inner_margin(Margin::symmetric(d.pad.0 as i8, d.pad.1 as i8));
        if let Some(stroke) = style.stroke {
            frame = frame.stroke(stroke);
        }
        if self.elevation > 0 {
            frame = frame.shadow(elevation_shadow(self.elevation));
        }

        let sense = if self.on_click.is_some() {
            Sense::click()
        } else {
            Sense::hover()
        };

        // Items are single-use per frame, so move them out of `self`.
        let mut content = self.content.take();
        let text = std::mem::take(&mut self.text);

        let mut draw_message = |ui: &mut Ui| {
            if let Some(render) = content.as_mut() {
                render(ui);
            } else {
                ui.add(
                    MudText::new(text.as_str())
                        .typo(Typo::Body2)
                        .inline(true)
                        .custom_color(ink),
                );
            }
        };

        let mut body_clicked = false;
        let mut close_clicked = false;
        let min_row = d.min_row;

        let inner = frame.show(ui, |ui| {
            let available = ui.available_width();
            let msg_w = message_width(available, has_icon, has_close);
            let measured = measure_message(ui, &mut draw_message, msg_w, align);
            let row_h = row_height(measured, has_icon, has_close, min_row);

            let (row, row_resp) = ui.allocate_exact_size(vec2(available, row_h), sense);
            let cols = layout_columns(row, self.right_to_left, has_icon, has_close);

            if let (Some(rect), Some(glyph)) = (cols.icon, icon_glyph.as_deref()) {
                ui.put(
                    rect,
                    MudIcon::new(glyph.to_owned())
                        .size_px(ICON_SIZE)
                        .custom_color(ink),
                );
            }

            let mut child = ui.new_child(
                UiBuilder::new()
                    .max_rect(cols.message)
                    .layout(Layout::top_down(align)),
            );
            draw_message(&mut child);

            // Three independent `Option`s, three independent sources:
            // - `cols.close_hit`   : `Option<Rect>` — the clickable region.
            // - `cols.close_glyph` : `Option<Rect>` — where to draw the glyph.
            // - `close_glyph`      : `Option<String>` — the SVG path to draw.
            if let (Some(hit), Some(glyph_rect), Some(glyph)) = (
                cols.close_hit,
                cols.close_glyph,
                close_glyph.as_deref(),
            ) {
                let hit_resp = ui.interact(hit, ui.scope_id().with("close"), Sense::click());
                if hit_resp.hovered() {
                    ui.painter().circle_filled(
                        hit.center(),
                        CLOSE_HIT * 0.5,
                        ui.visuals().widgets.hovered.weak_bg_fill,
                    );
                }
                ui.put(
                    glyph_rect,
                    MudIcon::new(glyph.to_owned())
                        .size_px(CLOSE_GLYPH)
                        .custom_color(ink),
                );
                close_clicked = hit_resp.clicked();
            }

            body_clicked = row_resp.clicked() && !close_clicked;
            row_resp
        });

        if body_clicked {
            if let Some(f) = self.on_click.as_mut() {
                f();
            }
        }
        if close_clicked {
            if let Some(f) = self.on_close.as_mut() {
                f();
            }
        }

        AlertResponse {
            response: inner.inner,
            body_clicked,
            close_clicked,
        }
    }
}

impl egui::Widget for MudAlert {
    /// Draws an alert with no content. Prefer [`MudAlert::show`] so you can read
    /// the close / body click result.
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

// ---------------------------------------------------------------------------
// AlertResponse
// ---------------------------------------------------------------------------

/// What [`MudAlert::show`] did this frame.
///
/// The two Blazor event callbacks are reported as data, the way egui reports
/// interaction: nothing is stored between frames.
#[derive(Debug)]
pub struct AlertResponse {
    /// The alert's content row. Excludes the padding, so hover is only reported
    /// over the actual content. Has `Sense::click()` when
    /// [`on_click`](MudAlert::on_click) is set.
    pub response: Response,
    /// `true` when the body was clicked this frame (and the close button was
    /// not).
    pub body_clicked: bool,
    /// `true` when the close button was clicked this frame.
    pub close_clicked: bool,
}

impl std::ops::Deref for AlertResponse {
    type Target = Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (no `egui::Context` required)
// ---------------------------------------------------------------------------

/// Padding and minimum row height for one density preset.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Density {
    pad: (f32, f32),
    min_row: f32,
}

const STANDARD_DENSITY: Density = Density {
    pad: (PADDING_X, PADDING_Y),
    min_row: MIN_ROW_HEIGHT,
};

const COMPACT_DENSITY: Density = Density {
    pad: (DENSE_PADDING_X, DENSE_PADDING_Y),
    min_row: DENSE_MIN_ROW_HEIGHT,
};

/// Resolved colours for one variant.
#[derive(Clone, Copy, Debug, PartialEq)]
struct AlertStyle {
    fill: Color32,
    stroke: Option<Stroke>,
    ink: Color32,
}

/// Geometry of the three columns inside the content row.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Columns {
    icon: Option<Rect>,
    message: Rect,
    close_hit: Option<Rect>,
    close_glyph: Option<Rect>,
}

/// Selects the density preset.
#[inline]
fn density(compact: bool) -> Density {
    if compact {
        COMPACT_DENSITY
    } else {
        STANDARD_DENSITY
    }
}

/// Maps a [`Severity`] to the [`Color`] role MudBlazor uses.
#[inline]
fn severity_color(severity: Severity) -> Color {
    match severity {
        Severity::Normal => Color::Default,
        Severity::Info => Color::Info,
        Severity::Success => Color::Success,
        Severity::Warning => Color::Warning,
        Severity::Error => Color::Error,
    }
}

/// The default glyph for a [`Severity`].
///
/// `Normal` and `Info` share `filled::INFO`, which is what MudBlazor renders.
#[inline]
fn severity_icon(severity: Severity) -> &'static str {
    use mud_egui_icons_material::filled;
    match severity {
        Severity::Normal | Severity::Info => filled::INFO,
        Severity::Success => filled::CHECK_CIRCLE,
        Severity::Warning => filled::WARNING,
        Severity::Error => filled::ERROR,
    }
}

/// The glyph to draw: the explicit override when present, else the severity's.
#[inline]
fn resolve_icon(custom: Option<&str>, severity: Severity) -> String {
    match custom {
        Some(glyph) => glyph.to_owned(),
        None => severity_icon(severity).to_owned(),
    }
}

/// Black or white, whichever reads better on `c`.
///
/// Uses the same `r + g + b > 384` threshold as `MudButton`, so a colour whose
/// channels sum to exactly `384` counts as *not* light and gets white ink.
#[inline]
fn contrast_text(c: Color32) -> Color32 {
    let sum = c.r() as u32 + c.g() as u32 + c.b() as u32;
    if sum > 384 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

/// Replaces `c`'s alpha channel, keeping its (unmultiplied) colour.
#[inline]
fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

/// Fill, border and ink for one variant.
fn resolve_style(accent: Color32, variant: Variant) -> AlertStyle {
    match variant {
        Variant::Filled => AlertStyle {
            fill: accent,
            stroke: None,
            ink: contrast_text(accent),
        },
        Variant::Outlined => AlertStyle {
            fill: Color32::TRANSPARENT,
            stroke: Some(Stroke::new(OUTLINE_STROKE_WIDTH, accent)),
            ink: accent,
        },
        _ => AlertStyle {
            fill: with_alpha(accent, TEXT_VARIANT_FILL_ALPHA),
            stroke: None,
            ink: accent,
        },
    }
}

/// Corner radius in `u8`, as `CornerRadius` expects.
#[inline]
fn corner_radius(square: bool) -> u8 {
    if square {
        0
    } else {
        radius_to_u8(DEFAULT_CORNER_RADIUS)
    }
}

/// Clamps and truncates a radius to the `u8` egui expects.
///
/// Mirrors `MudToggleGroup::radius_to_u8`: `NaN` becomes `0`, and both
/// infinities clamp to their respective bound.
#[inline]
fn radius_to_u8(radius: f32) -> u8 {
    radius.clamp(0.0, MAX_CORNER_RADIUS) as u8
}

/// Replaces a non-finite value with `fallback`.
#[inline]
fn finite_or(v: f32, fallback: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
}

/// Maps [`HorizontalAlignment`] to an egui cross-axis alignment.
///
/// `Start` / `End` flip with `right_to_left`; `Left` / `Right` never do.
#[inline]
fn resolve_alignment(alignment: HorizontalAlignment, right_to_left: bool) -> Align {
    match alignment {
        HorizontalAlignment::Left => Align::Min,
        HorizontalAlignment::Center => Align::Center,
        HorizontalAlignment::Right => Align::Max,
        HorizontalAlignment::Start if right_to_left => Align::Max,
        HorizontalAlignment::Start => Align::Min,
        HorizontalAlignment::End if right_to_left => Align::Min,
        HorizontalAlignment::End => Align::Max,
    }
}

/// Width reserved for the leading icon column (glyph plus its gap).
#[inline]
fn icon_column_width(has_icon: bool) -> f32 {
    if has_icon {
        ICON_SIZE + ICON_GAP
    } else {
        0.0
    }
}

/// Width reserved for the close column (gap plus hit area).
#[inline]
fn close_column_width(has_close: bool) -> f32 {
    if has_close {
        CLOSE_GAP + CLOSE_HIT
    } else {
        0.0
    }
}

/// Width left for the message after both side columns.
#[inline]
fn message_width(available: f32, has_icon: bool, has_close: bool) -> f32 {
    let total = icon_column_width(has_icon) + close_column_width(has_close);
    (finite_or(available, 0.0) - total).max(0.0)
}

/// Height of the content row: the tallest column, floored by the size preset.
#[inline]
fn row_height(measured: f32, has_icon: bool, has_close: bool, min_row: f32) -> f32 {
    let mut h = finite_or(measured, 0.0).max(0.0);
    if has_icon {
        h = h.max(ICON_SIZE);
    }
    if has_close {
        h = h.max(CLOSE_HIT);
    }
    h.max(min_row).max(0.0)
}

/// Places the icon, message and close columns inside `row`.
///
/// Right-to-left mirrors the order (close, message, icon); the message always
/// takes whatever is left.
fn layout_columns(row: Rect, right_to_left: bool, has_icon: bool, has_close: bool) -> Columns {
    let icon_w = icon_column_width(has_icon);
    let close_w = close_column_width(has_close);
    let msg_w = message_width(row.width(), has_icon, has_close);

    let icon_at = |x: f32| Rect::from_min_size(pos2(x, row.top()), vec2(ICON_SIZE, ICON_SIZE));
    let hit_at = |x: f32| Rect::from_min_size(pos2(x, row.top()), vec2(CLOSE_HIT, CLOSE_HIT));

    let (icon_x, message_x, close_x) = if right_to_left {
        (row.left() + close_w + msg_w, row.left() + close_w, row.left())
    } else {
        (row.left(), row.left() + icon_w, row.left() + icon_w + msg_w)
    };

    Columns {
        icon: has_icon.then(|| icon_at(icon_x)),
        message: Rect::from_min_size(pos2(message_x, row.top()), vec2(msg_w, row.height())),
        close_hit: has_close.then(|| hit_at(close_x)),
        close_glyph: has_close.then(|| close_glyph_rect(hit_at(close_x))),
    }
}

/// The square that a close glyph occupies, centred in its hit area.
#[inline]
fn close_glyph_rect(hit: Rect) -> Rect {
    let inset = (CLOSE_HIT - CLOSE_GLYPH) * 0.5;
    Rect::from_min_size(
        pos2(hit.left() + inset, hit.top() + inset),
        vec2(CLOSE_GLYPH, CLOSE_GLYPH),
    )
}

/// Drop shadow for an elevation level, clamped to [`MAX_ELEVATION`].
fn elevation_shadow(elevation: u8) -> Shadow {
    let e = elevation.min(MAX_ELEVATION) as f32;
    Shadow {
        offset: [0, (e * 0.5) as i8],
        blur: (e * 1.5) as u8,
        spread: 0,
        color: Color32::from_black_alpha(((e * 6.0) as u8).min(96)),
    }
}

/// Runs `draw` into an invisible probe `Ui` of width `width` and returns the
/// height it used.
fn measure_message(ui: &mut Ui, draw: &mut dyn FnMut(&mut Ui), width: f32, align: Align) -> f32 {
    let origin = ui.max_rect().min;
    let probe_rect = Rect::from_min_size(
        origin,
        vec2(finite_or(width, 0.0).max(1.0), MAX_PROBE_HEIGHT),
    );
    let mut probe = ui.new_child(
        UiBuilder::new()
            .max_rect(probe_rect)
            .layout(Layout::top_down(align))
            .invisible(),
    );
    draw(&mut probe);
    finite_or(probe.min_rect().height(), 0.0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn accent() -> Color32 {
        Color32::from_rgb(200, 30, 30)
    }

    fn row() -> Rect {
        Rect::from_min_size(pos2(10.0, 20.0), vec2(400.0, 40.0))
    }

    // --- severity mapping -------------------------------------------------

    #[test]
    fn severity_color_maps_every_level() {
        assert!(matches!(severity_color(Severity::Normal), Color::Default));
        assert!(matches!(severity_color(Severity::Info), Color::Info));
        assert!(matches!(severity_color(Severity::Success), Color::Success));
        assert!(matches!(severity_color(Severity::Warning), Color::Warning));
        assert!(matches!(severity_color(Severity::Error), Color::Error));
    }

    #[test]
    fn severity_icon_is_non_empty_and_normal_shares_info() {
        for s in Severity::ALL {
            assert!(!severity_icon(s).is_empty());
        }
        assert_eq!(severity_icon(Severity::Normal), severity_icon(Severity::Info));
        assert_ne!(
            severity_icon(Severity::Success),
            severity_icon(Severity::Error)
        );
    }

    #[test]
    fn resolve_icon_prefers_custom_glyph() {
        assert_eq!(
            resolve_icon(Some("M0 0"), Severity::Error),
            "M0 0".to_string()
        );
        assert_eq!(
            resolve_icon(None, Severity::Error),
            severity_icon(Severity::Error).to_string()
        );
    }

    // --- colour helpers ---------------------------------------------------

    #[test]
    fn contrast_text_picks_black_on_light_and_white_on_dark() {
        assert_eq!(contrast_text(Color32::WHITE), Color32::BLACK);
        assert_eq!(contrast_text(Color32::BLACK), Color32::WHITE);
        assert_eq!(contrast_text(Color32::from_rgb(255, 220, 0)), Color32::BLACK);
        assert_eq!(contrast_text(Color32::from_rgb(20, 40, 60)), Color32::WHITE);
    }

    #[test]
    fn contrast_text_threshold_boundary_is_not_light() {
        // 128 * 3 == 384 exactly, which counts as dark → white ink.
        assert_eq!(
            contrast_text(Color32::from_rgb(128, 128, 128)),
            Color32::WHITE
        );
        assert_eq!(
            contrast_text(Color32::from_rgb(128, 128, 129)),
            Color32::BLACK
        );
    }

    #[test]
    fn with_alpha_preserves_rgb_and_sets_alpha() {
        let c = with_alpha(Color32::from_rgb(10, 20, 30), 40);
        assert_eq!((c.r(), c.g(), c.b(), c.a()), (10, 20, 30, 40));
        assert_eq!(with_alpha(c, 255).a(), 255);
        assert_eq!(with_alpha(c, 0).a(), 0);
    }

    // --- variant resolution ----------------------------------------------

    #[test]
    fn text_variant_is_tinted_without_a_stroke() {
        let style = resolve_style(accent(), Variant::Text);
        assert_eq!(style.fill.a(), TEXT_VARIANT_FILL_ALPHA);
        assert_eq!((style.fill.r(), style.fill.g(), style.fill.b()), (200, 30, 30));
        assert!(style.stroke.is_none());
        assert_eq!(style.ink, accent());
    }

    #[test]
    fn outlined_variant_is_transparent_with_an_accent_stroke() {
        let style = resolve_style(accent(), Variant::Outlined);
        assert_eq!(style.fill, Color32::TRANSPARENT);
        assert_eq!(style.ink, accent());
        let stroke = style.stroke.expect("outlined must draw a border");
        assert_eq!(stroke.width, OUTLINE_STROKE_WIDTH);
        assert_eq!(stroke.color, accent());
    }

    #[test]
    fn filled_variant_uses_the_accent_and_contrast_ink() {
        let dark = Color32::from_rgb(10, 10, 10);
        let style = resolve_style(dark, Variant::Filled);
        assert_eq!(style.fill, dark);
        assert_eq!(style.ink, Color32::WHITE);
        assert!(style.stroke.is_none());

        let light = Color32::from_rgb(250, 250, 250);
        assert_eq!(resolve_style(light, Variant::Filled).ink, Color32::BLACK);
    }

    // --- density ----------------------------------------------------------

    #[test]
    fn density_preset_has_expected_padding_and_min_row() {
        assert_eq!(density(false), STANDARD_DENSITY);
        assert_eq!(density(true), COMPACT_DENSITY);
        assert!(COMPACT_DENSITY.pad.0 < STANDARD_DENSITY.pad.0);
        assert!(COMPACT_DENSITY.pad.1 < STANDARD_DENSITY.pad.1);
        assert!(COMPACT_DENSITY.min_row < STANDARD_DENSITY.min_row);
    }

    #[test]
    fn corner_radius_zero_when_square() {
        assert_eq!(corner_radius(true), 0);
        assert_eq!(corner_radius(false), 4);
    }

    #[test]
    fn radius_to_u8_clamps_and_truncates() {
        assert_eq!(radius_to_u8(4.0), 4);
        assert_eq!(radius_to_u8(4.9), 4);
        assert_eq!(radius_to_u8(-1.0), 0);
        assert_eq!(radius_to_u8(300.0), 255);
        assert_eq!(radius_to_u8(f32::NAN), 0);
        assert_eq!(radius_to_u8(f32::INFINITY), 255);
        assert_eq!(radius_to_u8(f32::NEG_INFINITY), 0);
    }

    #[test]
    fn finite_or_repairs_non_finite_values() {
        assert_eq!(finite_or(1.5, 9.0), 1.5);
        assert_eq!(finite_or(f32::NAN, 9.0), 9.0);
        assert_eq!(finite_or(f32::INFINITY, 9.0), 9.0);
    }

    // --- alignment --------------------------------------------------------

    #[test]
    fn resolve_alignment_maps_physical_edges() {
        assert_eq!(resolve_alignment(HorizontalAlignment::Left, false), Align::Min);
        assert_eq!(resolve_alignment(HorizontalAlignment::Left, true), Align::Min);
        assert_eq!(
            resolve_alignment(HorizontalAlignment::Center, false),
            Align::Center
        );
        assert_eq!(resolve_alignment(HorizontalAlignment::Right, false), Align::Max);
        assert_eq!(resolve_alignment(HorizontalAlignment::Right, true), Align::Max);
    }

    #[test]
    fn resolve_alignment_flips_logical_edges_in_rtl() {
        assert_eq!(resolve_alignment(HorizontalAlignment::Start, false), Align::Min);
        assert_eq!(resolve_alignment(HorizontalAlignment::Start, true), Align::Max);
        assert_eq!(resolve_alignment(HorizontalAlignment::End, false), Align::Max);
        assert_eq!(resolve_alignment(HorizontalAlignment::End, true), Align::Min);
    }

    // --- column widths ----------------------------------------------------

    #[test]
    fn column_width_truth_table() {
        assert_eq!(icon_column_width(false), 0.0);
        assert_eq!(icon_column_width(true), ICON_SIZE + ICON_GAP);
        assert_eq!(close_column_width(false), 0.0);
        assert_eq!(close_column_width(true), CLOSE_GAP + CLOSE_HIT);
    }

    #[test]
    fn message_width_subtracts_columns_and_never_goes_negative() {
        assert_eq!(message_width(400.0, false, false), 400.0);
        assert_eq!(message_width(400.0, true, false), 400.0 - 34.0);
        assert_eq!(message_width(400.0, false, true), 400.0 - 36.0);
        assert_eq!(message_width(400.0, true, true), 400.0 - 70.0);
        assert_eq!(message_width(10.0, true, true), 0.0);
        assert_eq!(message_width(f32::NAN, true, true), 0.0);
    }

    #[test]
    fn row_height_respects_content_and_floors() {
        assert_eq!(row_height(0.0, false, false, MIN_ROW_HEIGHT), MIN_ROW_HEIGHT);
        assert_eq!(
            row_height(0.0, false, false, DENSE_MIN_ROW_HEIGHT),
            DENSE_MIN_ROW_HEIGHT
        );
        assert_eq!(row_height(0.0, true, false, DENSE_MIN_ROW_HEIGHT), ICON_SIZE);
        assert_eq!(row_height(0.0, false, true, DENSE_MIN_ROW_HEIGHT), CLOSE_HIT);
        assert_eq!(row_height(120.0, true, true, MIN_ROW_HEIGHT), 120.0);
        assert_eq!(row_height(f32::NAN, true, true, MIN_ROW_HEIGHT), CLOSE_HIT);
        assert_eq!(row_height(-5.0, false, false, MIN_ROW_HEIGHT), MIN_ROW_HEIGHT);
    }

    // --- column layout ----------------------------------------------------

    #[test]
    fn layout_columns_orders_left_to_right() {
        let cols = layout_columns(row(), false, true, true);
        let icon = cols.icon.expect("icon");
        let close = cols.close_hit.expect("close");
        assert_eq!(icon.left(), row().left());
        assert_eq!(icon.right() + ICON_GAP, cols.message.left());
        assert_eq!(cols.message.right() + CLOSE_GAP, close.left());
        assert_eq!(close.right(), row().right());
        assert_eq!(icon.top(), row().top());
        assert_eq!(cols.message.height(), row().height());
    }

    #[test]
    fn layout_columns_mirrors_right_to_left() {
        let cols = layout_columns(row(), true, true, true);
        let icon = cols.icon.expect("icon");
        let close = cols.close_hit.expect("close");
        assert_eq!(close.left(), row().left());
        assert_eq!(icon.right(), row().right());
        assert!(close.right() < cols.message.left());
        assert!(cols.message.right() <= icon.left());
    }

    #[test]
    fn layout_columns_omits_absent_columns() {
        let none = layout_columns(row(), false, false, false);
        assert!(none.icon.is_none());
        assert!(none.close_hit.is_none());
        assert!(none.close_glyph.is_none());
        assert_eq!(none.message, row());

        let icon_only = layout_columns(row(), false, true, false);
        assert!(icon_only.icon.is_some());
        assert!(icon_only.close_hit.is_none());
        assert_eq!(icon_only.message.right(), row().right());
    }

    #[test]
    fn layout_columns_tiles_the_row_exactly() {
        for rtl in [false, true] {
            for has_icon in [false, true] {
                for has_close in [false, true] {
                    let cols = layout_columns(row(), rtl, has_icon, has_close);
                    let span = icon_column_width(has_icon)
                        + cols.message.width()
                        + close_column_width(has_close);
                    assert!((span - row().width()).abs() < f32::EPSILON);
                }
            }
        }
    }

    #[test]
    fn close_glyph_is_centred_in_its_hit_area() {
        let cols = layout_columns(row(), false, false, true);
        let hit = cols.close_hit.expect("close");
        let glyph = cols.close_glyph.expect("glyph");
        assert_eq!(glyph.center(), hit.center());
        assert_eq!(glyph.width(), CLOSE_GLYPH);
    }

    // --- elevation --------------------------------------------------------

    #[test]
    fn elevation_shadow_is_a_noop_at_zero_and_clamps() {
        let zero = elevation_shadow(0);
        assert_eq!(zero.offset, [0, 0]);
        assert_eq!(zero.blur, 0);
        assert_eq!(zero.color.a(), 0);

        assert_eq!(elevation_shadow(40), elevation_shadow(MAX_ELEVATION));
        assert!(elevation_shadow(8).blur > elevation_shadow(4).blur);
        assert!(elevation_shadow(8).color.a() > elevation_shadow(4).color.a());
    }

    // --- builders ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let a = MudAlert::new("hi");
        assert_eq!(a.text, "hi");
        assert!(a.content.is_none());
        assert!(matches!(a.severity, Severity::Normal));
        assert!(matches!(a.variant, Variant::Text));
        assert!(!a.dense);
        assert!(!a.square);
        assert_eq!(a.elevation, 0);
        assert!(!a.no_icon);
        assert!(a.icon.is_none());
        assert!(a.close_icon.is_none());
        assert!(!a.show_close_icon);
        assert!(matches!(a.content_alignment, HorizontalAlignment::Left));
        assert!(!a.right_to_left);
        assert!(a.on_click.is_none());
        assert!(a.on_close.is_none());
        assert!(a.tag.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let a = MudAlert::new("start")
            .text("message")
            .content(|ui| {
                ui.add(MudText::new("custom"));
            })
            .severity(Severity::Warning)
            .variant(Variant::Outlined)
            .dense(true)
            .square(true)
            .elevation(6)
            .no_icon(true)
            .icon("M0 0")
            .close_icon("M1 1")
            .show_close_icon(true)
            .content_alignment(HorizontalAlignment::End)
            .right_to_left(true)
            .on_click(|| {})
            .on_close(|| {})
            .classes(["my-2"]);

        assert_eq!(a.text, "message");
        assert!(a.content.is_some());
        assert!(matches!(a.severity, Severity::Warning));
        assert!(matches!(a.variant, Variant::Outlined));
        assert!(a.dense && a.square && a.no_icon && a.show_close_icon);
        assert_eq!(a.elevation, 6);
        assert_eq!(a.icon.as_deref(), Some("M0 0"));
        assert_eq!(a.close_icon.as_deref(), Some("M1 1"));
        assert!(matches!(a.content_alignment, HorizontalAlignment::End));
        assert!(a.right_to_left);
        assert!(a.on_click.is_some());
        assert!(a.on_close.is_some());
        assert!(!a.base.class_names().is_empty());
    }

    #[test]
    fn tag_round_trips_by_type() {
        let a = MudAlert::new("x").tag("page-banner");
        assert_eq!(a.tag_ref::<&str>(), Some(&"page-banner"));
        assert!(a.tag_ref::<u32>().is_none());
    }
}