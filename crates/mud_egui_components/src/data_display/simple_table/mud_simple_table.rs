//! `MudSimpleTable` — a lightweight, non-data-bound table with MudBlazor's
//! simple-table styling: striped rows, hover highlighting, borders, elevation,
//! dense rows, outlined/square variants, and a fixed header.
//!
//! Port of `MudSimpleTable.razor.cs` (MudBlazor 8.13.0).
//!
//! # Parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `Bordered`          | [`MudSimpleTable::bordered`] | `false` |
//! | `ChildContent`      | the closure passed to [`MudSimpleTable::show`] | required |
//! | `Dense`             | [`MudSimpleTable::dense`] | `false` |
//! | `Elevation`         | [`MudSimpleTable::elevation`] | `1` |
//! | `FixedHeader`       | [`MudSimpleTable::fixed_header`] | `false` |
//! | `Hover`             | [`MudSimpleTable::hover`] | `false` |
//! | `Outlined`          | [`MudSimpleTable::outlined`] | `false` |
//! | `Square`            | [`MudSimpleTable::square`] | `false` |
//! | `Striped`           | [`MudSimpleTable::striped`] | `false` |
//! | `Class` (base)      | [`MudSimpleTable::classes`] | `None` |
//! | `Style` (base)      | *(egui has no CSS; use `classes`)* | — |
//! | `Tag` (base)        | [`MudSimpleTable::tag`] / [`MudSimpleTable::tag_ref`] | `None` |
//! | `UserAttributes`    | **not ported** (no HTML tag) | — |
//!
//! # Table structure
//!
//! MudBlazor's `MudSimpleTable` uses raw HTML table tags, not `MudTh` / `MudTd`
//! sub-components (those belong to `MudTable<T>`). The egui port mirrors that
//! with three closure hooks:
//!
//! * `header(|h| …)` — one header row
//! * `row(|r| …)` — any number of body rows
//! * `footer(|f| …)` — one optional footer row
//!
//! Each row builder provides `cell(text)` / `cell_ui(closure)` (plus
//! `cell_width(text, px)` / `cell_ui_width(closure, px)` for explicit column
//! widths). Text cells render through [`MudText`], so typography and theme
//! updates propagate automatically.
//!
//! # Fixed header
//!
//! `fixed_header(true)` keeps the header visible while the body scrolls. Like
//! MudBlazor, it only takes effect when [`height`](MudSimpleTable::height) is
//! also set — the CSS `height` requirement in MudBlazor maps to the egui
//! `ScrollArea::max_height` requirement here.

use std::any::Any;
use std::sync::Arc;

use egui::{
    vec2, Color32, CornerRadius, Rect, Response, Sense, Stroke, StrokeKind, Ui, UiBuilder,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{StyleFlavor, Typo};
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

use crate::typography::mud_text::MudText;

use super::simple_table_internal::{CellContent, RowKind};

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Row height for a non-dense table.
pub const ROW_HEIGHT: f32 = 48.0;
/// Row height for a dense table.
pub const DENSE_ROW_HEIGHT: f32 = 32.0;
/// Horizontal padding inside a cell.
pub const CELL_PADDING_X: f32 = 12.0;
/// Vertical padding inside a cell.
pub const CELL_PADDING_Y: f32 = 8.0;
/// Maximum elevation (MudBlazor clamps at 24).
pub const MAX_ELEVATION: u8 = 24;
/// Fallback width when the available width is non-finite.
pub const FALLBACK_WIDTH: f32 = 1280.0;

// ---------------------------------------------------------------------------
// One cell
// ---------------------------------------------------------------------------

/// A single collected cell.
pub(crate) struct TableCell<'a> {
    /// Cell content: text (rendered via `MudText`) or custom UI.
    pub content: CellContent<'a>,
    /// Explicit column width in pixels, if set.
    pub width: Option<f32>,
}

// ---------------------------------------------------------------------------
// RowUi
// ---------------------------------------------------------------------------

