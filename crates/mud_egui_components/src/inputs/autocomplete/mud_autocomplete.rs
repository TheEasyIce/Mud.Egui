//! `MudAutocomplete<T>` — a text field with a filtered dropdown of
//! candidates, single-value selection only.
//!
//! Port of `MudAutocomplete<T>.razor.cs` (MudBlazor 8.13.0).
//!
//! # Composition, not reimplementation
//!
//! Like [`MudSelect`](crate::MudSelect), the port is built from two
//! primitives:
//!
//! | Primitive | Role |
//! |-----------|------|
//! | [`MudTextField`] | The editable trigger. Gives `Variant`, `Margin`, `Size`, `Color`, `Label`, `Placeholder`, `HelperText`, `Error`, `ErrorText`, `Clearable`, and the search-icon adornment for free. |
//! | [`MudPopover`] | The candidate dropdown. Anchor attachment, flip-when-overflowing, and Escape / outside-click dismissal all delegated. |
//!
//! Only new code is the **candidate list** (rows + keyboard focus) and the
//! **commit / coerce / reset logic**.
//!
//! # Async search is caller-owned
//!
//! MudBlazor's `SearchFunc` returns `Task<IEnumerable<T>>`. egui is
//! immediate-mode with no per-frame async runtime, so the port delegates the
//! search to the caller:
//!
//! * The caller writes `state.candidates` each frame from their own filter
//!   (or from a channel reader that receives async results).
//! * [`AutocompleteState::text_changed_since_search`] and
//!   [`AutocompleteState::mark_searched`] help avoid running the filter on
//!   every frame when the text has not changed.
//! * The component **never** invokes a search function itself.
//!
//! # MudBlazor parity — pass 1
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `Value`             | [`AutocompleteState::selected`] (`Option<T>`) | `None` |
//! | `Text`              | [`AutocompleteState::search_text`] | `""` |
//! | `ToStringFunc`      | [`MudAutocomplete::tostring_func`] | `None` |
//! | `SearchFunc`        | *(caller-owned candidates, see above)* | — |
//! | `Label`             | [`MudAutocomplete::label`] | `None` |
//! | `Placeholder`       | [`MudAutocomplete::placeholder`] | `None` |
//! | `HelperText`        | [`MudAutocomplete::helper_text`] | `None` |
//! | `Error`             | [`MudAutocomplete::error`] | `false` |
//! | `ErrorText`         | [`MudAutocomplete::error_text`] | `None` |
//! | `Required`          | [`MudAutocomplete::required`] | `false` (visual only) |
//! | `Disabled`          | [`MudAutocomplete::disabled`] | `false` |
//! | `ReadOnly`          | [`MudAutocomplete::readonly`] | `false` |
//! | `Clearable`         | [`MudAutocomplete::clearable`] | `true` |
//! | `Variant`           | [`MudAutocomplete::variant`] | [`Variant::Text`] |
//! | `Margin`            | [`MudAutocomplete::margin`] | [`Margin::None`] |
//! | `Color`             | [`MudAutocomplete::color`] | [`Color::Primary`] |
//! | `Size`              | [`MudAutocomplete::size`] | [`Size::Medium`] |
//! | `Dense`             | [`MudAutocomplete::dense`] | `false` |
//! | `Adornment` + `AdornmentIcon` + `AdornmentColor` | [`MudAutocomplete::adornment_icon`] / [`MudAutocomplete::adornment_color`] | `SEARCH` / `None` |
//! | `AnchorOrigin`      | [`MudAutocomplete::anchor_origin`] | `None` |
//! | `TransformOrigin`   | [`MudAutocomplete::transform_origin`] | `None` |
//! | `Position`          | [`MudAutocomplete::position`] | `None` |
//! | `MaxItems`          | [`MudAutocomplete::max_items`] | `None` |
//! | `MinCharacters`     | [`MudAutocomplete::min_characters`] | `0` |
//! | `OpenOnFocus`       | [`MudAutocomplete::open_on_focus`] | `true` |
//! | `ResetValueOnEmptyText` | [`MudAutocomplete::reset_value_on_empty_text`] | `false` |
//! | `NoItemsTemplate`   | [`MudAutocomplete::no_items_template`] | `None` |
//! | `AfterItemsTemplate` | [`MudAutocomplete::after_items_template`] | `None` |
//! | `ItemTemplate`      | [`MudAutocomplete::item_template`] | `None` |
//! | `OnOpen`            | [`MudAutocomplete::on_open`] | `None` |
//! | `OnClose`           | [`MudAutocomplete::on_close`] | `None` |
//! | `ChildContent`      | *(replaced by `ItemTemplate` + candidates)* | — |
//!
//! # Extensions (not in MudBlazor)
//!
//! | Extension | Description |
//! |-----------|-------------|
//! | [`AutocompleteState`] | Caller-owned search text + candidates + selection. |
//! | [`AutocompleteAction`] | Rich return value reporting what happened this frame. |
//! | [`AutocompleteResponse`] | Frame's `trigger` response, resulting `selected` / `search_text`, and `action`. |
//!
//! # Deferred to pass 2
//!
//! `CoerceValue`, `CoerceValueDelay`, `DebounceInterval`, `Strict`,
//! `SelectValueOnTab`, `ShowProgressIndicator`, `ProgressIndicatorColor`,
//! `ProgressIndicatorInPopoverTemplate`, `MoreItemsTemplate`,
//! `KeyDownPreventDefault`, `KeyUpPreventDefault`, `HelperTextOnFocus`,
//! `ShrinkLabel`, `DisableUnderLine`, `AriaLabel`, `AfterItemsTemplate` with
//! parameters, `NoItemsTemplate` with parameters.
//!
//! # Not ported
//!
//! * **`Class` / `Style` / `Tag` / `UserAttributes`** — no CSS strings or
//!   HTML tags in egui.
//!
//! # Text and icons
//!
//! Trigger label / placeholder / helper / error go through `MudText` (via
//! `MudTextField`). Candidate rows render text through [`MudText`]. The
//! search adornment and any per-item icons go through [`MudIcon`].
//!
//! # Example
//!
//! ```ignore
//! let mut state = AutocompleteState::<Fruit>::new("fruit-search");
//!
//! // Each frame, refresh candidates if the text changed.
//! if state.text_changed_since_search() {
//!     state.candidates = fruits
//!         .iter()
//!         .filter(|f| f.name.to_lowercase().contains(&state.search_text.to_lowercase()))
//!         .cloned()
//!         .collect();
//!     state.mark_searched();
//! }
//!
//! MudAutocomplete::new()
//!     .label("Fruit")
//!     .tostring_func(|f| f.name.clone())
//!     .show(ui, &mut state);
//! ```

