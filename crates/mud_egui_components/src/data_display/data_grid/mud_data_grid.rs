//! `MudDataGrid<T>` — a sortable, filterable, paginated data grid with
//! selection, ported from `MudDataGrid<T>.razor.cs` and `Column<T>.cs`
//! (MudBlazor 8.x).
//!
//! # Parity principle
//!
//! The MudBlazor parameter surface was verified before implementing. The grid
//! exposes the parameters that map cleanly onto egui's immediate-mode model:
//! sorting (`SortMode`, `AllowUnsorted`), filtering (`Filterable`,
//! `FilterMode`, `FilterCaseSensitivity`), pagination (`RowsPerPage`,
//! `CurrentPage`), selection (`MultiSelection`, `SelectOnRowClick`,
//! `SelectedItems`), presentation (`Bordered`, `Outlined`, `Hover`, `Striped`,
//! `Dense`, `Elevation`, `Square`), `Breakpoint`, `Height`, `Loading`,
//! `NoRecordsContent` (text form), `RowClass`, `RowClick`, `QuickFilter`,
//! `ToolBarContent`, `Class`, and `Tag`.
//!
//! Parameters that require a browser/DOM feature MudBlazor relies on
//! (`ServerData`, `Virtualize`, `EditMode`, `Groupable`, column resizing,
//! column reordering, popover-based `Simple` / `ColumnFilterMenu` filtering,
//! `ColumnsPanel`) are **not** implemented. Adding no-op builders for them
//! would be API invention. See `README.md` for the full rationale.
//!
//! # Per-frame model
//!
//! Items are borrowed for the duration of [`MudDataGrid::show`]; columns are
//! collected into a temporary `Vec<Column<'a, T>>` by the closure and dropped
//! afterwards. Sorting, filtering, pagination, and selection live in
//! [`DataGridState<T>`], which the caller owns across frames — the egui
//! equivalent of Blazor's two-way bindings (`SelectedItems`, `CurrentPage`,
//! `SortDefinitions`, `FilterDefinitions`).
//!
//! # Pure layer
//!
//! All layout and data-shaping decisions are pure functions
//! (`sort_indices`, `filter_indices`, `resolve_column_widths`,
//! `page_range`, `toggle_sort`, `toggle_selection`, …) so they are unit-tested
//! without an `egui::Context`. [`MudDataGrid::show`] only resolves the theme,
//! calls those helpers, allocates rectangles, and paints.

use std::any::Any;
use std::cmp::Ordering;
use std::rc::Rc;
use std::sync::Arc;

use egui::{
    vec2, Color32, CornerRadius, Direction, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    UiBuilder, Vec2,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::breakpoint::Breakpoint;
use mud_egui_core::enums::{Color, ColumnAlign, FilterMode, Size, SortDirection, SortMode, StyleFlavor, Typo, Variant};
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::typography::mud_text::MudText;
use crate::buttons::button::mud_button::MudButton;

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Width of the leading selection column, in logical pixels.
pub const SELECTION_COL_WIDTH: f32 = 48.0;

/// Row height for a non-dense grid.
pub const DEFAULT_ROW_HEIGHT: f32 = 52.0;

/// Row height for a dense grid.
pub const DENSE_ROW_HEIGHT: f32 = 36.0;

/// Header height for a non-dense grid.
pub const DEFAULT_HEADER_HEIGHT: f32 = 56.0;

/// Header height for a dense grid.
pub const DENSE_HEADER_HEIGHT: f32 = 40.0;

/// Height of the inline filter row when `FilterMode::ColumnFilterRow` is active.
pub const FILTER_ROW_HEIGHT: f32 = 48.0;

/// Height of the pager footer.
pub const FOOTER_HEIGHT: f32 = 48.0;

/// Default rows per page when the caller does not override it.
pub const DEFAULT_ROWS_PER_PAGE: usize = 10;

/// Fallback width used when the available width is non-finite.
pub const FALLBACK_WIDTH: f32 = 1280.0;

/// Horizontal padding inside a cell, in logical pixels.
pub const CELL_PADDING_X: f32 = 12.0;

/// Side length of a selection checkbox, in logical pixels.
const CHECKBOX_SIZE: f32 = 18.0;

/// Size of header sort / filter glyphs.
const HEADER_ICON_SIZE: f32 = 16.0;


/// One active sort, keyed by column id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortDefinition {
    /// The [`Column::id`] this sort applies to.
    pub column_id: String,
    /// The direction.
    pub direction: SortDirection,
}

impl SortDefinition {
    /// Creates a sort definition.
    pub fn new(column_id: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            column_id: column_id.into(),
            direction,
        }
    }
}

/// One active per-column filter.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ColumnFilter {
    /// The [`Column::id`] this filter applies to.
    pub column_id: String,
    /// The text to match against [`Column::filter_extractor`].
    pub value: String,
}

impl ColumnFilter {
    /// Creates a filter.
    pub fn new(column_id: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            column_id: column_id.into(),
            value: value.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// DataGridState
// ---------------------------------------------------------------------------

/// Caller-owned state that survives across frames.
///
/// This is the egui equivalent of MudBlazor's bindable parameters
/// `SelectedItems`, `CurrentPage`, `SortDefinitions`, and `FilterDefinitions`.
#[derive(Debug, Clone)]
pub struct DataGridState<T> {
    /// Active sorts, in priority order.
    pub sorts: Vec<SortDefinition>,
    /// Active per-column filters.
    pub filters: Vec<ColumnFilter>,
    /// Zero-based current page.
    pub current_page: usize,
    /// Rows per page. `0` means “all rows on one page”.
    pub rows_per_page: usize,
    /// Selected rows (single- or multi-selection).
    pub selected: Vec<T>,
    /// Text typed into the built-in search box (`show_search(true)`).
    pub quick_filter_text: String,

    /// Screen-space rectangles of the rows drawn in the most recent frame.
    /// Cleared and repopulated by [`MudDataGrid::show`].
    pub last_row_rects: Vec<Rect>,
    /// Number of items after filtering in the most recent frame.
    pub last_filtered_count: usize,
    /// Number of items passed to `show` in the most recent frame.
    pub last_total_count: usize,
}

impl<T> DataGridState<T> {
    /// Creates empty state with the MudBlazor default of 10 rows per page.
    pub fn new() -> Self {
        Self {
            sorts: Vec::new(),
            filters: Vec::new(),
            current_page: 0,
            rows_per_page: DEFAULT_ROWS_PER_PAGE,
            selected: Vec::new(),
            quick_filter_text: String::new(),
            last_row_rects: Vec::new(),
            last_filtered_count: 0,
            last_total_count: 0,
        }
    }

    /// Convenience: sets [`rows_per_page`](Self::rows_per_page).
    pub fn with_rows_per_page(mut self, n: usize) -> Self {
        self.rows_per_page = n;
        self
    }
}

impl<T> Default for DataGridState<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Column
// ---------------------------------------------------------------------------

/// A single column in a [`MudDataGrid`].
///
/// The cell renderer is a `Fn(&mut Ui, &T)`; it draws whatever the cell should
/// contain. Sorting and filtering use explicit closures so `T` needs no `Ord`
/// or `Display` bound — the same freedom MudBlazor's `Comparer` and
/// `PropertyColumn.Property` provide.
pub struct Column<'a, T: 'a> {
    /// Stable identifier used by sort and filter definitions.
    pub id: String,
    /// Header caption.
    pub title: String,
    /// Overrides the grid's `filterable` flag when set.
    pub sortable: Option<bool>,
    /// Overrides the grid's `filterable` flag when set.
    pub filterable: Option<bool>,
    /// Hidden columns are not rendered and take no width.
    pub hidden: bool,
    /// Horizontal alignment of the header and cells.
    pub align: ColumnAlign,
    /// Fixed width in pixels; `None` means flexible.
    pub width: Option<f32>,
    /// Draws the cell content.
    pub cell: Box<dyn Fn(&mut Ui, &T) + 'a>,
    /// Comparator used for sorting. A column without one is not sortable.
    pub comparator: Option<Box<dyn Fn(&T, &T) -> Ordering + 'a>>,
    /// Extracts the text compared against [`ColumnFilter::value`] and the
    /// search box. A column without one cannot be filtered.
    pub filter_extractor: Option<Box<dyn Fn(&T) -> String + 'a>>,
}

impl<'a, T: 'a> Column<'a, T> {
    /// Creates a column with a custom cell renderer. Sorting and filtering are
    /// opt-in; a plain column is display-only.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        cell: impl Fn(&mut Ui, &T) + 'a,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            sortable: None,
            filterable: None,
            hidden: false,
            align: ColumnAlign::Left,
            width: None,
            cell: Box::new(cell),
            comparator: None,
            filter_extractor: None,
        }
    }

    /// Creates a text column. The same extractor powers the cell, the column
    /// filter, and the search box.
    pub fn text(
        id: impl Into<String>,
        title: impl Into<String>,
        get: impl Fn(&T) -> String + 'a,
    ) -> Self {
        let get: Rc<dyn Fn(&T) -> String + 'a> = Rc::new(get);
        let cell_get = Rc::clone(&get);
        let filter_get = Rc::clone(&get);
        Self {
            id: id.into(),
            title: title.into(),
            sortable: None,
            filterable: None,
            hidden: false,
            align: ColumnAlign::Left,
            width: None,
            cell: Box::new(move |ui, row| {
                ui.add(MudText::new(cell_get(row)).typo(Typo::Body2));
            }),
            comparator: None,
            filter_extractor: Some(Box::new(move |row| filter_get(row))),
        }
    }

    /// Overrides sorting for this column.
    #[inline]
    pub fn sortable(mut self, b: bool) -> Self {
        self.sortable = Some(b);
        self
    }

    /// Overrides filtering for this column.
    #[inline]
    pub fn filterable(mut self, b: bool) -> Self {
        self.filterable = Some(b);
        self
    }

    /// Hides the column.
    #[inline]
    pub fn hidden(mut self, b: bool) -> Self {
        self.hidden = b;
        self
    }

    /// Sets the column alignment.
    #[inline]
    pub fn align(mut self, a: ColumnAlign) -> Self {
        self.align = a;
        self
    }

    /// Sets a fixed width in pixels.
    #[inline]
    pub fn width(mut self, px: f32) -> Self {
        self.width = Some(px.max(0.0));
        self
    }

    /// Provides the sort comparator.
    #[inline]
    pub fn sort_by(mut self, cmp: impl Fn(&T, &T) -> Ordering + 'a) -> Self {
        self.comparator = Some(Box::new(cmp));
        self
    }

    /// Provides the filter / search extractor.
    #[inline]
    pub fn filter_text(mut self, get: impl Fn(&T) -> String + 'a) -> Self {
        self.filter_extractor = Some(Box::new(get));
        self
    }

    /// Resolved sortability: the override if set, and a comparator must exist.
    #[inline]
    pub fn is_sortable(&self) -> bool {
        self.sortable.unwrap_or(true) && self.comparator.is_some()
    }

    /// Resolved filterability against the grid-level flag.
    #[inline]
    pub fn is_filterable(&self, grid_filterable: bool) -> bool {
        self.filterable.unwrap_or(grid_filterable) && self.filter_extractor.is_some()
    }
}

