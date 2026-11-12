//! `MudTable<T>` — a sortable, filterable, paginated table with multiselection,
//! responsive stacking, child rows, custom header/footer rows and a built-in
//! pager.
//!
//! Port of `MudTable<T>.razor.cs` + `MudTableBase.cs` + `MudTablePager.razor.cs`
//! (MudBlazor 8.13.0). See the module README for full parameter parity and
//! the list of deliberately-not-ported features (editing, grouping,
//! server-data, virtualization).

use std::any::Any;
use std::cmp::Ordering;
use std::sync::Arc;

use egui::{
    vec2, Color32, CornerRadius, Rect, Response, Sense, Stroke, StrokeKind, Ui, UiBuilder,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::breakpoint::Breakpoint;
use mud_egui_core::enums::{
    Color, ColumnAlign, SelectionMode, Size, SortDirection, StyleFlavor, Typo, Variant,
};
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::typography::mud_text::MudText;
use crate::buttons::button::mud_button::MudButton;
use crate::MudToggleGroup;

use super::table_internal::{CellContent, RowKind};

const REM_SIZE_PT: f32 = 14.0;

pub const ROW_HEIGHT: f32 = 48.0;
pub const DENSE_ROW_HEIGHT: f32 = 32.0;
pub const CELL_PADDING_X: f32 = 12.0;
pub const SELECTION_COL_WIDTH: f32 = 48.0;
const CHECKBOX_SIZE: f32 = 20.0;
const SORT_ICON_SIZE: f32 = 16.0;
pub const MAX_ELEVATION: u8 = 24;
pub const FALLBACK_WIDTH: f32 = 1280.0;
pub const DEFAULT_ROWS_PER_PAGE: usize = 10;
pub const DEFAULT_PAGE_SIZE_OPTIONS: [usize; 4] = [10, 25, 50, 100];

// ---------------------------------------------------------------------------
// Td
// ---------------------------------------------------------------------------

#[must_use = "Td does nothing until you add it to a row"]
pub struct Td<'a> {
    pub(crate) content: CellContent<'a>,
    pub(crate) label: Option<String>,
    pub(crate) hidden: bool,
    pub(crate) align: ColumnAlign,
    pub(crate) width: Option<f32>,
}

impl<'a> Td<'a> {
    pub fn text(text: impl Into<String>) -> Self {
        Self { content: CellContent::Text(text.into()), label: None, hidden: false, align: ColumnAlign::Left, width: None }
    }

    pub fn ui(f: impl FnMut(&mut Ui) + 'a) -> Self {
        Self { content: CellContent::Ui(Box::new(f)), label: None, hidden: false, align: ColumnAlign::Left, width: None }
    }

    #[inline]
    pub fn label(mut self, label: impl Into<String>) -> Self { self.label = Some(label.into()); self }

    #[inline]
    pub fn hidden(mut self, b: bool) -> Self { self.hidden = b; self }

    #[inline]
    pub fn align(mut self, a: ColumnAlign) -> Self { self.align = a; self }

    #[inline]
    pub fn width(mut self, px: f32) -> Self { self.width = Some(px.max(0.0)); self }
}

// ---------------------------------------------------------------------------
// Th
// ---------------------------------------------------------------------------

#[must_use = "Th does nothing until you add it to a header row"]
pub struct Th<'a, T> {
    pub(crate) content: CellContent<'a>,
    pub(crate) hidden: bool,
    pub(crate) align: ColumnAlign,
    pub(crate) width: Option<f32>,
    pub(crate) sortable: Option<bool>,
    pub(crate) sort_key: Option<String>,
    pub(crate) comparator: Option<Box<dyn Fn(&T, &T) -> Ordering + 'a>>,
}

impl<'a, T> Th<'a, T> {
    pub fn text(text: impl Into<String>) -> Self {
        Self { content: CellContent::Text(text.into()), hidden: false, align: ColumnAlign::Left, width: None, sortable: None, sort_key: None, comparator: None }
    }

    pub fn ui(f: impl FnMut(&mut Ui) + 'a) -> Self {
        Self { content: CellContent::Ui(Box::new(f)), hidden: false, align: ColumnAlign::Left, width: None, sortable: None, sort_key: None, comparator: None }
    }

    #[inline]
    pub fn sortable(mut self, b: bool) -> Self { self.sortable = Some(b); self }

    #[inline]
    pub fn sort_by(mut self, key: impl Into<String>, cmp: impl Fn(&T, &T) -> Ordering + 'a) -> Self {
        self.sort_key = Some(key.into());
        self.comparator = Some(Box::new(cmp));
        self.sortable = Some(true);
        self
    }

    #[inline]
    pub fn hidden(mut self, b: bool) -> Self { self.hidden = b; self }

    #[inline]
    pub fn align(mut self, a: ColumnAlign) -> Self { self.align = a; self }

    #[inline]
    pub fn width(mut self, px: f32) -> Self { self.width = Some(px.max(0.0)); self }

    #[inline]
    pub fn is_sortable(&self) -> bool {
        self.sortable.unwrap_or(true) && self.sort_key.is_some() && self.comparator.is_some()
    }
}

// ---------------------------------------------------------------------------
// RowUi / HeaderUi / FooterUi
// ---------------------------------------------------------------------------

pub struct RowUi<'a> { pub(crate) cells: Vec<Td<'a>> }

