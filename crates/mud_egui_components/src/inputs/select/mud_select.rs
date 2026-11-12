//! `MudSelect<T>` — a dropdown that shows one item at a time (or several in
//! multi-selection mode).
//!
//! Port of `MudSelect<T>.razor.cs` and `MudSelectItem<T>.razor.cs`
//! (MudBlazor 8.13.0).
//!
//! # Composition, not reimplementation
//!
//! `MudSelect` is built from two other primitives in this crate:
//!
//! | Primitive | Role |
//! |-----------|------|
//! | [`MudTextField`] with `readonly(true)` | The closed trigger. Gives `Variant`, `Margin`, `Size`, `Color`, `Label`, `Placeholder`, `HelperText`, `Error`, `ErrorText`, and the adornment chevron in one parameter sweep. This mirrors MudBlazor, whose `MudSelect` is itself a `MudTextField` in read-only mode. |
//! | [`MudPopover`] | The dropdown. Anchor attachment, flip-when-overflowing, backdrop, and Escape / outside-click dismissal are all delegated. |
//!
//! Only new code is the **list body** (rows, checkboxes, select-all) and the
//! **selection state machine** — and both are pure helpers tested without an
//! `egui::Context`.
//!
//! # MudBlazor parity — pass 1
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `Value` / `SelectedValues` | [`SelectState::selected`] (`Vec<T>`) | required |
//! | `ToStringFunc` | [`MudSelect::tostring_func`] | `None` |
//! | `Label`             | [`MudSelect::label`] | `None` |
//! | `Placeholder`       | [`MudSelect::placeholder`] | `None` |
//! | `HelperText`        | [`MudSelect::helper_text`] | `None` |
//! | `Error`             | [`MudSelect::error`] | `false` |
//! | `ErrorText`         | [`MudSelect::error_text`] | `None` |
//! | `Required`          | [`MudSelect::required`] | `false` (visual only) |
//! | `Disabled`          | [`MudSelect::disabled`] | `false` |
//! | `ReadOnly`          | [`MudSelect::readonly`] | `false` |
//! | `Clearable`         | [`MudSelect::clearable`] | `false` |
//! | `MultiSelection`    | [`MudSelect::multi_selection`] | `false` |
//! | `SelectAll`         | [`MudSelect::select_all`] | `false` |
//! | `Strict`            | [`MudSelect::strict`] | `false` (stored; see README) |
//! | `LockScroll`        | [`MudSelect::lock_scroll`] | `false` (stored; no-op) |
//! | `Variant`           | [`MudSelect::variant`] | [`Variant::Text`] |
//! | `Margin`            | [`MudSelect::margin`] | [`Margin::None`] |
//! | `Color`             | [`MudSelect::color`] | [`Color::Primary`] |
//! | `Size`              | [`MudSelect::size`] | [`Size::Medium`] |
//! | `FullWidth`         | [`MudSelect::full_width`] | `false` |
//! | `DisableUnderLine`  | [`MudSelect::disable_underline`] | `false` |
//! | `IconSize`          | [`MudSelect::icon_size`] | [`Size::Medium`] |
//! | `AdornmentColor`    | [`MudSelect::adornment_color`] | `None` |
//! | `OpenIcon`          | [`MudSelect::open_icon`] | `ARROW_DROP_DOWN` |
//! | `CloseIcon`         | [`MudSelect::close_icon`] | `ARROW_DROP_UP` |
//! | `AnchorOrigin`      | [`MudSelect::anchor_origin`] | `None` |
//! | `TransformOrigin`   | [`MudSelect::transform_origin`] | `None` |
//! | `Position`          | [`MudSelect::position`] | `None` |
//! | `MaxHeight`         | [`MudSelect::max_height`] | `300.0` |
//! | `PopoverClass`      | [`MudSelect::popover_class`] | `None` |
//! | `Dense`             | [`MudSelect::dense`] | `false` |
//! | `OnOpen`            | [`MudSelect::on_open`] | `None` |
//! | `OnClose`           | [`MudSelect::on_close`] | `None` |
//! | `OnClearButtonClick` | [`MudSelect::on_clear_button_click`] | `None` |
//! | `SelectedValuesChanged` | [`MudSelect::on_selected_values_changed`] | `None` |
//! | `ChildContent`      | the `add_items` argument of [`MudSelect::show`] | required |
//!
//! # MudBlazor parity — `MudSelectItem`
//!
//! See [`SelectItem`] for the full table.
//!
//! # Extensions (not in MudBlazor)
//!
//! | Extension | Description |
//! |-----------|-------------|
//! | [`SelectState`] | Caller-owned selection + popover state. |
//! | [`SelectAction`] | Rich return value reporting what happened this frame. |
//! | [`SelectResponse`] | The frame's `trigger` response, resulting `selected` vec, and `action`. |
//!
//! # Deferred to pass 2
//!
//! The following MudBlazor parameters are deliberately not implemented in
//! pass 1 (see the README's deferral table for the full list):
//!
//! `Counter`, `MaxLength`, `Pattern`, `For`, `Validation`,
//! `OnlyValidateIfDirty`, `RequiredError`, `Converter`, `Culture`, `Format`,
//! `Delimiter`, `MultiSelectionTextFunc`, `AutoFocus`, `Immediate`,
//! `InputMode`, `KeyDownPreventDefault`, `KeyUpPreventDefault`,
//! `TextUpdateSuppression`, `HelperTextOnFocus`, `ShrinkLabel`,
//! `InputClass`, `OuterClass`, `CheckedIcon`, `IndeterminateIcon`,
//! `ListClass`, `AdornmentAriaLabel`, `SelectRangeAsync`, `SelectOption`.
//!
//! # Not ported
//!
//! * **`Class`** (base) — accepted via [`MudSelect::classes`] on the trigger.
//! * **`Style` / `Tag` / `UserAttributes`** — no CSS strings or HTML tags in
//!   egui.
//!
//! # Text and icons
//!
//! The trigger's label, placeholder, helper text, error text, and text-form
//! adornment are rendered through [`MudText`] (via `MudTextField`). List rows
//! render text through [`MudText`], and icons plus checkbox glyphs through
//! [`MudIcon`].
//!
//! # Example
//!
//! ```ignore
//! let mut state = SelectState::<&str>::new("fruit").with_selected(["apple"]);
//!
//! let (resp, _) = MudSelect::new()
//!     .label("Fruit")
//!     .variant(Variant::Outlined)
//!     .show(ui, &mut state, |s| {
//!         s.text_item("apple", "Apple");
//!         s.text_item("banana", "Banana");
//!         s.text_item("cherry", "Cherry");
//!     });
//!
//! if resp.action.changed_selection() {
//!     log::info!("selection: {:?}", resp.selected);
//! }
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