use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Layout, Rect, Response, Sense, Ui,
    UiBuilder,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{
    AnchorOrigin, Color, Margin, Position, Size, StyleFlavor, TransformOrigin, Typo,
    Variant,
};
use mud_egui_icons_material::filled;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::inputs::text_field::{AdornmentEnd, MudTextField};
use crate::feedback::popover::MudPopover;
use crate::typography::mud_text::MudText;

use super::autocomplete_action::AutocompleteAction;
use super::autocomplete_state::AutocompleteState;

// =============================================================================
// Constants
// =============================================================================

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Height of one candidate row (normal density), in logical pixels.
pub const ROW_HEIGHT: f32 = 40.0;

/// Height of one candidate row (dense), in logical pixels.
pub const DENSE_ROW_HEIGHT: f32 = 32.0;

/// Horizontal padding inside a candidate row, in logical pixels.
pub const ROW_PADDING_X: f32 = 12.0;

/// Default maximum dropdown height (matches MudBlazor's `MaxHeight` default
/// on `MudAutocomplete`'s popover, which inherits 300 from `MudPopover`).
pub const DEFAULT_MAX_HEIGHT: f32 = 300.0;

// =============================================================================
// Pure helpers
// =============================================================================

/// Candidate row height for the given density.
#[inline]
pub fn row_height(dense: bool) -> f32 {
    if dense { DENSE_ROW_HEIGHT } else { ROW_HEIGHT }
}

/// Resolves a `T` value's display text.
///
/// Returns `"?"` when no `tostring_func` is provided — a visible marker that
/// the value cannot be rendered. Pass 1 does not require `T: Display` (that
/// would be more restrictive than MudBlazor, which accepts either a
/// `ToStringFunc` or a `T` that overrides `ToString()`).
pub fn display_text_of<T>(
    value: &T,
    tostring_func: Option<&dyn Fn(&T) -> String>,
) -> String {
    tostring_func
        .map(|f| f(value))
        .unwrap_or_else(|| "?".to_owned())
}

/// Advances the keyboard focus to the next candidate, clamped at `len - 1`.
/// Returns `None` for an empty list.
pub fn next_focused_index(current: Option<usize>, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(match current {
        Some(i) => (i + 1).min(len - 1),
        None => 0,
    })
}