impl<'a> RowUi<'a> {
    pub fn td(&mut self, cell: Td<'a>) { self.cells.push(cell); }
    pub fn text(&mut self, text: impl Into<String>) { self.cells.push(Td::text(text)); }
    pub fn ui(&mut self, f: impl FnMut(&mut Ui) + 'a) { self.cells.push(Td::ui(f)); }
    pub fn len(&self) -> usize { self.cells.len() }
    pub fn is_empty(&self) -> bool { self.cells.is_empty() }
}

pub struct HeaderUi<'a, T> { pub(crate) cells: Vec<Th<'a, T>> }

impl<'a, T> HeaderUi<'a, T> {
    pub fn th(&mut self, cell: Th<'a, T>) { self.cells.push(cell); }
    pub fn text(&mut self, text: impl Into<String>) { self.cells.push(Th::text(text)); }
    pub fn sort_th(&mut self, text: impl Into<String>, key: impl Into<String>, cmp: impl Fn(&T, &T) -> Ordering + 'a) {
        self.cells.push(Th::text(text).sort_by(key, cmp));
    }
    pub fn len(&self) -> usize { self.cells.len() }
    pub fn is_empty(&self) -> bool { self.cells.is_empty() }
}

pub struct FooterUi<'a> { pub(crate) cells: Vec<Td<'a>> }

impl<'a> FooterUi<'a> {
    pub fn td(&mut self, cell: Td<'a>) { self.cells.push(cell); }
    pub fn text(&mut self, text: impl Into<String>) { self.cells.push(Td::text(text)); }
    pub fn ui(&mut self, f: impl FnMut(&mut Ui) + 'a) { self.cells.push(Td::ui(f)); }
    pub fn len(&self) -> usize { self.cells.len() }
    pub fn is_empty(&self) -> bool { self.cells.is_empty() }
}

// ---------------------------------------------------------------------------
// TableUi
// ---------------------------------------------------------------------------

pub struct TableUi<'a, T> {
    pub(crate) header_rows: Vec<Vec<Th<'a, T>>>,
    pub(crate) footer_rows: Vec<Vec<Td<'a>>>,
    pub(crate) row_template: Option<Box<dyn FnMut(&mut RowUi<'a>, &T) + 'a>>,
    pub(crate) child_row: Option<Box<dyn FnMut(&mut RowUi<'a>, &T) + 'a>>,
}

impl<'a, T> TableUi<'a, T> {
    pub fn header<R>(&mut self, f: impl FnOnce(&mut HeaderUi<'a, T>) -> R) {
        let mut h = HeaderUi { cells: Vec::new() };
        let _ = f(&mut h);
        self.header_rows.push(h.cells);
    }
    pub fn footer<R>(&mut self, f: impl FnOnce(&mut FooterUi<'a>) -> R) {
        let mut fo = FooterUi { cells: Vec::new() };
        let _ = f(&mut fo);
        self.footer_rows.push(fo.cells);
    }
    pub fn row_template(&mut self, f: impl FnMut(&mut RowUi<'a>, &T) + 'a) {
        self.row_template = Some(Box::new(f));
    }
    pub fn child_row(&mut self, f: impl FnMut(&mut RowUi<'a>, &T) + 'a) {
        self.child_row = Some(Box::new(f));
    }
    pub fn header_row_count(&self) -> usize { self.header_rows.len() }
    pub fn footer_row_count(&self) -> usize { self.footer_rows.len() }
}

// ---------------------------------------------------------------------------
// TableState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TableState<T> {
    pub sort_label: Option<String>,
    pub sort_direction: SortDirection,
    pub current_page: usize,
    pub rows_per_page: usize,
    pub selected_item: Option<T>,
    pub selected_items: Vec<T>,
    pub last_row_rects: Vec<Rect>,
    pub last_filtered_count: usize,
    pub last_total_count: usize,
}

impl<T> Default for TableState<T> { fn default() -> Self { Self::new() } }

impl<T> TableState<T> {
    pub fn new() -> Self {
        Self {
            sort_label: None,
            sort_direction: SortDirection::None,
            current_page: 0,
            rows_per_page: DEFAULT_ROWS_PER_PAGE,
            selected_item: None,
            selected_items: Vec::new(),
            last_row_rects: Vec::new(),
            last_filtered_count: 0,
            last_total_count: 0,
        }
    }
    pub fn with_rows_per_page(mut self, n: usize) -> Self { self.rows_per_page = n; self }
    pub fn is_selected(&self, item: &T) -> bool where T: PartialEq {
        self.selected_item.as_ref() == Some(item) || self.selected_items.iter().any(|s| s == item)
    }
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

#[inline] pub fn row_height(dense: bool) -> f32 { if dense { DENSE_ROW_HEIGHT } else { ROW_HEIGHT } }

#[inline]
pub fn effective_row_height(dense: bool, item_size: Option<f32>) -> f32 {
    match item_size { Some(v) if v.is_finite() && v > 0.0 => v, _ => row_height(dense) }
}

#[inline]
pub fn breakpoint_px(bp: Breakpoint) -> f32 {
    match bp {
        Breakpoint::Xs => 0.0, Breakpoint::Sm => 600.0, Breakpoint::Md => 960.0,
        Breakpoint::Lg => 1280.0, Breakpoint::Xl => 1920.0, _ => f32::INFINITY,
    }
}

#[inline]
pub fn active_breakpoint(width: f32) -> Breakpoint {
    if width >= 1920.0 { Breakpoint::Xl }
    else if width >= 1280.0 { Breakpoint::Lg }
    else if width >= 960.0 { Breakpoint::Md }
    else if width >= 600.0 { Breakpoint::Sm }
    else { Breakpoint::Xs }
}

#[inline]
pub fn is_stacked(active: Breakpoint, breakpoint: Breakpoint) -> bool {
    breakpoint_px(active) < breakpoint_px(breakpoint)
}

pub fn toggle_sort(
    label: &mut Option<String>,
    direction: &mut SortDirection,
    clicked: &str,
    allow_unsorted: bool,
) -> bool {
    if label.as_deref() == Some(clicked) {
        *direction = direction.next(allow_unsorted);
        if !allow_unsorted && !direction.is_active() {
            *direction = SortDirection::Ascending;
        }
    } else {
        *label = Some(clicked.to_owned());
        *direction = SortDirection::Ascending;
    }
    true
}

pub fn sort_indices_by<T>(
    items: &[T], base: &[usize], direction: SortDirection,
    cmp: &dyn Fn(&T, &T) -> Ordering,
) -> Vec<usize> {
    let mut out = base.to_vec();
    match direction {
        SortDirection::Ascending => out.sort_by(|&a, &b| cmp(&items[a], &items[b])),
        SortDirection::Descending => out.sort_by(|&a, &b| cmp(&items[b], &items[a])),
        SortDirection::None => {}
    }
    out
}

#[inline]
pub fn page_count(total: usize, rows_per_page: usize) -> usize {
    if total == 0 { 0 } else if rows_per_page == 0 { 1 } else { (total + rows_per_page - 1) / rows_per_page }
}

#[inline]
pub fn clamp_page(page: usize, total: usize, rows_per_page: usize) -> usize {
    let pages = page_count(total, rows_per_page);
    if pages == 0 { 0 } else { page.min(pages - 1) }
}

#[inline]
pub fn page_range(total: usize, page: usize, rows_per_page: usize) -> (usize, usize) {
    if rows_per_page == 0 { return (0, total); }
    let start = page.saturating_mul(rows_per_page).min(total);
    let end = (start + rows_per_page).min(total);
    (start, end)
}

pub fn resolve_column_widths(explicit: &[Option<f32>], available: f32) -> Vec<f32> {
    let available = if available.is_finite() && available > 0.0 { available } else { FALLBACK_WIDTH };
    let mut fixed = 0.0_f32;
    let mut flex = 0usize;
    for w in explicit {
        match w { Some(v) => fixed += v.max(0.0), None => flex += 1 }
    }
    let per_flex = if flex > 0 { ((available - fixed).max(0.0)) / flex as f32 } else { 0.0 };
    explicit.iter().map(|w| w.unwrap_or(per_flex).max(0.0)).collect()
}

pub fn elevation_shadow(elevation: u8) -> egui::epaint::Shadow {
    let e = elevation.min(MAX_ELEVATION) as f32;
    egui::epaint::Shadow {
        offset: [0, (e * 0.4) as i8],
        blur: (e * 1.5) as u8,
        spread: 0,
        color: Color32::from_black_alpha(((e * 3.0) as u8).min(60)),
    }
}

pub fn toggle_selection<T: Clone + PartialEq>(
    selected_item: &mut Option<T>,
    selected_items: &mut Vec<T>,
    item: &T,
    multi: bool,
) -> bool {
    if multi {
        if let Some(pos) = selected_items.iter().position(|s| s == item) {
            selected_items.remove(pos);
        } else {
            selected_items.push(item.clone());
        }
        *selected_item = None;
    } else if selected_item.as_ref() == Some(item) {
        // keep
    } else {
        *selected_item = Some(item.clone());
        selected_items.clear();
    }
    true
}

#[inline]
fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

fn resolve_class_fill(
    class: &str,
    lookup: &dyn Fn(&str) -> Option<Color32>,
    bs: &mut BsStyle,
    tw: &mut TwStyle,
    flavor: StyleFlavor,
    tw_state: TwState,
) -> Option<Color32> {
    if class.trim().is_empty() { return None; }
    let base = ComponentBase::new().classes(class.split_whitespace());
    let frame = match flavor {
        StyleFlavor::Bootstrap => bs.build_frame(&base, lookup),
        StyleFlavor::Tailwind => tw.build_frame(&base, tw_state, REM_SIZE_PT, lookup),
    };
    if frame.fill != Color32::TRANSPARENT { Some(frame.fill) } else { None }
}

// ---------------------------------------------------------------------------
// Column resolution (pure)
// ---------------------------------------------------------------------------

pub(crate) fn resolve_column_count<'a, T>(
    explicit: Option<&[Option<f32>]>,
    header: &[Vec<Th<'a, T>>],
    footer: &[Vec<Td<'a>>],
    body: &[(usize, Vec<Td<'a>>)],
) -> usize {
    if let Some(w) = explicit {
        if !w.is_empty() { return w.len(); }
    }
    let h = header.iter().map(|r| r.len()).max().unwrap_or(0);
    let f = footer.iter().map(|r| r.len()).max().unwrap_or(0);
    let b = body.iter().map(|(_, r)| r.len()).max().unwrap_or(0);
    h.max(f).max(b).max(1)
}

pub(crate) fn resolve_column_hints<'a, T>(
    explicit: Option<&[Option<f32>]>,
    header: &[Vec<Th<'a, T>>],
    footer: &[Vec<Td<'a>>],
    body: &[(usize, Vec<Td<'a>>)],
    count: usize,
) -> Vec<Option<f32>> {
    if let Some(w) = explicit {
        if w.len() == count { return w.to_vec(); }
    }
    let mut hints = vec![None; count];
    for r in header {
        for (i, c) in r.iter().enumerate().take(count) {
            if hints[i].is_none() { hints[i] = c.width; }
        }
    }
    for r in footer {
        for (i, c) in r.iter().enumerate().take(count) {
            if hints[i].is_none() { hints[i] = c.width; }
        }
    }
    for (_, r) in body {
        for (i, c) in r.iter().enumerate().take(count) {
            if hints[i].is_none() { hints[i] = c.width; }
        }
    }
    hints
}

// ---------------------------------------------------------------------------
// DrawCtx
// ---------------------------------------------------------------------------

struct DrawCtx<'b> {
    row_h: f32,
    total_w: f32,
    header_fill: Color32,
    footer_fill: Color32,
    body_border: Color32,
    accent: Color32,
    bordered: bool,
    striped: bool,
    hover: bool,
    has_selection: bool,
    selection_changeable: bool,
    select_on_row_click: bool,
    loading: bool,
    loading_text: Option<&'b str>,
    no_records_text: Option<&'b str>,
    row_class: Option<&'b str>,
    widths: &'b [f32],
    palette: &'b Palette,
    flavor: StyleFlavor,
    tw_state: TwState,
    allow_unsorted: bool,
}

struct TableCellView<'c, 'a> {
    content: &'c mut CellContent<'a>,
    hidden: bool,
    align: ColumnAlign,
}

// ---------------------------------------------------------------------------
// Leaf drawing helpers
// ---------------------------------------------------------------------------

fn draw_row<'a>(
    ui: &mut Ui,
    cells: &mut [TableCellView<'_, 'a>],
    kind: RowKind,
    y: f32,
    ctx: &DrawCtx<'_>,
) -> Rect {
    let rect = Rect::from_min_size(egui::pos2(ui.min_rect().left(), y), vec2(ctx.total_w, ctx.row_h));
    let fill = match kind {
        RowKind::Header => ctx.header_fill,
        RowKind::Footer => ctx.footer_fill,
        RowKind::Body => Color32::TRANSPARENT,
    };
    if fill != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, 0.0, fill);
    }
    let mut x = rect.left();
    for (ci, cell) in cells.iter_mut().enumerate() {
        if cell.hidden { continue; }
        let w = ctx.widths.get(ci).copied().unwrap_or(0.0);
        let cell_rect = Rect::from_min_size(egui::pos2(x, rect.top()), vec2(w, ctx.row_h));
        if ctx.bordered && ci > 0 {
            ui.painter().line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                Stroke::new(1.0, ctx.body_border),
            );
        }
        let bold = matches!(kind, RowKind::Header | RowKind::Footer);
        draw_cell_in_rect(ui, cell_rect, &mut cell.content, cell.align, bold);
        x += w;
    }
    ui.painter().line_segment(
        [egui::pos2(rect.left(), rect.bottom()), egui::pos2(rect.right(), rect.bottom())],
        Stroke::new(1.0, ctx.body_border),
    );
    rect
}

fn draw_cell_in_rect(
    ui: &mut Ui,
    rect: Rect,
    content: &mut CellContent<'_>,
    align: ColumnAlign,
    bold: bool,
) {
    let inner = Rect::from_min_max(
        egui::pos2(rect.left() + CELL_PADDING_X, rect.top()),
        egui::pos2(rect.right() - CELL_PADDING_X, rect.bottom()),
    );
    let mut child = ui.new_child(UiBuilder::new().max_rect(inner));
    child.set_clip_rect(rect);
    let layout = match align {
        ColumnAlign::Left => egui::Layout::left_to_right(egui::Align::Center),
        ColumnAlign::Center => egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
        ColumnAlign::Right => egui::Layout::right_to_left(egui::Align::Center),
    };
    child.with_layout(layout, |ui| match content {
        CellContent::Text(t) => {
            let mut txt = MudText::new(t.clone()).typo(Typo::Body2);
            if bold { txt = txt.bold(true); }
            ui.add(txt);
        }
        CellContent::Ui(f) => f(ui),
    });
}

fn draw_cell_content(
    ui: &mut Ui,
    content: &mut CellContent<'_>,
    align: ColumnAlign,
    bold: bool,
    _fill_height: bool,
) {
    let layout = match align {
        ColumnAlign::Left => egui::Layout::left_to_right(egui::Align::Center),
        ColumnAlign::Center => egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
        ColumnAlign::Right => egui::Layout::right_to_left(egui::Align::Center),
    };
    ui.with_layout(layout, |ui| match content {
        CellContent::Text(t) => {
            let mut txt = MudText::new(t.clone()).typo(Typo::Body2);
            if bold { txt = txt.bold(true); }
            ui.add(txt);
        }
        CellContent::Ui(f) => f(ui),
    });
}

fn paint_checkbox(ui: &mut Ui, rect: Rect, checked: bool, accent: Color32) {
    if checked {
        ui.put(rect, MudIcon::new(mud_egui_icons_material::filled::CHECK_BOX.to_owned())
            .size_px(rect.width()).custom_color(accent));
    } else {
        ui.put(rect, MudIcon::new(mud_egui_icons_material::filled::CHECK_BOX_OUTLINE_BLANK.to_owned())
            .size_px(rect.width()).custom_color(ui.visuals().weak_text_color()));
    }
}

// ---------------------------------------------------------------------------
// Header / Footer / Body row drawers (free functions)
// ---------------------------------------------------------------------------

fn draw_header_row<'a, T: Clone + PartialEq>(
    ui: &mut Ui,
    row: &mut [Th<'a, T>],
    state: &mut TableState<T>,
    y: &mut f32,
    ctx: &DrawCtx<'_>,
) {
    // 1. Draw visual cells in a scope so `cells` is dropped before step 2.
    {
        let mut cells: Vec<TableCellView<'_, 'a>> = row.iter_mut().map(|th| TableCellView {
            content: &mut th.content,
            hidden: th.hidden,
            align: th.align,
        }).collect();
        draw_row(ui, &mut cells, RowKind::Header, *y, ctx);
    }

    // 2. Sort interaction on top.
    let mut x = ui.min_rect().left();
    for (ci, th) in row.iter_mut().enumerate() {
        if th.hidden { continue; }
        let w = ctx.widths.get(ci).copied().unwrap_or(0.0);
        if th.is_sortable() {
            let cr = Rect::from_min_size(egui::pos2(x, *y), vec2(w, ctx.row_h));
            let resp = ui.interact(cr, ui.scope_id().with(("mud_table_sort", ci)), Sense::click());
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                ui.painter().rect_filled(cr, 0.0, ui.visuals().widgets.hovered.weak_bg_fill);
            }
            if resp.clicked() {
                if let Some(key) = th.sort_key.clone() {
                    toggle_sort(&mut state.sort_label, &mut state.sort_direction, &key, ctx.allow_unsorted);
                }
            }
            if let Some(key) = th.sort_key.as_deref() {
                if state.sort_label.as_deref() == Some(key) && state.sort_direction.is_active() {
                    let glyph = match state.sort_direction {
                        SortDirection::Ascending => mud_egui_icons_material::filled::KEYBOARD_ARROW_UP,
                        _ => mud_egui_icons_material::filled::KEYBOARD_ARROW_DOWN,
                    };
                    let ir = Rect::from_center_size(
                        egui::pos2(cr.right() - CELL_PADDING_X - SORT_ICON_SIZE * 0.5, cr.center().y),
                        vec2(SORT_ICON_SIZE, SORT_ICON_SIZE),
                    );
                    ui.put(ir, MudIcon::new(glyph.to_owned()).size_px(SORT_ICON_SIZE).custom_color(ctx.accent));
                }
            }
        }
        x += w;
    }
    *y += ctx.row_h;
}

fn draw_footer_row<'a>(ui: &mut Ui, row: &mut [Td<'a>], y: &mut f32, ctx: &DrawCtx<'_>) {
    let mut cells: Vec<TableCellView<'_, 'a>> = row.iter_mut().map(|td| TableCellView {
        content: &mut td.content,
        hidden: td.hidden,
        align: td.align,
    }).collect();
    draw_row(ui, &mut cells, RowKind::Footer, *y, ctx);
    *y += ctx.row_h;
}

#[allow(clippy::too_many_arguments)]
fn draw_body_row<'a, T: Clone + PartialEq>(
    ui: &mut Ui,
    pos: usize,
    abs: usize,
    cells: &mut [Td<'a>],
    item: &T,
    state: &mut TableState<T>,
    child_row: &mut Option<Box<dyn FnMut(&mut RowUi<'a>, &T) + 'a>>,
    row_click: &mut Option<Box<dyn FnMut(&T, usize) + 'a>>,
    row_mouse_enter: &mut Option<Box<dyn FnMut(&T, usize) + 'a>>,
    row_class_func: Option<&dyn Fn(&T, usize) -> String>,
    bs_style: &mut BsStyle,
    tw_style: &mut TwStyle,
    y: &mut f32,
    ctx: &DrawCtx<'_>,
) {
    let row_rect = Rect::from_min_size(
        egui::pos2(ui.min_rect().left(), *y),
        vec2(ctx.total_w, ctx.row_h),
    );
    state.last_row_rects.push(row_rect);

    let resp = ui.interact(row_rect, ui.scope_id().with(("mud_table_row", abs)), Sense::click());
    let hovered = resp.hovered() && ctx.hover;
    let selected = state.is_selected(item);

    let class_str = row_class_func.map(|f| f(item, abs)).or_else(|| ctx.row_class.map(|s| s.to_owned()));
    let class_fill = class_str.and_then(|c| {
        let lookup = |key: &str| ctx.palette.resolve(key).map(Color32::from);
        resolve_class_fill(&c, &lookup, bs_style, tw_style, ctx.flavor, ctx.tw_state)
    });

    let fill = class_fill.unwrap_or_else(|| {
        if selected { with_alpha(ctx.accent, 28) }
        else if hovered { ui.visuals().widgets.hovered.weak_bg_fill }
        else if ctx.striped && pos % 2 == 1 { ui.visuals().faint_bg_color }
        else { Color32::TRANSPARENT }
    });
    if fill != Color32::TRANSPARENT {
        ui.painter().rect_filled(row_rect, 0.0, fill);
    }

    let mut cells_offset = 0usize;
    if ctx.has_selection {
        let cb = Rect::from_center_size(
            egui::pos2(row_rect.left() + SELECTION_COL_WIDTH * 0.5, row_rect.center().y),
            vec2(CHECKBOX_SIZE, CHECKBOX_SIZE),
        );
        paint_checkbox(ui, cb, selected, ctx.accent);
        let cb_resp = ui.interact(cb, ui.scope_id().with(("mud_table_cb", abs)), Sense::click());
        if cb_resp.clicked() && ctx.selection_changeable {
            toggle_selection(&mut state.selected_item, &mut state.selected_items, item, true);
        }
        cells_offset = 1;
    }

    let mut x = row_rect.left() + if ctx.has_selection { SELECTION_COL_WIDTH } else { 0.0 };
    for (ci, cell) in cells.iter_mut().enumerate() {
        if cell.hidden { continue; }
        let w = ctx.widths.get(ci + cells_offset).copied().unwrap_or(0.0);
        let cr = Rect::from_min_size(egui::pos2(x, row_rect.top()), vec2(w, ctx.row_h));
        if ctx.bordered && ci > 0 {
            ui.painter().line_segment(
                [egui::pos2(x, row_rect.top()), egui::pos2(x, row_rect.bottom())],
                Stroke::new(1.0, ctx.body_border),
            );
        }
        draw_cell_in_rect(ui, cr, &mut cell.content, cell.align, false);
        x += w;
    }

    ui.painter().line_segment(
        [egui::pos2(row_rect.left(), row_rect.bottom()), egui::pos2(row_rect.right(), row_rect.bottom())],
        Stroke::new(1.0, ctx.body_border),
    );

    if resp.clicked() {
        if ctx.select_on_row_click && ctx.selection_changeable && ctx.has_selection {
            toggle_selection(&mut state.selected_item, &mut state.selected_items, item, true);
        } else if ctx.selection_changeable && !ctx.has_selection {
            toggle_selection(&mut state.selected_item, &mut state.selected_items, item, false);
        }
        if let Some(rc) = row_click.as_mut() { rc(item, abs); }
    }
    if resp.hovered() {
        if let Some(enter) = row_mouse_enter.as_mut() { enter(item, abs); }
    }

    *y += ctx.row_h;

    // Child row.
    if let Some(cb) = child_row.as_mut() {
        let mut cr = RowUi { cells: Vec::new() };
        cb(&mut cr, item);
        for (ci, cell) in cr.cells.iter_mut().enumerate() {
            if cell.hidden { continue; }
            let w = ctx.widths.get(ci).copied().unwrap_or(ctx.total_w);
            let crect = Rect::from_min_size(
                egui::pos2(ui.min_rect().left() + if ci == 0 { 0.0 } else { 0.0 }, *y),
                vec2(w, ctx.row_h),
            );
            draw_cell_in_rect(ui, crect, &mut cell.content, cell.align, false);
        }
        *y += ctx.row_h;
    }
}

// ---------------------------------------------------------------------------
// Pager
// ---------------------------------------------------------------------------

fn draw_table_pager<T>(
    ui: &mut Ui,
    table: &mut MudTable<'_, T>,
    state: &mut TableState<T>,
    total: usize,
) {
    let pages = page_count(total, state.rows_per_page);
    let (start, end) = page_range(total, state.current_page, state.rows_per_page);

    ui.horizontal_wrapped(|ui| {
        if !table.hide_rows_per_page && !table.page_size_options.is_empty() {
            ui.add(MudText::new(table.rows_per_page_string.clone()).typo(Typo::Caption).color(Color::TextSecondary));
            ui.add_space(4.0);
            let mut sel = vec![state.rows_per_page];
            MudToggleGroup::new()
                .selection_mode(SelectionMode::Single)
                .size(Size::Small)
                .show(ui, &mut sel, |g| {
                    for &opt in &table.page_size_options {
                        let label = if opt == usize::MAX { table.all_items_text.clone() } else { opt.to_string() };
                        g.item(opt, label);
                    }
                });
            if let Some(&chosen) = sel.first() {
                if chosen != state.rows_per_page {
                    state.rows_per_page = chosen;
                    state.current_page = 0;
                }
            }
            ui.add_space(12.0);
        }
        let info = if total == 0 { "0 of 0".to_owned() } else { format!("{}-{} of {}", start + 1, end, total) };
        ui.add(MudText::new(info).typo(Typo::Caption).color(Color::TextSecondary));
        if pages > 1 {
            ui.add_space(8.0);
            if MudButton::new("Previous").variant(Variant::Text).size(Size::Small)
                .disabled(state.current_page == 0).show(ui).clicked()
            {
                state.current_page = state.current_page.saturating_sub(1);
            }
            ui.add(MudText::new(format!("{}/{}", state.current_page + 1, pages)).typo(Typo::Body2).color(Color::TextSecondary));
            if MudButton::new("Next").variant(Variant::Text).size(Size::Small)
                .disabled(state.current_page + 1 >= pages).show(ui).clicked()
            {
                state.current_page = (state.current_page + 1).min(pages - 1);
            }
        }
    });
}

// ---------------------------------------------------------------------------
// MudTable
// ---------------------------------------------------------------------------

#[must_use = "MudTable does nothing until you call `.show(ui, items, state, |t| ..)`"]
pub struct MudTable<'a, T> {
    pub allow_unsorted: bool,
    pub filter: Option<Box<dyn Fn(&T) -> bool + 'a>>,
    pub pager: bool,
    pub current_page_override: Option<usize>,
    pub rows_per_page_override: Option<usize>,
    pub page_size_options: Vec<usize>,
    pub rows_per_page_string: String,
    pub all_items_text: String,
    pub hide_rows_per_page: bool,
    pub multi_selection: bool,
    pub select_on_row_click: bool,
    pub selection_changeable: bool,
    pub bordered: bool,
    pub dense: bool,
    pub elevation: u8,
    pub hover: bool,
    pub outlined: bool,
    pub square: bool,
    pub striped: bool,
    pub breakpoint: Breakpoint,
    pub item_size: Option<f32>,
    pub height: Option<f32>,
    pub fixed_header: bool,
    pub fixed_footer: bool,
    pub horizontal_scrollbar: bool,
    pub column_widths: Option<Vec<Option<f32>>>,
    pub row_class: Option<String>,
    pub row_class_func: Option<Box<dyn Fn(&T, usize) -> String + 'a>>,
    pub row_click: Option<Box<dyn FnMut(&T, usize) + 'a>>,
    pub row_mouse_enter: Option<Box<dyn FnMut(&T, usize) + 'a>>,
    pub row_mouse_leave: Option<Box<dyn FnMut(&T, usize) + 'a>>,
    pub toolbar: Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    pub loading: bool,
    pub loading_progress_color: Color,
    pub no_records_text: Option<String>,
    pub loading_text: Option<String>,
    pub aria_label: Option<String>,
    pub base: ComponentBase,
    pub container_classes: Option<String>,
    pub header_classes: Option<String>,
    pub footer_classes: Option<String>,
    pub flavor: StyleFlavor,
    pub bs_style: BsStyle,
    pub tw_style: TwStyle,
    pub tw_state: TwState,
    tag_value: Option<Arc<dyn Any + Send + Sync>>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a, T> Default for MudTable<'a, T> { fn default() -> Self { Self::new() } }

impl<'a, T> MudTable<'a, T> {
    pub fn new() -> Self {
        Self {
            allow_unsorted: true, filter: None, pager: true,
            current_page_override: None, rows_per_page_override: None,
            page_size_options: DEFAULT_PAGE_SIZE_OPTIONS.to_vec(),
            rows_per_page_string: "Rows per page:".to_owned(),
            all_items_text: "All".to_owned(),
            hide_rows_per_page: false,
            multi_selection: false, select_on_row_click: true, selection_changeable: true,
            bordered: false, dense: false, elevation: 1, hover: false,
            outlined: false, square: false, striped: false,
            breakpoint: Breakpoint::Xs, item_size: None, height: None,
            fixed_header: false, fixed_footer: false, horizontal_scrollbar: false,
            column_widths: None, row_class: None, row_class_func: None,
            row_click: None, row_mouse_enter: None, row_mouse_leave: None,
            toolbar: None, loading: false,
            loading_progress_color: Color::Info,
            no_records_text: None, loading_text: None, aria_label: None,
            base: ComponentBase::new(), container_classes: None,
            header_classes: None, footer_classes: None,
            flavor: StyleFlavor::default(), bs_style: BsStyle::default(),
            tw_style: TwStyle::default(), tw_state: TwState::default(),
            tag_value: None, _marker: std::marker::PhantomData,
        }
    }

    #[inline] pub fn allow_unsorted(mut self, b: bool) -> Self { self.allow_unsorted = b; self }
    #[inline] pub fn filter(mut self, f: impl Fn(&T) -> bool + 'a) -> Self { self.filter = Some(Box::new(f)); self }
    #[inline] pub fn pager(mut self, b: bool) -> Self { self.pager = b; self }
    #[inline] pub fn current_page(mut self, page: usize) -> Self { self.current_page_override = Some(page); self }
    #[inline] pub fn rows_per_page(mut self, n: usize) -> Self { self.rows_per_page_override = Some(n); self }
    pub fn page_size_options(mut self, options: impl IntoIterator<Item = usize>) -> Self { self.page_size_options = options.into_iter().collect(); self }
    #[inline] pub fn rows_per_page_string(mut self, s: impl Into<String>) -> Self { self.rows_per_page_string = s.into(); self }
    #[inline] pub fn all_items_text(mut self, s: impl Into<String>) -> Self { self.all_items_text = s.into(); self }
    #[inline] pub fn hide_rows_per_page(mut self, b: bool) -> Self { self.hide_rows_per_page = b; self }
    #[inline] pub fn multi_selection(mut self, b: bool) -> Self { self.multi_selection = b; self }
    #[inline] pub fn select_on_row_click(mut self, b: bool) -> Self { self.select_on_row_click = b; self }
    #[inline] pub fn selection_changeable(mut self, b: bool) -> Self { self.selection_changeable = b; self }
    #[inline] pub fn bordered(mut self, b: bool) -> Self { self.bordered = b; self }
    #[inline] pub fn dense(mut self, b: bool) -> Self { self.dense = b; self }
    #[inline] pub fn elevation(mut self, e: u8) -> Self { self.elevation = e; self }
    #[inline] pub fn hover(mut self, b: bool) -> Self { self.hover = b; self }
    #[inline] pub fn outlined(mut self, b: bool) -> Self { self.outlined = b; self }
    #[inline] pub fn square(mut self, b: bool) -> Self { self.square = b; self }
    #[inline] pub fn striped(mut self, b: bool) -> Self { self.striped = b; self }
    #[inline] pub fn breakpoint(mut self, bp: Breakpoint) -> Self { self.breakpoint = bp; self }
    #[inline] pub fn item_size(mut self, px: f32) -> Self { self.item_size = Some(px.max(0.0)); self }
    #[inline] pub fn height(mut self, h: f32) -> Self { self.height = Some(h.max(0.0)); self }
    #[inline] pub fn fixed_header(mut self, b: bool) -> Self { self.fixed_header = b; self }
    #[inline] pub fn fixed_footer(mut self, b: bool) -> Self { self.fixed_footer = b; self }
    #[inline] pub fn horizontal_scrollbar(mut self, b: bool) -> Self { self.horizontal_scrollbar = b; self }
    pub fn column_widths(mut self, widths: impl IntoIterator<Item = Option<f32>>) -> Self { self.column_widths = Some(widths.into_iter().collect()); self }
    #[inline] pub fn row_class(mut self, s: impl Into<String>) -> Self { self.row_class = Some(s.into()); self }
    #[inline] pub fn row_class_func(mut self, f: impl Fn(&T, usize) -> String + 'a) -> Self { self.row_class_func = Some(Box::new(f)); self }
    #[inline] pub fn on_row_click(mut self, f: impl FnMut(&T, usize) + 'a) -> Self { self.row_click = Some(Box::new(f)); self }
    #[inline] pub fn on_row_mouse_enter(mut self, f: impl FnMut(&T, usize) + 'a) -> Self { self.row_mouse_enter = Some(Box::new(f)); self }
    #[inline] pub fn on_row_mouse_leave(mut self, f: impl FnMut(&T, usize) + 'a) -> Self { self.row_mouse_leave = Some(Box::new(f)); self }
    #[inline] pub fn toolbar(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self { self.toolbar = Some(Box::new(f)); self }
    #[inline] pub fn loading(mut self, b: bool) -> Self { self.loading = b; self }
    #[inline] pub fn loading_progress_color(mut self, c: Color) -> Self { self.loading_progress_color = c; self }
    #[inline] pub fn no_records_text(mut self, s: impl Into<String>) -> Self { self.no_records_text = Some(s.into()); self }
    #[inline] pub fn loading_text(mut self, s: impl Into<String>) -> Self { self.loading_text = Some(s.into()); self }
    #[inline] pub fn aria_label(mut self, s: impl Into<String>) -> Self { self.aria_label = Some(s.into()); self }
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }
    #[inline] pub fn container_classes(mut self, s: impl Into<String>) -> Self { self.container_classes = Some(s.into()); self }
    #[inline] pub fn header_classes(mut self, s: impl Into<String>) -> Self { self.header_classes = Some(s.into()); self }
    #[inline] pub fn footer_classes(mut self, s: impl Into<String>) -> Self { self.footer_classes = Some(s.into()); self }
    #[inline] pub fn flavor(mut self, f: StyleFlavor) -> Self { self.flavor = f; self }
    pub fn tag<U: Any + Send + Sync>(mut self, value: U) -> Self { self.tag_value = Some(Arc::new(value)); self }
    pub fn tag_ref<U: Any + Send + Sync>(&self) -> Option<&U> { self.tag_value.as_ref()?.downcast_ref::<U>() }

    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        items: &[T],
        state: &mut TableState<T>,
        add_content: impl FnOnce(&mut TableUi<'a, T>) -> R,
    ) -> (Response, R)
    where
        T: Clone + PartialEq,
    {
        if let Some(n) = self.rows_per_page_override { state.rows_per_page = n; }
        if let Some(p) = self.current_page_override { state.current_page = p; }
        state.last_total_count = items.len();

        let mut collector = TableUi {
            header_rows: Vec::new(), footer_rows: Vec::new(),
            row_template: None, child_row: None,
        };
        let user_result = add_content(&mut collector);
        let TableUi { header_rows, footer_rows, row_template, child_row } = collector;
        let mut header_rows = header_rows;
        let mut footer_rows = footer_rows;
        let mut row_template = row_template;
        let mut child_row = child_row;

        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);

        self.tw_state.dark = ui.visuals().dark_mode;
        let mut frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => self.tw_style.build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
        };
        if self.elevation > 0 { frame = frame.shadow(elevation_shadow(self.elevation)); }

        let radius = if self.square { 0 } else { 4 };
        let row_h = effective_row_height(self.dense, self.item_size);
        let body_border = ui.visuals().widgets.noninteractive.bg_stroke.color;
        let accent: Color32 = Color::Primary.resolve(palette).unwrap_or_else(|| palette.primary.into());
        let header_fill: Color32 = self.header_classes.as_deref().and_then(|c| {
            resolve_class_fill(c, &lookup, &mut self.bs_style, &mut self.tw_style, self.flavor, self.tw_state)
        }).unwrap_or_else(|| palette.surface.into());
        let footer_fill: Color32 = self.footer_classes.as_deref().and_then(|c| {
            resolve_class_fill(c, &lookup, &mut self.bs_style, &mut self.tw_style, self.flavor, self.tw_state)
        }).unwrap_or(header_fill);

        let filtered: Vec<usize> = items.iter().enumerate()
            .filter(|(_, it)| self.filter.as_ref().map_or(true, |f| f(it)))
            .map(|(i, _)| i).collect();
        state.last_filtered_count = filtered.len();

        let sorted = {
            let mut out = filtered.clone();
            if state.sort_direction.is_active() {
                if let Some(label) = state.sort_label.as_deref() {
                    let cmp = header_rows.iter().flatten()
                        .find(|th| th.sort_key.as_deref() == Some(label))
                        .and_then(|th| th.comparator.as_deref());
                    if let Some(cmp) = cmp {
                        out = sort_indices_by(items, &filtered, state.sort_direction, cmp);
                    }
                }
            }
            out
        };

        state.current_page = clamp_page(state.current_page, sorted.len(), state.rows_per_page);
        let (start, end) = page_range(sorted.len(), state.current_page, state.rows_per_page);
        let page_items: Vec<usize> = sorted[start..end].to_vec();

        let mut page_rows: Vec<(usize, Vec<Td<'a>>)> = Vec::with_capacity(page_items.len());
        if let Some(tpl) = row_template.as_mut() {
            for &abs in &page_items {
                let mut row = RowUi { cells: Vec::new() };
                tpl(&mut row, &items[abs]);
                page_rows.push((abs, row.cells));
            }
        }

        let column_count = resolve_column_count(
            self.column_widths.as_deref(), &header_rows, &footer_rows, &page_rows,
        );
        let hints = resolve_column_hints(
            self.column_widths.as_deref(), &header_rows, &footer_rows, &page_rows, column_count,
        );

        let outer = frame.show(ui, |ui| {
            if let Some(tb) = self.toolbar.as_mut() { tb(ui); ui.add_space(4.0); }

            let available = ui.available_width();
            let available = if available.is_finite() && available > 0.0 { available } else { FALLBACK_WIDTH };
            let active = active_breakpoint(available);
            let stacked = is_stacked(active, self.breakpoint);

            let has_selection = self.multi_selection;
            let mut explicit = hints.clone();
            if has_selection { explicit.insert(0, Some(SELECTION_COL_WIDTH)); }
            let widths = resolve_column_widths(&explicit, available);
            let total_w: f32 = widths.iter().sum();

            // Derived immutable borrows used by drawing.
            let row_class_func_ref: Option<&dyn Fn(&T, usize) -> String> =
                self.row_class_func.as_deref();

            let ctx = DrawCtx {
                row_h, total_w, header_fill, footer_fill, body_border, accent,
                bordered: self.bordered, striped: self.striped, hover: self.hover,
                has_selection, selection_changeable: self.selection_changeable,
                select_on_row_click: self.select_on_row_click,
                loading: self.loading,
                loading_text: self.loading_text.as_deref(),
                no_records_text: self.no_records_text.as_deref(),
                row_class: self.row_class.as_deref(),
                widths: &widths,
                palette,
                flavor: self.flavor,
                tw_state: self.tw_state,
                allow_unsorted: self.allow_unsorted,
            };

            if stacked {
                state.last_row_rects.clear();
                ui.vertical(|ui| {
                    for (abs, cells) in page_rows.iter_mut() {
                        let item = &items[*abs];
                        let selected = state.is_selected(item);
                        let card = egui::Frame::default()
                            .fill(if selected { with_alpha(accent, 28) }
                            else if self.striped { ui.visuals().faint_bg_color }
                            else { ui.visuals().widgets.noninteractive.bg_fill })
                            .stroke(Stroke::new(1.0, body_border))
                            .corner_radius(CornerRadius::same(radius as u8))
                            .inner_margin(egui::Margin::same(10))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                if has_selection {
                                    let mut checked = selected;
                                    if ui.add(egui::Checkbox::without_text(&mut checked)).changed()
                                        && self.selection_changeable
                                    {
                                        toggle_selection(
                                            &mut state.selected_item, &mut state.selected_items,
                                            item, true,
                                        );
                                    }
                                }
                                for cell in cells.iter_mut().filter(|c| !c.hidden) {
                                    if let Some(label) = cell.label.as_deref() {
                                        ui.add(MudText::new(label.to_owned()).typo(Typo::Caption).color(Color::TextSecondary));
                                    }
                                    draw_cell_content(ui, &mut cell.content, cell.align, false, false);
                                    ui.add_space(2.0);
                                }
                            });
                        state.last_row_rects.push(card.response.rect);

                        if let Some(rc) = self.row_click.as_mut() {
                            if ui.interact(
                                card.response.rect,
                                ui.scope_id().with(("mud_table_card", *abs)),
                                Sense::click(),
                            ).clicked() {
                                rc(item, *abs);
                            }
                        }

                        if let Some(child) = child_row.as_mut() {
                            let mut cr = RowUi { cells: Vec::new() };
                            child(&mut cr, item);
                            for cell in cr.cells.iter_mut().filter(|c| !c.hidden) {
                                draw_cell_content(ui, &mut cell.content, cell.align, false, false);
                            }
                        }
                        ui.add_space(6.0);
                    }
                });
            } else {
                state.last_row_rects.clear();
                let header_h = header_rows.len() as f32 * row_h;
                let footer_h = footer_rows.len() as f32 * row_h;
                let body_h = page_rows.len() as f32 * row_h;
                let total_h = header_h + body_h + footer_h;
                let (table_rect, _) = ui.allocate_exact_size(vec2(total_w, total_h.max(row_h)), Sense::hover());
                let mut y = table_rect.top();

                let skip_header = self.fixed_header && self.height.is_some();
                let skip_footer = self.fixed_footer && self.height.is_some();

                // Fixed header (drawn before the scroll area).
                if skip_header {
                    for row in header_rows.iter_mut() {
                        draw_header_row(ui, row, state, &mut y, &ctx);
                    }
                }

                let cap = self.height.map(|h| {
                    (h - if skip_header { header_h } else { 0.0 }
                        - if skip_footer { footer_h } else { 0.0 })
                        .max(row_h)
                });

                // Content (single `run_content` closure, dropped at end of block).
                {
                    let mut run_content = |ui: &mut Ui, yy: &mut f32| {
                        if !skip_header {
                            for row in header_rows.iter_mut() {
                                draw_header_row(ui, row, state, yy, &ctx);
                            }
                        }
                        if page_rows.is_empty() {
                            let msg = if self.loading {
                                self.loading_text.as_deref().unwrap_or("Loading…")
                            } else {
                                self.no_records_text.as_deref().unwrap_or("No records to display")
                            };
                            let empty = Rect::from_min_size(egui::pos2(ui.min_rect().left(), *yy), vec2(total_w, row_h));
                            ui.painter().rect_filled(empty, 0.0, ui.visuals().faint_bg_color);
                            let mut child = ui.new_child(
                                UiBuilder::new().max_rect(empty).layout(
                                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                                ),
                            );
                            child.add(MudText::new(msg.to_owned()).typo(Typo::Body2).color(Color::TextSecondary));
                            *yy += row_h;
                        } else {
                            for (pos, (abs, cells)) in page_rows.iter_mut().enumerate() {
                                draw_body_row(
                                    ui, pos, *abs, cells, &items[*abs], state,
                                    &mut child_row,
                                    &mut self.row_click,
                                    &mut self.row_mouse_enter,
                                    row_class_func_ref,
                                    &mut self.bs_style,
                                    &mut self.tw_style,
                                    yy,
                                    &ctx,
                                );
                            }
                        }
                        if !skip_footer {
                            for row in footer_rows.iter_mut() {
                                draw_footer_row(ui, row, yy, &ctx);
                            }
                        }
                    };

                    if let Some(cap) = cap {
                        egui::ScrollArea::vertical().max_height(cap).auto_shrink([false, false])
                            .show(ui, |ui| {
                                let mut yy = y;
                                run_content(ui, &mut yy);
                            });
                    } else {
                        run_content(ui, &mut y);
                    }
                }

                // Fixed footer (drawn after the scroll area).
                if skip_footer {
                    for row in footer_rows.iter_mut() {
                        draw_footer_row(ui, row, &mut y, &ctx);
                    }
                }

                if self.outlined {
                    ui.painter().rect_stroke(
                        table_rect, CornerRadius::same(radius as u8),
                        Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
                        StrokeKind::Inside,
                    );
                }
            }

            if self.pager {
                ui.add_space(6.0);
                draw_table_pager(ui, &mut self, state, sorted.len());
            }

            if self.loading {
                ui.ctx().request_repaint();
                let overlay = ui.max_rect();
                ui.painter().rect_filled(overlay, CornerRadius::same(radius as u8), Color32::from_black_alpha(28));
                ui.put(overlay, egui::Spinner::new());
            }
        });

        self.tw_state.hovered = outer.response.hovered();
        self.tw_state.active = outer.response.is_pointer_button_down_on();

        if let Some(label) = &self.aria_label {
            outer.response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Table, true, label));
        }

        (outer.response, user_result)
    }
}

// ---------------------------------------------------------------------------
// Tests (unchanged)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Person { name: String, age: u32 }

