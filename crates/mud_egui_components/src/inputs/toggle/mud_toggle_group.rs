//! A container that manages multiple [`MudToggleItem`]s for single, multi, or
//! toggle selection.
//!
//! Port of `MudToggleGroup.cs` / `MudToggleItem.cs` (MudBlazor 8.13.0). The
//! public parameter surface mirrors the Blazor component:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `CheckMark` | [`MudToggleGroup::check_mark`] |
//! | `CheckMarkClass` | [`MudToggleGroup::check_mark_class`] |
//! | `Color` | [`MudToggleGroup::color`] |
//! | `Delimiters` | [`MudToggleGroup::delimiters`] |
//! | `Disabled` | [`MudToggleGroup::disabled`] |
//! | `FixedContent` | [`MudToggleGroup::fixed_content`] |
//! | `Outlined` | [`MudToggleGroup::outlined`] |
//! | `RightToLeft` | [`MudToggleGroup::right_to_left`] |
//! | `Ripple` | [`MudToggleGroup::ripple`] |
//! | `SelectedClass` | [`MudToggleGroup::selected_class`] |
//! | `SelectionMode` | [`MudToggleGroup::selection_mode`] |
//! | `Size` | [`MudToggleGroup::size`] |
//! | `Vertical` | [`MudToggleGroup::vertical`] |
//! | `Value` / `ValueChanged` | caller-owned `&mut Vec<T>` in [`MudToggleGroup::show`] |
//! | `Values` / `ValuesChanged` | caller-owned `&mut Vec<T>` in [`MudToggleGroup::show`] |
//!
//! # Selection modes
//!
//! | [`SelectionMode`] | Clicking an unselected item | Clicking a selected item |
//! |-------------------|-----------------------------|--------------------------|
//! | `Single`          | replaces the selection      | no change                |
//! | `Toggle`          | replaces the selection      | clears the selection     |
//! | `Multi`           | adds it to the selection    | removes it               |
//!
//! # Borders
//!
//! An outlined group looks like one segmented control: one rounded border
//! around the whole group, 1 px dividers between cells, square inner corners.
//! Individual [`MudButton`] outlines cannot be joined like that, so outlined
//! groups render *borderless* buttons (`Text` when unselected, `Filled` when
//! selected) and paint the shared border and dividers themselves. The selected
//! cell receives a square-cornered accent under-fill so the button’s rounded
//! fill does not leave notches at the seams.
//!
//! # Sizing
//!
//! Horizontal groups are content-sized: each button takes its natural width and
//! height. Two situations switch to stretched layout: a [`vertical`](MudToggleGroup::vertical)
//! group, and a group containing at least one item with custom
//! [`content`](MudToggleItem::content).
//!
//! # Custom content
//!
//! Custom cells size themselves to their content. Because all cells of a row
//! must share one height (and the shared border and dividers must line up),
//! each custom closure is run **twice per frame**: once into an invisible probe
//! `Ui` to measure it, and once for real. Closures are therefore `FnMut` and
//! should be free of side effects.
//!
//! # Per-frame model
//!
//! Items are collected into a temporary `Vec` on every frame by the closure
//! passed to [`MudToggleGroup::show`], rendered once, and dropped. The group
//! owns no state between frames: the selection lives entirely in the caller’s
//! `Vec<T>`. Because items are single-use temporaries, rendering **moves** their
//! text and icon strings into the underlying [`MudButton`] instead of cloning
//! them.
//!
//! # Performance
//!
//! - String fields are moved out with `mem::take`; no `Clone` on the hot path.
//! - Theme palette is fetched at most once per frame and only when a colour is
//!   actually needed (border, divider, selection fill).
//! - Separator count and cell width are computed with integer arithmetic and
//!   saturating operations; no allocations.
//! - Custom-content heights are measured once per item and stored in a
//!   reusable `Vec`; plain button groups allocate nothing for heights.
//! - Ripple state lives in `egui::Memory` keyed by item index, so the caller
//!   does not need to provide per-item state.

use egui::{
    vec2, Align, Color32, CornerRadius, Layout, Rect, Response, Sense, Shape, Stroke,
    StrokeKind, Ui, UiBuilder, Vec2,
};

use crate::buttons::button::mud_button::MudButton;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, SelectionMode, Size, Variant};
use mud_egui_theming::theme::MudTheme;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Thickness, in logical pixels, of the delimiter line drawn between items.
const SEPARATOR_THICKNESS: f32 = 1.0;

/// Inner padding, in logical pixels, between the border of a custom-content
/// cell and the rectangle handed to the item’s content closure.
const CUSTOM_CONTENT_PADDING: f32 = 12.0;

/// Alpha of the white overlay painted over a hovered custom-content cell.
const HOVER_OVERLAY_ALPHA: u8 = 10;

/// Width of the shared outline stroke drawn around outlined groups.
const OUTLINE_STROKE_WIDTH: f32 = 1.0;

/// Outline / divider colour used when the theme palette cannot resolve the
/// group’s [`Color`].
const FALLBACK_OUTLINE_COLOR: Color32 = Color32::GRAY;

/// Default corner radius, in logical pixels, of a freshly created group.
const DEFAULT_CORNER_RADIUS: f32 = 4.0;

/// Height of the invisible probe `Ui` used to measure custom content.
///
/// It must be finite: an infinite probe height makes content that stretches to
/// the available height measure as infinite, which poisons parent scroll areas
/// with NaN offsets.
const MAX_CUSTOM_CONTENT_HEIGHT: f32 = 2048.0;

/// Duration of the ripple animation, in seconds.
const RIPPLE_DURATION: f32 = 0.35;

/// Maximum radius of the ripple circle as a multiple of the cell’s shortest
/// side.
const RIPPLE_MAX_RADIUS_FACTOR: f32 = 1.2;

// ---------------------------------------------------------------------------
// MudToggleItem
// ---------------------------------------------------------------------------