use crate::data_display::icons::mud_icon::MudIcon;
use crate::inputs::text_field::{AdornmentEnd, MudTextField};
use crate::feedback::popover::MudPopover;
use crate::typography::mud_text::MudText;

use super::select_action::SelectAction;
use super::select_item::SelectItem;
use super::select_state::{SelectState, SelectUi};

// =============================================================================
// Constants
// =============================================================================

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Height of one list row (normal density), in logical pixels.
pub const ROW_HEIGHT: f32 = 40.0;

/// Height of one list row (dense), in logical pixels.
pub const DENSE_ROW_HEIGHT: f32 = 32.0;

/// Horizontal padding inside a list row, in logical pixels.
pub const ROW_PADDING_X: f32 = 12.0;

/// Gap between a leading icon and its text, in logical pixels.
const ICON_GAP: f32 = 8.0;

/// Gap between the leading checkbox and the row content, in logical pixels.
const CHECKBOX_GAP: f32 = 8.0;

/// Default maximum dropdown height (matches MudBlazor's `MaxHeight = 300`).
pub const DEFAULT_MAX_HEIGHT: f32 = 300.0;

/// Delimiter between multiple selected values in the trigger.
pub const MULTI_DELIMITER: &str = ", ";

// =============================================================================
// Pure helpers
// =============================================================================

/// Row height for the given density.
///
/// | `dense` | Height |
/// |---------|--------|
/// | `false` | [`ROW_HEIGHT`] (40 px) |
/// | `true`  | [`DENSE_ROW_HEIGHT`] (32 px) |
#[inline]
pub fn row_height(dense: bool) -> f32 {
    if dense { DENSE_ROW_HEIGHT } else { ROW_HEIGHT }
}

/// Resolves the trigger's adornment glyph: the close glyph when the popover
/// is open, the open glyph when closed.
///
/// The return lifetime is tied to both glyph arguments (they come from
/// `self.open_icon` / `self.close_icon`, which share the caller's
/// lifetime).
#[inline]
pub fn trigger_icon<'a>(
    state_open: bool,
    open_icon: &'a str,
    close_icon: &'a str,
) -> &'a str {
    if state_open { close_icon } else { open_icon }
}

/// Looks up the display text of `value` among `items`, if a matching item
/// exists.
///
/// This is how MudBlazor prefers an item's `Text` over the select's
/// `ToStringFunc` for the trigger display. The return lifetime is tied to
/// `items` — the only source of `&str`.
pub fn item_text_for<'a, T: PartialEq>(
    value: &T,
    items: &'a [SelectItem<T>],
) -> Option<&'a str> {
    items
        .iter()
        .find(|i| &i.value == value)
        .and_then(|i| i.text.as_deref())
}

/// Computes the trigger's display text for the current selection.
///
/// Per selected value the precedence is:
///
/// 1. The matching item's `text`.
/// 2. `tostring(&value)` if a `tostring` closure is provided.
/// 3. `"?"` — a visible marker that the value was bound but not rendered.
///
/// Multiple selections are joined with [`MULTI_DELIMITER`].
///
/// `tostring` is generic over any closure-like value (`Fn`, `dyn Fn`, a
/// captured closure, …) so callers can pass `Some(&closure)`,
/// `Some(boxed.as_ref())` (a trait object), or `None`. The `?Sized` bound
/// is what allows `&dyn Fn(..)` — the `Box<dyn Fn>` field on [`MudSelect`]
/// dereferences to one, and it is the lifetime-plumbing that lets the
/// method work with any closure lifetime rather than only `'static`.
pub fn compute_display_text<T, F>(
    selected: &[T],
    items: &[SelectItem<T>],
    tostring: Option<&F>,
) -> String
where
    T: PartialEq,
    F: Fn(&T) -> String + ?Sized,
{
    if selected.is_empty() {
        return String::new();
    }
    selected
        .iter()
        .map(|v| {
            item_text_for(v, items)
                .map(|s| s.to_owned())
                .or_else(|| tostring.map(|f| f(v)))
                .unwrap_or_else(|| "?".to_owned())
        })
        .collect::<Vec<_>>()
        .join(MULTI_DELIMITER)
}

/// Applies one click on `value` to `selected`, honouring `multi`.
///
/// | Mode | Unselected item | Selected item |
/// |------|-----------------|---------------|
/// | single (`multi = false`) | replace selection | no-op |
/// | multi (`multi = true`)   | append | remove |
///
/// Returns the action taken, or [`SelectAction::None`] when nothing changed.
pub fn apply_selection<T: Clone + PartialEq>(
    selected: &mut Vec<T>,
    value: &T,
    multi: bool,
) -> SelectAction<T> {
    if let Some(pos) = selected.iter().position(|v| v == value) {
        if multi {
            selected.remove(pos);
            SelectAction::Deselected { value: value.clone() }
        } else {
            SelectAction::None
        }
    } else if multi {
        selected.push(value.clone());
        SelectAction::Selected {
            value: value.clone(),
            multi: true,
        }
    } else {
        selected.clear();
        selected.push(value.clone());
        SelectAction::Selected {
            value: value.clone(),
            multi: false,
        }
    }
}

/// Selects or deselects every enabled item in one shot.
///
/// * `select == true` — every non-disabled item's value is added to
///   `selected` if not already present.
/// * `select == false` — `selected` is cleared entirely.
///
/// Disabled items are always skipped. Returns [`SelectAction::SelectAll`].
pub fn apply_select_all<T: Clone + PartialEq>(
    selected: &mut Vec<T>,
    items: &[SelectItem<T>],
    select: bool,
) -> SelectAction<T> {
    if select {
        for item in items.iter().filter(|i| !i.disabled) {
            if !selected.iter().any(|v| v == &item.value) {
                selected.push(item.value.clone());
            }
        }
    } else {
        selected.clear();
    }
    SelectAction::SelectAll { select }
}

/// Whether every enabled item is selected (drives the tri-state select-all
/// checkbox). Disabled items are ignored.
///
/// Returns `false` for an empty item list — the "all" state of "no items"
/// would be vacuous, and the checkbox should show "unchecked" rather than
/// "checked" in that case.
pub fn all_enabled_selected<T: PartialEq>(selected: &[T], items: &[SelectItem<T>]) -> bool {
    let enabled: Vec<&SelectItem<T>> = items.iter().filter(|i| !i.disabled).collect();
    !enabled.is_empty()
        && enabled
        .iter()
        .all(|i| selected.iter().any(|v| v == &i.value))
}