/// Builder passed to `header` / `row` / `footer` closures.
///
/// The row's structural kind (header / body / footer) is decided by which
/// hook the caller used; `RowUi` only collects cells.
pub struct RowUi<'a> {
    pub(crate) cells: Vec<TableCell<'a>>,
}

impl<'a> RowUi<'a> {
    /// Adds a text cell. Rendered through [`MudText`].
    pub fn cell(&mut self, text: impl Into<String>) {
        self.cells.push(TableCell {
            content: CellContent::Text(text.into()),
            width: None,
        });
    }

    /// Adds a text cell with an explicit column width in pixels.
    pub fn cell_width(&mut self, text: impl Into<String>, width: f32) {
        self.cells.push(TableCell {
            content: CellContent::Text(text.into()),
            width: Some(width.max(0.0)),
        });
    }

    /// Adds a custom-UI cell. The closure receives the cell's child `Ui`.
    ///
    /// Use this for anything that is not a plain string: a `MudButton`, an
    /// icon + text pair, a nested widget, etc.
    pub fn cell_ui(&mut self, f: impl FnMut(&mut Ui) + 'a) {
        self.cells.push(TableCell {
            content: CellContent::Ui(Box::new(f)),
            width: None,
        });
    }

    /// Adds a custom-UI cell with an explicit column width.
    pub fn cell_ui_width(&mut self, f: impl FnMut(&mut Ui) + 'a, width: f32) {
        self.cells.push(TableCell {
            content: CellContent::Ui(Box::new(f)),
            width: Some(width.max(0.0)),
        });
    }

    /// Number of cells added so far.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Whether no cells have been added.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

// ---------------------------------------------------------------------------
// SimpleTableUi
// ---------------------------------------------------------------------------

/// Collector passed to [`MudSimpleTable::show`]'s closure.
pub struct SimpleTableUi<'a> {
    pub(crate) header: Option<Vec<TableCell<'a>>>,
    pub(crate) rows: Vec<Vec<TableCell<'a>>>,
    pub(crate) footer: Option<Vec<TableCell<'a>>>,
}