// ---------------------------------------------------------------------------
// GridUi
// ---------------------------------------------------------------------------

/// Collector passed to [`MudDataGrid::show`]'s closure.
pub struct GridUi<'a, T: 'a> {
    columns: Vec<Column<'a, T>>,
}

impl<'a, T: 'a> GridUi<'a, T> {
    /// Adds a fully configured column.
    pub fn column(&mut self, column: Column<'a, T>) {
        self.columns.push(column);
    }

    /// Adds a text column (see [`Column::text`]).
    pub fn text_column(
        &mut self,
        id: impl Into<String>,
        title: impl Into<String>,
        get: impl Fn(&T) -> String + 'a,
    ) {
        self.columns.push(Column::text(id, title, get));
    }

    /// Number of columns registered so far.
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    /// Whether no columns have been registered.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
}

// ---------------------------------------------------------------------------
// MudDataGrid
// ---------------------------------------------------------------------------

/// A sortable, filterable, paginated data grid with selection.
#[must_use = "MudDataGrid does nothing until you call `.show(ui, items, state, ..)`"]
pub struct MudDataGrid<'a, T: 'a> {
    // Sorting
    /// How many columns may be sorted at once. Mirrors `SortMode`.
    pub sort_mode: SortMode,
    /// Whether a third “unsorted” toggle state is allowed. Mirrors
    /// `AllowUnsorted`.
    pub allow_unsorted: bool,

    // Filtering
    /// Whether filtering is enabled at all. Mirrors `Filterable`.
    pub filterable: bool,
    /// Where filter inputs render. Mirrors `FilterMode`.
    pub filter_mode: FilterMode,
    /// Case sensitivity for string filtering. Mirrors
    /// `FilterCaseSensitivity`.
    pub filter_case_sensitive: bool,
    /// Renders a built-in search box above the header (extension).
    pub show_search: bool,

    // Selection
    /// Allows more than one selected row. Mirrors `MultiSelection`.
    pub multi_selection: bool,
    /// Toggles a row's checkbox when the row itself is clicked. Mirrors
    /// `SelectOnRowClick` (default `true`).
    pub select_on_row_click: bool,

    // Presentation
    /// Left/right borders on each column. Mirrors `Bordered`.
    pub bordered: bool,
    /// Outline around the whole grid. Mirrors `Outlined`.
    pub outlined: bool,
    /// Row highlight on hover. Mirrors `Hover`.
    pub hover: bool,
    /// Alternating row background. Mirrors `Striped`.
    pub striped: bool,
    /// Compact padding and row height. Mirrors `Dense`.
    pub dense: bool,
    /// Removes corner rounding. Mirrors `Square`.
    pub square: bool,
    /// Drop-shadow depth (0–24). Mirrors `Elevation` (default `1`).
    pub elevation: u8,
    /// Below this breakpoint the grid stacks each row vertically. Mirrors
    /// `Breakpoint`.
    pub breakpoint: Breakpoint,

    // Data / layout
    /// Maximum body height; enables a vertical scroll area. Mirrors `Height`.
    pub height: Option<f32>,
    /// Overrides [`DataGridState::current_page`] when set. Mirrors
    /// `CurrentPage`.
    pub current_page: Option<usize>,
    /// Overrides [`DataGridState::rows_per_page`] when set. Mirrors
    /// `RowsPerPage`.
    pub rows_per_page: Option<usize>,
    /// Shows the pager footer. Mirrors the presence of `PagerContent`.
    pub show_pager: bool,
    /// Overrides the resolved wrapper frame. egui analogue of `Style`.
    pub frame_override: Option<egui::Frame>,

    // Status
    /// Shows a loading overlay. Mirrors `Loading`.
    pub loading: bool,
    /// Colour of the loading indicator. Mirrors `LoadingProgressColor`.
    pub loading_progress_color: Color,
    /// Text shown when there are no rows. Mirrors the text form of
    /// `NoRecordsContent`.
    pub no_records_text: Option<String>,
    /// Accessible label published on the grid's node.
    pub aria_label: Option<String>,

    // Callbacks / content
    /// Extra row CSS classes, resolved through the grid's flavour. Mirrors
    /// `RowClass`.
    pub row_class: Option<Box<dyn Fn(&T, usize) -> String + 'a>>,
    /// Invoked when a row is clicked. Mirrors `RowClick`.
    pub row_click: Option<Box<dyn FnMut(&T, usize) + 'a>>,
    /// Extra predicate every row must satisfy. Mirrors `QuickFilter`.
    pub quick_filter: Option<Box<dyn Fn(&T) -> bool + 'a>>,
    /// Custom toolbar content above the header. Mirrors `ToolBarContent`.
    pub toolbar: Option<Box<dyn FnMut(&mut Ui) + 'a>>,

    // CSS utilities
    /// Base component state (utility classes).
    pub base: ComponentBase,
    /// Bootstrap or Tailwind resolver.
    pub flavor: StyleFlavor,
    /// Bootstrap resolver state.
    pub bs_style: BsStyle,
    /// Tailwind resolver state.
    pub tw_style: TwStyle,
    /// Tailwind interaction state.
    pub tw_state: TwState,

    /// Arbitrary user data (`Tag`).
    tag_value: Option<Arc<dyn Any + Send + Sync>>,
}

impl<'a, T: 'a> Default for MudDataGrid<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: 'a> MudDataGrid<'a, T> {
    /// Creates a grid with MudBlazor defaults.
    pub fn new() -> Self {
        Self {
            sort_mode: SortMode::Multiple,
            allow_unsorted: false,
            filterable: false,
            filter_mode: FilterMode::ColumnFilterRow,
            filter_case_sensitive: false,
            show_search: false,
            multi_selection: false,
            select_on_row_click: true,
            bordered: false,
            outlined: false,
            hover: false,
            striped: false,
            dense: false,
            square: false,
            elevation: 1,
            breakpoint: Breakpoint::Xs,
            height: None,
            current_page: None,
            rows_per_page: None,
            show_pager: true,
            frame_override: None,
            loading: false,
            loading_progress_color: Color::Info,
            no_records_text: None,
            aria_label: None,
            row_class: None,
            row_click: None,
            quick_filter: None,
            toolbar: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
            tag_value: None,
        }
    }

    // ── Builders: sorting ──────────────────────────────────────────────────

    #[inline]
    pub fn sort_mode(mut self, m: SortMode) -> Self {
        self.sort_mode = m;
        self
    }