/// Moves the keyboard focus to the previous candidate, clamped at `0`.
/// Returns `None` for an empty list.
pub fn prev_focused_index(current: Option<usize>, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(match current {
        Some(i) => i.saturating_sub(1),
        None => 0,
    })
}

/// Commits `value` as the selection, sets `search_text` to `value`'s display
/// text, closes the popover, and clears the focused candidate.
///
/// This is the single source of truth for the "commit" behaviour shared by
/// candidate clicks and keyboard Enter.
pub fn commit_selection<T: Clone>(
    state: &mut AutocompleteState<T>,
    value: T,
    tostring_func: Option<&dyn Fn(&T) -> String>,
) -> AutocompleteAction<T> {
    state.search_text = display_text_of(&value, tostring_func);
    state.selected = Some(value.clone());
    state.popover.close();
    state.focused_candidate = None;
    AutocompleteAction::Selected { value }
}

/// Normalises a [`Position`] into `(AnchorOrigin, TransformOrigin)`. `Start` /
/// `End` are resolved against `right_to_left` first.
pub fn position_to_origins(
    position: Position,
    right_to_left: bool,
) -> (AnchorOrigin, TransformOrigin) {
    match position.resolve(right_to_left) {
        Position::Top => (AnchorOrigin::Top, TransformOrigin::BottomLeft),
        Position::Left => (AnchorOrigin::Left, TransformOrigin::CenterRight),
        Position::Right => (AnchorOrigin::Right, TransformOrigin::CenterLeft),
        _ => (AnchorOrigin::Bottom, TransformOrigin::TopLeft),
    }
}

#[inline]
fn with_alpha(c: Color32, alpha: f32) -> Color32 {
    if alpha >= 1.0 {
        return c;
    }
    if alpha <= 0.0 {
        return Color32::TRANSPARENT;
    }
    Color32::from_rgba_unmultiplied(
        c.r(),
        c.g(),
        c.b(),
        (c.a() as f32 * alpha).round() as u8,
    )
}

// =============================================================================
// Response
// =============================================================================

/// The result of one [`MudAutocomplete::show`] frame.
#[derive(Debug)]
pub struct AutocompleteResponse<T> {
    /// The trigger's response (union of edit and adornment regions).
    pub trigger: Response,
    /// Selection **after** this frame's commit, if any.
    pub selected: Option<T>,
    /// Search text **after** this frame's edits.
    pub search_text: String,
    /// What happened this frame.
    pub action: AutocompleteAction<T>,
}

// =============================================================================
// MudAutocomplete
// =============================================================================

/// A text field with a filtered dropdown of candidates (single selection).
#[must_use = "MudAutocomplete does nothing until you call `.show(ui, state)`"]
pub struct MudAutocomplete<'a, T> {
    // ── Data ───────────────────────────────────────────────────────────────
    /// Maps a candidate value to its display text (`ToStringFunc`). When
    /// absent, candidates render as `"?"` and the trigger keeps whatever the
    /// user typed on blur.
    pub tostring_func: Option<Box<dyn Fn(&T) -> String + 'a>>,

    // ── Validation (visual only — see README) ──────────────────────────────
    pub error: bool,
    pub error_text: Option<String>,
    pub required: bool,

    // ── Behaviour ──────────────────────────────────────────────────────────
    pub disabled: bool,
    pub readonly: bool,
    pub clearable: bool,
    pub open_on_focus: bool,
    pub reset_value_on_empty_text: bool,
    pub min_characters: usize,
    pub max_items: Option<usize>,

    // ── Trigger appearance ─────────────────────────────────────────────────
    pub label: Option<String>,
    pub placeholder: Option<String>,
    pub helper_text: Option<String>,
    pub variant: Variant,
    pub margin: Margin,
    pub color: Color,
    pub size: Size,
    pub dense: bool,

    // ── Adornment ──────────────────────────────────────────────────────────
    pub adornment_icon: String,
    pub adornment_color: Option<Color>,

    // ── Placement ──────────────────────────────────────────────────────────
    pub position: Option<Position>,
    pub anchor_origin: Option<AnchorOrigin>,
    pub transform_origin: Option<TransformOrigin>,
    pub right_to_left: bool,
    pub max_height: Option<f32>,
    pub popover_class: Option<String>,

    // ── Templates ──────────────────────────────────────────────────────────
    pub item_template: Option<Box<dyn FnMut(&mut Ui, &T) + 'a>>,
    pub no_items_template: Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    pub after_items_template: Option<Box<dyn FnMut(&mut Ui) + 'a>>,

    // ── Callbacks ──────────────────────────────────────────────────────────
    pub on_open: Option<Box<dyn FnMut() + 'a>>,
    pub on_close: Option<Box<dyn FnMut() + 'a>>,

    // ── Utilities ──────────────────────────────────────────────────────────
    pub base: ComponentBase,
    pub flavor: StyleFlavor,
    pub bs_style: BsStyle,
    pub tw_style: TwStyle,
    pub tw_state: TwState,
}

