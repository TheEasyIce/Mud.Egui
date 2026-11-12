//! A file picker and drag-and-drop surface.
//!
//! Port of `MudFileUpload<T>.razor.cs` (MudBlazor 8.13.0). The public
//! parameter surface mirrors the Blazor component exactly:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `Accept` | [`MudFileUpload::accept`] |
//! | `ActivatorContent` | [`MudFileUpload::activator_content`] |
//! | `AppendMultipleFiles` | [`MudFileUpload::append_multiple_files`] |
//! | `Disabled` | [`MudFileUpload::disabled`] |
//! | `Files` | the caller-owned `&mut Vec<PathBuf>` |
//! | `FilesChanged` | [`FileUploadResponse::changed`] |
//! | `Hidden` | [`MudFileUpload::hidden`] (default `true`) |
//! | `MaxFileSize` | [`MudFileUpload::max_file_size`] |
//! | `MaximumFileCount` | [`MudFileUpload::maximum_file_count`] (default `10`) |
//! | `Class` | [`MudFileUpload::classes`] (stored, not resolved) |
//! | `Tag` | [`MudFileUpload::tag`] / [`MudFileUpload::tag_ref`] |
//! | `InputClass` / `InputStyle` | not ported: egui has no CSS strings |
//! | `Style` / `UserAttributes` | not ported: no CSS strings, no HTML tag |
//!
//! # `Hidden` is the mode switch
//!
//! | `hidden` | Mode |
//! |----------|------|
//! | `true` (default) | Button only. Opens the OS file picker on click. |
//! | `false` | Button **and** a dashed drop zone. Files can be dropped anywhere on the component. |
//!
//! # `MaximumFileCount` is per-pick
//!
//! The docs are explicit: it caps the files returned by **one** picker call, not
//! the total list. `apply_selection` does not enforce a total cap; check
//! `files.len()` yourself if you need one.
//!
//! # Blocking file dialog
//!
//! `rfd::FileDialog::pick_files()` blocks the UI thread. The README documents
//! the async alternative (`rfd::AsyncFileDialog` on a worker thread with a
//! channel); this file keeps the synchronous call for simplicity and because it
//! is what the pre-existing port did.
//!
//! # Performance
//!
//! - Theme fetched once per frame, and only when a colour is needed.
//! - Extension parsing runs once per frame; an empty `accept` short-circuits.
//! - Every validation decision is a pure function over `&Path`, `&[String]` and
//!   `Option<u64>`.
//! - The chip list is built only when `files` is non-empty and the component is
//!   not in drag-and-drop mode.
//! - Animations (`animate_bool_with_time`) cost nothing when idle.

use std::path::{Path, PathBuf};

use egui::{
    pos2, vec2, Align2, Color32, CornerRadius, CursorIcon, FontId, Id, Pos2, Rect, Response,
    Sense, Stroke, StrokeKind, Ui, WidgetInfo, WidgetType,
};
use epaint::Shadow;

use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, Size, Variant};
use mud_egui_theming::theme::MudTheme;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum button width, in points.
const MIN_BUTTON_WIDTH: f32 = 200.0;

/// Drop-zone height at `Size::Medium`, in points.
const DROP_ZONE_HEIGHT: f32 = 180.0;

/// Corner radius of the button and drop zone, in points.
const CORNER_RADIUS: f32 = 4.0;

/// Animation duration for hover/focus transitions, in seconds.
const ANIM_SECONDS: f32 = 0.12;

/// Dash length and gap for the drop-zone border, in points.
const DASH_LEN: f32 = 8.0;
const DASH_GAP: f32 = 6.0;

// ---------------------------------------------------------------------------
// MudFileUpload
// ---------------------------------------------------------------------------