/// Whether some (but not all) enabled items are selected.
pub fn some_enabled_selected<T: PartialEq>(selected: &[T], items: &[SelectItem<T>]) -> bool {
    items
        .iter()
        .filter(|i| !i.disabled)
        .any(|i| selected.iter().any(|v| v == &i.value))
}

/// The glyph to use for the select-all tri-state checkbox.
///
/// | `all` | `some` | Glyph |
/// |-------|--------|-------|
/// | `true`  | —   | `CHECK_BOX` |
/// | `false` | `true`  | `INDETERMINATE_CHECK_BOX` |
/// | `false` | `false` | `CHECK_BOX_OUTLINE_BLANK` |
#[inline]
pub fn select_all_icon(all: bool, some: bool) -> &'static str {
    if all {
        filled::CHECK_BOX
    } else if some {
        filled::INDETERMINATE_CHECK_BOX
    } else {
        filled::CHECK_BOX_OUTLINE_BLANK
    }
}

/// The glyph to use for a per-item checkbox.
#[inline]
pub fn item_checkbox_icon(selected: bool) -> &'static str {
    if selected {
        filled::CHECK_BOX
    } else {
        filled::CHECK_BOX_OUTLINE_BLANK
    }
}

/// Multiplies the alpha channel of `c` by `alpha`.
///
/// * `alpha >= 1.0` → `c` unchanged.
/// * `alpha <= 0.0` → [`Color32::TRANSPARENT`].
/// * Otherwise the result is `c`'s RGB with `c.a() * alpha` as its new
///   alpha.
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

/// Normalises a [`Position`] into an `(AnchorOrigin, TransformOrigin)` pair.
///
/// `Position::Start` / `End` are resolved against `right_to_left` before the
/// mapping; physical variants pass through.
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

// =============================================================================
// Response
// =============================================================================

/// The result of one [`MudSelect::show`] frame.
///
/// `trigger` is the union of the `MudTextField` trigger's internal regions
/// (edit area, adornment, clear button), so `.clicked()` reflects a click
/// on any part of the closed dropdown. `selected` is the **post-click**
/// selection state, so callers can inspect it without comparing before and
/// after. `action` reports what changed this frame.
#[derive(Debug)]
pub struct SelectResponse<T> {
    /// The trigger's response (see [`MudTextField::show`]).
    pub trigger: Response,
    /// The selection state **after** this frame's click was applied.
    pub selected: Vec<T>,
    /// What happened this frame.
    pub action: SelectAction<T>,
}

// =============================================================================
// MudSelect
// =============================================================================

/// A dropdown that shows one item at a time (or several in multi mode).
///
/// Constructed with [`MudSelect::new`] and configured through the fluent
/// builder. Selection lives in [`SelectState`], which the caller persists
/// across frames — the same caller-owned-state pattern used by `MudTable`,
/// `MudTreeView`, `MudChipSet`, `MudNavMenu`, `MudPagination`, `MudStepper`,
/// `MudTabs`, and `MudMenu`.
#[must_use = "MudSelect does nothing until you call `.show(ui, state, |s| ..)`"]
pub struct MudSelect<'a, T> {
    // ── Data ───────────────────────────────────────────────────────────────
    /// Maps a value to display text when an item has no `text` and for the
    /// trigger when the selected value has no matching item
    /// (`ToStringFunc`).
    pub tostring_func: Option<Box<dyn Fn(&T) -> String + 'a>>,

    // ── Validation (visual only — see README) ──────────────────────────────
    /// Switches the trigger accent to `palette.error` (`Error`).
    pub error: bool,
    /// Error text shown in place of the helper text when `error` is `true`
    /// (`ErrorText`).
    pub error_text: Option<String>,
    /// Marks the field as required visually (`Required`). Pass 1 ships
    /// visual-only — no form-level validation is performed.
    pub required: bool,

    // ── Behaviour ──────────────────────────────────────────────────────────
    /// Disables the trigger and every list item (`Disabled`).
    pub disabled: bool,
    /// Prevents opening the dropdown (`ReadOnly`).
    pub readonly: bool,
    /// Shows a clear button on the trigger in single-selection mode
    /// (`Clearable`).
    pub clearable: bool,
    /// Enables multi-value selection (`MultiSelection`).
    pub multi_selection: bool,
    /// Renders a tri-state "select all" row above the item list in
    /// multi-selection mode (`SelectAll`).
    pub select_all: bool,
    /// Stored for parity (`Strict`). Pass 1 does not enforce it — see the
    /// README's differences section.
    pub strict: bool,
    /// Stored for parity (`LockScroll`). egui `Area`s are non-scrolling, so
    /// this is a no-op.
    pub lock_scroll: bool,

    // ── Trigger appearance ─────────────────────────────────────────────────
    /// Caption above the trigger (`Label`).
    pub label: Option<String>,
    /// Placeholder shown in the trigger when nothing is selected
    /// (`Placeholder`).
    pub placeholder: Option<String>,
    /// Helper text below the trigger (`HelperText`).
    pub helper_text: Option<String>,
    /// Trigger visual variant (`Variant`).
    pub variant: Variant,
    /// Trigger vertical density (`Margin`).
    pub margin: Margin,
    /// Trigger focus accent colour (`Color`).
    pub color: Color,
    /// Trigger size preset (`Size`).
    pub size: Size,
    /// Stretches the trigger to the available width (`FullWidth`). The
    /// dropdown always matches the trigger width regardless of this flag
    /// (MudBlazor's `MudSelect` behaves the same way for the popover).
    pub full_width: bool,
    /// Hides the underline on `Text` / `Filled` variants (`DisableUnderLine`).
    pub disable_underline: bool,
    /// Icon size preset (`IconSize`).
    pub icon_size: Size,

    // ── Adornment / open-close icons ───────────────────────────────────────
    /// Overrides the trigger chevron colour (`AdornmentColor`).
    pub adornment_color: Option<Color>,
    /// Glyph shown while the dropdown is closed (`OpenIcon`). Defaults to
    /// `ARROW_DROP_DOWN`.
    pub open_icon: String,
    /// Glyph shown while the dropdown is open (`CloseIcon`). Defaults to
    /// `ARROW_DROP_UP`.
    pub close_icon: String,

    // ── Placement ──────────────────────────────────────────────────────────
    /// Coarse placement helper (`Position`). When set, overrides
    /// `anchor_origin` and `transform_origin`.
    pub position: Option<Position>,
    /// Attach edge (`AnchorOrigin`). Ignored when `position` is set.
    pub anchor_origin: Option<AnchorOrigin>,
    /// Transform corner (`TransformOrigin`). Ignored when `position` is set.
    pub transform_origin: Option<TransformOrigin>,
    /// RTL flag for `Position::Start` / `End`.
    pub right_to_left: bool,
    /// Max height for the item list (`MaxHeight`). Defaults to `300.0`.
    pub max_height: Option<f32>,
    /// Extra classes on the popover paper (`PopoverClass`).
    pub popover_class: Option<String>,

    // ── List appearance ────────────────────────────────────────────────────
    /// Compact list rows (`Dense`).
    pub dense: bool,

    // ── Callbacks ──────────────────────────────────────────────────────────
    /// Fires when the dropdown opens (`OnOpen`).
    pub on_open: Option<Box<dyn FnMut() + 'a>>,
    /// Fires when the dropdown closes (`OnClose`).
    pub on_close: Option<Box<dyn FnMut() + 'a>>,
    /// Fires when the clear button is pressed (`OnClearButtonClick`).
    pub on_clear_button_click: Option<Box<dyn FnMut() + 'a>>,
    /// Fires after any selection change with the new selection
    /// (`SelectedValuesChanged`).
    pub on_selected_values_changed: Option<Box<dyn FnMut(&[T]) + 'a>>,

    // ── Utilities ──────────────────────────────────────────────────────────
    /// Utility classes on the trigger (`Class`).
    pub base: ComponentBase,
    /// Bootstrap or Tailwind resolver.
    pub flavor: StyleFlavor,
    /// Bootstrap resolver state.
    pub bs_style: BsStyle,
    /// Tailwind resolver state.
    pub tw_style: TwStyle,
    /// Tailwind interaction state.
    pub tw_state: TwState,
}