impl<'a, T> Default for MudAutocomplete<'a, T> {
    fn default() -> Self {
        Self {
            tostring_func: None,
            error: false,
            error_text: None,
            required: false,
            disabled: false,
            readonly: false,
            clearable: true,
            open_on_focus: true,
            reset_value_on_empty_text: false,
            min_characters: 0,
            max_items: None,
            label: None,
            placeholder: None,
            helper_text: None,
            variant: Variant::Text,
            margin: Margin::None,
            color: Color::Primary,
            size: Size::Medium,
            dense: false,
            adornment_icon: filled::SEARCH.to_owned(),
            adornment_color: None,
            position: None,
            anchor_origin: None,
            transform_origin: None,
            right_to_left: false,
            max_height: Some(DEFAULT_MAX_HEIGHT),
            popover_class: None,
            item_template: None,
            no_items_template: None,
            after_items_template: None,
            on_open: None,
            on_close: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }
}

impl<'a, T> MudAutocomplete<'a, T> {
    /// Creates an autocomplete with MudBlazor defaults.
    pub fn new() -> Self {
        Self::default()
    }

    // ── Builders ───────────────────────────────────────────────────────────

    #[inline] pub fn tostring_func(mut self, f: impl Fn(&T) -> String + 'a) -> Self { self.tostring_func = Some(Box::new(f)); self }
    #[inline] pub fn error(mut self, b: bool) -> Self { self.error = b; self }
    #[inline] pub fn error_text(mut self, s: impl Into<String>) -> Self { self.error_text = Some(s.into()); self }
    #[inline] pub fn required(mut self, b: bool) -> Self { self.required = b; self }
    #[inline] pub fn disabled(mut self, b: bool) -> Self { self.disabled = b; self }
    #[inline] pub fn readonly(mut self, b: bool) -> Self { self.readonly = b; self }
    #[inline] pub fn clearable(mut self, b: bool) -> Self { self.clearable = b; self }
    #[inline] pub fn open_on_focus(mut self, b: bool) -> Self { self.open_on_focus = b; self }
    #[inline] pub fn reset_value_on_empty_text(mut self, b: bool) -> Self { self.reset_value_on_empty_text = b; self }
    #[inline] pub fn min_characters(mut self, n: usize) -> Self { self.min_characters = n; self }
    #[inline] pub fn max_items(mut self, n: usize) -> Self { self.max_items = Some(n); self }
    #[inline] pub fn label(mut self, s: impl Into<String>) -> Self { self.label = Some(s.into()); self }
    #[inline] pub fn placeholder(mut self, s: impl Into<String>) -> Self { self.placeholder = Some(s.into()); self }
    #[inline] pub fn helper_text(mut self, s: impl Into<String>) -> Self { self.helper_text = Some(s.into()); self }
    #[inline] pub fn variant(mut self, v: Variant) -> Self { self.variant = v; self }
    #[inline] pub fn margin(mut self, m: Margin) -> Self { self.margin = m; self }
    #[inline] pub fn color(mut self, c: Color) -> Self { self.color = c; self }
    #[inline] pub fn size(mut self, s: Size) -> Self { self.size = s; self }
    #[inline] pub fn dense(mut self, b: bool) -> Self { self.dense = b; self }
    #[inline] pub fn adornment_icon(mut self, s: impl Into<String>) -> Self { self.adornment_icon = s.into(); self }
    #[inline] pub fn adornment_color(mut self, c: Color) -> Self { self.adornment_color = Some(c); self }
    #[inline] pub fn position(mut self, p: Position) -> Self { self.position = Some(p); self }
    #[inline] pub fn anchor_origin(mut self, a: AnchorOrigin) -> Self { self.anchor_origin = Some(a); self }
    #[inline] pub fn transform_origin(mut self, t: TransformOrigin) -> Self { self.transform_origin = Some(t); self }
    #[inline] pub fn right_to_left(mut self, b: bool) -> Self { self.right_to_left = b; self }
    #[inline] pub fn max_height(mut self, px: f32) -> Self { self.max_height = Some(px); self }
    #[inline] pub fn popover_class(mut self, s: impl Into<String>) -> Self { self.popover_class = Some(s.into()); self }
    #[inline] pub fn item_template(mut self, f: impl FnMut(&mut Ui, &T) + 'a) -> Self { self.item_template = Some(Box::new(f)); self }
    #[inline] pub fn no_items_template(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self { self.no_items_template = Some(Box::new(f)); self }
    #[inline] pub fn after_items_template(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self { self.after_items_template = Some(Box::new(f)); self }
    #[inline] pub fn on_open(mut self, f: impl FnMut() + 'a) -> Self { self.on_open = Some(Box::new(f)); self }
    #[inline] pub fn on_close(mut self, f: impl FnMut() + 'a) -> Self { self.on_close = Some(Box::new(f)); self }
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }
    #[inline] pub fn flavor(mut self, f: StyleFlavor) -> Self { self.flavor = f; self }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the trigger and, when open, the candidate dropdown.
    ///
    /// The caller's `state.candidates` are read (never written) — populate
    /// them from your own filter before calling `show`. See the module docs.
    pub fn show(
        mut self,
        ui: &mut Ui,
        state: &mut AutocompleteState<T>,
    ) -> AutocompleteResponse<T>
    where
        T: Clone + PartialEq,
    {
        // 1. Render the trigger. `state.search_text` is edited in place.
        let mut action: AutocompleteAction<T> = AutocompleteAction::None;
        let was_open = state.popover.open;

        let trigger_icon_color = self.adornment_color.unwrap_or(Color::Inherit);
        let mut trigger = MudTextField::new(&mut state.search_text)
            .disabled(self.disabled)
            .readonly(self.readonly)
            .error(self.error)
            .variant(self.variant)
            .margin(self.margin)
            .color(self.color)
            .size(self.size)
            .adornment_end(AdornmentEnd::icon(self.adornment_icon.clone()))
            .adornment_end_color(trigger_icon_color)
            .clear_icon(filled::CLEAR.to_owned());
        if let Some(s) = self.label.clone() {
            trigger = trigger.label(s);
        }
        if let Some(s) = self.placeholder.clone() {
            trigger = trigger.placeholder(s);
        }
        if let Some(s) = self.helper_text.clone() {
            trigger = trigger.helper_text(s);
        }
        if let Some(s) = self.error_text.clone() {
            trigger = trigger.error_text(s);
        }
        if self.clearable {
            trigger = trigger.clearable(true);
        }
        let trigger_resp = trigger.show(ui);

        // 2. Handle text-changed → reset keyboard focus.
        if state.search_text != state.last_searched_text {
            state.focused_candidate = None;
        }

        // 3. Open / close logic driven by user interaction.
        let char_count = state.search_text.chars().count();
        let meets_min_characters = char_count >= self.min_characters;

        if !self.disabled && !self.readonly {
            if trigger_resp.response.clicked() {
                state.popover.open = !state.popover.open;
            }
            if trigger_resp.has_focus && self.open_on_focus && meets_min_characters {
                state.popover.open = true;
            }
            if !meets_min_characters && state.popover.open {
                state.popover.open = false;
            }
        }

        // 4. Blur handling — coerce / reset.
        if trigger_resp.lost_focus {
            if state.search_text.is_empty() && self.reset_value_on_empty_text {
                state.selected = None;
                action = AutocompleteAction::Cleared;
            } else {
                // Revert the field to the selected item's display text.
                state.search_text = state
                    .selected
                    .as_ref()
                    .map(|v| display_text_of(v, self.tostring_func.as_deref()))
                    .unwrap_or_default();
            }
            state.popover.open = false;
            state.focused_candidate = None;
        }

        // 5. Keyboard navigation (only while the field has focus and the
        //    popover is open with candidates).
        let candidate_count = state.candidates.len();
        if trigger_resp.has_focus && state.popover.open && candidate_count > 0 {
            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                state.focused_candidate =
                    next_focused_index(state.focused_candidate, candidate_count);
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                state.focused_candidate =
                    prev_focused_index(state.focused_candidate, candidate_count);
            }
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Some(idx) = state.focused_candidate {
                    if let Some(v) = state.candidates.get(idx).cloned() {
                        action = commit_selection(state, v, self.tostring_func.as_deref());
                    }
                }
            }
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                state.popover.open = false;
                state.focused_candidate = None;
            }
        }

        // 6. Render popover.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let text_primary: Color32 = palette.text_primary.into();
        let text_secondary: Color32 = palette.text_secondary.into();
        let text_disabled: Color32 = palette.text_disabled.into();
        let hover_fill = ui.visuals().widgets.hovered.weak_bg_fill;
        let focus_fill = with_alpha(palette.primary.into(), 0.12);
        let divider: Color32 = palette.divider.into();

        let (anchor_origin, transform_origin) = match self.position {
            Some(p) => position_to_origins(p, self.right_to_left),
            None => (
                self.anchor_origin.unwrap_or(AnchorOrigin::Bottom),
                self.transform_origin.unwrap_or(TransformOrigin::TopLeft),
            ),
        };

        let dense = self.dense;
        let max_items = self.max_items;
        let max_height = self.max_height;
        let tostring = self.tostring_func.as_deref();
        let anchor_rect = trigger_resp.response.rect;

        // Size hint from candidate count.
        let visible_count = max_items
            .map(|m| m.min(state.candidates.len()))
            .unwrap_or(state.candidates.len());
        let size_hint = vec2(
            0.0,
            (visible_count as f32 * row_height(dense) + 8.0)
                .min(max_height.unwrap_or(DEFAULT_MAX_HEIGHT)),
        );

        let mut popover = MudPopover::new()
            .anchor_origin(anchor_origin)
            .transform_origin(transform_origin)
            .right_to_left(self.right_to_left)
            .match_anchor_width(true)
            .paper(true)
            .elevation(8)
            .size_hint(size_hint);
        if let Some(c) = self.popover_class.clone() {
            popover = popover.classes([c]);
        }

        // Take the templates out of `self` so we can call them while iterating
        // `state.candidates` immutably and holding `&mut self.bs_style` etc.
        let mut item_template = self.item_template.take();
        let mut no_items_template = self.no_items_template.take();
        let mut after_items_template = self.after_items_template.take();
        let base_id = state.popover.id.with("candidates");

        // Written inside the popover closure, applied after it returns.
        // `commit_selection(state, ..)` requires `&mut state` (the whole
        // struct) and cannot run while `&mut state.popover` is held by the
        // closure, so we defer the mutation.
        let mut clicked_candidate: Option<usize> = None;

        let _ = popover.show(ui, &mut state.popover, anchor_rect, |pop_ui| {
            egui::ScrollArea::vertical()
                .max_height(max_height.unwrap_or(DEFAULT_MAX_HEIGHT))
                .auto_shrink([false, false])
                .show(pop_ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;

                    let n = max_items
                        .map(|m| m.min(state.candidates.len()))
                        .unwrap_or(state.candidates.len());

                    if n == 0 {
                        if let Some(tpl) = no_items_template.as_mut() {
                            tpl(ui);
                        } else {
                            let h = row_height(dense);
                            let (rect, _) = ui.allocate_exact_size(
                                vec2(ui.available_width(), h),
                                Sense::hover(),
                            );
                            let mut child = ui.new_child(
                                UiBuilder::new().max_rect(rect).layout(
                                    Layout::left_to_right(egui::Align::Center),
                                ),
                            );
                            child.add(
                                MudText::new("No results".to_owned())
                                    .typo(Typo::Body2)
                                    .custom_color(text_disabled),
                            );
                        }
                    } else {
                        for i in 0..n {
                            let is_focused = state.focused_candidate == Some(i);
                            // Snapshot the candidate reference, but we cannot
                            // borrow `state.candidates` mutably later.
                            let clicked = render_candidate_row(
                                ui,
                                &state.candidates[i],
                                i,
                                is_focused,
                                dense,
                                tostring,
                                &mut item_template,
                                text_primary,
                                hover_fill,
                                focus_fill,
                                base_id.with(("row", i)),
                            );
                            if clicked {
                                clicked_candidate = Some(i);
                                break;
                            }
                        }
                    }

                    if let Some(tpl) = after_items_template.as_mut() {
                        ui.painter().line_segment(
                            [
                                pos2(ui.cursor().min.x, ui.cursor().min.y),
                                pos2(
                                    ui.cursor().min.x + ui.available_width(),
                                    ui.cursor().min.y,
                                ),
                            ],
                            egui::Stroke::new(1.0, divider),
                        );
                        tpl(ui);
                    }
                });
        });
        // 6b. Apply any deferred candidate click. This runs after the
        // popover closure has returned, so `state.popover` is no longer
        // mutably borrowed and `commit_selection` can take `&mut state`.
        if let Some(idx) = clicked_candidate {
            if let Some(v) = state.candidates.get(idx).cloned() {
                action = commit_selection(state, v, tostring);
            }
        }

        // 7. Open/close transition callbacks.
        let now_open = state.popover.open;
        if now_open != was_open {
            if now_open {
                if let Some(cb) = self.on_open.as_mut() {
                    cb();
                }
                if !action.changed_selection() {
                    action = AutocompleteAction::Opened;
                }
            } else {
                if let Some(cb) = self.on_close.as_mut() {
                    cb();
                }
                if !action.changed_selection() {
                    action = AutocompleteAction::Closed;
                }
            }
        }

        AutocompleteResponse {
            trigger: trigger_resp.response,
            selected: state.selected.clone(),
            search_text: state.search_text.clone(),
            action,
        }
    }
}