/// A single toggle button inside a [`MudToggleGroup`].
///
/// Items are cheap, single-use descriptions of a button. They are created
/// every frame and consumed by rendering: `text` and icon strings are moved
/// into the underlying [`MudButton`], so an item should not be inspected after
/// it has been rendered.
///
/// Mirrors `MudToggleItem<T>` in MudBlazor. The Blazor `ChildContent`
/// (`RenderFragment<bool>`) is represented by the Rust [`content`](Self::content)
/// closure.
#[must_use]
pub struct MudToggleItem<T> {
    /// The value this item represents. It is what gets pushed into / removed
    /// from the caller’s selection `Vec<T>` and what selection membership is
    /// tested against.
    pub value: T,
    /// The caption displayed inside the button. Mirrors MudBlazor `Text`.
    pub text: String,
    /// The glyph shown while this item is selected and the group’s
    /// [`check_mark`](MudToggleGroup::check_mark) option is on. Falls back to
    /// a Material check icon. Mirrors MudBlazor `SelectedIcon`.
    pub selected_icon: Option<String>,
    /// The glyph shown while this item is *not* selected and the group’s
    /// [`check_mark`](MudToggleGroup::check_mark) option is on. With
    /// [`fixed_content`](MudToggleGroup::fixed_content) and no explicit icon,
    /// an invisible placeholder keeps the layout from shifting on selection.
    /// Mirrors MudBlazor `UnselectedIcon`.
    pub unselected_icon: Option<String>,
    /// Disables only this item. The group’s [`disabled`](MudToggleGroup::disabled)
    /// flag disables every item regardless. Mirrors MudBlazor `Disabled`.
    pub disabled: bool,
    /// Custom content renderer. When present it replaces the standard button
    /// entirely and receives the item’s current `is_selected` flag. It is
    /// `FnMut` because it runs twice per frame (measure, then draw).
    /// Mirrors MudBlazor `ChildContent`.
    pub content: Option<Box<dyn FnMut(&mut Ui, bool) + 'static>>,
}

impl<T> MudToggleItem<T> {
    /// Creates an item for `value` that displays `text` with no icons, no
    /// custom content, and not disabled.
    pub fn new(value: T, text: impl Into<String>) -> Self {
        Self {
            value,
            text: text.into(),
            selected_icon: None,
            unselected_icon: None,
            disabled: false,
            content: None,
        }
    }

    /// Sets the checkmark glyph shown when this item is selected.
    #[inline]
    pub fn selected_icon(mut self, icon: impl Into<String>) -> Self {
        self.selected_icon = Some(icon.into());
        self
    }

    /// Sets the glyph shown when unselected (used with `check_mark` and
    /// `fixed_content`).
    #[inline]
    pub fn unselected_icon(mut self, icon: impl Into<String>) -> Self {
        self.unselected_icon = Some(icon.into());
        self
    }

    /// Disables this individual item. A disabled item never reports a click
    /// and its custom content is disabled.
    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// Full custom content. The closure receives the `is_selected` flag and
    /// replaces the standard button for this item. The cell grows to fit it.
    /// The whole cell is clickable, including any widgets inside it.
    #[inline]
    pub fn content(mut self, f: impl FnMut(&mut Ui, bool) + 'static) -> Self {
        self.content = Some(Box::new(f));
        self
    }
}

// ---------------------------------------------------------------------------
// MudToggleGroup
// ---------------------------------------------------------------------------

/// A container that manages a list of toggle items.
///
/// Configure it with the builder methods, then call [`MudToggleGroup::show`].
///
/// Mirrors `MudToggleGroup<T>` in MudBlazor 8.13.0. The Blazor two-way bindings
/// `Value` / `ValueChanged` and `Values` / `ValuesChanged` are unified into the
/// `selected: &mut Vec<T>` argument of [`show`](Self::show), which is the
/// idiomatic egui equivalent of a two-way binding.
#[must_use = "MudToggleGroup does nothing until you call `.show(ui, ...)`"]
pub struct MudToggleGroup {
    /// Shared component state (utility classes). Stored for parity with other
    /// components; not read while rendering.
    pub base: ComponentBase,
    /// How clicks change the selection. Mirrors MudBlazor `SelectionMode`.
    pub selection_mode: SelectionMode,
    /// Show a leading check glyph (selected / unselected) on every item.
    /// Mirrors MudBlazor `CheckMark`.
    pub check_mark: bool,
    /// CSS classes applied to checkmark icons. Mirrors MudBlazor
    /// `CheckMarkClass`. Accepted for API parity; egui styling is applied via
    /// `base.classes` and the active `egui::Style`.
    pub check_mark_class: Option<String>,
    /// CSS classes applied to selected items only. Mirrors MudBlazor
    /// `SelectedClass`. Accepted for API parity; egui styling is applied via
    /// `base.classes` and the active `egui::Style`.
    pub selected_class: Option<String>,
    /// Reserve the check glyph’s space on unselected items so toggling does
    /// not change the item’s width. Mirrors MudBlazor `FixedContent`.
    pub fixed_content: bool,
    /// Disables all items; clicks are ignored and custom content is disabled.
    /// Mirrors MudBlazor `Disabled`.
    pub disabled: bool,
    /// Stack items vertically instead of laying them out in a row. Vertical
    /// groups always span the available width. Mirrors MudBlazor `Vertical`.
    pub vertical: bool,
    /// Reverse the visual order of items. Normally set by the surrounding
    /// layout direction, not directly. Mirrors MudBlazor `RightToLeft`
    /// (cascading parameter).
    pub right_to_left: bool,
    /// Draw thin delimiter lines between items. Only drawn when `outlined`
    /// is off, because outlined groups already draw their own dividers.
    /// Mirrors MudBlazor `Delimiters` (default `true`).
    pub delimiters: bool,
    /// Draw one shared border and dividers around the group (the segmented
    /// control look) instead of flat text buttons. Mirrors MudBlazor
    /// `Outlined` (default `true`).
    pub outlined: bool,
    /// Theme colour used for selection fill, borders and buttons. Mirrors
    /// MudBlazor `Color` (default `Primary`).
    pub color: Color,
    /// Size preset controlling the minimum height of custom cells. Mirrors
    /// MudBlazor `Size` (default `Medium`).
    pub size: Size,
    /// Enables the ripple effect on click. Mirrors MudBlazor `Ripple`
    /// (default `true`). In egui the ripple is a short expanding circle drawn
    /// by the group; it does not allocate per-item state.
    pub ripple: bool,
}

impl Default for MudToggleGroup {
    /// Single selection, horizontal, outlined, medium, primary, ripple on.
    fn default() -> Self {
        Self {
            base: ComponentBase::new(),
            selection_mode: SelectionMode::Single,
            check_mark: false,
            check_mark_class: None,
            selected_class: None,
            fixed_content: false,
            disabled: false,
            vertical: false,
            right_to_left: false,
            delimiters: true,
            outlined: true,
            color: Color::Primary,
            size: Size::Medium,
            ripple: true,
        }
    }
}