impl<'a, T> Default for MudSelect<'a, T> {
    fn default() -> Self {
        Self {
            tostring_func: None,
            error: false,
            error_text: None,
            required: false,
            disabled: false,
            readonly: false,
            clearable: false,
            multi_selection: false,
            select_all: false,
            strict: false,
            lock_scroll: false,
            label: None,
            placeholder: None,
            helper_text: None,
            variant: Variant::Text,
            margin: Margin::None,
            color: Color::Primary,
            size: Size::Medium,
            full_width: false,
            disable_underline: false,
            icon_size: Size::Medium,
            adornment_color: None,
            open_icon: filled::ARROW_DROP_DOWN.to_owned(),
            close_icon: filled::ARROW_DROP_UP.to_owned(),
            position: None,
            anchor_origin: None,
            transform_origin: None,
            right_to_left: false,
            max_height: Some(DEFAULT_MAX_HEIGHT),
            popover_class: None,
            dense: false,
            on_open: None,
            on_close: None,
            on_clear_button_click: None,
            on_selected_values_changed: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }
}

impl<'a, T> MudSelect<'a, T> {
    /// Creates a select with MudBlazor defaults.
    pub fn new() -> Self {
        Self::default()
    }

    // ── Data builders ──────────────────────────────────────────────────────

    /// Sets the value-to-display-text mapping (`ToStringFunc`).
    ///
    /// Only consulted when a selected value has no matching item with a
    /// `text` set — item text wins. See
    /// [`compute_display_text`] for the full precedence.
    #[inline]
    pub fn tostring_func(mut self, f: impl Fn(&T) -> String + 'a) -> Self {
        self.tostring_func = Some(Box::new(f));
        self
    }

    // ── Validation builders ────────────────────────────────────────────────

    /// Switches the trigger accent to `palette.error` (`Error`).
    #[inline]
    pub fn error(mut self, b: bool) -> Self {
        self.error = b;
        self
    }

    /// Sets the error text shown in place of the helper text when
    /// [`error`](Self::error) is `true` (`ErrorText`).
    #[inline]
    pub fn error_text(mut self, s: impl Into<String>) -> Self {
        self.error_text = Some(s.into());
        self
    }

    /// Marks the field as visually required (`Required`). Pass 1 does not
    /// perform form validation.
    #[inline]
    pub fn required(mut self, b: bool) -> Self {
        self.required = b;
        self
    }

    // ── Behaviour builders ─────────────────────────────────────────────────

    /// Disables the trigger and every list item (`Disabled`).
    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// Prevents opening the dropdown (`ReadOnly`).
    #[inline]
    pub fn readonly(mut self, b: bool) -> Self {
        self.readonly = b;
        self
    }

    /// Shows a clear button on the trigger in single-selection mode
    /// (`Clearable`).
    #[inline]
    pub fn clearable(mut self, b: bool) -> Self {
        self.clearable = b;
        self
    }

    /// Enables multi-value selection (`MultiSelection`).
    #[inline]
    pub fn multi_selection(mut self, b: bool) -> Self {
        self.multi_selection = b;
        self
    }

    /// Renders a tri-state "select all" row above the item list in
    /// multi-selection mode (`SelectAll`).
    #[inline]
    pub fn select_all(mut self, b: bool) -> Self {
        self.select_all = b;
        self
    }

    /// Stored for parity (`Strict`). See README.
    #[inline]
    pub fn strict(mut self, b: bool) -> Self {
        self.strict = b;
        self
    }

    /// Stored for parity (`LockScroll`). No-op in egui.
    #[inline]
    pub fn lock_scroll(mut self, b: bool) -> Self {
        self.lock_scroll = b;
        self
    }

    // ── Trigger appearance builders ────────────────────────────────────────

    /// Caption above the trigger (`Label`).
    #[inline]
    pub fn label(mut self, s: impl Into<String>) -> Self {
        self.label = Some(s.into());
        self
    }

    /// Placeholder shown in the trigger when nothing is selected
    /// (`Placeholder`).
    #[inline]
    pub fn placeholder(mut self, s: impl Into<String>) -> Self {
        self.placeholder = Some(s.into());
        self
    }

    /// Helper text below the trigger (`HelperText`).
    #[inline]
    pub fn helper_text(mut self, s: impl Into<String>) -> Self {
        self.helper_text = Some(s.into());
        self
    }