    fn people() -> Vec<Person> {
        vec![
            Person { name: "Charlie".into(), age: 30 },
            Person { name: "Alice".into(), age: 25 },
            Person { name: "Bob".into(), age: 35 },
        ]
    }

    #[test]
    fn row_height_matches_density() {
        assert_eq!(row_height(false), ROW_HEIGHT);
        assert_eq!(row_height(true), DENSE_ROW_HEIGHT);
    }

    #[test]
    fn item_size_overrides_density() {
        assert_eq!(effective_row_height(false, None), ROW_HEIGHT);
        assert_eq!(effective_row_height(true, None), DENSE_ROW_HEIGHT);
        assert_eq!(effective_row_height(false, Some(64.0)), 64.0);
        assert_eq!(effective_row_height(false, Some(f32::NAN)), ROW_HEIGHT);
        assert_eq!(effective_row_height(false, Some(-1.0)), ROW_HEIGHT);
    }

    #[test]
    fn breakpoint_pixels_match_bootstrap() {
        assert_eq!(breakpoint_px(Breakpoint::Xs), 0.0);
        assert_eq!(breakpoint_px(Breakpoint::Sm), 600.0);
        assert_eq!(breakpoint_px(Breakpoint::Md), 960.0);
        assert_eq!(breakpoint_px(Breakpoint::Lg), 1280.0);
        assert_eq!(breakpoint_px(Breakpoint::Xl), 1920.0);
    }