impl MudToggleGroup {
    /// Creates a group with default settings (see [`Default`]).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the selection mode.
    #[inline] pub fn selection_mode(mut self, m: SelectionMode) -> Self { self.selection_mode = m; self }
    /// Enables or disables the leading check glyph.
    #[inline] pub fn check_mark(mut self, b: bool) -> Self { self.check_mark = b; self }
    /// Sets the CSS classes applied to checkmark icons.
    #[inline] pub fn check_mark_class(mut self, class: impl Into<String>) -> Self { self.check_mark_class = Some(class.into()); self }
    /// Sets the CSS classes applied to selected items.
    #[inline] pub fn selected_class(mut self, class: impl Into<String>) -> Self { self.selected_class = Some(class.into()); self }
    /// Enables or disables reserving space for the unselected check glyph.
    #[inline] pub fn fixed_content(mut self, b: bool) -> Self { self.fixed_content = b; self }
    /// Enables or disables the whole group.
    #[inline] pub fn disabled(mut self, b: bool) -> Self { self.disabled = b; self }
    /// Chooses vertical (`true`) or horizontal (`false`) layout.
    #[inline] pub fn vertical(mut self, b: bool) -> Self { self.vertical = b; self }
    /// Reverses the item order in the horizontal layout.
    #[inline] pub fn right_to_left(mut self, b: bool) -> Self { self.right_to_left = b; self }
    /// Enables or disables delimiter lines (effective only when not outlined).
    #[inline] pub fn delimiters(mut self, b: bool) -> Self { self.delimiters = b; self }
    /// Chooses the outlined (`true`) or flat (`false`) look.
    #[inline] pub fn outlined(mut self, b: bool) -> Self { self.outlined = b; self }
    /// Sets the theme colour.
    #[inline] pub fn color(mut self, c: Color) -> Self { self.color = c; self }
    /// Sets the size preset.
    #[inline] pub fn size(mut self, s: Size) -> Self { self.size = s; self }
    /// Enables or disables the ripple effect.
    #[inline] pub fn ripple(mut self, b: bool) -> Self { self.ripple = b; self }

    /// Adds utility classes to the group’s [`ComponentBase`].
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Minimum height of custom cells for the configured [`Size`].
    #[inline]
    fn cell_height(&self) -> f32 {
        cell_height_for(&self.size)
    }