    /// Trigger visual variant (`Variant`).
    #[inline]
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    /// Trigger vertical density (`Margin`).
    #[inline]
    pub fn margin(mut self, m: Margin) -> Self {
        self.margin = m;
        self
    }

    /// Trigger focus accent colour (`Color`).
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Trigger size preset (`Size`).
    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    /// Stretches the trigger to the available width (`FullWidth`).
    #[inline]
    pub fn full_width(mut self, b: bool) -> Self {
        self.full_width = b;
        self
    }

    /// Hides the underline on `Text` / `Filled` variants (`DisableUnderLine`).
    #[inline]
    pub fn disable_underline(mut self, b: bool) -> Self {
        self.disable_underline = b;
        self
    }

    /// Icon size preset (`IconSize`).
    #[inline]
    pub fn icon_size(mut self, s: Size) -> Self {
        self.icon_size = s;
        self
    }

    // ── Adornment / open-close icon builders ───────────────────────────────

    /// Overrides the trigger chevron colour (`AdornmentColor`).
    #[inline]
    pub fn adornment_color(mut self, c: Color) -> Self {
        self.adornment_color = Some(c);
        self
    }

    /// Glyph shown while the dropdown is closed (`OpenIcon`). Defaults to
    /// `ARROW_DROP_DOWN`.
    #[inline]
    pub fn open_icon(mut self, s: impl Into<String>) -> Self {
        self.open_icon = s.into();
        self
    }

    /// Glyph shown while the dropdown is open (`CloseIcon`). Defaults to
    /// `ARROW_DROP_UP`.
    #[inline]
    pub fn close_icon(mut self, s: impl Into<String>) -> Self {
        self.close_icon = s.into();
        self
    }

    // ── Placement builders ─────────────────────────────────────────────────

    /// Coarse placement helper (`Position`). Overrides `anchor_origin` and
    /// `transform_origin`.
    #[inline]
    pub fn position(mut self, p: Position) -> Self {
        self.position = Some(p);
        self
    }

    /// Attach edge (`AnchorOrigin`). Ignored when [`position`](Self::position)
    /// is set.
    #[inline]
    pub fn anchor_origin(mut self, a: AnchorOrigin) -> Self {
        self.anchor_origin = Some(a);
        self
    }

    /// Transform corner (`TransformOrigin`). Ignored when
    /// [`position`](Self::position) is set.
    #[inline]
    pub fn transform_origin(mut self, t: TransformOrigin) -> Self {
        self.transform_origin = Some(t);
        self
    }

    /// RTL flag for `Position::Start` / `End`.
    #[inline]
    pub fn right_to_left(mut self, b: bool) -> Self {
        self.right_to_left = b;
        self
    }

    /// Max height for the item list (`MaxHeight`).
    #[inline]
    pub fn max_height(mut self, px: f32) -> Self {
        self.max_height = Some(px);
        self
    }

    /// Extra classes on the popover paper (`PopoverClass`).
    #[inline]
    pub fn popover_class(mut self, s: impl Into<String>) -> Self {
        self.popover_class = Some(s.into());
        self
    }

    // ── List appearance builders ───────────────────────────────────────────

    /// Compact list rows (`Dense`).
    #[inline]
    pub fn dense(mut self, b: bool) -> Self {
        self.dense = b;
        self
    }

    // ── Callback builders ──────────────────────────────────────────────────

    /// Fires when the dropdown opens (`OnOpen`).
    #[inline]
    pub fn on_open(mut self, f: impl FnMut() + 'a) -> Self {
        self.on_open = Some(Box::new(f));
        self
    }

    /// Fires when the dropdown closes (`OnClose`).
    #[inline]
    pub fn on_close(mut self, f: impl FnMut() + 'a) -> Self {
        self.on_close = Some(Box::new(f));
        self
    }

    /// Fires when the clear button is pressed (`OnClearButtonClick`).
    #[inline]
    pub fn on_clear_button_click(mut self, f: impl FnMut() + 'a) -> Self {
        self.on_clear_button_click = Some(Box::new(f));
        self
    }

    /// Fires after any selection change with the new selection
    /// (`SelectedValuesChanged`).
    #[inline]
    pub fn on_selected_values_changed(mut self, f: impl FnMut(&[T]) + 'a) -> Self {
        self.on_selected_values_changed = Some(Box::new(f));
        self
    }

    // ── Utility builders ───────────────────────────────────────────────────