impl<'a> SimpleTableUi<'a> {
    /// Sets the header row. Calling it again replaces the previous header.
    pub fn header<R>(&mut self, f: impl FnOnce(&mut RowUi<'a>) -> R) {
        let mut row = RowUi { cells: Vec::new() };
        let _ = f(&mut row);
        self.header = Some(row.cells);
    }

    /// Adds a body row.
    pub fn row<R>(&mut self, f: impl FnOnce(&mut RowUi<'a>) -> R) {
        let mut row = RowUi { cells: Vec::new() };
        let _ = f(&mut row);
        self.rows.push(row.cells);
    }

    /// Sets the footer row. Calling it again replaces the previous footer.
    pub fn footer<R>(&mut self, f: impl FnOnce(&mut RowUi<'a>) -> R) {
        let mut row = RowUi { cells: Vec::new() };
        let _ = f(&mut row);
        self.footer = Some(row.cells);
    }

    /// Number of collected body rows (header/footer excluded).
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Whether any body rows have been collected.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Row height for the given density.
#[inline]
pub fn row_height(dense: bool) -> f32 {
    if dense { DENSE_ROW_HEIGHT } else { ROW_HEIGHT }
}

/// Resolves column widths from per-column explicit hints.
///
/// `explicit` has one entry per column; `None` means "flexible". The result
/// sums to `available` (when at least one column is flexible) or is the sum
/// of the explicit widths (when every column is fixed).
pub fn resolve_column_widths(explicit: &[Option<f32>], available: f32) -> Vec<f32> {
    let available = if available.is_finite() && available > 0.0 {
        available
    } else {
        FALLBACK_WIDTH
    };
    let mut fixed: f32 = 0.0;
    let mut flex_count = 0usize;
    for w in explicit {
        match w {
            Some(v) => fixed += v.max(0.0),
            None => flex_count += 1,
        }
    }
    let per_flex = if flex_count > 0 {
        ((available - fixed).max(0.0)) / flex_count as f32
    } else {
        0.0
    };
    explicit
        .iter()
        .map(|w| w.unwrap_or(per_flex).max(0.0))
        .collect()
}

/// Column count of a table: the widest row.
pub(crate) fn column_count<'a>(
    header: Option<&[TableCell<'a>]>,
    rows: &[Vec<TableCell<'a>>],
    footer: Option<&[TableCell<'a>]>,
) -> usize {
    let mut max = header.map_or(0, |h| h.len());
    for r in rows {
        max = max.max(r.len());
    }
    if let Some(f) = footer {
        max = max.max(f.len());
    }
    max
}

/// Per-column explicit width hint (first non-`None` encountered).
pub(crate) fn column_hints<'a>(
    header: Option<&[TableCell<'a>]>,
    rows: &[Vec<TableCell<'a>>],
    footer: Option<&[TableCell<'a>]>,
    count: usize,
) -> Vec<Option<f32>> {
    let mut hints = vec![None; count];
    let mut scan = |cells: &[TableCell<'a>]| {
        for (i, c) in cells.iter().enumerate().take(count) {
            if hints[i].is_none() {
                hints[i] = c.width;
            }
        }
    };
    if let Some(h) = header {
        scan(h);
    }
    for r in rows {
        scan(r);
    }
    if let Some(f) = footer {
        scan(f);
    }
    hints
}

/// Maps a MudBlazor elevation (0–24) to an egui shadow.
pub fn elevation_shadow(elevation: u8) -> egui::epaint::Shadow {
    let e = elevation.min(MAX_ELEVATION) as f32;
    egui::epaint::Shadow {
        offset: [0, (e * 0.4) as i8],
        blur: (e * 1.5) as u8,
        spread: 0,
        color: Color32::from_black_alpha(((e * 3.0) as u8).min(60)),
    }
}

// ---------------------------------------------------------------------------
// MudSimpleTable
// ---------------------------------------------------------------------------

/// A lightweight, non-data-bound table.
#[must_use = "MudSimpleTable does nothing until you call `.show(ui, |t| ..)`"]
pub struct MudSimpleTable<'a> {
    /// Left/right borders on every cell.
    pub bordered: bool,
    /// Compact row heights.
    pub dense: bool,
    /// Drop-shadow depth (0–24; clamped).
    pub elevation: u8,
    /// Keeps the header visible while the body scrolls (requires `height`).
    pub fixed_header: bool,
    /// Highlights rows on hover.
    pub hover: bool,
    /// Draws an outline around the table.
    pub outlined: bool,
    /// Removes the rounded corners.
    pub square: bool,
    /// Alternating row backgrounds.
    pub striped: bool,
    /// Maximum body height; enables a vertical scroll area.
    pub height: Option<f32>,

    /// Utility classes for the wrapper frame.
    pub base: ComponentBase,
    /// Bootstrap or Tailwind resolver.
    pub flavor: StyleFlavor,
    /// Bootstrap resolver state.
    pub bs_style: BsStyle,
    /// Tailwind resolver state.
    pub tw_style: TwStyle,
    /// Tailwind interaction state.
    pub tw_state: TwState,

    tag_value: Option<Arc<dyn Any + Send + Sync>>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Default for MudSimpleTable<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> MudSimpleTable<'a> {
    /// Creates a table with MudBlazor defaults.
    pub fn new() -> Self {
        Self {
            bordered: false,
            dense: false,
            elevation: 1,
            fixed_header: false,
            hover: false,
            outlined: false,
            square: false,
            striped: false,
            height: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
            tag_value: None,
            _marker: std::marker::PhantomData,
        }
    }

    // ── Builders ───────────────────────────────────────────────────────────

    /// Left/right borders on every cell (`Bordered`).
    #[inline]
    pub fn bordered(mut self, b: bool) -> Self {
        self.bordered = b;
        self
    }

    /// Compact row heights (`Dense`).
    #[inline]
    pub fn dense(mut self, b: bool) -> Self {
        self.dense = b;
        self
    }

    /// Drop-shadow depth (`Elevation`, default `1`, clamped to 24).
    #[inline]
    pub fn elevation(mut self, e: u8) -> Self {
        self.elevation = e;
        self
    }

    /// Keeps the header visible while the body scrolls (`FixedHeader`).
    ///
    /// Only effective when [`height`](Self::height) is also set.
    #[inline]
    pub fn fixed_header(mut self, b: bool) -> Self {
        self.fixed_header = b;
        self
    }

    /// Highlights rows on hover (`Hover`).
    #[inline]
    pub fn hover(mut self, b: bool) -> Self {
        self.hover = b;
        self
    }

    /// Draws an outline around the table (`Outlined`).
    #[inline]
    pub fn outlined(mut self, b: bool) -> Self {
        self.outlined = b;
        self
    }

    /// Removes the rounded corners (`Square`).
    #[inline]
    pub fn square(mut self, b: bool) -> Self {
        self.square = b;
        self
    }

    /// Alternating row backgrounds (`Striped`).
    #[inline]
    pub fn striped(mut self, b: bool) -> Self {
        self.striped = b;
        self
    }

    /// Sets a maximum body height; enables a vertical scroll area.
    #[inline]
    pub fn height(mut self, h: f32) -> Self {
        self.height = Some(h.max(0.0));
        self
    }

    /// Adds utility classes to the wrapper frame (`Class`).
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Selects the utility-class resolver.
    #[inline]
    pub fn flavor(mut self, f: StyleFlavor) -> Self {
        self.flavor = f;
        self
    }

    /// Attaches arbitrary user data (`Tag`).
    pub fn tag<T: Any + Send + Sync>(mut self, value: T) -> Self {
        self.tag_value = Some(Arc::new(value));
        self
    }

    /// Reads the tag back if it is of type `U`.
    pub fn tag_ref<U: Any + Send + Sync>(&self) -> Option<&U> {
        self.tag_value.as_ref()?.downcast_ref::<U>()
    }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the table and returns `(response, add_content_result)`.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        add_content: impl FnOnce(&mut SimpleTableUi<'a>) -> R,
    ) -> (Response, R) {
        // 1. Collect content.
        let mut collector = SimpleTableUi {
            header: None,
            rows: Vec::new(),
            footer: None,
        };
        let user_result = add_content(&mut collector);
        let SimpleTableUi {
            header,
            mut rows,
            footer,
        } = collector;

        // 2. Theme.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);

        // 3. Wrapper frame.
        self.tw_state.dark = ui.visuals().dark_mode;
        let mut frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => self
                .tw_style
                .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
        };
        if self.elevation > 0 {
            frame = frame.shadow(elevation_shadow(self.elevation));
        }

        // 4. Resolve geometry.
        //
        // `palette.surface` is a `MudColor`, which implements `Into<Color32>`.
        // `Palette::resolve(&str)` takes a `MudPalette*` key, so it is not the
        // right method here.
        let radius = if self.square { 0 } else { 4 };
        let row_h = row_height(self.dense);
        let outline_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        let border_color = outline_color;
        let header_fill: Color32 = palette.surface.into();

        // 5. Render.
        let outer = frame.show(ui, |ui| {
            let available = ui.available_width();
            let available = if available.is_finite() && available > 0.0 {
                available
            } else {
                FALLBACK_WIDTH
            };

            let cols = column_count(header.as_deref(), &rows, footer.as_deref());
            let hints = column_hints(header.as_deref(), &rows, footer.as_deref(), cols);
            let widths = resolve_column_widths(&hints, available);
            let content_width: f32 = widths.iter().sum();

            let can_fixed =
                self.fixed_header && self.height.is_some() && cols > 0;

            // Helper that draws a row at `(x, y)` with a given kind.
            let mut draw_row = |ui: &mut Ui,
                                cells: &mut Vec<TableCell<'a>>,
                                kind: RowKind,
                                y: f32,
                                row_idx: Option<usize>,
                                widths: &[f32]|
                                -> Rect {
                let is_header = matches!(kind, RowKind::Header);
                let is_footer = matches!(kind, RowKind::Footer);
                let h = row_h;
                let rect = Rect::from_min_size(
                    egui::pos2(ui.min_rect().left(), y),
                    vec2(content_width, h),
                );

                // Determine background for body rows.
                let fill = if is_header {
                    header_fill
                } else if is_footer {
                    header_fill
                } else if self.striped && row_idx.map_or(false, |i| i % 2 == 1) {
                    ui.visuals().faint_bg_color
                } else {
                    Color32::TRANSPARENT
                };

                // Hover detection for body rows.
                let row_resp = ui.interact(
                    rect,
                    ui.scope_id().with(("simple_row", y.to_bits())),
                    if self.hover && !is_header && !is_footer {
                        Sense::hover()
                    } else {
                        Sense::hover()
                    },
                );
                let hovered = self.hover
                    && !is_header
                    && !is_footer
                    && row_resp.hovered();

                let final_fill = if hovered {
                    ui.visuals().widgets.hovered.weak_bg_fill
                } else {
                    fill
                };
                if final_fill != Color32::TRANSPARENT {
                    ui.painter().rect_filled(rect, 0.0, final_fill);
                }

                // Cells.
                let mut x = rect.left();
                for (ci, cell) in cells.iter_mut().enumerate() {
                    let w = widths.get(ci).copied().unwrap_or(0.0);
                    let cell_rect =
                        Rect::from_min_size(egui::pos2(x, rect.top()), vec2(w, h));

                    if self.bordered && ci > 0 {
                        ui.painter().line_segment(
                            [
                                egui::pos2(x, rect.top()),
                                egui::pos2(x, rect.bottom()),
                            ],
                            Stroke::new(1.0, border_color),
                        );
                    }

                    let inner = Rect::from_min_max(
                        egui::pos2(cell_rect.left() + CELL_PADDING_X, cell_rect.top()),
                        egui::pos2(cell_rect.right() - CELL_PADDING_X, cell_rect.bottom()),
                    );
                    let mut child = ui.new_child(
                        UiBuilder::new()
                            .max_rect(inner)
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );
                    child.set_clip_rect(cell_rect);

                    match &mut cell.content {
                        CellContent::Text(text) => {
                            let typo = if is_header || is_footer {
                                Typo::Subtitle2
                            } else {
                                Typo::Body2
                            };
                            let mut t = MudText::new(text.clone()).typo(typo);
                            if is_header || is_footer {
                                t = t.bold(true);
                            }
                            child.add(t);
                        }
                        CellContent::Ui(f) => f(&mut child),
                    }

                    x += w;
                }

                // Bottom border.
                ui.painter().line_segment(
                    [
                        egui::pos2(rect.left(), rect.bottom()),
                        egui::pos2(rect.right(), rect.bottom()),
                    ],
                    Stroke::new(1.0, border_color),
                );

                rect
            };

            // 5a. Allocate the outer response rect.
            let body_rows = rows.len();
            let footer_h = if footer.is_some() { row_h } else { 0.0 };
            let header_h = if header.is_some() { row_h } else { 0.0 };
            let body_h = body_rows as f32 * row_h;
            let total_h = header_h + body_h + footer_h;

            let (table_rect, table_resp) =
                ui.allocate_exact_size(vec2(content_width, total_h), Sense::hover());

            // 5b. Draw.
            let mut cursor_y = table_rect.top();

            if can_fixed {
                // Fixed header: header + scrollable body + footer.
                if let Some(mut h) = header {
                    draw_row(ui, &mut h, RowKind::Header, cursor_y, None, &widths);
                    cursor_y += row_h;
                }
                let cap = self
                    .height
                    .map(|hh| (hh - header_h - footer_h).max(row_h))
                    .unwrap_or(body_h);
                egui::ScrollArea::vertical()
                    .max_height(cap)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut y = cursor_y;
                        for (idx, mut r) in rows.into_iter().enumerate() {
                            draw_row(ui, &mut r, RowKind::Body, y, Some(idx), &widths);
                            y += row_h;
                        }
                    });
                cursor_y += body_h;
                if let Some(mut f) = footer {
                    draw_row(ui, &mut f, RowKind::Footer, cursor_y, None, &widths);
                }
            } else if let Some(cap) = self.height {
                // Scrolling the whole table (header included).
                egui::ScrollArea::vertical()
                    .max_height(cap)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut y = cursor_y;
                        if let Some(mut h) = header {
                            draw_row(ui, &mut h, RowKind::Header, y, None, &widths);
                            y += row_h;
                        }
                        for (idx, mut r) in rows.into_iter().enumerate() {
                            draw_row(ui, &mut r, RowKind::Body, y, Some(idx), &widths);
                            y += row_h;
                        }
                        if let Some(mut f) = footer {
                            draw_row(ui, &mut f, RowKind::Footer, y, None, &widths);
                        }
                    });
            } else {
                // No scroll: draw everything inline.
                if let Some(mut h) = header {
                    draw_row(ui, &mut h, RowKind::Header, cursor_y, None, &widths);
                    cursor_y += row_h;
                }
                for (idx, mut r) in rows.into_iter().enumerate() {
                    draw_row(ui, &mut r, RowKind::Body, cursor_y, Some(idx), &widths);
                    cursor_y += row_h;
                }
                if let Some(mut f) = footer {
                    draw_row(ui, &mut f, RowKind::Footer, cursor_y, None, &widths);
                }
            }

            // Outline.
            if self.outlined {
                ui.painter().rect_stroke(
                    table_rect,
                    CornerRadius::same(radius),
                    Stroke::new(1.0, outline_color),
                    StrokeKind::Inside,
                );
            }

            table_resp
        });

        (outer.inner, user_result)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- row heights -------------------------------------------------------

    #[test]
    fn row_height_matches_density() {
        assert_eq!(row_height(false), ROW_HEIGHT);
        assert_eq!(row_height(true), DENSE_ROW_HEIGHT);
    }

    // --- column width resolution ------------------------------------------

    #[test]
    fn widths_all_flexible_split_equally() {
        let w = resolve_column_widths(&[None, None, None], 600.0);
        assert_eq!(w, vec![200.0, 200.0, 200.0]);
    }

    #[test]
    fn widths_all_fixed_sum_is_total() {
        let w = resolve_column_widths(&[Some(100.0), Some(80.0), Some(120.0)], 999.0);
        assert_eq!(w, vec![100.0, 80.0, 120.0]);
    }

    #[test]
    fn widths_mixed_fixed_fills_remaining() {
        let w = resolve_column_widths(&[Some(100.0), None, None], 500.0);
        assert_eq!(w, vec![100.0, 200.0, 200.0]);
        assert!((w.iter().sum::<f32>() - 500.0).abs() < 1e-3);
    }

    #[test]
    fn widths_clamp_when_fixed_exceeds_available() {
        let w = resolve_column_widths(&[Some(400.0), None], 300.0);
        assert_eq!(w, vec![400.0, 0.0]);
    }

    #[test]
    fn widths_repair_non_finite_available() {
        let w = resolve_column_widths(&[None, None], f32::NAN);
        assert_eq!(w, vec![FALLBACK_WIDTH / 2.0, FALLBACK_WIDTH / 2.0]);
    }

    #[test]
    fn widths_empty_input_is_empty() {
        assert!(resolve_column_widths(&[], 500.0).is_empty());
    }

    // --- column count ------------------------------------------------------

    fn cell<'a>(text: &str) -> TableCell<'a> {
        TableCell {
            content: CellContent::Text(text.into()),
            width: None,
        }
    }

    #[test]
    fn column_count_uses_widest_row() {
        let header = vec![cell("a"), cell("b"), cell("c")];
        let rows = vec![vec![cell("1"), cell("2")], vec![cell("1"), cell("2"), cell("3")]];
        assert_eq!(column_count(Some(&header), &rows, None), 3);
    }

    #[test]
    fn column_count_handles_missing_sections() {
        assert_eq!(column_count(None, &[], None), 0);
        assert_eq!(column_count(None, &[vec![cell("x")]], None), 1);
    }

    // --- column hints ------------------------------------------------------

    #[test]
    fn column_hints_take_first_explicit_width() {
        let header = vec![
            TableCell {
                content: CellContent::Text("h".into()),
                width: None,
            },
            TableCell {
                content: CellContent::Text("h".into()),
                width: Some(80.0),
            },
        ];
        let rows = vec![vec![
            TableCell {
                content: CellContent::Text("r".into()),
                width: Some(200.0),
            },
            TableCell {
                content: CellContent::Text("r".into()),
                width: Some(999.0),
            },
        ]];
        let hints = column_hints(Some(&header), &rows, None, 2);
        assert_eq!(hints, vec![Some(200.0), Some(80.0)]);
    }

    // --- elevation ---------------------------------------------------------

    #[test]
    fn elevation_zero_is_no_shadow() {
        let s = elevation_shadow(0);
        assert_eq!(s.blur, 0);
        assert_eq!(s.color.a(), 0);
    }

    #[test]
    fn elevation_clamps_above_max() {
        let a = elevation_shadow(MAX_ELEVATION);
        let b = elevation_shadow(200);
        assert_eq!(a.offset, b.offset);
        assert_eq!(a.blur, b.blur);
        assert_eq!(a.color, b.color);
    }

    #[test]
    fn elevation_scales_monotonically() {
        let s1 = elevation_shadow(1);
        let s8 = elevation_shadow(8);
        let s16 = elevation_shadow(16);
        assert!(s1.blur < s8.blur);
        assert!(s8.blur < s16.blur);
    }

    // --- defaults ----------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let t: MudSimpleTable<'static> = MudSimpleTable::new();
        assert!(!t.bordered);
        assert!(!t.dense);
        assert_eq!(t.elevation, 1);
        assert!(!t.fixed_header);
        assert!(!t.hover);
        assert!(!t.outlined);
        assert!(!t.square);
        assert!(!t.striped);
        assert!(t.height.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let t = MudSimpleTable::new()
            .bordered(true)
            .dense(true)
            .elevation(4)
            .fixed_header(true)
            .hover(true)
            .outlined(true)
            .square(true)
            .striped(true)
            .height(300.0)
            .classes(["a"]);
        assert!(t.bordered && t.dense && t.fixed_header && t.hover);
        assert!(t.outlined && t.square && t.striped);
        assert_eq!(t.elevation, 4);
        assert_eq!(t.height, Some(300.0));
    }

    #[test]
    fn height_clamps_negative() {
        let t = MudSimpleTable::new().height(-10.0);
        assert_eq!(t.height, Some(0.0));
    }

    // --- tag ---------------------------------------------------------------

    #[test]
    fn tag_round_trips_by_type() {
        let t = MudSimpleTable::new().tag("section");
        assert_eq!(t.tag_ref::<&str>(), Some(&"section"));
        assert!(t.tag_ref::<u32>().is_none());
    }

    // --- collector ---------------------------------------------------------

    #[test]
    fn collector_preserves_row_order() {
        let mut c: SimpleTableUi<'static> = SimpleTableUi {
            header: None,
            rows: Vec::new(),
            footer: None,
        };
        c.header(|h| {
            h.cell("A");
            h.cell("B");
        });
        c.row(|r| r.cell("1"));
        c.row(|r| r.cell("2"));
        c.footer(|f| f.cell("T"));
        assert_eq!(c.row_count(), 2);
        assert!(c.header.is_some());
        assert!(c.footer.is_some());
        assert_eq!(c.header.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn collector_header_replaces_previous() {
        let mut c: SimpleTableUi<'static> = SimpleTableUi {
            header: None,
            rows: Vec::new(),
            footer: None,
        };
        c.header(|h| h.cell("first"));
        c.header(|h| {
            h.cell("second");
            h.cell("third");
        });
        assert_eq!(c.header.as_ref().unwrap().len(), 2);
    }
}