    /// Collects the items described by `add_items`, renders them, and applies
    /// at most one click to `selected` according to the selection mode.
    ///
    /// * An empty group allocates a zero-sized response and renders nothing.
    /// * Disabled groups never modify `selected`.
    /// * The returned [`Response`] belongs to the group container, not to an
    ///   individual item; compare `selected` before and after to detect a
    ///   change.
    ///
    /// The `add_items` closure runs exactly once per call. Custom-content
    /// closures run twice per frame (see the module docs).
    pub fn show<T, R>(
        self,
        ui: &mut Ui,
        selected: &mut Vec<T>,
        add_items: impl FnOnce(&mut ToggleGroupUi<T>) -> R,
    ) -> Response
    where
        T: Clone + PartialEq,
    {
        let mut items: Vec<MudToggleItem<T>> = Vec::new();
        {
            let mut group_ui = ToggleGroupUi { items: &mut items };
            let _ = add_items(&mut group_ui);
        }

        if items.is_empty() {
            return ui.allocate_response(Vec2::ZERO, Sense::hover());
        }

        let count = items.len();
        let wants_sep = wants_separator(self.delimiters, self.outlined);
        let has_custom = items.iter().any(|i| i.content.is_some());
        let any_selected = items.iter().any(|i| selected.contains(&i.value));
        let chrome = self.outlined;
        let stretch = is_stretched(self.vertical, has_custom);
        let wants_accent = needs_accent(has_custom, chrome, any_selected);

        // Resolve theme colours at most once per frame, and only when some
        // code path actually needs them.
        let (accent, theme_sep): (Option<Color32>, Option<Color32>) =
            if needs_theme(wants_sep, wants_accent) {
                let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
                let palette = theme.current_palette();
                let accent = if wants_accent {
                    self.color.resolve(palette)
                } else {
                    None
                };
                let sep = if wants_sep {
                    Some(Color::Default.resolve(palette).unwrap_or_else(|| {
                        ui.visuals().widgets.noninteractive.bg_stroke.color
                    }))
                } else {
                    None
                };
                (accent, sep)
            } else {
                (None, None)
            };
        let sep_color = theme_sep.or_else(|| chrome_divider(chrome, accent));

        let min_h = self.cell_height();
        let radius = radius_to_u8(DEFAULT_CORNER_RADIUS);
        let avail = ui.available_width();
        let sep_n = separator_count(sep_color.is_some(), count);
        let cell_w = if self.vertical {
            let mut natural = 0.0_f32;
            if !has_custom {
                for item in items.iter() {
                    let is_sel = selected.contains(&item.value);
                    let mut probe_item = MudToggleItem::new((), item.text.clone());
                    probe_item.selected_icon = item.selected_icon.clone();
                    probe_item.unselected_icon = item.unselected_icon.clone();
                    let btn = self.make_button(&mut probe_item, is_sel, false);
                    natural = natural.max(measure_widget_width(ui, btn, avail));
                }
            }
            vertical_cell_width(avail, natural, has_custom)
        } else {
            cell_width(avail, count, sep_n)
        };

        // Measure custom content up front so every cell of a row can share one
        // height. Empty (and allocation-free) for plain button groups.
        let mut heights: Vec<f32> = Vec::new();
        if has_custom {
            heights.reserve(count);
            let inner_w = custom_inner_width(cell_w);
            for item in items.iter_mut() {
                let is_sel = selected.contains(&item.value);
                let h = match item.content.as_deref_mut() {
                    Some(content) => {
                        padded_cell_height(measure_content(ui, content, is_sel, inner_w), min_h)
                    }
                    None => min_h,
                };
                heights.push(h);
            }
        }
        let row_h = heights.iter().copied().fold(min_h, f32::max);

        let mut clicked_value: Option<T> = None;
        let mut clicked_item_idx: Option<usize> = None;

        let response = if self.vertical {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                for (i, item) in items.iter_mut().enumerate() {
                    if i > 0 {
                        if let Some(color) = sep_color {
                            paint_separator(ui, vec2(cell_w, SEPARATOR_THICKNESS), color);
                        }
                    }
                    let cell = CellLayout {
                        width: cell_w,
                        height: heights.get(i).copied().unwrap_or(min_h),
                        corners: cell_corners(i, count, true, radius),
                        stretch,
                    };
                    let is_sel = selected.contains(&item.value);
                    let out = self.render_item(ui, item, is_sel, cell, accent, i, count);
                    if out.clicked && !self.disabled && !item.disabled {
                        clicked_value = Some(item.value.clone());
                        clicked_item_idx = Some(i);
                    }
                }
            })
                .response
        } else {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                if has_custom {
                    ui.set_min_height(row_h);
                }
                let mut prev_h = row_h;

                for pos in 0..count {
                    if pos > 0 {
                        if let Some(color) = sep_color {
                            paint_separator(ui, vec2(SEPARATOR_THICKNESS, prev_h), color);
                        }
                    }
                    let idx = display_index(pos, count, self.right_to_left);
                    let item = &mut items[idx];
                    let cell = CellLayout {
                        width: cell_w,
                        height: row_h,
                        corners: cell_corners(pos, count, false, radius),
                        stretch,
                    };
                    let is_sel = selected.contains(&item.value);
                    let out = self.render_item(ui, item, is_sel, cell, accent, idx, count);
                    prev_h = out.height;
                    if out.clicked && !self.disabled && !item.disabled {
                        clicked_value = Some(item.value.clone());
                        clicked_item_idx = Some(idx);
                    }
                }
            })
                .response
        };

        if chrome {
            let stroke = Stroke::new(
                OUTLINE_STROKE_WIDTH,
                accent.unwrap_or(FALLBACK_OUTLINE_COLOR),
            );
            ui.painter().rect_stroke(
                response.rect,
                CornerRadius::same(radius),
                stroke,
                StrokeKind::Inside,
            );
        }

        if let (Some(value), Some(idx)) = (clicked_value, clicked_item_idx) {
            if self.ripple {
                start_ripple(ui.ctx(), response.id.with(idx));
            }
            let _ = apply_selection(&self.selection_mode, selected, value);
        }

        response
    }

    /// Renders one item into its cell and reports whether it was clicked this
    /// frame and how tall the cell turned out.
    fn render_item<T>(
        &self,
        ui: &mut Ui,
        item: &mut MudToggleItem<T>,
        is_selected: bool,
        cell: CellLayout,
        accent: Option<Color32>,
        item_idx: usize,
        _count: usize,
    ) -> ItemOutcome {
        if let Some(content) = item.content.as_deref_mut() {
            let clicked = self.render_custom(ui, content, is_selected, cell, accent, item.disabled);
            return ItemOutcome {
                clicked,
                height: cell.height,
            };
        }
        self.render_standard(ui, item, is_selected, cell, accent, item_idx)
    }

    /// Custom-content path: paints the selection fill, hover overlay, focus
    /// ring, and ripple (outer corners only) and hands the padded interior to
    /// the user’s closure.
    fn render_custom(
        &self,
        ui: &mut Ui,
        content: &mut dyn FnMut(&mut Ui, bool),
        is_selected: bool,
        cell: CellLayout,
        accent: Option<Color32>,
        item_disabled: bool,
    ) -> bool {
        let effective_disabled = self.disabled || item_disabled;
        let sense = if effective_disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let (rect, resp) = ui.allocate_exact_size(vec2(cell.width, cell.height), sense);

        if let Some(bg) = custom_background(is_selected, accent) {
            ui.painter().rect_filled(rect, cell.corners, bg);
        }
        if resp.hovered() && !effective_disabled {
            ui.painter().rect_filled(
                rect,
                cell.corners,
                Color32::from_white_alpha(HOVER_OVERLAY_ALPHA),
            );
        }
        if resp.has_focus() {
            let focus_color = accent.unwrap_or(Color32::WHITE);
            ui.painter().rect_stroke(
                rect,
                cell.corners,
                Stroke::new(2.0, focus_color),
                StrokeKind::Outside,
            );
        }

        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(custom_content_rect(rect))
                .layout(Layout::top_down(Align::LEFT)),
        );
        if effective_disabled {
            child.disable();
        }
        content(&mut child, is_selected);
        resp.clicked() && !effective_disabled
    }

    /// Standard path: a borderless [`MudButton`]. Stretched cells get a
    /// full-width button; content-sized cells add it at its natural width.
    fn render_standard<T>(
        &self,
        ui: &mut Ui,
        item: &mut MudToggleItem<T>,
        is_selected: bool,
        cell: CellLayout,
        accent: Option<Color32>,
        item_idx: usize,
    ) -> ItemOutcome {
        let effective_disabled = self.disabled || item.disabled;
        let btn = self.make_button(item, is_selected, effective_disabled);
        let underfill = underfill_color(is_selected, effective_disabled, accent);
        let slot = underfill.map(|_| ui.painter().add(Shape::Noop));

        let (clicked, rect) = if cell.stretch {
            ui.allocate_ui_with_layout(
                vec2(cell.width, 0.0),
                Layout::left_to_right(Align::Min),
                |ui| {
                    let resp = ui.add(btn.full_width(true));
                    (resp.clicked(), resp.rect)
                },
            )
                .inner
        } else {
            let resp = ui.add(btn);
            (resp.clicked(), resp.rect)
        };

        if let (Some(idx), Some(color)) = (slot, underfill) {
            ui.painter()
                .set(idx, Shape::rect_filled(rect, cell.corners, color));
        }

        if self.ripple && clicked {
            paint_ripple(ui, rect, cell.corners, accent, item_idx);
        }

        ItemOutcome {
            clicked: clicked && !effective_disabled,
            height: rect.height(),
        }
    }

    /// Builds the [`MudButton`] that represents `item`.
    ///
    /// The item’s `text` and icon strings are **moved** out (`take`) rather
    /// than cloned; items are single-use per frame.
    fn make_button<T>(
        &self,
        item: &mut MudToggleItem<T>,
        is_selected: bool,
        effective_disabled: bool,
    ) -> MudButton {
        let mut btn = MudButton::new(std::mem::take(&mut item.text))
            .variant(button_variant(is_selected))
            .color(self.color)
            .size(self.size)
            .drop_shadow(false)
            .disabled(effective_disabled);

        let leading = resolve_leading_icon(
            self.check_mark,
            self.fixed_content,
            is_selected,
            item.selected_icon.take(),
            item.unselected_icon.take(),
        );

        if let Some(icon) = leading {
            btn = btn.start_icon(icon.glyph);
            if icon.hidden {
                btn = btn.icon_color(Color::Custom(Color32::TRANSPARENT));
            }
        }

        btn
    }
}

// ---------------------------------------------------------------------------
// ToggleGroupUi
// ---------------------------------------------------------------------------

/// Collector handle passed to [`MudToggleGroup::show`].
///
/// It only records items; nothing is rendered until the closure returns.
pub struct ToggleGroupUi<'a, T> {
    items: &'a mut Vec<MudToggleItem<T>>,
}