// =============================================================================
// Candidate row rendering
// =============================================================================

/// Renders one candidate row. Returns `true` if it was clicked.
#[allow(clippy::too_many_arguments)]
fn render_candidate_row<'a, T>(
    ui: &mut Ui,
    candidate: &T,
    _index: usize,
    is_focused: bool,
    dense: bool,
    tostring_func: Option<&dyn Fn(&T) -> String>,
    item_template: &mut Option<Box<dyn FnMut(&mut Ui, &T) + 'a>>,
    text_primary: Color32,
    hover_fill: Color32,
    focus_fill: Color32,
    id: egui::Id,
) -> bool {
    let height = row_height(dense);
    let (rect, resp) =
        ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click());
    let _ = id;

    let hovered = resp.hovered();
    let fill = if is_focused {
        focus_fill
    } else if hovered {
        hover_fill
    } else {
        Color32::TRANSPARENT
    };
    if fill != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, fill);
    }
    if hovered {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }

    let inner = Rect::from_min_max(
        pos2(rect.left() + ROW_PADDING_X, rect.top()),
        pos2(rect.right() - ROW_PADDING_X, rect.bottom()),
    );

    if let Some(tpl) = item_template.as_mut() {
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(inner)
                .layout(Layout::left_to_right(egui::Align::Center)),
        );
        child.set_clip_rect(inner);
        tpl(&mut child, candidate);
    } else {
        let text = display_text_of(candidate, tostring_func);
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(inner)
                .layout(Layout::left_to_right(egui::Align::Center)),
        );
        child.set_clip_rect(inner);
        child.add(
            MudText::new(text.clone())
                .typo(Typo::Body2)
                .custom_color(text_primary),
        );
    }

    resp.clicked()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_height_matches_density() {
        assert_eq!(row_height(false), ROW_HEIGHT);
        assert_eq!(row_height(true), DENSE_ROW_HEIGHT);
    }

    #[test]
    fn display_text_uses_tostring_when_present() {
        let f = |v: &i32| format!("N{v}");
        assert_eq!(display_text_of(&3, Some(&f)), "N3");
    }

    #[test]
    fn display_text_falls_back_to_question_mark() {
        assert_eq!(
            display_text_of(&3, None::<&dyn Fn(&i32) -> String>),
            "?"
        );
    }

    #[test]
    fn next_focused_clamps_and_handles_empty() {
        assert_eq!(next_focused_index(None, 0), None);
        assert_eq!(next_focused_index(Some(0), 0), None);
        assert_eq!(next_focused_index(None, 3), Some(0));
        assert_eq!(next_focused_index(Some(0), 3), Some(1));
        assert_eq!(next_focused_index(Some(2), 3), Some(2));
    }

    #[test]
    fn prev_focused_clamps_and_handles_empty() {
        assert_eq!(prev_focused_index(None, 0), None);
        assert_eq!(prev_focused_index(None, 3), Some(0));
        assert_eq!(prev_focused_index(Some(0), 3), Some(0));
        assert_eq!(prev_focused_index(Some(2), 3), Some(1));
    }

    #[test]
    fn commit_selection_writes_text_and_closes() {
        let mut state: AutocompleteState<i32> = AutocompleteState::new("t");
        state.open();
        state.focused_candidate = Some(0);
        let f = |v: &i32| format!("N{v}");
        let action = commit_selection(&mut state, 7, Some(&f));
        assert_eq!(state.search_text, "N7");
        assert_eq!(state.selected, Some(7));
        assert!(!state.is_open());
        assert!(state.focused_candidate.is_none());
        assert!(matches!(action, AutocompleteAction::Selected { value: 7 }));
    }

    #[test]
    fn commit_selection_uses_question_mark_without_tostring() {
        let mut state: AutocompleteState<i32> = AutocompleteState::new("t");
        commit_selection(&mut state, 7, None);
        assert_eq!(state.search_text, "?");
        assert_eq!(state.selected, Some(7));
    }

    #[test]
    fn position_defaults_to_bottom() {
        let (a, t) = position_to_origins(Position::Bottom, false);
        assert_eq!(a, AnchorOrigin::Bottom);
        assert_eq!(t, TransformOrigin::TopLeft);
    }

    #[test]
    fn position_top_flips() {
        let (a, t) = position_to_origins(Position::Top, false);
        assert_eq!(a, AnchorOrigin::Top);
        assert_eq!(t, TransformOrigin::BottomLeft);
    }

    #[test]
    fn defaults_match_mudblazor() {
        let a: MudAutocomplete<i32> = MudAutocomplete::new();
        assert!(a.tostring_func.is_none());
        assert!(!a.error && !a.required);
        assert!(!a.disabled && !a.readonly);
        assert!(a.clearable);
        assert!(a.open_on_focus);
        assert!(!a.reset_value_on_empty_text);
        assert_eq!(a.min_characters, 0);
        assert!(a.max_items.is_none());
        assert!(matches!(a.variant, Variant::Text));
        assert!(matches!(a.margin, Margin::None));
        assert!(matches!(a.color, Color::Primary));
        assert!(matches!(a.size, Size::Medium));
        assert!(!a.dense);
        assert_eq!(a.adornment_icon, filled::SEARCH);
        assert!(a.adornment_color.is_none());
        assert!(a.position.is_none());
        assert!(a.anchor_origin.is_none());
        assert!(a.transform_origin.is_none());
        assert!(!a.right_to_left);
        assert_eq!(a.max_height, Some(DEFAULT_MAX_HEIGHT));
        assert!(a.popover_class.is_none());
        assert!(a.item_template.is_none());
        assert!(a.no_items_template.is_none());
        assert!(a.after_items_template.is_none());
        assert!(a.on_open.is_none());
        assert!(a.on_close.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let a: MudAutocomplete<i32> = MudAutocomplete::new()
            .tostring_func(|v| format!("N{v}"))
            .error(true)
            .error_text("err")
            .required(true)
            .disabled(true)
            .readonly(true)
            .clearable(false)
            .open_on_focus(false)
            .reset_value_on_empty_text(true)
            .min_characters(2)
            .max_items(5)
            .label("L")
            .placeholder("P")
            .helper_text("H")
            .variant(Variant::Outlined)
            .margin(Margin::Dense)
            .color(Color::Error)
            .size(Size::Large)
            .dense(true)
            .adornment_icon("S")
            .adornment_color(Color::Warning)
            .position(Position::Top)
            .anchor_origin(AnchorOrigin::Top)
            .transform_origin(TransformOrigin::BottomLeft)
            .right_to_left(true)
            .max_height(200.0)
            .popover_class("pc")
            .item_template(|_, _| {})
            .no_items_template(|_| {})
            .after_items_template(|_| {})
            .on_open(|| {})
            .on_close(|| {})
            .classes(["cls"]);

        assert!(a.tostring_func.is_some());
        assert!(a.error && a.required);
        assert!(a.disabled && a.readonly);
        assert!(!a.clearable && !a.open_on_focus);
        assert!(a.reset_value_on_empty_text);
        assert_eq!(a.min_characters, 2);
        assert_eq!(a.max_items, Some(5));
        assert!(matches!(a.variant, Variant::Outlined));
        assert!(matches!(a.margin, Margin::Dense));
        assert!(matches!(a.color, Color::Error));
        assert!(matches!(a.size, Size::Large));
        assert!(a.dense);
        assert_eq!(a.adornment_icon, "S");
        assert_eq!(a.adornment_color, Some(Color::Warning));
        assert_eq!(a.position, Some(Position::Top));
        assert_eq!(a.anchor_origin, Some(AnchorOrigin::Top));
        assert_eq!(a.transform_origin, Some(TransformOrigin::BottomLeft));
        assert!(a.right_to_left);
        assert_eq!(a.max_height, Some(200.0));
        assert_eq!(a.popover_class.as_deref(), Some("pc"));
        assert!(a.item_template.is_some());
        assert!(a.no_items_template.is_some());
        assert!(a.after_items_template.is_some());
        assert!(a.on_open.is_some());
        assert!(a.on_close.is_some());
        assert!(a.base.has_class("cls"));
    }
}