    #[test]
    fn active_breakpoint_edges() {
        assert_eq!(active_breakpoint(0.0), Breakpoint::Xs);
        assert_eq!(active_breakpoint(599.0), Breakpoint::Xs);
        assert_eq!(active_breakpoint(600.0), Breakpoint::Sm);
        assert_eq!(active_breakpoint(960.0), Breakpoint::Md);
        assert_eq!(active_breakpoint(1920.0), Breakpoint::Xl);
    }

    #[test]
    fn stacking_is_strictly_below() {
        assert!(!is_stacked(Breakpoint::Xs, Breakpoint::Xs));
        assert!(is_stacked(Breakpoint::Xs, Breakpoint::Sm));
        assert!(!is_stacked(Breakpoint::Sm, Breakpoint::Sm));
        assert!(is_stacked(Breakpoint::Sm, Breakpoint::Md));
        assert!(!is_stacked(Breakpoint::Lg, Breakpoint::Md));
    }

    #[test]
    fn toggle_sort_cycles_ascending_and_descending() {
        let mut label = None;
        let mut dir = SortDirection::None;
        toggle_sort(&mut label, &mut dir, "name", false);
        assert_eq!(label.as_deref(), Some("name"));
        assert_eq!(dir, SortDirection::Ascending);
        toggle_sort(&mut label, &mut dir, "name", false);
        assert_eq!(dir, SortDirection::Descending);
        toggle_sort(&mut label, &mut dir, "name", false);
        assert_eq!(dir, SortDirection::Ascending);
    }