impl<T> ToggleGroupUi<'_, T> {
    /// Adds a plain text item for `value`.
    pub fn item(&mut self, value: T, text: impl Into<String>) {
        self.items.push(MudToggleItem::new(value, text));
    }

    /// Adds a fully configured [`MudToggleItem`] (icons, custom content, ...).
    pub fn item_full(&mut self, item: MudToggleItem<T>) {
        self.items.push(item);
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (no `egui::Context` required)
// ---------------------------------------------------------------------------

/// Geometry and style decisions for one cell.
#[derive(Clone, Copy)]
struct CellLayout {
    width: f32,
    height: f32,
    corners: CornerRadius,
    stretch: bool,
}

/// What rendering one item reports back to [`MudToggleGroup::show`].
#[derive(Clone, Copy)]
struct ItemOutcome {
    clicked: bool,
    height: f32,
}

/// Minimum cell height in logical pixels for a [`Size`] preset.
#[inline]
fn cell_height_for(size: &Size) -> f32 {
    match size {
        Size::ExtraSmall => 24.0,
        Size::Small => 32.0,
        Size::Medium => 40.0,
        Size::Large => 48.0,
        Size::ExtraLarge => 56.0,
    }
}

/// Converts a corner radius to the `u8` egui expects: negatives and NaN
/// become `0`, values above `255` saturate, fractions truncate.
#[inline]
fn radius_to_u8(radius: f32) -> u8 {
    radius.clamp(0.0, 255.0) as u8
}

/// Replaces non-finite measurements with `0.0`.
#[inline]
fn finite_or_zero(v: f32) -> f32 {
    if v.is_finite() { v } else { 0.0 }
}

/// Whether the group uses stretched (fixed-width cell) layout.
#[inline]
fn is_stretched(vertical: bool, has_custom_content: bool) -> bool {
    vertical || has_custom_content
}

/// Width of one cell in stretched horizontal layout.
fn cell_width(available: f32, count: usize, separators: usize) -> f32 {
    if count == 0 {
        return 0.0;
    }
    let available = finite_or_zero(available);
    ((available - separators as f32 * SEPARATOR_THICKNESS) / count as f32).max(0.0)
}

/// Cell width of a vertical group.
fn vertical_cell_width(available: f32, natural: f32, has_custom: bool) -> f32 {
    let avail = finite_or_zero(available);
    if has_custom {
        return avail;
    }
    let natural = finite_or_zero(natural);
    if natural <= 0.0 {
        avail
    } else if available.is_finite() {
        natural.min(avail)
    } else {
        natural
    }
}

/// Width available to custom content inside a cell of `cell_width`.
#[inline]
fn custom_inner_width(cell_width: f32) -> f32 {
    (finite_or_zero(cell_width) - 2.0 * CUSTOM_CONTENT_PADDING).max(0.0)
}

/// Final height of a custom cell: measured content plus padding, never below
/// the size preset’s minimum height.
#[inline]
fn padded_cell_height(measured_content: f32, min_height: f32) -> f32 {
    (finite_or_zero(measured_content) + 2.0 * CUSTOM_CONTENT_PADDING).max(min_height)
}

/// Number of delimiter lines drawn between `count` items.
#[inline]
fn separator_count(has_separator: bool, count: usize) -> usize {
    if has_separator {
        count.saturating_sub(1)
    } else {
        0
    }
}

/// Maps a visual position to the index of the item drawn there.
#[inline]
fn display_index(pos: usize, count: usize, right_to_left: bool) -> usize {
    if right_to_left {
        count.saturating_sub(pos + 1)
    } else {
        pos
    }
}

/// Corner rounding for the cell at visual position `pos` of `count`: only the
/// corners on the outside of the group are rounded.
fn cell_corners(pos: usize, count: usize, vertical: bool, radius: u8) -> CornerRadius {
    let first = pos == 0;
    let last = pos + 1 >= count;
    let (nw, ne, sw, se) = if vertical {
        (first, first, last, last)
    } else {
        (first, last, first, last)
    };
    let r = |on: bool| if on { radius } else { 0 };
    CornerRadius {
        nw: r(nw),
        ne: r(ne),
        sw: r(sw),
        se: r(se),
    }
}

/// Delimiter lines are drawn only for flat groups.
#[inline]
fn wants_separator(delimiters: bool, outlined: bool) -> bool {
    delimiters && !outlined
}

/// Whether the accent colour is required.
#[inline]
fn needs_accent(has_custom_content: bool, chrome: bool, any_selected: bool) -> bool {
    has_custom_content || chrome || any_selected
}

/// The theme is fetched only when a delimiter colour or the accent colour is
/// required.
#[inline]
fn needs_theme(wants_separator: bool, wants_accent: bool) -> bool {
    wants_separator || wants_accent
}

/// Divider colour for outlined groups.
#[inline]
fn chrome_divider(chrome: bool, accent: Option<Color32>) -> Option<Color32> {
    if chrome {
        Some(accent.unwrap_or(FALLBACK_OUTLINE_COLOR))
    } else {
        None
    }
}

/// The [`Variant`] of a standard button.
#[inline]
fn button_variant(is_selected: bool) -> Variant {
    if is_selected {
        Variant::Filled
    } else {
        Variant::Text
    }
}

/// Square-cornered fill placed under a selected standard button.
#[inline]
fn underfill_color(is_selected: bool, disabled: bool, accent: Option<Color32>) -> Option<Color32> {
    if is_selected && !disabled {
        accent
    } else {
        None
    }
}

/// Background fill of a custom-content cell.
#[inline]
fn custom_background(is_selected: bool, accent: Option<Color32>) -> Option<Color32> {
    if is_selected {
        Some(accent.unwrap_or(Color32::TRANSPARENT))
    } else {
        None
    }
}

/// The interior rectangle handed to a custom-content closure.
#[inline]
fn custom_content_rect(cell: Rect) -> Rect {
    cell.shrink(CUSTOM_CONTENT_PADDING)
}

/// Applies one click on `value` to `selected` according to `mode`.
///
/// Returns `true` when the selection changed.
fn apply_selection<T: PartialEq>(mode: &SelectionMode, selected: &mut Vec<T>, value: T) -> bool {
    match mode {
        SelectionMode::Single => {
            if selected.contains(&value) {
                false
            } else {
                selected.clear();
                selected.push(value);
                true
            }
        }
        SelectionMode::Toggle => {
            if selected.contains(&value) {
                selected.clear();
            } else {
                selected.clear();
                selected.push(value);
            }
            true
        }
        SelectionMode::Multi => {
            if let Some(p) = selected.iter().position(|v| v == &value) {
                selected.remove(p);
            } else {
                selected.push(value);
            }
            true
        }
    }
}

/// A resolved leading icon for a standard button.
#[derive(Debug, Clone, PartialEq, Eq)]
struct LeadingIcon {
    glyph: String,
    hidden: bool,
}

/// Decides which leading icon (if any) a standard button shows.
///
/// MudBlazor has no plain `Icon` parameter on `MudToggleItem`; icons are
/// driven exclusively by `CheckMark` and the selected/unselected icon pair.
fn resolve_leading_icon(
    check_mark: bool,
    fixed_content: bool,
    is_selected: bool,
    selected_icon: Option<String>,
    unselected_icon: Option<String>,
) -> Option<LeadingIcon> {
    if !check_mark {
        return None;
    }
    if is_selected {
        Some(LeadingIcon {
            glyph: selected_icon.unwrap_or_else(default_check_icon),
            hidden: false,
        })
    } else if fixed_content {
        let hidden = unselected_icon.is_none();
        Some(LeadingIcon {
            glyph: unselected_icon.unwrap_or_else(default_check_icon),
            hidden,
        })
    } else {
        unselected_icon.map(|glyph| LeadingIcon {
            glyph,
            hidden: false,
        })
    }
}

/// Runs `content` once inside an invisible probe `Ui` and returns the height
/// it used.
fn measure_content(
    ui: &mut Ui,
    content: &mut dyn FnMut(&mut Ui, bool),
    is_selected: bool,
    inner_width: f32,
) -> f32 {
    let origin = ui.max_rect().min;
    let probe_rect = Rect::from_min_size(origin, vec2(inner_width, MAX_CUSTOM_CONTENT_HEIGHT));
    let mut probe = ui.new_child(
        UiBuilder::new()
            .max_rect(probe_rect)
            .layout(Layout::top_down(Align::LEFT))
            .invisible(),
    );
    content(&mut probe, is_selected);
    finite_or_zero(probe.min_rect().height())
}

/// Adds `widget` to an invisible probe `Ui` and returns the natural width it
/// takes.
fn measure_widget_width(ui: &mut Ui, widget: impl egui::Widget, available: f32) -> f32 {
    let width = if available.is_finite() && available > 0.0 {
        available
    } else {
        4096.0
    };
    let origin = ui.max_rect().min;
    let probe_rect = Rect::from_min_size(origin, vec2(width, MAX_CUSTOM_CONTENT_HEIGHT));
    let mut probe = ui.new_child(UiBuilder::new().max_rect(probe_rect).invisible());
    finite_or_zero(probe.add(widget).rect.width())
}

/// Allocates `size` and fills it with `color` to draw one delimiter line.
fn paint_separator(ui: &mut Ui, size: Vec2, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, 0.0, color);
}