    #[inline]
    pub fn allow_unsorted(mut self, b: bool) -> Self {
        self.allow_unsorted = b;
        self
    }

    // ── Builders: filtering ────────────────────────────────────────────────

    #[inline]
    pub fn filterable(mut self, b: bool) -> Self {
        self.filterable = b;
        self
    }

    #[inline]
    pub fn filter_mode(mut self, m: FilterMode) -> Self {
        self.filter_mode = m;
        self
    }

    #[inline]
    pub fn filter_case_sensitive(mut self, b: bool) -> Self {
        self.filter_case_sensitive = b;
        self
    }

    /// Renders a built-in search box (extension; MudBlazor exposes
    /// `QuickFilter` as a predicate instead).
    #[inline]
    pub fn show_search(mut self, b: bool) -> Self {
        self.show_search = b;
        self
    }

    // ── Builders: selection ────────────────────────────────────────────────

    #[inline]
    pub fn multi_selection(mut self, b: bool) -> Self {
        self.multi_selection = b;
        self
    }

    #[inline]
    pub fn select_on_row_click(mut self, b: bool) -> Self {
        self.select_on_row_click = b;
        self
    }

    // ── Builders: presentation ─────────────────────────────────────────────

    #[inline]
    pub fn bordered(mut self, b: bool) -> Self {
        self.bordered = b;
        self
    }

    #[inline]
    pub fn outlined(mut self, b: bool) -> Self {
        self.outlined = b;
        self
    }

    #[inline]
    pub fn hover(mut self, b: bool) -> Self {
        self.hover = b;
        self
    }

    #[inline]
    pub fn striped(mut self, b: bool) -> Self {
        self.striped = b;
        self
    }

    #[inline]
    pub fn dense(mut self, b: bool) -> Self {
        self.dense = b;
        self
    }

    #[inline]
    pub fn square(mut self, b: bool) -> Self {
        self.square = b;
        self
    }

    #[inline]
    pub fn elevation(mut self, e: u8) -> Self {
        self.elevation = e;
        self
    }

    #[inline]
    pub fn breakpoint(mut self, bp: Breakpoint) -> Self {
        self.breakpoint = bp;
        self
    }

    // ── Builders: data / layout ────────────────────────────────────────────

    #[inline]
    pub fn height(mut self, h: f32) -> Self {
        self.height = Some(h);
        self
    }

    #[inline]
    pub fn current_page(mut self, p: usize) -> Self {
        self.current_page = Some(p);
        self
    }

    #[inline]
    pub fn rows_per_page(mut self, n: usize) -> Self {
        self.rows_per_page = Some(n);
        self
    }

    #[inline]
    pub fn show_pager(mut self, b: bool) -> Self {
        self.show_pager = b;
        self
    }

    #[inline]
    pub fn frame(mut self, f: egui::Frame) -> Self {
        self.frame_override = Some(f);
        self
    }

    // ── Builders: status ───────────────────────────────────────────────────

    #[inline]
    pub fn loading(mut self, b: bool) -> Self {
        self.loading = b;
        self
    }

    #[inline]
    pub fn loading_progress_color(mut self, c: Color) -> Self {
        self.loading_progress_color = c;
        self
    }

    #[inline]
    pub fn no_records_text(mut self, s: impl Into<String>) -> Self {
        self.no_records_text = Some(s.into());
        self
    }

    #[inline]
    pub fn aria_label(mut self, s: impl Into<String>) -> Self {
        self.aria_label = Some(s.into());
        self
    }

    // ── Builders: callbacks ────────────────────────────────────────────────