/// A file picker and drag-and-drop surface.
///
/// Mirrors `MudFileUpload<T>` in MudBlazor 8.13.0. The Blazor two-way binding
/// `Files` / `FilesChanged` becomes a caller-owned `&mut Vec<PathBuf>` plus a
/// [`FileUploadResponse::changed`] report, the same split `MudAlert` and
/// `MudCollapse` use.
#[must_use = "MudFileUpload does nothing until you call `.show(ui)`"]
pub struct MudFileUpload<'a> {
    /// Shared component state (utility classes). Stored for parity.
    pub base: ComponentBase,
    /// The bound file list. Mirrors MudBlazor `Files`.
    pub files: &'a mut Vec<PathBuf>,
    /// Comma-separated extensions, e.g. `".png, .jpg"`. Mirrors `Accept`.
    pub accept: Option<&'a str>,
    /// Appends to the list instead of replacing it. Mirrors
    /// `AppendMultipleFiles`.
    pub append_multiple_files: bool,
    /// Prevents interaction. Mirrors `Disabled`.
    pub disabled: bool,
    /// `true` = button only; `false` = button **and** drop zone. Mirrors
    /// `Hidden` (default `true`).
    pub hidden: bool,
    /// Max bytes per file. Mirrors `MaxFileSize`.
    pub max_file_size: Option<u64>,
    /// Max files returned by one picker call. Mirrors `MaximumFileCount`
    /// (default `10`).
    pub maximum_file_count: usize,
    /// Whether the component allows multiple files at all. Derived from the
    /// binding type in MudBlazor; here it is set explicitly with `single()`.
    pub multiple: bool,
    /// Visual variant of the button. Mirrors the `MudButton`-style variant
    /// MudBlazor uses for the activator.
    pub variant: Variant,
    /// Accent colour. Mirrors the `MudButton`-style colour.
    pub color: Color,
    /// Size preset. Mirrors the `MudButton`-style size.
    pub size: Size,
    /// Shadow depth (0–24).
    pub elevation: u8,
    /// Corner radius override.
    pub corner_radius: f32,
    /// Accessible label. Extension.
    pub aria_label: Option<&'a str>,
    /// Button label. Extension (MudBlazor uses `ActivatorContent`).
    pub label: Option<&'a str>,
    /// Custom activator content. Mirrors `ActivatorContent`.
    pub activator_content: Option<Box<dyn FnOnce(&mut Ui) -> Response + 'a>>,
    /// Arbitrary user data. Mirrors `Tag`.
    pub tag: Option<Box<dyn std::any::Any + Send + Sync>>,
}

impl<'a> MudFileUpload<'a> {
    /// Creates an upload bound to `files` with MudBlazor’s default settings.
    pub fn new(files: &'a mut Vec<PathBuf>) -> Self {
        Self {
            base: ComponentBase::new(),
            files,
            accept: None,
            append_multiple_files: false,
            disabled: false,
            hidden: true,
            max_file_size: None,
            maximum_file_count: 10,
            multiple: true,
            variant: Variant::Filled,
            color: Color::Primary,
            size: Size::Medium,
            elevation: 0,
            corner_radius: CORNER_RADIUS,
            aria_label: None,
            label: None,
            activator_content: None,
            tag: None,
        }
    }

    /// Restricts to one file. Mirrors the `T = IBrowserFile` case.
    #[inline]
    pub fn single(mut self) -> Self {
        self.multiple = false;
        self
    }

    /// Sets the accepted extensions. Mirrors `Accept`.
    #[inline]
    pub fn accept(mut self, s: &'a str) -> Self {
        self.accept = Some(s);
        self
    }

    /// Appends instead of replacing. Mirrors `AppendMultipleFiles`.
    #[inline]
    pub fn append_multiple_files(mut self, b: bool) -> Self {
        self.append_multiple_files = b;
        self
    }

    /// Disables interaction. Mirrors `Disabled`.
    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// `true` = button only; `false` = button **and** drop zone. Mirrors
    /// `Hidden`.
    #[inline]
    pub fn hidden(mut self, b: bool) -> Self {
        self.hidden = b;
        self
    }

    /// Caps bytes per file. Mirrors `MaxFileSize`.
    #[inline]
    pub fn max_file_size(mut self, bytes: u64) -> Self {
        self.max_file_size = Some(bytes);
        self
    }

    /// Caps the files returned by one picker call. Mirrors
    /// `MaximumFileCount`; clamped to at least `1`.
    #[inline]
    pub fn maximum_file_count(mut self, n: usize) -> Self {
        self.maximum_file_count = n.max(1);
        self
    }