/// The Material check glyph used when an item supplies no `selected_icon`.
fn default_check_icon() -> String {
    mud_egui_icons_material::filled::CHECK.to_string()
}

// ---------------------------------------------------------------------------
// Ripple
// ---------------------------------------------------------------------------

/// Starts a ripple animation for the given response id.
///
/// Reads the current frame time **before** acquiring the memory lock.
/// `egui::Context` uses a non-reentrant `RwLock`, so calling `ctx.input(..)`
/// inside `ctx.memory_mut(..)` self-deadlocks. Always take the two locks
/// sequentially, never nested.
fn start_ripple(ctx: &egui::Context, id: egui::Id) {
    let time = ctx.input(|i| i.time);
    ctx.memory_mut(|m| m.data.insert_temp(id, time));
}

/// Paints a ripple on `rect` if one was started for `idx` recently.
fn paint_ripple(
    ui: &mut Ui,
    rect: Rect,
    corners: CornerRadius,
    accent: Option<Color32>,
    idx: usize,
) {
    let id = ui.id().with(idx);
    let start: Option<f64> = ui.ctx().memory(|m| m.data.get_temp(id));
    let Some(start) = start else { return };
    let now = ui.input(|i| i.time);
    let elapsed = (now - start) as f32;
    if elapsed >= RIPPLE_DURATION {
        ui.ctx().memory_mut(|m| m.data.remove_temp::<f64>(id));
        return;
    }
    let t = (elapsed / RIPPLE_DURATION).clamp(0.0, 1.0);
    let radius = rect.width().min(rect.height()) * 0.5 * RIPPLE_MAX_RADIUS_FACTOR * t;
    let alpha = ((1.0 - t) * 64.0) as u8;
    let color = accent.unwrap_or(Color32::WHITE);
    let color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
    let center = rect.center();
    ui.painter()
        .circle_filled(center, radius, color);
    ui.ctx().request_repaint();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    // --- sizing -----------------------------------------------------------

    #[test]
    fn cell_height_matches_every_size_preset() {
        assert_eq!(cell_height_for(&Size::ExtraSmall), 24.0);
        assert_eq!(cell_height_for(&Size::Small), 32.0);
        assert_eq!(cell_height_for(&Size::Medium), 40.0);
        assert_eq!(cell_height_for(&Size::Large), 48.0);
        assert_eq!(cell_height_for(&Size::ExtraLarge), 56.0);
    }

    #[test]
    fn group_cell_height_follows_builder_size() {
        assert_eq!(MudToggleGroup::new().cell_height(), 40.0);
        assert_eq!(MudToggleGroup::new().size(Size::Large).cell_height(), 48.0);
    }

    #[test]
    fn radius_to_u8_clamps_and_truncates() {
        assert_eq!(radius_to_u8(4.0), 4);
        assert_eq!(radius_to_u8(4.9), 4);
        assert_eq!(radius_to_u8(0.0), 0);
        assert_eq!(radius_to_u8(-3.0), 0);
        assert_eq!(radius_to_u8(300.0), 255);
        assert_eq!(radius_to_u8(f32::NAN), 0);
        assert_eq!(radius_to_u8(f32::INFINITY), 255);
        assert_eq!(radius_to_u8(f32::NEG_INFINITY), 0);
    }

    #[test]
    fn finite_or_zero_repairs_non_finite_values() {
        assert_eq!(finite_or_zero(12.5), 12.5);
        assert_eq!(finite_or_zero(f32::NAN), 0.0);
        assert_eq!(finite_or_zero(f32::INFINITY), 0.0);
        assert_eq!(finite_or_zero(f32::NEG_INFINITY), 0.0);
    }

    #[test]
    fn is_stretched_truth_table() {
        assert!(!is_stretched(false, false));
        assert!(is_stretched(true, false));
        assert!(is_stretched(false, true));
        assert!(is_stretched(true, true));
    }

    #[test]
    fn cell_width_splits_space_after_separators() {
        assert_eq!(cell_width(100.0, 4, 3), 24.25);
        assert_eq!(cell_width(100.0, 2, 1), 49.5);
        assert_eq!(cell_width(90.0, 3, 0), 30.0);
    }

    #[test]
    fn cell_width_never_negative_or_non_finite() {
        assert_eq!(cell_width(2.0, 4, 3), 0.0);
        assert_eq!(cell_width(100.0, 0, 0), 0.0);
        assert_eq!(cell_width(f32::INFINITY, 2, 0), 0.0);
        assert_eq!(cell_width(f32::NAN, 2, 1), 0.0);
    }

    #[test]
    fn vertical_cell_width_is_content_sized_unless_custom() {
        assert_eq!(vertical_cell_width(500.0, 120.0, false), 120.0);
        assert_eq!(vertical_cell_width(100.0, 120.0, false), 100.0);
        assert_eq!(vertical_cell_width(500.0, 120.0, true), 500.0);
    }

    #[test]
    fn vertical_cell_width_repairs_bad_inputs() {
        assert_eq!(vertical_cell_width(500.0, 0.0, false), 500.0);
        assert_eq!(vertical_cell_width(500.0, f32::NAN, false), 500.0);
        assert_eq!(vertical_cell_width(500.0, -5.0, false), 500.0);
        assert_eq!(vertical_cell_width(f32::INFINITY, 120.0, false), 120.0);
    }

    #[test]
    fn custom_inner_width_subtracts_padding_and_never_goes_negative() {
        assert_eq!(custom_inner_width(100.0), 76.0);
        assert_eq!(custom_inner_width(24.0), 0.0);
        assert_eq!(custom_inner_width(10.0), 0.0);
        assert_eq!(custom_inner_width(f32::NAN), 0.0);
        assert_eq!(custom_inner_width(f32::INFINITY), 0.0);
    }

    #[test]
    fn padded_cell_height_adds_padding_but_respects_minimum() {
        assert_eq!(padded_cell_height(16.0, 40.0), 40.0);
        assert_eq!(padded_cell_height(50.0, 40.0), 74.0);
        assert_eq!(padded_cell_height(17.0, 40.0), 41.0);
    }

    #[test]
    fn padded_cell_height_repairs_bad_measurements() {
        assert_eq!(padded_cell_height(f32::NAN, 40.0), 40.0);
        assert_eq!(padded_cell_height(f32::INFINITY, 40.0), 40.0);
        assert_eq!(padded_cell_height(-100.0, 40.0), 40.0);
    }

    #[test]
    fn separator_count_is_between_items_only() {
        assert_eq!(separator_count(true, 4), 3);
        assert_eq!(separator_count(true, 2), 1);
        assert_eq!(separator_count(true, 1), 0);
        assert_eq!(separator_count(true, 0), 0);
        assert_eq!(separator_count(false, 4), 0);
    }

    #[test]
    fn display_index_reverses_without_allocating() {
        assert_eq!(display_index(0, 3, false), 0);
        assert_eq!(display_index(2, 3, false), 2);
        assert_eq!(display_index(0, 3, true), 2);
        assert_eq!(display_index(1, 3, true), 1);
        assert_eq!(display_index(2, 3, true), 0);
    }

    #[test]
    fn display_index_saturates_when_out_of_range() {
        assert_eq!(display_index(5, 3, true), 0);
        assert_eq!(display_index(0, 0, true), 0);
    }

    #[test]
    fn custom_content_rect_shrinks_by_padding() {
        let cell = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 40.0));
        let inner = custom_content_rect(cell);
        assert_eq!(inner.min, pos2(12.0, 12.0));
        assert_eq!(inner.max, pos2(88.0, 28.0));
    }

    // --- corner rounding --------------------------------------------------

    #[test]
    fn horizontal_corners_round_left_of_first_and_right_of_last() {
        let first = cell_corners(0, 3, false, 4);
        assert_eq!((first.nw, first.sw, first.ne, first.se), (4, 4, 0, 0));
        let mid = cell_corners(1, 3, false, 4);
        assert_eq!((mid.nw, mid.sw, mid.ne, mid.se), (0, 0, 0, 0));
        let last = cell_corners(2, 3, false, 4);
        assert_eq!((last.nw, last.sw, last.ne, last.se), (0, 0, 4, 4));
    }

    #[test]
    fn vertical_corners_round_top_of_first_and_bottom_of_last() {
        let first = cell_corners(0, 3, true, 6);
        assert_eq!((first.nw, first.ne, first.sw, first.se), (6, 6, 0, 0));
        let mid = cell_corners(1, 3, true, 6);
        assert_eq!((mid.nw, mid.ne, mid.sw, mid.se), (0, 0, 0, 0));
        let last = cell_corners(2, 3, true, 6);
        assert_eq!((last.nw, last.ne, last.sw, last.se), (0, 0, 6, 6));
    }

    #[test]
    fn single_cell_rounds_every_corner() {
        assert_eq!(cell_corners(0, 1, false, 5), CornerRadius::same(5));
        assert_eq!(cell_corners(0, 1, true, 5), CornerRadius::same(5));
    }

    #[test]
    fn zero_radius_means_no_rounding() {
        assert_eq!(cell_corners(0, 3, false, 0), CornerRadius::same(0));
        assert_eq!(cell_corners(2, 3, true, 0), CornerRadius::same(0));
    }

    // --- theme / paint decisions -----------------------------------------

    #[test]
    fn wants_separator_truth_table() {
        assert!(wants_separator(true, false));
        assert!(!wants_separator(true, true));
        assert!(!wants_separator(false, false));
        assert!(!wants_separator(false, true));
    }

    #[test]
    fn needs_accent_truth_table() {
        assert!(!needs_accent(false, false, false));
        assert!(needs_accent(true, false, false));
        assert!(needs_accent(false, true, false));
        assert!(needs_accent(false, false, true));
        assert!(needs_accent(true, true, true));
    }

    #[test]
    fn needs_theme_truth_table() {
        assert!(!needs_theme(false, false));
        assert!(needs_theme(true, false));
        assert!(needs_theme(false, true));
        assert!(needs_theme(true, true));
    }

    #[test]
    fn chrome_divider_uses_accent_or_fallback() {
        let red = Color32::from_rgb(200, 10, 10);
        assert_eq!(chrome_divider(true, Some(red)), Some(red));
        assert_eq!(chrome_divider(true, None), Some(FALLBACK_OUTLINE_COLOR));
        assert_eq!(chrome_divider(false, Some(red)), None);
        assert_eq!(chrome_divider(false, None), None);
    }

    #[test]
    fn buttons_are_borderless_filled_or_text() {
        assert!(matches!(button_variant(true), Variant::Filled));
        assert!(matches!(button_variant(false), Variant::Text));
    }

    #[test]
    fn underfill_only_for_enabled_selected_cells_with_an_accent() {
        let red = Color32::from_rgb(200, 10, 10);
        assert_eq!(underfill_color(true, false, Some(red)), Some(red));
        assert_eq!(underfill_color(true, false, None), None);
        assert_eq!(underfill_color(true, true, Some(red)), None);
        assert_eq!(underfill_color(false, false, Some(red)), None);
        assert_eq!(underfill_color(false, true, None), None);
    }

    #[test]
    fn custom_background_only_when_selected() {
        let red = Color32::from_rgb(200, 10, 10);
        assert_eq!(custom_background(true, Some(red)), Some(red));
        assert_eq!(custom_background(true, None), Some(Color32::TRANSPARENT));
        assert_eq!(custom_background(false, Some(red)), None);
        assert_eq!(custom_background(false, None), None);
    }

    // --- selection --------------------------------------------------------

    #[test]
    fn single_mode_replaces_and_ignores_reclick() {
        let mut sel = vec![1];
        assert!(apply_selection(&SelectionMode::Single, &mut sel, 2));
        assert_eq!(sel, vec![2]);
        assert!(!apply_selection(&SelectionMode::Single, &mut sel, 2));
        assert_eq!(sel, vec![2]);
    }

    #[test]
    fn single_mode_selects_from_empty() {
        let mut sel: Vec<i32> = Vec::new();
        assert!(apply_selection(&SelectionMode::Single, &mut sel, 7));
        assert_eq!(sel, vec![7]);
    }

    #[test]
    fn toggle_mode_clears_on_reclick_and_replaces_otherwise() {
        let mut sel = vec![1];
        assert!(apply_selection(&SelectionMode::Toggle, &mut sel, 2));
        assert_eq!(sel, vec![2]);
        assert!(apply_selection(&SelectionMode::Toggle, &mut sel, 2));
        assert!(sel.is_empty());
        assert!(apply_selection(&SelectionMode::Toggle, &mut sel, 3));
        assert_eq!(sel, vec![3]);
    }

    #[test]
    fn toggle_mode_clears_every_prepopulated_entry() {
        let mut sel = vec![1, 2, 3];
        assert!(apply_selection(&SelectionMode::Toggle, &mut sel, 2));
        assert!(sel.is_empty());
    }

    #[test]
    fn multi_mode_adds_and_removes_preserving_order() {
        let mut sel: Vec<i32> = Vec::new();
        apply_selection(&SelectionMode::Multi, &mut sel, 1);
        apply_selection(&SelectionMode::Multi, &mut sel, 2);
        apply_selection(&SelectionMode::Multi, &mut sel, 3);
        assert_eq!(sel, vec![1, 2, 3]);
        apply_selection(&SelectionMode::Multi, &mut sel, 2);
        assert_eq!(sel, vec![1, 3]);
        apply_selection(&SelectionMode::Multi, &mut sel, 1);
        apply_selection(&SelectionMode::Multi, &mut sel, 3);
        assert!(sel.is_empty());
    }

    #[test]
    fn selection_works_with_non_copy_values() {
        let mut sel: Vec<String> = Vec::new();
        apply_selection(&SelectionMode::Multi, &mut sel, "a".to_string());
        apply_selection(&SelectionMode::Multi, &mut sel, "b".to_string());
        apply_selection(&SelectionMode::Multi, &mut sel, "a".to_string());
        assert_eq!(sel, vec!["b".to_string()]);
    }

    // --- icon resolution --------------------------------------------------

    #[test]
    fn default_check_icon_is_not_empty() {
        assert!(!default_check_icon().is_empty());
    }

    #[test]
    fn no_icon_when_check_mark_is_off() {
        assert_eq!(
            resolve_leading_icon(false, false, false, Some("s".into()), Some("u".into())),
            None
        );
        assert_eq!(
            resolve_leading_icon(false, true, true, Some("s".into()), Some("u".into())),
            None
        );
    }

    #[test]
    fn selected_check_uses_custom_or_default_glyph() {
        let custom = resolve_leading_icon(true, false, true, Some("s".into()), None);
        assert_eq!(
            custom,
            Some(LeadingIcon {
                glyph: "s".into(),
                hidden: false
            })
        );
        let default = resolve_leading_icon(true, false, true, None, None);
        assert_eq!(
            default,
            Some(LeadingIcon {
                glyph: default_check_icon(),
                hidden: false
            })
        );
    }

    #[test]
    fn unselected_fixed_content_uses_hidden_placeholder() {
        let placeholder = resolve_leading_icon(true, true, false, None, None);
        assert_eq!(
            placeholder,
            Some(LeadingIcon {
                glyph: default_check_icon(),
                hidden: true
            })
        );
        let explicit = resolve_leading_icon(true, true, false, None, Some("u".into()));
        assert_eq!(
            explicit,
            Some(LeadingIcon {
                glyph: "u".into(),
                hidden: false
            })
        );
    }

    #[test]
    fn unselected_without_fixed_content_shows_only_explicit_icon() {
        let explicit = resolve_leading_icon(true, false, false, None, Some("u".into()));
        assert_eq!(
            explicit,
            Some(LeadingIcon {
                glyph: "u".into(),
                hidden: false
            })
        );
        assert_eq!(resolve_leading_icon(true, false, false, None, None), None);
    }

    // --- builders & collectors -------------------------------------------

    #[test]
    fn group_defaults_match_mudblazor() {
        let g = MudToggleGroup::new();
        assert!(matches!(g.selection_mode, SelectionMode::Single));
        assert!(!g.check_mark);
        assert!(g.check_mark_class.is_none());
        assert!(g.selected_class.is_none());
        assert!(!g.fixed_content);
        assert!(!g.disabled);
        assert!(!g.vertical);
        assert!(!g.right_to_left);
        assert!(g.delimiters);
        assert!(g.outlined);
        assert!(matches!(g.color, Color::Primary));
        assert!(matches!(g.size, Size::Medium));
        assert!(g.ripple);
    }

    #[test]
    fn group_builders_set_every_field() {
        let g = MudToggleGroup::new()
            .selection_mode(SelectionMode::Multi)
            .check_mark(true)
            .check_mark_class("cm")
            .selected_class("sel")
            .fixed_content(true)
            .disabled(true)
            .vertical(true)
            .right_to_left(true)
            .delimiters(false)
            .outlined(false)
            .color(Color::Secondary)
            .size(Size::Small)
            .ripple(false)
            .classes(["a", "b"]);
        assert!(matches!(g.selection_mode, SelectionMode::Multi));
        assert!(g.check_mark && g.fixed_content && g.disabled);
        assert!(g.vertical && g.right_to_left);
        assert!(!g.delimiters && !g.outlined);
        assert!(matches!(g.color, Color::Secondary));
        assert!(matches!(g.size, Size::Small));
        assert!(!g.ripple);
        assert_eq!(g.check_mark_class.as_deref(), Some("cm"));
        assert_eq!(g.selected_class.as_deref(), Some("sel"));
    }

    #[test]
    fn item_new_has_no_icons_or_content() {
        let item = MudToggleItem::new(5, "Five");
        assert_eq!(item.value, 5);
        assert_eq!(item.text, "Five");
        assert!(item.selected_icon.is_none());
        assert!(item.unselected_icon.is_none());
        assert!(!item.disabled);
        assert!(item.content.is_none());
    }

    #[test]
    fn item_builders_set_icons_disabled_and_content() {
        let item = MudToggleItem::new("v", "V")
            .selected_icon("s")
            .unselected_icon("u")
            .disabled(true)
            .content(|_ui, _selected| {});
        assert_eq!(item.selected_icon.as_deref(), Some("s"));
        assert_eq!(item.unselected_icon.as_deref(), Some("u"));
        assert!(item.disabled);
        assert!(item.content.is_some());
    }

    #[test]
    fn collector_preserves_insertion_order() {
        let mut items: Vec<MudToggleItem<i32>> = Vec::new();
        {
            let mut ui = ToggleGroupUi { items: &mut items };
            ui.item(1, "one");
            ui.item_full(MudToggleItem::new(2, "two").selected_icon("x"));
            ui.item(3, String::from("three"));
        }
        let values: Vec<i32> = items.iter().map(|i| i.value).collect();
        assert_eq!(values, vec![1, 2, 3]);
        assert_eq!(items[1].selected_icon.as_deref(), Some("x"));
        assert_eq!(items[2].text, "three");
    }
}