    #[test]
    fn toggle_sort_allows_unsorted_third_state() {
        let mut label = None;
        let mut dir = SortDirection::None;
        toggle_sort(&mut label, &mut dir, "name", true);
        toggle_sort(&mut label, &mut dir, "name", true);
        toggle_sort(&mut label, &mut dir, "name", true);
        assert_eq!(dir, SortDirection::None);
    }

    #[test]
    fn sort_indices_by_ascending_and_descending() {
        let data = people();
        let base: Vec<usize> = (0..data.len()).collect();
        let cmp = |a: &Person, b: &Person| a.name.cmp(&b.name);
        let asc = sort_indices_by(&data, &base, SortDirection::Ascending, &cmp);
        assert_eq!(
            asc.iter().map(|&i| data[i].name.as_str()).collect::<Vec<_>>(),
            vec!["Alice", "Bob", "Charlie"]
        );
        let desc = sort_indices_by(&data, &base, SortDirection::Descending, &cmp);
        assert_eq!(
            desc.iter().map(|&i| data[i].name.as_str()).collect::<Vec<_>>(),
            vec!["Charlie", "Bob", "Alice"]
        );
    }

    #[test]
    fn page_count_handles_zero_and_zero_size() {
        assert_eq!(page_count(0, 10), 0);
        assert_eq!(page_count(25, 10), 3);
        assert_eq!(page_count(20, 10), 2);
        assert_eq!(page_count(25, 0), 1);
    }