    /// Adds utility classes to the trigger (`Class`).
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

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the trigger and, when open, the dropdown.
    ///
    /// `'a` on the impl block constrains the item closures; the item
    /// closures themselves are supplied per-call via `add_items` so the
    /// method signature does not need a fresh lifetime parameter.
    ///
    /// Returns `(SelectResponse<T>, R)` where `R` is whatever `add_items`
    /// returned.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        state: &mut SelectState<T>,
        add_items: impl FnOnce(&mut SelectUi<T>) -> R,
    ) -> (SelectResponse<T>, R)
    where
        T: Clone + PartialEq,
    {
        // 1. Collect items.
        let mut collector = SelectUi { items: Vec::new() };
        let user_result = add_items(&mut collector);
        let mut items = collector.items;

        // 2. Compute the trigger's display string.
        //
        // `as_deref()` gives `Option<&(dyn Fn(&T) -> String + 'a)>`. The
        // generic `F: Fn(..) + ?Sized` bound on `compute_display_text`
        // accepts this without forcing a `'static` coercion.
        let mut display = compute_display_text(
            &state.selected,
            &items,
            self.tostring_func.as_deref(),
        );

        // 3. Resolve trigger config.
        let mut action: SelectAction<T> = SelectAction::None;
        let icon_str = trigger_icon(state.popover.open, &self.open_icon, &self.close_icon);

        // 4. Trigger — a MudTextField in readonly mode.
        let trigger_icon_color = self.adornment_color.unwrap_or(Color::Inherit);
        let mut trigger = MudTextField::new(&mut display)
            .readonly(true)
            .disabled(self.disabled)
            .error(self.error)
            .variant(self.variant)
            .margin(self.margin)
            .color(self.color)
            .size(self.size)
            .adornment_end(AdornmentEnd::icon(icon_str.to_owned()))
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
        if self.clearable && !self.multi_selection {
            trigger = trigger.clearable(true);
        }
        let trigger_resp = trigger.show(ui);

        // 5. Toggle on trigger click.
        //
        // The trigger's response is the union of the field, edit, and
        // adornment regions — so a click on the chevron toggles the dropdown
        // exactly as a click on the edit area does (see MudTextField's
        // module docs, "Click propagation").
        let was_open = state.popover.open;
        if trigger_resp.response.clicked() && !self.disabled && !self.readonly {
            state.popover.open = !state.popover.open;
        }
        if trigger_resp.response.clicked() && display.is_empty() && !state.selected.is_empty() {
            if let Some(cb) = self.on_clear_button_click.as_mut() {
                cb();
            }
            state.selected.clear();
            action = SelectAction::Cleared;
        }
        let now_open = state.popover.open;
        if now_open != was_open {
            if now_open {
                if let Some(cb) = self.on_open.as_mut() {
                    cb();
                }
                action = SelectAction::Opened;
            } else {
                if let Some(cb) = self.on_close.as_mut() {
                    cb();
                }
                action = SelectAction::Closed;
            }
        }

        // 6. Dropdown — a MudPopover with the item list.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let text_primary: Color32 = palette.text_primary.into();
        let text_secondary: Color32 = palette.text_secondary.into();
        let text_disabled: Color32 = palette.text_disabled.into();
        let hover_fill = ui.visuals().widgets.hovered.weak_bg_fill;
        let accent: Color32 = palette.primary.into();
        let checkbox_color: Color32 = if self.error {
            palette.error.into()
        } else {
            accent
        };
        let divider: Color32 = palette.divider.into();

        let (anchor_origin, transform_origin) = match self.position {
            Some(p) => position_to_origins(p, self.right_to_left),
            None => (
                self.anchor_origin.unwrap_or(AnchorOrigin::Bottom),
                self.transform_origin.unwrap_or(TransformOrigin::TopLeft),
            ),
        };

        let mut popover = MudPopover::new()
            .anchor_origin(anchor_origin)
            .transform_origin(transform_origin)
            .right_to_left(self.right_to_left)
            .match_anchor_width(true)
            .paper(true)
            .elevation(8);
        if let Some(mh) = self.max_height {
            popover = popover.size_hint(vec2(0.0, mh.min(200.0)));
        }
        if let Some(c) = self.popover_class.clone() {
            popover = popover.classes([c]);
        }

        let base_id = state.popover.id.with("items");
        let dense = self.dense;
        let multi = self.multi_selection;
        let show_select_all = self.select_all && self.multi_selection;
        let max_height = self.max_height;
        let mut selected_snapshot = state.selected.clone();
        let mut pick: Option<SelectAction<T>> = None;
        let mut close_after_pick = false;

        let _ = popover.show(ui, &mut state.popover, trigger_resp.response.rect, |pop_ui| {
            egui::ScrollArea::vertical()
                .max_height(max_height.unwrap_or(DEFAULT_MAX_HEIGHT))
                .auto_shrink([false, false])
                .show(pop_ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;

                    // Select-all row (multi mode only).
                    if show_select_all {
                        let all = all_enabled_selected(&selected_snapshot, &items);
                        let some = some_enabled_selected(&selected_snapshot, &items);
                        let glyph = select_all_icon(all, some);
                        let resp = render_select_all_row(
                            ui,
                            glyph,
                            checkbox_color,
                            text_primary,
                            text_secondary,
                            hover_fill,
                        );
                        if resp {
                            let act = apply_select_all(&mut selected_snapshot, &items, !all);
                            pick = Some(act);
                        }
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
                    }

                    // Item rows.
                    for (i, item) in items.iter_mut().enumerate() {
                        let is_sel = selected_snapshot.iter().any(|v| v == &item.value);
                        let clicked = render_item_row(
                            ui,
                            item,
                            &base_id.with(("row", i)),
                            is_sel,
                            dense,
                            multi,
                            checkbox_color,
                            text_primary,
                            text_secondary,
                            text_disabled,
                            hover_fill,
                        );
                        if clicked && !item.disabled {
                            let act =
                                apply_selection(&mut selected_snapshot, &item.value, multi);
                            if !matches!(act, SelectAction::None) {
                                pick = Some(act);
                                close_after_pick = !multi;
                            }
                        }
                    }
                });
        });

        // 7. Commit the pick.
        if let Some(act) = pick {
            state.selected = selected_snapshot;
            if let Some(cb) = self.on_selected_values_changed.as_mut() {
                cb(&state.selected);
            }
            action = act;
            if close_after_pick {
                state.popover.open = false;
                if let Some(cb) = self.on_close.as_mut() {
                    cb();
                }
            }
        }

        (
            SelectResponse {
                trigger: trigger_resp.response,
                selected: state.selected.clone(),
                action,
            },
            user_result,
        )
    }
}

// =============================================================================
// Row rendering helpers
// =============================================================================

/// Renders one item row and returns whether it was clicked.
///
/// Takes `item` by `&mut` so it can `take()` the `child` closure before
/// creating an immutable borrow of the item's `text` / `icon` fields.
#[allow(clippy::too_many_arguments)]
fn render_item_row<T>(
    ui: &mut Ui,
    item: &mut SelectItem<T>,
    id: &egui::Id,
    is_selected: bool,
    dense: bool,
    multi: bool,
    checkbox_color: Color32,
    text_primary: Color32,
    text_secondary: Color32,
    text_disabled: Color32,
    hover_fill: Color32,
) -> bool {
    let height = row_height(dense);
    let (rect, resp) = ui.allocate_exact_size(
        vec2(ui.available_width(), height),
        if item.disabled {
            Sense::hover()
        } else {
            Sense::click()
        },
    );
    let _ = id;

    let hovered = resp.hovered() && !item.disabled;
    if hovered {
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, hover_fill);
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }

    let text_color = if item.disabled {
        text_disabled
    } else if is_selected {
        checkbox_color
    } else {
        text_primary
    };

    let mut child_opt = item.child.take();
    let icon_opt = item.icon.clone();
    let text_opt = item.text.clone();
    let icon_px = (height * 0.5).min(20.0);
    let checkbox_px = (height * 0.42).max(16.0);

    let mut x = rect.left() + ROW_PADDING_X;
    let y_center = rect.center().y;

    // Checkbox (multi mode only).
    if multi {
        let cb_rect = Rect::from_center_size(
            pos2(x + checkbox_px * 0.5, y_center),
            vec2(checkbox_px, checkbox_px),
        );
        ui.put(
            cb_rect,
            MudIcon::new(item_checkbox_icon(is_selected).to_owned())
                .size_px(checkbox_px)
                .custom_color(if item.disabled {
                    text_disabled
                } else {
                    checkbox_color
                }),
        );
        x += checkbox_px + CHECKBOX_GAP;
    }

    // Leading icon.
    if let Some(icon) = icon_opt {
        let icon_rect = Rect::from_center_size(
            pos2(x + icon_px * 0.5, y_center),
            vec2(icon_px, icon_px),
        );
        ui.put(
            icon_rect,
            MudIcon::new(icon).size_px(icon_px).custom_color(if item.disabled {
                text_disabled
            } else {
                text_secondary
            }),
        );
        x += icon_px + ICON_GAP;
    }

    // Text or custom content.
    let content_rect = Rect::from_min_max(
        pos2(x, rect.top()),
        pos2(rect.right() - ROW_PADDING_X, rect.bottom()),
    );
    if let Some(child) = child_opt.as_mut() {
        let mut child_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(content_rect)
                .layout(Layout::left_to_right(egui::Align::Center)),
        );
        child_ui.set_clip_rect(content_rect);
        child(&mut child_ui);
    } else if let Some(text) = text_opt.as_deref() {
        let mut text_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(content_rect)
                .layout(Layout::left_to_right(egui::Align::Center)),
        );
        text_ui.set_clip_rect(content_rect);
        text_ui.add(
            MudText::new(text.to_owned())
                .typo(Typo::Body2)
                .custom_color(text_color),
        );
    }

    let label = text_opt.unwrap_or_default();
    let enabled = !item.disabled;
    resp.clone().widget_info(|| {
        egui::WidgetInfo::selected(egui::Role::Button, enabled, is_selected, label.as_str())
    });

    resp.clicked() && !item.disabled
}