    #[inline]
    pub fn row_class(mut self, f: impl Fn(&T, usize) -> String + 'a) -> Self {
        self.row_class = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn row_click(mut self, f: impl FnMut(&T, usize) + 'a) -> Self {
        self.row_click = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn quick_filter(mut self, f: impl Fn(&T) -> bool + 'a) -> Self {
        self.quick_filter = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn toolbar(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self {
        self.toolbar = Some(Box::new(f));
        self
    }

    // ── Builders: CSS / tag ────────────────────────────────────────────────

    #[inline]
    pub fn flavor(mut self, f: StyleFlavor) -> Self {
        self.flavor = f;
        self
    }

    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Attaches arbitrary user data (`Tag`).
    pub fn tag<Tag: Any + Send + Sync>(mut self, value: Tag) -> Self {
        self.tag_value = Some(Arc::new(value));
        self
    }

    /// Reads the tag back if it is of type `U`.
    pub fn tag_ref<U: Any + Send + Sync>(&self) -> Option<&U> {
        self.tag_value.as_ref()?.downcast_ref::<U>()
    }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the grid and returns `(response, add_columns_result)`.
    ///
    /// `state` is updated in place: page clamping, selection, sort and filter
    /// changes are written back so the caller can persist them.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        items: &[T],
        state: &mut DataGridState<T>,
        add_columns: impl FnOnce(&mut GridUi<'a, T>) -> R,
    ) -> (Response, R)
    where
        T: Clone + PartialEq,
    {
        // 1. Apply parameter overrides onto the caller-owned state.
        if let Some(rpp) = self.rows_per_page {
            state.rows_per_page = rpp;
        }
        if let Some(p) = self.current_page {
            state.current_page = p;
        }
        state.last_total_count = items.len();

        // 2. Collect columns.
        let mut grid_ui = GridUi {
            columns: Vec::new(),
        };
        let user_result = add_columns(&mut grid_ui);
        let columns = grid_ui.columns;

        if columns.iter().all(|c| c.hidden) {
            state.last_filtered_count = 0;
            state.last_row_rects.clear();
            return (
                ui.allocate_response(Vec2::ZERO, Sense::hover()),
                user_result,
            );
        }

        // 3. Theme.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| -> Option<Color32> { palette.resolve(key).map(|c| Color32::from(c)) };

        // 4. Wrapper frame.
        self.tw_state.dark = ui.visuals().dark_mode;
        let mut frame = if let Some(f) = self.frame_override.take() {
            f
        } else {
            match self.flavor {
                StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
                StyleFlavor::Tailwind => self.tw_style.build_frame(
                    &self.base,
                    self.tw_state,
                    REM_SIZE_PT,
                    &lookup,
                ),
            }
        };
        if self.elevation > 0 {
            frame = frame.shadow(elevation_shadow(self.elevation));
        }

        // 5. Take callbacks out of `self` so the render closure only captures
        //    disjoint fields.
        let mut row_click = self.row_click.take();
        let row_class = self.row_class.take();
        let mut toolbar = self.toolbar.take();
        let quick_filter = self.quick_filter.take();

        // 6. Geometry constants.
        let row_h = row_height(self.dense);
        let header_h = header_height(self.dense);
        let show_filter_row =
            self.filterable && self.filter_mode == FilterMode::ColumnFilterRow;
        let filter_h = if show_filter_row { FILTER_ROW_HEIGHT } else { 0.0 };

        let radius = if self.square { 0.0 } else { 4.0 };
        let accent: Color32 = Color::Primary
            .resolve(palette)
            .unwrap_or_else(|| palette.primary.into());
        let selection_accent = accent;

        let outer = frame.show(ui, |ui| {
            // 7. Toolbar / search.
            if let Some(t) = toolbar.as_mut() {
                t(ui);
                ui.add_space(4.0);
            }
            if self.show_search {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut state.quick_filter_text)
                            .hint_text("Search…")
                            .desired_width(220.0),
                    );
                });
                ui.add_space(4.0);
            }

            // 8. Filtering + sorting + pagination (pure).
            let filtered = filter_indices(
                items,
                &state.filters,
                &columns,
                &state.quick_filter_text,
                quick_filter.as_deref(),
                self.filter_case_sensitive,
            );
            let sorted = sort_indices(items, &state.sorts, &columns, &filtered);
            state.last_filtered_count = sorted.len();

            let total = sorted.len();
            state.current_page = clamp_page(state.current_page, total, state.rows_per_page);
            let (start, end) = page_range(total, state.current_page, state.rows_per_page);
            let page_items = &sorted[start..end];

            // 9. Available width and responsive decision.
            let available = ui.available_width();
            let available = if available.is_finite() && available > 0.0 {
                available
            } else {
                FALLBACK_WIDTH
            };
            let active_bp = active_breakpoint(available);
            let stacked = is_stacked(active_bp, self.breakpoint);

            if stacked {
                // ── Responsive card layout ────────────────────────────────
                state.last_row_rects.clear();
                let mut card_rects = Vec::with_capacity(page_items.len());
                ui.vertical(|ui| {
                    for (pos, &abs_idx) in page_items.iter().enumerate() {
                        let item = &items[abs_idx];
                        let selected = state.selected.iter().any(|s| s == item);
                        let card = ui.scope(|ui| {
                            let card_frame = egui::Frame::default()
                                .fill(row_fill(
                                    pos,
                                    selected,
                                    self.striped,
                                    false,
                                    accent,
                                    ui,
                                ))
                                .stroke(Stroke::new(
                                    1.0,
                                    ui.visuals().widgets.noninteractive.bg_stroke.color,
                                ))
                                .corner_radius(CornerRadius::same(radius as u8))
                                .inner_margin(egui::Margin::same(10));
                            card_frame.show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                if self.multi_selection {
                                    let mut checked = selected;
                                    if ui
                                        .add(
                                            egui::Checkbox::without_text(&mut checked),
                                        )
                                        .changed()
                                    {
                                        toggle_selection(
                                            &mut state.selected,
                                            item,
                                            true,
                                        );
                                    }
                                }
                                for col in columns.iter().filter(|c| !c.hidden) {
                                    ui.horizontal(|ui| {
                                        ui.add(
                                            MudText::new(col.title.clone())
                                                .typo(Typo::Caption)
                                                .color(Color::TextSecondary),
                                        );
                                        ui.add_space(6.0);
                                        (col.cell)(ui, item);
                                    });
                                }
                            });
                        });
                        card_rects.push(card.response.rect);
                        if let Some(rc) = row_click.as_mut() {
                            let card_id = ui.scope_id().with(("dg_card", abs_idx));
                            if ui
                                .interact(
                                    card.response.rect,
                                    card_id,
                                    Sense::click(),
                                )
                                .clicked()
                            {
                                rc(item, abs_idx);
                            }
                        }
                    }
                });
                state.last_row_rects = card_rects;

                if self.show_pager {
                    draw_pager(ui, state, total, self.dense);
                }
            } else {
                // ── Table layout ──────────────────────────────────────────
                let has_selection = self.multi_selection;
                let widths = resolve_column_widths(
                    &columns,
                    available,
                    SELECTION_COL_WIDTH,
                    has_selection,
                );
                let widths = &widths;
                let content_width: f32 = widths.iter().sum();

                // Header.
                let (header_rect, _) = ui.allocate_exact_size(
                    vec2(content_width, header_h),
                    Sense::hover(),
                );
                draw_header(
                    ui,
                    header_rect,
                    &columns,
                    widths,
                    &mut state.sorts,
                    self.sort_mode,
                    self.allow_unsorted,
                    self.filterable,
                    has_selection,
                    self.bordered,
                    self.dense,
                    accent,
                    &mut state.selected,
                    items,
                    &sorted,
                );

                // Filter row.
                if show_filter_row {
                    let (filter_rect, _) = ui.allocate_exact_size(
                        vec2(content_width, filter_h),
                        Sense::hover(),
                    );
                    draw_filter_row(
                        ui,
                        filter_rect,
                        &columns,
                        widths,
                        &mut state.filters,
                        has_selection,
                        self.bordered,
                        accent,
                    );
                }

                // Body.
                let rows = page_items.len();
                let body_h = rows as f32 * row_h;
                let footer_h = if self.show_pager { FOOTER_HEIGHT } else { 0.0 };
                let body_cap = self
                    .height
                    .map(|h| (h - header_h - filter_h - footer_h).max(row_h * 1.5));

                let mut draw_body = |ui: &mut Ui| {
                    state.last_row_rects.clear();
                    let (body_rect, _) = ui.allocate_exact_size(
                        vec2(content_width, body_h.max(row_h)),
                        Sense::hover(),
                    );

                    if page_items.is_empty() {
                        let msg = self
                            .no_records_text
                            .as_deref()
                            .unwrap_or("No records to display");
                        ui.painter().rect_filled(
                            body_rect,
                            CornerRadius::same(radius as u8),
                            ui.visuals().faint_bg_color,
                        );
                        ui.put(
                            body_rect,
                            egui::Label::new(
                                egui::RichText::new(msg)
                                    .size(14.0)
                                    .color(ui.visuals().weak_text_color()),
                            ),
                        );
                        return;
                    }

                    let mut y = body_rect.top();
                    for (pos, &abs_idx) in page_items.iter().enumerate() {
                        let item = &items[abs_idx];
                        let row_rect = Rect::from_min_size(
                            egui::pos2(body_rect.left(), y),
                            vec2(content_width, row_h),
                        );
                        state.last_row_rects.push(row_rect);

                        let selected = state.selected.iter().any(|s| s == item);
                        let row_id = ui.scope_id().with(("dg_row", abs_idx));
                        let row_resp = ui.interact(
                            row_rect,
                            row_id,
                            Sense::click(),
                        );
                        let hovered = row_resp.hovered() && self.hover;

                        let class_fill = row_class
                            .as_ref()
                            .and_then(|rc| resolve_row_class_fill(&rc(item, abs_idx), &lookup, &mut self.bs_style, &mut self.tw_style, self.flavor, self.tw_state));

                        let fill = class_fill.unwrap_or_else(|| {
                            row_fill(pos, selected, self.striped, hovered, accent, ui)
                        });
                        ui.painter()
                            .rect_filled(row_rect, 0.0, fill);

                        // Selection checkbox.
                        let mut x = row_rect.left();
                        if has_selection {
                            let cb_rect = Rect::from_center_size(
                                egui::pos2(
                                    x + SELECTION_COL_WIDTH * 0.5,
                                    row_rect.center().y,
                                ),
                                vec2(CHECKBOX_SIZE, CHECKBOX_SIZE),
                            );
                            let cb_id = ui.scope_id().with(("dg_cb", abs_idx));
                            let cb_resp = ui.interact(
                                cb_rect,
                                cb_id,
                                Sense::click(),
                            );
                            paint_checkbox(ui, cb_rect, selected, selection_accent);
                            if cb_resp.clicked() {
                                toggle_selection(
                                    &mut state.selected,
                                    item,
                                    self.multi_selection,
                                );
                            }
                            x += SELECTION_COL_WIDTH;
                        }

                        // Cells.
                        let mut col_x = x;
                        for (vi, col) in columns.iter().filter(|c| !c.hidden).enumerate() {
                            let w = widths[vi + if has_selection { 1 } else { 0 }];
                            let cell_rect =
                                Rect::from_min_size(egui::pos2(col_x, row_rect.top()), vec2(w, row_h));
                            if self.bordered && vi > 0 {
                                ui.painter().line_segment(
                                    [
                                        egui::pos2(col_x, row_rect.top()),
                                        egui::pos2(col_x, row_rect.bottom()),
                                    ],
                                    Stroke::new(
                                        1.0,
                                        ui.visuals().widgets.noninteractive.bg_stroke.color,
                                    ),
                                );
                            }
                            let inner = Rect::from_min_max(
                                egui::pos2(cell_rect.left() + CELL_PADDING_X, cell_rect.top()),
                                egui::pos2(cell_rect.right() - CELL_PADDING_X, cell_rect.bottom()),
                            );
                            let layout = match col.align {
                                ColumnAlign::Left => egui::Layout::left_to_right(egui::Align::Center),
                                ColumnAlign::Center => egui::Layout::centered_and_justified(Direction::LeftToRight),
                                ColumnAlign::Right => egui::Layout::right_to_left(egui::Align::Center),
                            };
                            let mut child = ui.new_child(
                                UiBuilder::new().max_rect(inner).layout(layout),
                            );
                            child.set_clip_rect(cell_rect);
                            (col.cell)(&mut child, item);
                            col_x += w;
                        }

                        // Row click.
                        if row_resp.clicked() {
                            if self.select_on_row_click && has_selection {
                                toggle_selection(
                                    &mut state.selected,
                                    item,
                                    self.multi_selection,
                                );
                            }
                            if let Some(rc) = row_click.as_mut() {
                                rc(item, abs_idx);
                            }
                        }

                        y += row_h;
                    }
                };

                if let Some(cap) = body_cap {
                    egui::ScrollArea::vertical()
                        .max_height(cap)
                        .auto_shrink([false, false])
                        .show(ui, |ui| draw_body(ui));
                } else {
                    draw_body(ui);
                }

                // Footer.
                if self.show_pager {
                    let (footer_rect, _) = ui.allocate_exact_size(
                        vec2(content_width, FOOTER_HEIGHT),
                        Sense::hover(),
                    );
                    let mut footer_ui = ui.new_child(
                        UiBuilder::new()
                            .max_rect(footer_rect)
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );
                    draw_pager(&mut footer_ui, state, total, self.dense);
                }

                // Outlined border.
                if self.outlined {
                    let full = Rect::from_min_size(
                        header_rect.min,
                        vec2(content_width, ui.cursor().min.y - header_rect.min.y),
                    );
                    ui.painter().rect_stroke(
                        full,
                        CornerRadius::same(radius as u8),
                        Stroke::new(
                            1.0,
                            ui.visuals().widgets.noninteractive.bg_stroke.color,
                        ),
                        StrokeKind::Inside,
                    );
                }
            }

            // Loading overlay.
            if self.loading {
                ui.ctx().request_repaint();
                let overlay = ui.max_rect();
                ui.painter().rect_filled(
                    overlay,
                    CornerRadius::same(radius as u8),
                    Color32::from_black_alpha(28),
                );
                ui.put(overlay, egui::Spinner::new());
            }
        });

        self.tw_state.hovered = outer.response.hovered();
        self.tw_state.active = outer.response.is_pointer_button_down_on();

        if let Some(label) = &self.aria_label {
            outer
                .response
                .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Table, true, label));
        }

        (outer.response, user_result)
    }
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Pixels at which a breakpoint activates.
///
/// Mirrors the Bootstrap breakpoints MudBlazor uses. Unknown / `None`
/// variants fall back to “never stacks”.
#[inline]
pub fn breakpoint_px(bp: Breakpoint) -> f32 {
    match bp {
        Breakpoint::Xs => 0.0,
        Breakpoint::Sm => 600.0,
        Breakpoint::Md => 960.0,
        Breakpoint::Lg => 1280.0,
        Breakpoint::Xl => 1920.0,
        _ => f32::INFINITY,
    }
}