    #[test]
    fn clamp_page_bounds() {
        assert_eq!(clamp_page(0, 25, 10), 0);
        assert_eq!(clamp_page(9, 25, 10), 2);
        assert_eq!(clamp_page(0, 0, 10), 0);
    }

    #[test]
    fn page_range_slices() {
        assert_eq!(page_range(25, 0, 10), (0, 10));
        assert_eq!(page_range(25, 2, 10), (20, 25));
        assert_eq!(page_range(25, 0, 0), (0, 25));
        assert_eq!(page_range(5, 3, 10), (5, 5));
    }

    #[test]
    fn widths_split_remaining_space() {
        assert_eq!(resolve_column_widths(&[None, None, None], 600.0), vec![200.0, 200.0, 200.0]);
    }

    #[test]
    fn widths_mixed_fixed_and_flexible() {
        assert_eq!(resolve_column_widths(&[Some(100.0), None, None], 500.0), vec![100.0, 200.0, 200.0]);
    }

    #[test]
    fn widths_repair_non_finite_available() {
        assert_eq!(resolve_column_widths(&[None, None], f32::NAN), vec![FALLBACK_WIDTH / 2.0, FALLBACK_WIDTH / 2.0]);
    }

    #[test]
    fn widths_over_constrained_clamp_to_zero() {
        assert_eq!(resolve_column_widths(&[Some(400.0), None], 300.0), vec![400.0, 0.0]);
    }