    /// Sets the button variant.
    #[inline]
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    /// Sets the accent colour.
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Sets the size preset.
    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    /// Sets the shadow depth (clamped to 24).
    #[inline]
    pub fn elevation(mut self, e: u8) -> Self {
        self.elevation = e.min(24);
        self
    }

    /// Sets the corner radius.
    #[inline]
    pub fn corner_radius(mut self, r: f32) -> Self {
        self.corner_radius = r;
        self
    }

    /// Sets the button label.
    #[inline]
    pub fn label(mut self, s: &'a str) -> Self {
        self.label = Some(s);
        self
    }

    /// Sets the accessible label.
    #[inline]
    pub fn aria_label(mut self, s: &'a str) -> Self {
        self.aria_label = Some(s);
        self
    }

    /// Replaces the button content. Mirrors `ActivatorContent`.
    #[inline]
    pub fn activator_content(mut self, f: impl FnOnce(&mut Ui) -> Response + 'a) -> Self {
        self.activator_content = Some(Box::new(f));
        self
    }

    /// Adds utility classes. Stored for parity.
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Attaches arbitrary user data. Mirrors `Tag`.
    #[inline]
    pub fn tag<T: std::any::Any + Send + Sync>(mut self, tag: T) -> Self {
        self.tag = Some(Box::new(tag));
        self
    }

    /// Reads user data attached with [`tag`](Self::tag).
    #[inline]
    pub fn tag_ref<T: std::any::Any + Send + Sync>(&self) -> Option<&T> {
        self.tag.as_ref().and_then(|b| b.downcast_ref::<T>())
    }