/// The breakpoint that is active at `width`.
#[inline]
pub fn active_breakpoint(width: f32) -> Breakpoint {
    if width >= 1920.0 {
        Breakpoint::Xl
    } else if width >= 1280.0 {
        Breakpoint::Lg
    } else if width >= 960.0 {
        Breakpoint::Md
    } else if width >= 600.0 {
        Breakpoint::Sm
    } else {
        Breakpoint::Xs
    }
}

/// Whether the grid should stack at the active breakpoint.
///
/// Stacking occurs strictly *below* the configured breakpoint.
#[inline]
pub fn is_stacked(active: Breakpoint, configured: Breakpoint) -> bool {
    breakpoint_px(active) < breakpoint_px(configured)
}

/// Row height for the given density.
#[inline]
pub fn row_height(dense: bool) -> f32 {
    if dense {
        DENSE_ROW_HEIGHT
    } else {
        DEFAULT_ROW_HEIGHT
    }
}

/// Header height for the given density.
#[inline]
pub fn header_height(dense: bool) -> f32 {
    if dense {
        DENSE_HEADER_HEIGHT
    } else {
        DEFAULT_HEADER_HEIGHT
    }
}

/// Direction a sort should advance to after a header click.
#[inline]
pub fn next_sort_direction(current: SortDirection, allow_unsorted: bool) -> SortDirection {
    current.next(allow_unsorted)
}

/// Current direction applied to `column_id`, or [`SortDirection::None`].
#[inline]
pub fn sort_direction_for(sorts: &[SortDefinition], column_id: &str) -> SortDirection {
    sorts
        .iter()
        .find(|s| s.column_id == column_id)
        .map(|s| s.direction)
        .unwrap_or(SortDirection::None)
}

/// Applies one header click to the sort list.
///
/// Returns `true` when the list changed.
pub fn toggle_sort(
    sorts: &mut Vec<SortDefinition>,
    column_id: &str,
    mode: SortMode,
    allow_unsorted: bool,
) -> bool {
    if mode == SortMode::None {
        return false;
    }
    let existing = sorts.iter().position(|s| s.column_id == column_id);
    match (mode, existing) {
        (SortMode::Single, Some(pos)) => {
            let next = sorts[pos].direction.next(allow_unsorted);
            if next == SortDirection::None {
                sorts.clear();
            } else {
                sorts.clear();
                sorts.push(SortDefinition::new(column_id, next));
            }
            true
        }
        (SortMode::Single, None) => {
            sorts.clear();
            sorts.push(SortDefinition::new(column_id, SortDirection::Ascending));
            true
        }
        (SortMode::Multiple, Some(pos)) => {
            let next = sorts[pos].direction.next(allow_unsorted);
            if next == SortDirection::None {
                sorts.remove(pos);
            } else {
                sorts[pos].direction = next;
            }
            true
        }
        (SortMode::Multiple, None) => {
            sorts.push(SortDefinition::new(column_id, SortDirection::Ascending));
            true
        }
        (SortMode::None, _) => false,
    }
}