    #[test]
    fn column_count_prefers_explicit_widths() {
        let explicit = [None, None, None];
        assert_eq!(resolve_column_count::<Person>(Some(&explicit), &[], &[], &[]), 3);
    }

    #[test]
    fn column_count_uses_widest_section() {
        let header = vec![vec![Th::<Person>::text("a"), Th::<Person>::text("b")]];
        let footer = vec![vec![Td::text("x"), Td::text("y"), Td::text("z")]];
        let body = vec![(0usize, vec![Td::text("1")])];
        assert_eq!(resolve_column_count::<Person>(None, &header, &footer, &body), 3);
    }

    #[test]
    fn column_count_defaults_to_one() {
        assert_eq!(resolve_column_count::<Person>(None, &[], &[], &[]), 1);
    }

    #[test]
    fn column_hints_take_first_explicit_width() {
        let header = vec![vec![Th::<Person>::text("h"), Th::<Person>::text("h").width(80.0)]];
        let body = vec![(0usize, vec![Td::text("r").width(200.0), Td::text("r")])];
        assert_eq!(resolve_column_hints::<Person>(None, &header, &[], &body, 2), vec![Some(200.0), Some(80.0)]);
    }

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
        assert_eq!(a.blur, b.blur);
        assert_eq!(a.offset, b.offset);
        assert_eq!(a.color, b.color);
    }

    #[test]
    fn single_selection_replaces_and_keeps() {
        let mut item = None;
        let mut items = Vec::new();
        let a = people()[0].clone();
        let b = people()[1].clone();
        toggle_selection(&mut item, &mut items, &a, false);
        assert_eq!(item, Some(a.clone()));
        toggle_selection(&mut item, &mut items, &a, false);
        assert_eq!(item, Some(a));
        toggle_selection(&mut item, &mut items, &b, false);
        assert_eq!(item, Some(b));
    }

    #[test]
    fn multi_selection_toggles_membership() {
        let mut item = None;
        let mut items = Vec::new();
        let a = people()[0].clone();
        let b = people()[1].clone();
        toggle_selection(&mut item, &mut items, &a, true);
        toggle_selection(&mut item, &mut items, &b, true);
        assert_eq!(items.len(), 2);
        toggle_selection(&mut item, &mut items, &a, true);
        assert_eq!(items, vec![b]);
    }

    #[test]
    fn state_is_selected_covers_both_shapes() {
        let mut st = TableState::<Person>::new();
        let p = people()[0].clone();
        st.selected_item = Some(p.clone());
        assert!(st.is_selected(&p));
        st.selected_item = None;
        st.selected_items.push(p.clone());
        assert!(st.is_selected(&p));
        assert!(!st.is_selected(&people()[1]));
    }

    #[test]
    fn table_defaults_match_mudblazor() {
        let t: MudTable<'static, Person> = MudTable::new();
        assert!(t.allow_unsorted && t.pager && !t.multi_selection);
        assert!(t.select_on_row_click && t.selection_changeable);
        assert_eq!(t.elevation, 1);
        assert_eq!(t.rows_per_page_string, "Rows per page:");
        assert_eq!(t.all_items_text, "All");
    }

    #[test]
    fn builders_set_every_field() {
        let t = MudTable::<Person>::new()
            .allow_unsorted(false)
            .filter(|p| p.age > 18)
            .pager(false)
            .current_page(2)
            .rows_per_page(5)
            .page_size_options([5, 10])
            .rows_per_page_string("Per page")
            .all_items_text("Everything")
            .hide_rows_per_page(true)
            .multi_selection(true)
            .select_on_row_click(false)
            .selection_changeable(false)
            .bordered(true).dense(true).elevation(4).hover(true)
            .outlined(true).square(true).striped(true)
            .breakpoint(Breakpoint::Md).item_size(64.0).height(300.0)
            .fixed_header(true).fixed_footer(true).horizontal_scrollbar(true)
            .column_widths([Some(100.0), None])
            .row_class("bg-surface")
            .row_class_func(|_, _| "x".into())
            .on_row_click(|_, _| {}).on_row_mouse_enter(|_, _| {}).on_row_mouse_leave(|_, _| {})
            .toolbar(|_| {})
            .loading(true).loading_progress_color(Color::Success)
            .no_records_text("empty").loading_text("busy").aria_label("people")
            .container_classes("c").header_classes("h").footer_classes("f")
            .classes(["a"]);
        assert!(!t.allow_unsorted && t.filter.is_some() && !t.pager);
        assert_eq!(t.current_page_override, Some(2));
        assert_eq!(t.rows_per_page_override, Some(5));
        assert!(t.multi_selection && t.bordered && t.striped);
    }

    #[test]
    fn tag_round_trips_by_type() {
        let t = MudTable::<Person>::new().tag("schedule");
        assert_eq!(t.tag_ref::<&str>(), Some(&"schedule"));
        assert!(t.tag_ref::<u32>().is_none());
    }

    #[test]
    fn th_sortability_requires_key_and_comparator() {
        assert!(!Th::<Person>::text("Name").is_sortable());
        assert!(Th::<Person>::text("Name").sort_by("name", |a, b| a.name.cmp(&b.name)).is_sortable());
        assert!(!Th::<Person>::text("Name").sort_by("name", |a, b| a.name.cmp(&b.name)).sortable(false).is_sortable());
    }

    #[test]
    fn collectors_preserve_order() {
        let mut t: TableUi<'static, Person> = TableUi {
            header_rows: Vec::new(), footer_rows: Vec::new(),
            row_template: None, child_row: None,
        };
        t.header(|h| { h.text("Name"); h.sort_th("Age", "age", |a, b| a.age.cmp(&b.age)); });
        t.footer(|f| f.text("Total"));
        t.row_template(|r, p| { r.text(p.name.clone()); r.text(p.age.to_string()); });
        t.child_row(|r, _| r.text("child"));
        assert_eq!(t.header_row_count(), 1);
        assert_eq!(t.footer_row_count(), 1);
        assert!(t.header_rows[0][1].is_sortable());
    }

    #[test]
    fn multiple_header_calls_append_rows() {
        let mut t: TableUi<'static, Person> = TableUi {
            header_rows: Vec::new(), footer_rows: Vec::new(),
            row_template: None, child_row: None,
        };
        t.header(|h| h.text("A"));
        t.header(|h| h.text("B"));
        assert_eq!(t.header_row_count(), 2);
    }
}