    /// Draws the component and reports this frame’s outcome.
    pub fn show(mut self, ui: &mut Ui) -> FileUploadResponse {
        let id = ui.id().with("file_upload");
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let accent = self
            .color
            .resolve(palette)
            .unwrap_or_else(|| ui.visuals().selection.bg_fill);

        let extensions = parse_extensions(self.accept);
        let metrics = SizeMetrics::resolve(self.size);

        // --- render surface ------------------------------------------------
        let resp = if self.hidden {
            self.draw_button(ui, id, &metrics, accent)
        } else {
            self.draw_drop_zone(ui, id, &metrics, accent)
        };

        // --- picker --------------------------------------------------------
        let mut changed = false;
        if !self.disabled && resp.clicked() {
            if let Some(picked) = pick_files(self.multiple, &extensions) {
                let (valid, _err) =
                    filter_files(&picked, &extensions, self.max_file_size, self.maximum_file_count);
                if !valid.is_empty() {
                    apply_selection(
                        self.files,
                        valid,
                        self.multiple,
                        self.append_multiple_files,
                    );
                    changed = true;
                }
            }
        }

        // --- drag and drop (only when Hidden = false) ----------------------
        if !self.disabled && !self.hidden {
            let dropped: Vec<PathBuf> = ui.ctx().input(|i| {
                i.raw
                    .dropped_files
                    .iter()
                    .map(|d| d.path().to_path_buf())
                    .collect()
            });
            if !dropped.is_empty() && resp.contains_pointer() {
                let (valid, _err) = filter_files(
                    &dropped,
                    &extensions,
                    self.max_file_size,
                    self.maximum_file_count,
                );
                if !valid.is_empty() {
                    apply_selection(
                        self.files,
                        valid,
                        self.multiple,
                        self.append_multiple_files,
                    );
                    changed = true;
                }
            }
        }

        // --- chips ---------------------------------------------------------
        if !self.files.is_empty() && self.hidden {
            ui.add_space(metrics.padding_y);
            let mut to_remove = Vec::new();
            ui.horizontal_wrapped(|ui| {
                for (index, path) in self.files.iter().enumerate() {
                    if draw_chip(ui, path, accent, self.disabled, &metrics) {
                        to_remove.push(index);
                    }
                }
            });
            if !self.disabled && !to_remove.is_empty() {
                for index in to_remove.into_iter().rev() {
                    self.files.remove(index);
                }
                changed = true;
            }
        }

        // --- accessibility -------------------------------------------------
        if let Some(label) = self.aria_label {
            let enabled = !self.disabled;
            resp.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, label));
        }

        FileUploadResponse { response: resp, changed }
    }

    // ---------------------------------------------------------------------
    // Rendering
    // ---------------------------------------------------------------------

    fn draw_button(
        &mut self,
        ui: &mut Ui,
        id: Id,
        metrics: &SizeMetrics,
        accent: Color32,
    ) -> Response {
        let alpha = if self.disabled { 0.4 } else { 1.0 };
        let width = ui.available_width().min(MIN_BUTTON_WIDTH).max(80.0);
        let (rect, resp) = ui.allocate_exact_size(vec2(width, metrics.height), Sense::click());

        let hover = ui
            .ctx()
            .animate_bool_with_time(id.with("hover"), resp.hovered() && !self.disabled, ANIM_SECONDS);
        let focus = ui
            .ctx()
            .animate_bool_with_time(id.with("focus"), resp.has_focus() && !self.disabled, ANIM_SECONDS);

        let fill = match self.variant {
            Variant::Filled => lerp_color(
                accent.gamma_multiply(alpha),
                accent.gamma_multiply(alpha).gamma_multiply(0.85),
                hover,
            ),
            _ => Color32::TRANSPARENT,
        };
        let radius = CornerRadius::same(radius_u8(self.corner_radius));

        // --- shadow (painted behind) ---------------------------------------
        // `RectShape::with_shadow` does not exist; a shadow is a `Shadow`
        // turned into its own `RectShape` via `as_shape`. Paint it first, then
        // the fill on top.
        if self.elevation > 0 && matches!(self.variant, Variant::Filled) && !self.disabled {
            let shadow = Shadow {
                offset: [0, self.elevation as i8],
                blur: (self.elevation * 2) as u8,
                spread: 0,
                color: Color32::from_black_alpha((self.elevation * 4).min(80)),
            };
            ui.painter().add(shadow.as_shape(rect, radius));
        }
        ui.painter().rect_filled(rect, radius, fill);

        if matches!(self.variant, Variant::Outlined) {
            let stroke = Stroke::new(if focus > 0.0 { 2.0 } else { 1.0 }, accent.gamma_multiply(alpha));
            ui.painter()
                .rect_stroke(rect, radius, stroke, StrokeKind::Inside);
        }

        if focus > 0.0 && !self.disabled {
            ui.painter().rect_stroke(
                rect.expand(focus * 3.0),
                radius,
                Stroke::new(2.0, accent.gamma_multiply(0.5 * focus)),
                StrokeKind::Outside,
            );
        }

        if let Some(content) = self.activator_content.take() {
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(*ui.layout()),
            );
            content(&mut child);
        } else {
            let text = self.label.unwrap_or("Browse files");
            let ink = match self.variant {
                Variant::Filled => contrast_text(accent).gamma_multiply(alpha),
                _ => accent.gamma_multiply(alpha),
            };
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                text,
                FontId::proportional(metrics.font_size),
                ink,
            );
        }

        if self.disabled {
            resp.on_hover_cursor(CursorIcon::NotAllowed)
        } else {
            resp.on_hover_cursor(CursorIcon::PointingHand)
        }
    }

    fn draw_drop_zone(
        &mut self,
        ui: &mut Ui,
        id: Id,
        metrics: &SizeMetrics,
        accent: Color32,
    ) -> Response {
        let width = ui.available_width().max(250.0);
        let height = DROP_ZONE_HEIGHT * (metrics.height / 36.0);
        let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click());

        let hovering = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
        let anim = ui
            .ctx()
            .animate_bool_with_time(id.with("drop"), hovering, ANIM_SECONDS);

        let stroke_color = lerp_color(
            Color32::from_gray(120),
            accent,
            if self.disabled { 0.0 } else { anim },
        );
        let inner = rect.shrink(4.0 + anim * 2.0);

        draw_dashed_rect(
            ui.painter(),
            inner,
            stroke_color,
            2.0,
            DASH_LEN,
            DASH_GAP,
            self.corner_radius,
        );

        let text = self.label.unwrap_or("Drag files here or click to browse");
        let ink = ui.visuals().text_color().gamma_multiply(if self.disabled { 0.4 } else { 1.0 });
        ui.painter().text(
            inner.center(),
            Align2::CENTER_CENTER,
            text,
            FontId::proportional(metrics.font_size * 1.1),
            ink,
        );

        if self.disabled {
            resp.on_hover_cursor(CursorIcon::NotAllowed)
        } else {
            resp.on_hover_cursor(CursorIcon::PointingHand)
        }
    }
}