/// Returns row indices sorted by the active sort definitions.
///
/// `base` is the pre-filtered index list; the returned list is a stable
/// multi-key sort of it. Columns without a comparator (or not sortable) are
/// skipped.
pub fn sort_indices<T>(
    items: &[T],
    sorts: &[SortDefinition],
    columns: &[Column<'_, T>],
    base: &[usize],
) -> Vec<usize> {
    let mut indices = base.to_vec();
    if sorts.is_empty() || columns.is_empty() {
        return indices;
    }
    indices.sort_by(|&a, &b| {
        for s in sorts {
            if !s.direction.is_active() {
                continue;
            }
            let Some(col) = columns.iter().find(|c| c.id == s.column_id) else {
                continue;
            };
            if !col.is_sortable() {
                continue;
            }
            let Some(cmp) = col.comparator.as_ref() else {
                continue;
            };
            let ord = cmp(&items[a], &items[b]);
            let ord = match s.direction {
                SortDirection::Descending => ord.reverse(),
                _ => ord,
            };
            if ord != Ordering::Equal {
                return ord;
            }
        }
        Ordering::Equal
    });
    indices
}

/// Returns indices of rows that pass every active filter and the quick filter.
pub fn filter_indices<T>(
    items: &[T],
    filters: &[ColumnFilter],
    columns: &[Column<'_, T>],
    quick_text: &str,
    quick_predicate: Option<&dyn Fn(&T) -> bool>,
    case_sensitive: bool,
) -> Vec<usize> {
    let needle = quick_text.trim();
    items
        .iter()
        .enumerate()
        .filter(|(_, item)| {
            if let Some(pred) = quick_predicate {
                if !pred(item) {
                    return false;
                }
            }
            if !needle.is_empty() {
                let matched = columns.iter().any(|col| {
                    let Some(extract) = col.filter_extractor.as_ref() else {
                        return false;
                    };
                    contains(extract(item), needle, case_sensitive)
                });
                if !matched {
                    return false;
                }
            }
            for f in filters {
                let value = f.value.trim();
                if value.is_empty() {
                    continue;
                }
                let Some(col) = columns.iter().find(|c| c.id == f.column_id) else {
                    continue;
                };
                let Some(extract) = col.filter_extractor.as_ref() else {
                    continue;
                };
                if !contains(extract(item), value, case_sensitive) {
                    return false;
                }
            }
            true
        })
        .map(|(i, _)| i)
        .collect()
}

#[inline]
fn contains(haystack: String, needle: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        haystack.contains(needle)
    } else {
        haystack.to_lowercase().contains(&needle.to_lowercase())
    }
}

/// Number of pages for `total` rows.
#[inline]
pub fn page_count(total: usize, rows_per_page: usize) -> usize {
    if total == 0 {
        0
    } else if rows_per_page == 0 {
        1
    } else {
        (total + rows_per_page - 1) / rows_per_page
    }
}

/// Clamps `page` into the valid range.
#[inline]
pub fn clamp_page(page: usize, total: usize, rows_per_page: usize) -> usize {
    let pages = page_count(total, rows_per_page);
    if pages == 0 {
        0
    } else {
        page.min(pages - 1)
    }
}

/// Half-open `[start, end)` range of items on `page`.
#[inline]
pub fn page_range(total: usize, page: usize, rows_per_page: usize) -> (usize, usize) {
    if rows_per_page == 0 {
        return (0, total);
    }
    let start = page.saturating_mul(rows_per_page).min(total);
    let end = (start + rows_per_page).min(total);
    (start, end)
}

/// Resolves column widths for the visible columns.
///
/// The returned vector begins with [`SELECTION_COL_WIDTH`] when
/// `has_selection` is `true`, followed by one entry per non-hidden column.
/// Fixed-width columns are honoured; the remainder is split equally among
/// flexible columns. Hidden columns contribute nothing.
pub fn resolve_column_widths<T>(
    columns: &[Column<'_, T>],
    available: f32,
    selection_width: f32,
    has_selection: bool,
) -> Vec<f32> {
    let available = if available.is_finite() && available > 0.0 {
        available
    } else {
        FALLBACK_WIDTH
    };
    let visible: Vec<&Column<'_, T>> = columns.iter().filter(|c| !c.hidden).collect();
    let mut fixed: f32 = if has_selection { selection_width } else { 0.0 };
    let mut flex_count = 0usize;
    for col in &visible {
        match col.width {
            Some(w) => fixed += w.max(0.0),
            None => flex_count += 1,
        }
    }
    let remaining = (available - fixed).max(0.0);
    let per_flex = if flex_count > 0 {
        remaining / flex_count as f32
    } else {
        0.0
    };

    let mut widths = Vec::with_capacity(visible.len() + usize::from(has_selection));
    if has_selection {
        widths.push(selection_width);
    }
    for col in visible {
        widths.push(col.width.unwrap_or(per_flex).max(0.0));
    }
    widths
}

/// Toggles `item` in `selected`, honouring single vs multi selection.
///
/// Returns `true` when the selection changed.
pub fn toggle_selection<T: Clone + PartialEq>(
    selected: &mut Vec<T>,
    item: &T,
    multi: bool,
) -> bool {
    if let Some(pos) = selected.iter().position(|s| s == item) {
        selected.remove(pos);
        true
    } else {
        if !multi {
            selected.clear();
        }
        selected.push(item.clone());
        true
    }
}

/// Row background for the table layout.
fn row_fill(
    pos: usize,
    selected: bool,
    striped: bool,
    hovered: bool,
    accent: Color32,
    ui: &Ui,
) -> Color32 {
    if selected {
        return with_alpha(accent, 28);
    }
    if hovered {
        return ui.visuals().widgets.hovered.weak_bg_fill;
    }
    if striped && pos % 2 == 1 {
        return ui.visuals().faint_bg_color;
    }
    Color32::TRANSPARENT
}

#[inline]
fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

/// Maps a MudBlazor elevation (0–24) to an egui shadow.
pub fn elevation_shadow(elevation: u8) -> egui::epaint::Shadow {
    let e = elevation.min(24) as f32;
    egui::epaint::Shadow {
        offset: [0, (e * 0.4) as i8],
        blur: (e * 1.5) as u8,
        spread: 0,
        color: Color32::from_black_alpha(((e * 3.0) as u8).min(60)),
    }
}

/// Paints a Mud-style checkbox.
///
/// `ui` is `&mut Ui` because `Ui::put` needs mutable access. The icon string
/// is materialised to `String` so `MudIcon::new(..)` has a concrete target
/// type (`&'static str` alone is ambiguous against `Into<String>` /
/// `Into<Arc<str>>`).
fn paint_checkbox(ui: &mut Ui, rect: Rect, checked: bool, accent: Color32) {
    let radius = CornerRadius::same(3);
    if checked {
        ui.painter().rect_filled(rect, radius, accent);
        ui.put(
            rect.shrink(2.0),
            MudIcon::new(mud_egui_icons_material::filled::CHECK.to_owned())
                .size_px(rect.width() - 4.0)
                .custom_color(Color32::WHITE),
        );
    } else {
        ui.painter()
            .rect_stroke(rect, radius, Stroke::new(1.5, accent), StrokeKind::Inside);
    }
}

/// Resolves a row-class string into a fill colour via the grid's flavour.
fn resolve_row_class_fill(
    class: &str,
    lookup: &dyn Fn(&str) -> Option<Color32>,
    bs: &mut BsStyle,
    tw: &mut TwStyle,
    flavor: StyleFlavor,
    tw_state: TwState,
) -> Option<Color32> {
    if class.trim().is_empty() {
        return None;
    }
    let base = ComponentBase::new().classes(class.split_whitespace());
    let frame = match flavor {
        StyleFlavor::Bootstrap => bs.build_frame(&base, lookup),
        StyleFlavor::Tailwind => tw.build_frame(&base, tw_state, REM_SIZE_PT, lookup),
    };
    if frame.fill != Color32::TRANSPARENT {
        Some(frame.fill)
    } else {
        None
    }
}

/// Draws the header row and wires sort clicks / select-all.
#[allow(clippy::too_many_arguments)]
fn draw_header<T: Clone + PartialEq>(
    ui: &mut Ui,
    rect: Rect,
    columns: &[Column<'_, T>],
    widths: &[f32],
    sorts: &mut Vec<SortDefinition>,
    sort_mode: SortMode,
    allow_unsorted: bool,
    filterable: bool,
    has_selection: bool,
    bordered: bool,
    dense: bool,
    accent: Color32,
    selected: &mut Vec<T>,
    items: &[T],
    filtered: &[usize],
) {
    ui.painter()
        .rect_filled(rect, 0.0, ui.visuals().widgets.noninteractive.bg_fill);
    let mut x = rect.left();

    if has_selection {
        let all_selected = !filtered.is_empty()
            && filtered
            .iter()
            .all(|&i| selected.iter().any(|s| s == &items[i]));
        let cb_rect = Rect::from_center_size(
            egui::pos2(x + SELECTION_COL_WIDTH * 0.5, rect.center().y),
            vec2(CHECKBOX_SIZE, CHECKBOX_SIZE),
        );
        paint_checkbox(ui, cb_rect, all_selected, accent);
        let select_all_id = ui.scope_id().with("dg_select_all");
        let resp = ui.interact(
            cb_rect,
            select_all_id,
            Sense::click(),
        );
        if resp.clicked() {
            if all_selected {
                selected.clear();
            } else {
                for &i in filtered {
                    let item = &items[i];
                    if !selected.iter().any(|s| s == item) {
                        selected.push(item.clone());
                    }
                }
            }
        }
        x += SELECTION_COL_WIDTH;
    }

    for (vi, col) in columns.iter().filter(|c| !c.hidden).enumerate() {
        let w = widths[vi + usize::from(has_selection)];
        let cell = Rect::from_min_size(egui::pos2(x, rect.top()), vec2(w, rect.height()));
        if bordered && vi > 0 {
            ui.painter().line_segment(
                [
                    egui::pos2(x, rect.top()),
                    egui::pos2(x, rect.bottom()),
                ],
                Stroke::new(
                    1.0,
                    ui.visuals().widgets.noninteractive.bg_stroke.color,
                ),
            );
        }

        let sort_dir = sort_direction_for(sorts, &col.id);
        let sortable = sort_mode != SortMode::None && col.is_sortable();
        let head_id = ui.scope_id().with(("dg_head", col.id.as_str()));
        let resp = ui.interact(
            cell,
            head_id,
            if sortable { Sense::click() } else { Sense::hover() },
        );
        if sortable {
            if resp.hovered() {
                ui.painter()
                    .rect_filled(cell, 0.0, ui.visuals().widgets.hovered.weak_bg_fill);
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resp.clicked() {
                toggle_sort(sorts, &col.id, sort_mode, allow_unsorted);
            }
        }

        let inner = Rect::from_min_max(
            egui::pos2(cell.left() + CELL_PADDING_X, cell.top()),
            egui::pos2(cell.right() - CELL_PADDING_X, cell.bottom()),
        );
        let layout = match col.align {
            ColumnAlign::Left => egui::Layout::left_to_right(egui::Align::Center),
            ColumnAlign::Center => egui::Layout::centered_and_justified(Direction::LeftToRight),
            ColumnAlign::Right => egui::Layout::right_to_left(egui::Align::Center),
        };
        let mut child = ui.new_child(UiBuilder::new().max_rect(inner).layout(layout));
        child.add(MudText::new(col.title.clone()).typo(Typo::Subtitle2).bold(true));

        // Sort indicator.
        if sort_dir.is_active() {
            let icon = match sort_dir {
                SortDirection::Ascending => mud_egui_icons_material::filled::KEYBOARD_ARROW_UP,
                SortDirection::Descending => mud_egui_icons_material::filled::KEYBOARD_ARROW_DOWN,
                SortDirection::None => "",
            };
            let icon_rect = Rect::from_center_size(
                egui::pos2(cell.right() - CELL_PADDING_X - HEADER_ICON_SIZE * 0.5, cell.center().y),
                vec2(HEADER_ICON_SIZE, HEADER_ICON_SIZE),
            );
            ui.put(
                icon_rect,
                MudIcon::new(icon).size_px(HEADER_ICON_SIZE).custom_color(accent),
            );
        } else if filterable && col.is_filterable(filterable) {
            let icon_rect = Rect::from_center_size(
                egui::pos2(cell.right() - CELL_PADDING_X - HEADER_ICON_SIZE * 0.5, cell.center().y),
                vec2(HEADER_ICON_SIZE, HEADER_ICON_SIZE),
            );
            ui.put(
                icon_rect,
                MudIcon::new(mud_egui_icons_material::filled::FILTER_ALT)
                    .size_px(HEADER_ICON_SIZE)
                    .custom_color(ui.visuals().weak_text_color()),
            );
        }
        let _ = dense;
        x += w;
    }
}

/// Draws the inline filter row.
fn draw_filter_row<T>(
    ui: &mut Ui,
    rect: Rect,
    columns: &[Column<'_, T>],
    widths: &[f32],
    filters: &mut Vec<ColumnFilter>,
    has_selection: bool,
    bordered: bool,
    accent: Color32,
) {
    ui.painter()
        .rect_filled(rect, 0.0, ui.visuals().faint_bg_color);
    let mut x = rect.left();
    if has_selection {
        x += SELECTION_COL_WIDTH;
    }
    for (vi, col) in columns.iter().filter(|c| !c.hidden).enumerate() {
        let w = widths[vi + usize::from(has_selection)];
        let cell = Rect::from_min_size(egui::pos2(x, rect.top()), vec2(w, rect.height()));
        if bordered && vi > 0 {
            ui.painter().line_segment(
                [
                    egui::pos2(x, rect.top()),
                    egui::pos2(x, rect.bottom()),
                ],
                Stroke::new(
                    1.0,
                    ui.visuals().widgets.noninteractive.bg_stroke.color,
                ),
            );
        }
        if col.is_filterable(true) {
            let value = filter_value_mut(filters, &col.id);
            let input_rect = cell.shrink2(vec2(CELL_PADDING_X, 10.0));
            ui.put(
                input_rect,
                egui::TextEdit::singleline(value)
                    .hint_text("Filter")
                    .desired_width(input_rect.width()),
            );
        }
        let _ = accent;
        x += w;
    }
}

/// Ensures a `ColumnFilter` exists for `id` and returns its value.
fn filter_value_mut<'s>(filters: &'s mut Vec<ColumnFilter>, id: &str) -> &'s mut String {
    if let Some(pos) = filters.iter().position(|f| f.column_id == id) {
        return &mut filters[pos].value;
    }
    filters.push(ColumnFilter::new(id, ""));
    &mut filters.last_mut().expect("just pushed").value
}

/// Draws the pager footer.
fn draw_pager<T>(ui: &mut Ui, state: &mut DataGridState<T>, total: usize, dense: bool) {
    let rows_per_page = state.rows_per_page;
    let pages = page_count(total, rows_per_page);
    let (start, end) = page_range(total, state.current_page, rows_per_page);

    ui.horizontal(|ui| {
        let info = if total == 0 {
            "0 of 0".to_owned()
        } else {
            format!("{}-{} of {}", start + 1, end, total)
        };
        ui.add(
            MudText::new(info)
                .typo(if dense { Typo::Caption } else { Typo::Body2 })
                .color(Color::TextSecondary),
        );

        if pages <= 1 {
            return;
        }

        ui.add_space(12.0);
        let prev_disabled = state.current_page == 0;
        if MudButton::new("Previous")
            .variant(Variant::Text)
            .size(Size::Small)
            .disabled(prev_disabled)
            .show(ui)
            .clicked()
        {
            state.current_page = state.current_page.saturating_sub(1);
        }
        ui.add(
            MudText::new(format!("{}/{}", state.current_page + 1, pages))
                .typo(Typo::Body2)
                .color(Color::TextSecondary),
        );
        let next_disabled = state.current_page + 1 >= pages;
        if MudButton::new("Next")
            .variant(Variant::Text)
            .size(Size::Small)
            .disabled(next_disabled)
            .show(ui)
            .clicked()
        {
            state.current_page = (state.current_page + 1).min(pages - 1);
        }
    });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Person {
        name: String,
        age: u32,
        city: String,
    }

    fn p(name: &str, age: u32, city: &str) -> Person {
        Person {
            name: name.into(),
            age,
            city: city.into(),
        }
    }

    fn people() -> Vec<Person> {
        vec![
            p("Charlie", 30, "Berlin"),
            p("Alice", 25, "Amsterdam"),
            p("Bob", 35, "Berlin"),
            p("Dora", 28, "Amsterdam"),
        ]
    }

    fn columns() -> Vec<Column<'static, Person>> {
        vec![
            Column::new("name", "Name", |_: &mut egui::Ui, _: &Person| {})
                .sort_by(|a, b| a.name.cmp(&b.name))
                .filter_text(|r| r.name.clone()),
            Column::new("age", "Age", |_: &mut egui::Ui, _: &Person| {})
                .sort_by(|a, b| a.age.cmp(&b.age))
                .filter_text(|r| r.age.to_string()),
            Column::new("city", "City", |_: &mut egui::Ui, _: &Person| {})
                .sort_by(|a, b| a.city.cmp(&b.city))
                .filter_text(|r| r.city.clone()),
        ]
    }

    fn all_indices(n: usize) -> Vec<usize> {
        (0..n).collect()
    }

    // --- sort direction ----------------------------------------------------

    #[test]
    fn direction_cycles_without_unsorted() {
        assert_eq!(
            SortDirection::None.next(false),
            SortDirection::Ascending
        );
        assert_eq!(
            SortDirection::Ascending.next(false),
            SortDirection::Descending
        );
        assert_eq!(
            SortDirection::Descending.next(false),
            SortDirection::Ascending
        );
    }

    #[test]
    fn direction_cycles_with_unsorted() {
        assert_eq!(
            SortDirection::Descending.next(true),
            SortDirection::None
        );
    }

    #[test]
    fn reversed_is_involutive() {
        for d in [
            SortDirection::None,
            SortDirection::Ascending,
            SortDirection::Descending,
        ] {
            assert_eq!(d.reversed().reversed(), d);
        }
    }

    // --- toggle_sort -------------------------------------------------------

    #[test]
    fn single_mode_replaces_previous_sort() {
        let mut sorts = vec![SortDefinition::new("name", SortDirection::Ascending)];
        toggle_sort(&mut sorts, "age", SortMode::Single, false);
        assert_eq!(sorts.len(), 1);
        assert_eq!(sorts[0].column_id, "age");
        assert_eq!(sorts[0].direction, SortDirection::Ascending);
    }

    #[test]
    fn single_mode_clears_on_third_toggle_with_unsorted() {
        let mut sorts = Vec::new();
        toggle_sort(&mut sorts, "name", SortMode::Single, true); // Asc
        toggle_sort(&mut sorts, "name", SortMode::Single, true); // Desc
        toggle_sort(&mut sorts, "name", SortMode::Single, true); // None
        assert!(sorts.is_empty());
    }

    #[test]
    fn multiple_mode_appends_in_priority_order() {
        let mut sorts = Vec::new();
        toggle_sort(&mut sorts, "city", SortMode::Multiple, false);
        toggle_sort(&mut sorts, "age", SortMode::Multiple, false);
        assert_eq!(sorts.len(), 2);
        assert_eq!(sorts[0].column_id, "city");
        assert_eq!(sorts[1].column_id, "age");
    }

    #[test]
    fn multiple_mode_removes_when_unsorted_is_allowed() {
        let mut sorts = vec![SortDefinition::new("name", SortDirection::Descending)];
        toggle_sort(&mut sorts, "name", SortMode::Multiple, true);
        assert!(sorts.is_empty());
    }

    #[test]
    fn sort_mode_none_never_changes() {
        let mut sorts = Vec::new();
        assert!(!toggle_sort(&mut sorts, "name", SortMode::None, true));
        assert!(sorts.is_empty());
    }

    // --- sort_indices ------------------------------------------------------

    #[test]
    fn sort_indices_ascending_by_name() {
        let data = people();
        let cols = columns();
        let sorts = vec![SortDefinition::new("name", SortDirection::Ascending)];
        let order = sort_indices(&data, &sorts, &cols, &all_indices(data.len()));
        let names: Vec<&str> = order.iter().map(|&i| data[i].name.as_str()).collect();
        assert_eq!(names, vec!["Alice", "Bob", "Charlie", "Dora"]);
    }

    #[test]
    fn sort_indices_descending_by_age() {
        let data = people();
        let cols = columns();
        let sorts = vec![SortDefinition::new("age", SortDirection::Descending)];
        let order = sort_indices(&data, &sorts, &cols, &all_indices(data.len()));
        let ages: Vec<u32> = order.iter().map(|&i| data[i].age).collect();
        assert_eq!(ages, vec![35, 30, 28, 25]);
    }

    #[test]
    fn sort_indices_multi_key_city_then_age() {
        let data = people();
        let cols = columns();
        let sorts = vec![
            SortDefinition::new("city", SortDirection::Ascending),
            SortDefinition::new("age", SortDirection::Ascending),
        ];
        let order = sort_indices(&data, &sorts, &cols, &all_indices(data.len()));
        let pairs: Vec<(&str, u32)> = order
            .iter()
            .map(|&i| (data[i].city.as_str(), data[i].age))
            .collect();
        assert_eq!(
            pairs,
            vec![
                ("Amsterdam", 25),
                ("Amsterdam", 28),
                ("Berlin", 30),
                ("Berlin", 35),
            ]
        );
    }

    #[test]
    fn sort_indices_skips_columns_without_comparator() {
        let data = people();
        let cols = vec![Column::<Person>::new("name", "Name", |_, _| {})];
        let sorts = vec![SortDefinition::new("name", SortDirection::Ascending)];
        let order = sort_indices(&data, &sorts, &cols, &all_indices(data.len()));
        assert_eq!(order, vec![0, 1, 2, 3]);
    }

    #[test]
    fn sort_indices_empty_sorts_preserves_order() {
        let data = people();
        let order = sort_indices(&data, &[], &columns(), &all_indices(data.len()));
        assert_eq!(order, vec![0, 1, 2, 3]);
    }

    // --- filter_indices ----------------------------------------------------

    #[test]
    fn filter_indices_case_insensitive_by_default() {
        let data = people();
        let cols = columns();
        let filters = vec![ColumnFilter::new("name", "ali")];
        let out = filter_indices(&data, &filters, &cols, "", None, false);
        assert_eq!(out, vec![1]);
    }

    #[test]
    fn filter_indices_case_sensitive() {
        let data = people();
        let cols = columns();
        let filters = vec![ColumnFilter::new("name", "ali")];
        let out = filter_indices(&data, &filters, &cols, "", None, true);
        assert!(out.is_empty());
    }

    #[test]
    fn filter_indices_empty_value_is_inactive() {
        let data = people();
        let cols = columns();
        let filters = vec![ColumnFilter::new("name", "  ")];
        let out = filter_indices(&data, &filters, &cols, "", None, false);
        assert_eq!(out.len(), data.len());
    }

    #[test]
    fn filter_indices_and_semantics() {
        let data = people();
        let cols = columns();
        let filters = vec![
            ColumnFilter::new("city", "Berlin"),
            ColumnFilter::new("name", "bob"),
        ];
        let out = filter_indices(&data, &filters, &cols, "", None, false);
        assert_eq!(out, vec![2]);
    }

    #[test]
    fn filter_indices_quick_text_matches_any_filterable_column() {
        let data = people();
        let cols = columns();
        let out = filter_indices(&data, &[], &cols, "amsterdam", None, false);
        assert_eq!(out, vec![1, 3]);
    }

    #[test]
    fn filter_indices_quick_predicate() {
        let data = people();
        let cols = columns();
        let pred = |r: &Person| r.age > 30;
        let out = filter_indices(&data, &[], &cols, "", Some(&pred), false);
        assert_eq!(out, vec![2]);
    }

    // --- pagination --------------------------------------------------------

    #[test]
    fn page_count_handles_zero_rows_and_zero_size() {
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

    // --- widths ------------------------------------------------------------

    #[test]
    fn widths_split_remaining_space_equally() {
        let cols = vec![
            Column::<Person>::new("a", "A", |_, _| {}).width(100.0),
            Column::<Person>::new("b", "B", |_, _| {}),
            Column::<Person>::new("c", "C", |_, _| {}),
        ];
        let w = resolve_column_widths(&cols, 600.0, SELECTION_COL_WIDTH, false);
        assert_eq!(w, vec![100.0, 250.0, 250.0]);
    }

    #[test]
    fn widths_with_selection_column() {
        let cols = vec![
            Column::<Person>::new("a", "A", |_, _| {}).width(100.0),
            Column::<Person>::new("b", "B", |_, _| {}),
            Column::<Person>::new("c", "C", |_, _| {}),
        ];
        let w = resolve_column_widths(&cols, 648.0, SELECTION_COL_WIDTH, true);
        assert_eq!(w, vec![SELECTION_COL_WIDTH, 100.0, 250.0, 250.0]);
        assert_eq!(w.iter().sum::<f32>(), 648.0);
    }

    #[test]
    fn widths_skip_hidden_columns() {
        let cols = vec![
            Column::<Person>::new("a", "A", |_, _| {}).width(100.0),
            Column::<Person>::new("b", "B", |_, _| {}).hidden(true),
            Column::<Person>::new("c", "C", |_, _| {}),
        ];
        let w = resolve_column_widths(&cols, 500.0, SELECTION_COL_WIDTH, false);
        assert_eq!(w, vec![100.0, 400.0]);
    }

    #[test]
    fn widths_repair_non_finite_available() {
        let cols = vec![Column::<Person>::new("a", "A", |_, _| {})];
        let w = resolve_column_widths(&cols, f32::NAN, SELECTION_COL_WIDTH, false);
        assert_eq!(w, vec![FALLBACK_WIDTH]);
    }

    // --- selection ---------------------------------------------------------

    #[test]
    fn selection_multi_toggles_and_preserves_order() {
        let mut sel: Vec<Person> = Vec::new();
        let a = p("A", 1, "X");
        let b = p("B", 2, "Y");
        toggle_selection(&mut sel, &a, true);
        toggle_selection(&mut sel, &b, true);
        assert_eq!(sel, vec![a.clone(), b.clone()]);
        toggle_selection(&mut sel, &a, true);
        assert_eq!(sel, vec![b]);
    }

    #[test]
    fn selection_single_replaces() {
        let mut sel: Vec<Person> = Vec::new();
        let a = p("A", 1, "X");
        let b = p("B", 2, "Y");
        toggle_selection(&mut sel, &a, false);
        toggle_selection(&mut sel, &b, false);
        assert_eq!(sel, vec![b]);
    }

    // --- breakpoints -------------------------------------------------------

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
        assert_eq!(active_breakpoint(1280.0), Breakpoint::Lg);
        assert_eq!(active_breakpoint(1920.0), Breakpoint::Xl);
    }

    #[test]
    fn stacking_is_strictly_below_the_configured_breakpoint() {
        assert!(!is_stacked(Breakpoint::Xs, Breakpoint::Xs));
        assert!(is_stacked(Breakpoint::Xs, Breakpoint::Sm));
        assert!(!is_stacked(Breakpoint::Sm, Breakpoint::Sm));
        assert!(is_stacked(Breakpoint::Sm, Breakpoint::Md));
        assert!(!is_stacked(Breakpoint::Lg, Breakpoint::Md));
    }

    // --- heights -----------------------------------------------------------

    #[test]
    fn heights_match_density() {
        assert_eq!(row_height(false), DEFAULT_ROW_HEIGHT);
        assert_eq!(row_height(true), DENSE_ROW_HEIGHT);
        assert_eq!(header_height(false), DEFAULT_HEADER_HEIGHT);
        assert_eq!(header_height(true), DENSE_HEADER_HEIGHT);
    }

    // --- defaults ----------------------------------------------------------

    #[test]
    fn grid_defaults_match_mudblazor() {
        let g: MudDataGrid<'static, Person> = MudDataGrid::new();
        assert_eq!(g.sort_mode, SortMode::Multiple);
        assert!(!g.allow_unsorted);
        assert!(!g.filterable);
        assert_eq!(g.filter_mode, FilterMode::ColumnFilterRow);
        assert!(!g.filter_case_sensitive);
        assert!(!g.multi_selection);
        assert!(g.select_on_row_click);
        assert!(!g.bordered && !g.outlined && !g.hover && !g.striped && !g.dense && !g.square);
        assert_eq!(g.elevation, 1);
        assert_eq!(breakpoint_px(g.breakpoint), 0.0);
        assert!(g.show_pager);
        assert!(!g.loading);
        assert!(g.rows_per_page.is_none());
        assert!(g.current_page.is_none());
    }

    #[test]
    fn column_defaults() {
        let c = Column::<Person>::new("id", "Id", |_, _| {});
        assert!(c.sortable.is_none());
        assert!(c.filterable.is_none());
        assert!(!c.hidden);
        assert_eq!(c.align, ColumnAlign::Left);
        assert!(c.width.is_none());
        assert!(!c.is_sortable());
    }

    #[test]
    fn text_column_installs_filter_extractor() {
        let c = Column::<Person>::text("name", "Name", |p| p.name.clone());
        assert!(c.filter_extractor.is_some());
        assert!(!c.is_sortable()); // no comparator yet
    }

    #[test]
    fn column_alignment_maps_to_egui() {
        assert_eq!(ColumnAlign::Left.to_egui(), egui::Align::Min);
        assert_eq!(ColumnAlign::Center.to_egui(), egui::Align::Center);
        assert_eq!(ColumnAlign::Right.to_egui(), egui::Align::Max);
    }

    // --- tag ---------------------------------------------------------------

    #[test]
    fn tag_round_trips_by_type() {
        let g: MudDataGrid<'static, Person> = MudDataGrid::new().tag("section");
        assert_eq!(g.tag_ref::<&str>(), Some(&"section"));
        assert!(g.tag_ref::<u32>().is_none());
    }

    // --- state -------------------------------------------------------------

    #[test]
    fn state_defaults() {
        let s: DataGridState<Person> = DataGridState::new();
        assert_eq!(s.rows_per_page, DEFAULT_ROWS_PER_PAGE);
        assert_eq!(s.current_page, 0);
        assert!(s.sorts.is_empty() && s.filters.is_empty() && s.selected.is_empty());
        assert!(s.quick_filter_text.is_empty());
    }

    #[test]
    fn state_with_rows_per_page() {
        let s: DataGridState<Person> = DataGridState::new().with_rows_per_page(25);
        assert_eq!(s.rows_per_page, 25);
    }

    // --- filter row helper -------------------------------------------------

    #[test]
    fn filter_value_mut_creates_entry_once() {
        let mut filters = Vec::new();
        filter_value_mut(&mut filters, "name").push('a');
        filter_value_mut(&mut filters, "name").push('b');
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].value, "ab");
    }
}