/// Renders the select-all row and returns whether it was clicked.
#[allow(clippy::too_many_arguments)]
fn render_select_all_row(
    ui: &mut Ui,
    glyph: &str,
    checkbox_color: Color32,
    text_primary: Color32,
    _text_secondary: Color32,
    hover_fill: Color32,
) -> bool {
    let height = row_height(false);
    let (rect, resp) =
        ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click());
    let hovered = resp.hovered();
    if hovered {
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, hover_fill);
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }

    let checkbox_px = (height * 0.42).max(16.0);
    let y_center = rect.center().y;
    let x = rect.left() + ROW_PADDING_X;

    let cb_rect = Rect::from_center_size(
        pos2(x + checkbox_px * 0.5, y_center),
        vec2(checkbox_px, checkbox_px),
    );
    ui.put(
        cb_rect,
        MudIcon::new(glyph.to_owned())
            .size_px(checkbox_px)
            .custom_color(checkbox_color),
    );

    let text_x = x + checkbox_px + CHECKBOX_GAP;
    let text_rect = Rect::from_min_max(
        pos2(text_x, rect.top()),
        pos2(rect.right() - ROW_PADDING_X, rect.bottom()),
    );
    let mut text_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(text_rect)
            .layout(Layout::left_to_right(egui::Align::Center)),
    );
    text_ui.add(
        MudText::new("Select all".to_owned())
            .typo(Typo::Body2)
            .custom_color(text_primary),
    );

    resp.clicked()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // The three `None` call sites must annotate `F` — `compute_display_text`
    // is generic over `F: Fn(..) + ?Sized`, and `None` alone gives it no way
    // to infer `F`. `None::<&dyn Fn(&i32) -> String>` pins `F` to a trait
    // object, which is what the caller (`self.tostring_func.as_deref()`)
    // also produces.

    // ── row_height ─────────────────────────────────────────────────────────

    #[test]
    fn row_height_matches_density() {
        assert_eq!(row_height(false), ROW_HEIGHT);
        assert_eq!(row_height(true), DENSE_ROW_HEIGHT);
    }

    // ── trigger_icon ───────────────────────────────────────────────────────

    #[test]
    fn trigger_icon_returns_open_or_close() {
        assert_eq!(trigger_icon(false, "O", "C"), "O");
        assert_eq!(trigger_icon(true, "O", "C"), "C");
    }

    // ── item_text_for ──────────────────────────────────────────────────────

    #[test]
    fn item_text_lookup() {
        let items = vec![SelectItem::text(1, "One"), SelectItem::new(2)];
        assert_eq!(item_text_for(&1, &items), Some("One"));
        assert_eq!(item_text_for(&2, &items), None);
        assert_eq!(item_text_for(&99, &items), None);
    }

    // ── compute_display_text ───────────────────────────────────────────────

    #[test]
    fn compute_display_empty_selection() {
        let items: Vec<SelectItem<i32>> = Vec::new();
        assert_eq!(
            compute_display_text(&[], &items, None::<&dyn Fn(&i32) -> String>),
            ""
        );
    }

    #[test]
    fn compute_display_uses_item_text_first() {
        let items = vec![SelectItem::text(1, "One")];
        let ts = |v: &i32| format!("TS:{v}");
        assert_eq!(compute_display_text(&[1], &items, Some(&ts)), "One");
    }

    #[test]
    fn compute_display_falls_back_to_tostring() {
        let items = vec![SelectItem::new(1)];
        let ts = |v: &i32| format!("TS:{v}");
        assert_eq!(compute_display_text(&[1], &items, Some(&ts)), "TS:1");
    }

    #[test]
    fn compute_display_unknown_is_question_mark() {
        let items: Vec<SelectItem<i32>> = Vec::new();
        assert_eq!(
            compute_display_text(&[1], &items, None::<&dyn Fn(&i32) -> String>),
            "?"
        );
    }

    #[test]
    fn compute_display_joins_with_delimiter() {
        let items = vec![
            SelectItem::text(1, "One"),
            SelectItem::text(2, "Two"),
            SelectItem::text(3, "Three"),
        ];
        assert_eq!(
            compute_display_text(&[1, 2, 3], &items, None::<&dyn Fn(&i32) -> String>),
            "One, Two, Three"
        );
    }

    // ── apply_selection ────────────────────────────────────────────────────

    #[test]
    fn apply_selection_single_replaces() {
        let mut sel = vec![1];
        let act = apply_selection(&mut sel, &2, false);
        assert_eq!(sel, vec![2]);
        assert!(matches!(
            act,
            SelectAction::Selected {
                value: 2,
                multi: false
            }
        ));
    }

    #[test]
    fn apply_selection_single_reclick_is_noop() {
        let mut sel = vec![1];
        let act = apply_selection(&mut sel, &1, false);
        assert_eq!(sel, vec![1]);
        assert!(matches!(act, SelectAction::None));
    }

    #[test]
    fn apply_selection_multi_toggles() {
        let mut sel: Vec<i32> = Vec::new();
        apply_selection(&mut sel, &1, true);
        apply_selection(&mut sel, &2, true);
        assert_eq!(sel, vec![1, 2]);
        apply_selection(&mut sel, &1, true);
        assert_eq!(sel, vec![2]);
    }

    // ── apply_select_all ───────────────────────────────────────────────────

    #[test]
    fn select_all_adds_every_enabled() {
        let items = vec![
            SelectItem::text(1, "A"),
            SelectItem::text(2, "B"),
            SelectItem::new(3).disabled(true),
        ];
        let mut sel: Vec<i32> = Vec::new();
        apply_select_all(&mut sel, &items, true);
        assert_eq!(sel, vec![1, 2]);
    }

    #[test]
    fn select_all_false_clears() {
        let items = vec![SelectItem::text(1, "A")];
        let mut sel = vec![1];
        apply_select_all(&mut sel, &items, false);
        assert!(sel.is_empty());
    }

    // ── all_enabled_selected / some_enabled_selected ──────────────────────

    #[test]
    fn all_enabled_selected_requires_every_enabled() {
        let items = vec![SelectItem::text(1, "A"), SelectItem::text(2, "B")];
        assert!(!all_enabled_selected(&[1], &items));
        assert!(all_enabled_selected(&[1, 2], &items));
    }

    #[test]
    fn all_enabled_selected_handles_empty() {
        let items: Vec<SelectItem<i32>> = Vec::new();
        assert!(!all_enabled_selected(&[], &items));
    }

    #[test]
    fn some_enabled_selected_partial() {
        let items = vec![SelectItem::text(1, "A"), SelectItem::text(2, "B")];
        assert!(!some_enabled_selected(&[], &items));
        assert!(some_enabled_selected(&[1], &items));
        assert!(some_enabled_selected(&[1, 2], &items));
    }

    // ── select_all_icon / item_checkbox_icon ──────────────────────────────

    #[test]
    fn select_all_icon_tri_state() {
        assert_eq!(select_all_icon(true, true), filled::CHECK_BOX);
        assert_eq!(
            select_all_icon(false, true),
            filled::INDETERMINATE_CHECK_BOX
        );
        assert_eq!(
            select_all_icon(false, false),
            filled::CHECK_BOX_OUTLINE_BLANK
        );
    }

    #[test]
    fn item_checkbox_icon_matches_selection() {
        assert_eq!(item_checkbox_icon(true), filled::CHECK_BOX);
        assert_eq!(item_checkbox_icon(false), filled::CHECK_BOX_OUTLINE_BLANK);
    }

    // ── position_to_origins ───────────────────────────────────────────────

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

    // ── Defaults ───────────────────────────────────────────────────────────

    #[test]
    fn defaults_match_mudblazor() {
        let s: MudSelect<i32> = MudSelect::new();
        assert!(s.tostring_func.is_none());
        assert!(!s.error && !s.required);
        assert!(!s.disabled && !s.readonly && !s.clearable);
        assert!(!s.multi_selection && !s.select_all && !s.strict && !s.lock_scroll);
        assert!(matches!(s.variant, Variant::Text));
        assert!(matches!(s.margin, Margin::None));
        assert!(matches!(s.color, Color::Primary));
        assert!(matches!(s.size, Size::Medium));
        assert!(!s.full_width && !s.disable_underline);
        assert!(s.adornment_color.is_none());
        assert_eq!(s.open_icon, filled::ARROW_DROP_DOWN);
        assert_eq!(s.close_icon, filled::ARROW_DROP_UP);
        assert!(s.position.is_none());
        assert!(s.anchor_origin.is_none());
        assert!(s.transform_origin.is_none());
        assert!(!s.right_to_left);
        assert_eq!(s.max_height, Some(DEFAULT_MAX_HEIGHT));
        assert!(!s.dense);
    }

    // ── Builders ───────────────────────────────────────────────────────────

    #[test]
    fn builders_set_every_field() {
        let s: MudSelect<i32> = MudSelect::<i32>::new()
            .tostring_func(|v| v.to_string())
            .error(true)
            .error_text("err")
            .required(true)
            .disabled(true)
            .readonly(true)
            .clearable(true)
            .multi_selection(true)
            .select_all(true)
            .strict(true)
            .lock_scroll(true)
            .label("L")
            .placeholder("P")
            .helper_text("H")
            .variant(Variant::Outlined)
            .margin(Margin::Dense)
            .color(Color::Error)
            .size(Size::Large)
            .full_width(true)
            .disable_underline(true)
            .icon_size(Size::Small)
            .adornment_color(Color::Warning)
            .open_icon("O")
            .close_icon("C")
            .position(Position::Top)
            .anchor_origin(AnchorOrigin::Top)
            .transform_origin(TransformOrigin::BottomLeft)
            .right_to_left(true)
            .max_height(200.0)
            .popover_class("pc")
            .dense(true)
            .on_open(|| {})
            .on_close(|| {})
            .on_clear_button_click(|| {})
            .on_selected_values_changed(|_| {})
            .classes(["cls"]);

        assert!(s.tostring_func.is_some());
        assert!(s.error && s.required);
        assert!(s.disabled && s.readonly && s.clearable);
        assert!(s.multi_selection && s.select_all && s.strict && s.lock_scroll);
        assert!(matches!(s.variant, Variant::Outlined));
        assert!(matches!(s.margin, Margin::Dense));
        assert!(matches!(s.color, Color::Error));
        assert!(matches!(s.size, Size::Large));
        assert!(s.full_width && s.disable_underline);
        assert!(matches!(s.icon_size, Size::Small));
        assert_eq!(s.adornment_color, Some(Color::Warning));
        assert_eq!(s.open_icon, "O");
        assert_eq!(s.close_icon, "C");
        assert_eq!(s.position, Some(Position::Top));
        assert_eq!(s.anchor_origin, Some(AnchorOrigin::Top));
        assert_eq!(s.transform_origin, Some(TransformOrigin::BottomLeft));
        assert!(s.right_to_left);
        assert_eq!(s.max_height, Some(200.0));
        assert_eq!(s.popover_class.as_deref(), Some("pc"));
        assert!(s.dense);
        assert!(s.on_open.is_some());
        assert!(s.on_close.is_some());
        assert!(s.on_clear_button_click.is_some());
        assert!(s.on_selected_values_changed.is_some());
        assert!(s.base.has_class("cls"));
    }
}