impl<'a> egui::Widget for MudFileUpload<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

// ---------------------------------------------------------------------------
// FileUploadResponse
// ---------------------------------------------------------------------------

/// What [`MudFileUpload::show`] did this frame.
#[derive(Debug)]
pub struct FileUploadResponse {
    /// The component’s `Response`.
    pub response: Response,
    /// `true` on the frame the file list changed.
    pub changed: bool,
}

impl std::ops::Deref for FileUploadResponse {
    type Target = Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

// ---------------------------------------------------------------------------
// SizeMetrics
// ---------------------------------------------------------------------------

/// Height and padding for a [`Size`].
#[derive(Clone, Copy, Debug, PartialEq)]
struct SizeMetrics {
    height: f32,
    font_size: f32,
    padding_y: f32,
}

impl SizeMetrics {
    fn resolve(size: Size) -> Self {
        match size {
            Size::ExtraSmall | Size::Small => Self {
                height: 28.0,
                font_size: 12.0,
                padding_y: 4.0,
            },
            Size::Medium => Self {
                height: 36.0,
                font_size: 14.0,
                padding_y: 6.0,
            },
            Size::Large | Size::ExtraLarge => Self {
                height: 48.0,
                font_size: 16.0,
                padding_y: 8.0,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Parses a comma-separated extension list into lowercase, dot-prefixed
/// strings. Non-dot tokens are ignored.
fn parse_extensions(accept: Option<&str>) -> Vec<String> {
    accept
        .map(|s| {
            s.split(',')
                .map(|t| t.trim().to_lowercase())
                .filter(|t| t.starts_with('.') && t.len() > 1)
                .collect()
        })
        .unwrap_or_default()
}

/// `true` when `path`’s extension is in `extensions`, or when the list is
/// empty.
fn matches_accept(path: &Path, extensions: &[String]) -> bool {
    if extensions.is_empty() {
        return true;
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => extensions.contains(&format!(".{}", ext.to_lowercase())),
        None => false,
    }
}

/// Filters `paths` by extension and size, and truncates the batch to
/// `maximum_file_count`. Returns the valid subset; the error string is `None`
/// because MudBlazor surfaces per-file failures through validation, not a
/// single error.
fn filter_files(
    paths: &[PathBuf],
    extensions: &[String],
    max_size: Option<u64>,
    maximum_file_count: usize,
) -> (Vec<PathBuf>, Option<String>) {
    let limit = maximum_file_count.max(1);
    let mut valid = Vec::with_capacity(paths.len().min(limit));
    for path in paths {
        if valid.len() >= limit {
            break;
        }
        if !matches_accept(path, extensions) {
            continue;
        }
        if let Some(max) = max_size {
            let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            if size > max {
                continue;
            }
        }
        valid.push(path.clone());
    }
    (valid, None)
}

/// Applies `picked` to `files` according to `multiple` and `append`.
fn apply_selection(
    files: &mut Vec<PathBuf>,
    picked: Vec<PathBuf>,
    multiple: bool,
    append: bool,
) {
    if multiple {
        if append {
            files.extend(picked);
        } else {
            *files = picked;
        }
    } else if let Some(first) = picked.into_iter().next() {
        *files = vec![first];
    }
}

/// Opens the OS file picker. **Blocks the UI thread.**
fn pick_files(multiple: bool, extensions: &[String]) -> Option<Vec<PathBuf>> {
    let mut dialog = rfd::FileDialog::new().set_title("Select files");
    if !extensions.is_empty() {
        let bare: Vec<&str> = extensions.iter().map(|s| s.trim_start_matches('.')).collect();
        dialog = dialog.add_filter("Allowed types", &bare);
    }
    if multiple {
        dialog.pick_files()
    } else {
        dialog.pick_file().map(|p| vec![p])
    }
}

/// Draws one removable chip and returns `true` when the × was clicked.
fn draw_chip(
    ui: &mut Ui,
    path: &Path,
    accent: Color32,
    disabled: bool,
    metrics: &SizeMetrics,
) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
    let mut remove = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let _ = ui.add(
            egui::Button::new(
                egui::RichText::new(name)
                    .color(contrast_text(accent))
                    .size(metrics.font_size * 0.9),
            )
                .fill(accent)
                .min_size(vec2(0.0, metrics.height * 0.7))
                .corner_radius(CornerRadius {
                    nw: 12,
                    ne: 0,
                    sw: 12,
                    se: 0,
                }),
        );
        let close = ui.add_enabled(
            !disabled,
            egui::Button::new(
                egui::RichText::new("×")
                    .color(contrast_text(accent))
                    .size(metrics.font_size),
            )
                .fill(accent)
                .min_size(vec2(metrics.height * 0.7, metrics.height * 0.7))
                .corner_radius(CornerRadius {
                    nw: 0,
                    ne: 12,
                    sw: 0,
                    se: 12,
                }),
        );
        if close.clicked() {
            remove = true;
        }
    });
    remove
}

/// Draws a dashed rounded rectangle.
fn draw_dashed_rect(
    painter: &egui::Painter,
    rect: Rect,
    color: Color32,
    thickness: f32,
    dash_len: f32,
    gap: f32,
    corner_radius: f32,
) {
    let stroke = Stroke::new(thickness, color);
    let r = corner_radius.max(0.0).min(rect.width().min(rect.height()) * 0.5);
    let edges = [
        (pos2(rect.left() + r, rect.top()), pos2(rect.right() - r, rect.top())),
        (pos2(rect.right(), rect.top() + r), pos2(rect.right(), rect.bottom() - r)),
        (pos2(rect.right() - r, rect.bottom()), pos2(rect.left() + r, rect.bottom())),
        (pos2(rect.left(), rect.bottom() - r), pos2(rect.left(), rect.top() + r)),
    ];
    for (start, end) in edges {
        draw_dashed_line(painter, start, end, stroke, dash_len, gap);
    }
}

/// Draws one dashed line segment.
fn draw_dashed_line(
    painter: &egui::Painter,
    start: Pos2,
    end: Pos2,
    stroke: Stroke,
    dash_len: f32,
    gap: f32,
) {
    let total = (end - start).length();
    if total <= 0.0 {
        return;
    }
    let dir = (end - start) / total;
    let mut t = 0.0;
    while t < total {
        let t_end = (t + dash_len).min(total);
        painter.line_segment([start + dir * t, start + dir * t_end], stroke);
        t = t_end + gap;
    }
}

/// Linear interpolation between two colours.
fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| (x as f32 * (1.0 - t) + y as f32 * t) as u8;
    Color32::from_rgba_unmultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}

/// Black or white, whichever reads better on `c`.
fn contrast_text(c: Color32) -> Color32 {
    let sum = c.r() as u32 + c.g() as u32 + c.b() as u32;
    if sum > 384 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

/// Clamps a pixel value into `u8` for `CornerRadius`.
fn radius_u8(px: f32) -> u8 {
    if px.is_finite() {
        px.clamp(0.0, 255.0) as u8
    } else {
        0
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_extensions_normalizes_and_filters() {
        assert_eq!(
            parse_extensions(Some(".png, .JPG ,.gif")),
            vec![".png", ".jpg", ".gif"]
        );
        assert!(parse_extensions(Some("image/*, png")).is_empty());
        assert!(parse_extensions(None).is_empty());
    }

    #[test]
    fn matches_accept_is_case_insensitive() {
        let exts = vec![".png".into(), ".jpg".into()];
        assert!(matches_accept(Path::new("a.PNG"), &exts));
        assert!(matches_accept(Path::new("a.jpg"), &exts));
        assert!(!matches_accept(Path::new("a.pdf"), &exts));
        assert!(matches_accept(Path::new("a.pdf"), &[]));
    }

    #[test]
    fn filter_files_respects_extension_and_limit() {
        let files = vec![
            PathBuf::from("a.png"),
            PathBuf::from("b.jpg"),
            PathBuf::from("c.pdf"),
        ];
        let exts = vec![".png".into(), ".jpg".into()];
        let (valid, _) = filter_files(&files, &exts, None, 10);
        assert_eq!(valid, vec![PathBuf::from("a.png"), PathBuf::from("b.jpg")]);

        let (limited, _) = filter_files(&files, &exts, None, 1);
        assert_eq!(limited, vec![PathBuf::from("a.png")]);
    }

    #[test]
    fn apply_selection_replaces_or_appends() {
        let mut files = vec![PathBuf::from("old.txt")];
        apply_selection(&mut files, vec![PathBuf::from("new.txt")], true, false);
        assert_eq!(files, vec![PathBuf::from("new.txt")]);

        let mut files = vec![PathBuf::from("old.txt")];
        apply_selection(&mut files, vec![PathBuf::from("new.txt")], true, true);
        assert_eq!(files, vec![PathBuf::from("old.txt"), PathBuf::from("new.txt")]);
    }

    #[test]
    fn single_selection_keeps_only_the_first() {
        let mut files = vec![PathBuf::from("old.txt")];
        apply_selection(
            &mut files,
            vec![PathBuf::from("a.txt"), PathBuf::from("b.txt")],
            false,
            false,
        );
        assert_eq!(files, vec![PathBuf::from("a.txt")]);
    }

    #[test]
    fn size_metrics_fold_edge_sizes() {
        assert_eq!(SizeMetrics::resolve(Size::ExtraSmall), SizeMetrics::resolve(Size::Small));
        assert_eq!(SizeMetrics::resolve(Size::ExtraLarge), SizeMetrics::resolve(Size::Large));
    }

    #[test]
    fn defaults_match_mudblazor() {
        let mut files = Vec::new();
        let upload = MudFileUpload::new(&mut files);
        assert!(upload.hidden);
        assert_eq!(upload.maximum_file_count, 10);
        assert!(upload.multiple);
        assert!(!upload.append_multiple_files);
        assert!(!upload.disabled);
        assert!(upload.accept.is_none());
        assert!(upload.max_file_size.is_none());
        assert!(upload.tag.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let mut files = Vec::new();
        let upload = MudFileUpload::new(&mut files)
            .single()
            .accept(".png")
            .append_multiple_files(true)
            .disabled(true)
            .hidden(false)
            .max_file_size(1024)
            .maximum_file_count(3)
            .variant(Variant::Outlined)
            .color(Color::Success)
            .size(Size::Large)
            .elevation(4)
            .corner_radius(8.0)
            .label("Pick")
            .aria_label("Files")
            .activator_content(|ui| ui.label("Custom"))
            .classes(["mt-2"])
            .tag("upload");
        assert!(!upload.multiple);
        assert_eq!(upload.accept, Some(".png"));
        assert!(upload.append_multiple_files && upload.disabled);
        assert!(!upload.hidden);
        assert_eq!(upload.max_file_size, Some(1024));
        assert_eq!(upload.maximum_file_count, 3);
        assert!(matches!(upload.variant, Variant::Outlined));
        assert!(matches!(upload.color, Color::Success));
        assert!(matches!(upload.size, Size::Large));
        assert_eq!(upload.elevation, 4);
        assert_eq!(upload.corner_radius, 8.0);
        assert_eq!(upload.label, Some("Pick"));
        assert_eq!(upload.aria_label, Some("Files"));
        assert!(upload.activator_content.is_some());
        assert!(!upload.base.class_names().is_empty());
        assert_eq!(upload.tag_ref::<&str>(), Some(&"upload"));
    }

    #[test]
    fn maximum_file_count_is_clamped_to_one() {
        let mut files = Vec::new();
        let upload = MudFileUpload::new(&mut files).maximum_file_count(0);
        assert_eq!(upload.maximum_file_count, 1);
    }
}