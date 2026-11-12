//! `MudChipSet` — a high-performance container that coordinates a wrapping row
//! of [`MudChip`]s.
//!
//! Port of `MudChipSet<T>.razor.cs` (MudBlazor 8.13.0).
//!
//! # MudBlazor parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `AllClosable`       | [`MudChipSet::all_closable`] | `false` |
//! | `CheckMark`         | [`MudChipSet::check_mark`] | `false` |
//! | `CheckedIcon`       | [`MudChipSet::checked_icon`] | `None` |
//! | `ChildContent`      | closure to [`MudChipSet::show`] | required |
//! | `CloseIcon`         | [`MudChipSet::close_icon`] | `None` |
//! | `Color`             | [`MudChipSet::color`] | `None` |
//! | `Comparer`          | *(Rust `String` keys)* | — |
//! | `IconColor`         | [`MudChipSet::icon_color`] | `None` |
//! | `Label`             | [`MudChipSet::label`] | `None` |
//! | `Mandatory`         | [`MudChipSet::mandatory`] | `false` |
//! | `ReadOnly`          | [`MudChipSet::read_only`] | `false` |
//! | `SelectedChip`      | [`ChipSetState::selected_value`] | `None` |
//! | `SelectedValue`     | [`ChipSetState::selected_value`] | `None` |
//! | `SelectedValueChanged` | [`MudChipSet::on_selected_value_changed`] | `None` |
//! | `SelectedValues`    | [`ChipSetState::selected_values`] | `{}` |
//! | `SelectedValuesChanged` | [`MudChipSet::on_selected_values_changed`] | `None` |
//! | `SelectionMode`     | [`MudChipSet::selection_mode`] | `Single` |
//! | `Size`              | [`MudChipSet::size`] | `None` |
//! | `Variant`           | [`MudChipSet::variant`] | `None` |
//! | `Class` (base)      | *(not on the set; per-chip)* | — |
//! | `Style` (base)      | *(not on the set; per-chip)* | — |
//! | `Tag` (base)        | **not ported** (no HTML tag) | — |
//! | `UserAttributes` (base) | **not ported** | — |
//!
//! ### Extensions (not in MudBlazor)
//!
//! | Extension | Description |
//! |-----------|-------------|
//! | [`MudChipSet::disabled`] | Disables every chip via cascade. |
//! | [`MudChipSet::selected_color`] | Cascades `selected_color` to every chip. |
//! | [`MudChipSet::on_close`] | Set-level close callback; MudBlazor fires per-chip. |
//!
//! ### Not ported
//!
//! * **`Comparer`** — Rust uses `String` keys with `HashSet<String>`; equality
//!   is `String: PartialEq`.
//! * **`SelectedChip`** — MudBlazor exposes the selected `MudChip` instance;
//!   the port keeps the state as `String` keys (the caller's chip list owns
//!   the actual `MudChip` values).
//! * **`Tag` / `UserAttributes`** — no HTML tag.
//!
//! # Caller-owned state
//!
//! The set is a **config-only** builder; selection lives in a separate
//! [`ChipSetState`] that the caller owns across frames. This matches the
//! pattern used by `MudTable`, `MudTreeView`, `MudTimeline`, and `MudList`:
//!
//! ```ignore
//! struct MyPage { chip_state: ChipSetState }
//!
//! MudChipSet::new()
//!     .selection_mode(SelectionMode::Multi)
//!     .show(ui, &mut self.chip_state, |ctx| {
//!         ctx.chip(MudChip::new("Rust").value("rs"));
//!         ctx.chip(MudChip::new("Go").value("go"));
//!     });
//! // `self.chip_state.selected_values` now holds the clicked keys.
//! ```
//!
//! Callbacks ([`on_selected_value_changed`](MudChipSet::on_selected_value_changed),
//! [`on_selected_values_changed`](MudChipSet::on_selected_values_changed),
//! [`on_close`](MudChipSet::on_close)) are still available for "react
//! immediately" use cases, but are optional — the state is authoritative.
//!
//! # Selection semantics
//!
//! | Mode | Click unselected | Click selected | `mandatory` |
//! |------|------------------|----------------|-------------|
//! | `Single` | selects it | replaces the selection | n/a |
//! | `Toggle` | selects it | deselects it (unless `mandatory`) | prevents deselect |
//! | `Multi`  | adds it | removes it | n/a |
//!
//! `read_only(true)` prevents selection changes but keeps close handling.
//! `disabled(true)` cascades to every chip and suppresses both selection and
//! close.
//!
//! # Cascade
//!
//! [`apply_cascade`] fills in a chip's unset fields from the set config. The
//! precedence is:
//!
//! ```text
//! chip.field  >  set.field  >  MudChip default
//! ```
//!
//! The cascade runs in [`MudChipSetCtx::chip`] before the chip is rendered.

use std::collections::HashSet;

use egui::Ui;
use mud_egui_core::enums::{Color, SelectionMode, Size, Variant};

use crate::data_display::chip::mud_chip::MudChip;

// =============================================================================
// Pure helpers
// =============================================================================

/// The selection key for a chip: its `value` when set, otherwise its `text`.
///
/// This is what the set stores in [`ChipSetState::selected_value`] /
/// [`ChipSetState::selected_values`].
#[inline]
pub fn chip_key(chip: &MudChip) -> String {
    chip.value
        .clone()
        .unwrap_or_else(|| chip.text.clone())
}

/// Whether `key` is selected under `mode` and the given state.
#[inline]
pub fn is_key_selected(
    mode: SelectionMode,
    selected_value: Option<&str>,
    selected_values: &HashSet<String>,
    key: &str,
) -> bool {
    match mode {
        SelectionMode::Multi => selected_values.contains(key),
        SelectionMode::Single | SelectionMode::Toggle => selected_value == Some(key),
    }
}

/// Applies one click to the single-selection state.
///
/// Returns `true` when `current` changed.
#[inline]
pub fn apply_single_selection(current: &mut Option<String>, key: &str) -> bool {
    if current.as_deref() == Some(key) {
        return false;
    }
    *current = Some(key.to_owned());
    true
}

/// Applies one click to the toggle-selection state.
///
/// * Already selected and `mandatory == false` → clears the selection.
/// * Already selected and `mandatory == true`  → no change.
/// * Not selected                              → selects it.
///
/// Returns `true` when `current` changed.
#[inline]
pub fn apply_toggle_selection(
    current: &mut Option<String>,
    key: &str,
    mandatory: bool,
) -> bool {
    if current.as_deref() == Some(key) {
        if mandatory {
            return false;
        }
        *current = None;
        true
    } else {
        *current = Some(key.to_owned());
        true
    }
}

/// Applies one click to the multi-selection state.
///
/// Adds `key` when absent, removes it when present. Always returns `true`
/// because the set always changes (clicking toggles membership).
#[inline]
pub fn apply_multi_selection(values: &mut HashSet<String>, key: &str) -> bool {
    if values.contains(key) {
        values.remove(key);
    } else {
        values.insert(key.to_owned());
    }
    true
}

/// Removes `key` from the selection when its chip is closed.
///
/// Returns `true` when the selection actually changed. In `Multi` mode this
/// is `true` only if `key` was present (the old implementation fired the
/// callback unconditionally — a bug).
pub fn apply_close(
    mode: SelectionMode,
    selected_value: &mut Option<String>,
    selected_values: &mut HashSet<String>,
    key: &str,
) -> bool {
    match mode {
        SelectionMode::Multi => selected_values.remove(key),
        SelectionMode::Single | SelectionMode::Toggle => {
            if selected_value.as_deref() == Some(key) {
                *selected_value = None;
                true
            } else {
                false
            }
        }
    }
}

/// Applies the set's cascading defaults to `chip` in place.
///
/// Only fields the chip did not set are filled in:
///
/// * `None` optional fields → the set's value.
/// * `false` booleans the set has `true` for → `true`.
/// * `chip.disabled` is OR-ed with `set.disabled`.
pub fn apply_cascade(chip: &mut MudChip, set: &MudChipSet) {
    if chip.color.is_none() {
        chip.color = set.color;
    }
    if chip.icon_color.is_none() {
        chip.icon_color = set.icon_color;
    }
    if chip.selected_color.is_none() {
        chip.selected_color = set.selected_color;
    }
    if chip.variant.is_none() {
        chip.variant = set.variant;
    }
    if chip.size.is_none() {
        chip.size = set.size;
    }
    if chip.label.is_none() {
        chip.label = set.label;
    }
    if chip.checked_icon.is_none() {
        chip.checked_icon = set.checked_icon.clone();
    }
    if chip.close_icon.is_none() {
        chip.close_icon = set.close_icon.clone();
    }
    if !chip.show_check_mark {
        chip.show_check_mark = set.check_mark;
    }
    if !chip.closable {
        chip.closable = set.all_closable;
    }
    if set.disabled {
        chip.disabled = true;
    }
}

// =============================================================================
// ChipSetState
// =============================================================================

/// Caller-owned selection state for [`MudChipSet`].
///
/// This is the egui equivalent of MudBlazor's bindable `SelectedValue` /
/// `SelectedValues`. Persist it across frames and pass `&mut` to
/// [`MudChipSet::show`].
///
/// ```ignore
/// let mut chip_state = ChipSetState::new();
///
/// MudChipSet::new().show(ui, &mut chip_state, |ctx| {
///     ctx.chip(MudChip::new("Rust").value("rs"));
///     ctx.chip(MudChip::new("Go").value("go"));
/// });
///
/// for key in &chip_state.selected_values { /* … */ }
/// ```
#[derive(Debug, Clone, Default)]
pub struct ChipSetState {
    /// Currently selected key in `Single` / `Toggle` modes.
    pub selected_value: Option<String>,
    /// Currently selected keys in `Multi` mode.
    pub selected_values: HashSet<String>,
}

impl ChipSetState {
    /// Creates an empty state (nothing selected).
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: seeds the single-selection value.
    pub fn with_selected_value(mut self, v: impl Into<String>) -> Self {
        self.selected_value = Some(v.into());
        self
    }

    /// Builder: seeds the multi-selection set.
    pub fn with_selected_values<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.selected_values = values.into_iter().map(Into::into).collect();
        self
    }

    /// Clears both single and multi selection.
    pub fn clear(&mut self) {
        self.selected_value = None;
        self.selected_values.clear();
    }

    /// Whether `key` is selected under `mode`.
    #[inline]
    pub fn is_selected(&self, mode: SelectionMode, key: &str) -> bool {
        is_key_selected(
            mode,
            self.selected_value.as_deref(),
            &self.selected_values,
            key,
        )
    }

    /// Number of selected keys. Counts `selected_value` as 1 in
    /// `Single` / `Toggle` modes.
    pub fn selected_count(&self, mode: SelectionMode) -> usize {
        match mode {
            SelectionMode::Multi => self.selected_values.len(),
            SelectionMode::Single | SelectionMode::Toggle => {
                if self.selected_value.is_some() { 1 } else { 0 }
            }
        }
    }
}

// =============================================================================
// MudChipSet
// =============================================================================

/// A container that lays out [`MudChip`]s and manages their shared selection
/// state.
///
/// The set is **config-only**: all state lives in [`ChipSetState`], which the
/// caller passes to [`MudChipSet::show`].
#[must_use = "MudChipSet does nothing until you call `.show(ui, state, |ctx| …)`"]
pub struct MudChipSet {
    // ── Selection ──────────────────────────────────────────────────────────
    /// `Single` / `Toggle` / `Multi`. Default `Single`.
    pub selection_mode: SelectionMode,
    /// Prevents deselecting the last chip (`Toggle` mode only).
    pub mandatory: bool,
    /// Prevents every selection change.
    pub read_only: bool,
    /// Disables every chip via cascade.
    pub disabled: bool,

    // ── Cascade ────────────────────────────────────────────────────────────
    /// When `true`, every chip is closable.
    pub all_closable: bool,
    /// Accent colour cascaded to chips.
    pub color: Option<Color>,
    /// Leading-icon colour cascaded to chips.
    pub icon_color: Option<Color>,
    /// Selected-accent override cascaded to chips.
    pub selected_color: Option<Color>,
    /// Variant cascaded to chips.
    pub variant: Option<Variant>,
    /// Size cascaded to chips.
    pub size: Option<Size>,
    /// Label flag cascaded to chips.
    pub label: Option<bool>,
    /// Check-mark flag cascaded to chips.
    pub check_mark: bool,
    /// Checked glyph cascaded to chips.
    pub checked_icon: Option<String>,
    /// Close glyph cascaded to chips.
    pub close_icon: Option<String>,

    // ── Callbacks ──────────────────────────────────────────────────────────
    /// Fired after `selected_value` changes.
    pub on_selected_value_changed: Option<Box<dyn FnMut(Option<String>)>>,
    /// Fired after `selected_values` changes.
    pub on_selected_values_changed: Option<Box<dyn FnMut(HashSet<String>)>>,
    /// Fired after a chip's close button is clicked.
    pub on_close: Option<Box<dyn FnMut(String)>>,
}

impl Default for MudChipSet {
    fn default() -> Self {
        Self {
            selection_mode: SelectionMode::Single,
            mandatory: false,
            read_only: false,
            disabled: false,
            all_closable: false,
            color: None,
            icon_color: None,
            selected_color: None,
            variant: None,
            size: None,
            label: None,
            check_mark: false,
            checked_icon: None,
            close_icon: None,
            on_selected_value_changed: None,
            on_selected_values_changed: None,
            on_close: None,
        }
    }
}

impl MudChipSet {
    /// Creates a set with MudBlazor defaults (`Single` mode).
    pub fn new() -> Self {
        Self::default()
    }

    // ── Builder methods ────────────────────────────────────────────────────

    /// Sets the selection mode (`SelectionMode`, default `Single`).
    #[inline]
    pub fn selection_mode(mut self, m: SelectionMode) -> Self {
        self.selection_mode = m;
        self
    }

    /// Prevents deselecting the last chip (`Mandatory`).
    #[inline]
    pub fn mandatory(mut self, b: bool) -> Self {
        self.mandatory = b;
        self
    }

    /// Prevents every selection change (`ReadOnly`).
    #[inline]
    pub fn read_only(mut self, b: bool) -> Self {
        self.read_only = b;
        self
    }

    /// Disables every chip via cascade.
    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// Makes every chip closable (`AllClosable`).
    #[inline]
    pub fn all_closable(mut self, b: bool) -> Self {
        self.all_closable = b;
        self
    }

    /// Cascades `Color` to every chip.
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }

    /// Cascades `IconColor` to every chip.
    #[inline]
    pub fn icon_color(mut self, c: Color) -> Self {
        self.icon_color = Some(c);
        self
    }

    /// Cascades `SelectedColor` to every chip.
    #[inline]
    pub fn selected_color(mut self, c: Color) -> Self {
        self.selected_color = Some(c);
        self
    }

    /// Cascades `Variant` to every chip.
    #[inline]
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = Some(v);
        self
    }

    /// Cascades `Size` to every chip.
    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        self.size = Some(s);
        self
    }

    /// Cascades `Label` to every chip.
    #[inline]
    pub fn label(mut self, b: bool) -> Self {
        self.label = Some(b);
        self
    }

    /// Cascades `CheckMark` to every chip.
    #[inline]
    pub fn check_mark(mut self, b: bool) -> Self {
        self.check_mark = b;
        self
    }

    /// Cascades `CheckedIcon` to every chip.
    #[inline]
    pub fn checked_icon(mut self, i: impl Into<String>) -> Self {
        self.checked_icon = Some(i.into());
        self
    }

    /// Cascades `CloseIcon` to every chip.
    #[inline]
    pub fn close_icon(mut self, i: impl Into<String>) -> Self {
        self.close_icon = Some(i.into());
        self
    }

    // ── Callbacks ──────────────────────────────────────────────────────────

    /// Fired after `selected_value` changes (`SelectedValueChanged`).
    #[inline]
    pub fn on_selected_value_changed(
        mut self,
        f: impl FnMut(Option<String>) + 'static,
    ) -> Self {
        self.on_selected_value_changed = Some(Box::new(f));
        self
    }

    /// Fired after `selected_values` changes (`SelectedValuesChanged`).
    #[inline]
    pub fn on_selected_values_changed(
        mut self,
        f: impl FnMut(HashSet<String>) + 'static,
    ) -> Self {
        self.on_selected_values_changed = Some(Box::new(f));
        self
    }

    /// Fired when a chip's close button is clicked.
    #[inline]
    pub fn on_close(mut self, f: impl FnMut(String) + 'static) -> Self {
        self.on_close = Some(Box::new(f));
        self
    }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Renders the set into a wrapping horizontal row.
    ///
    /// `state` is mutated in place: `selected_value` / `selected_values` are
    /// updated as chips are clicked or closed. Persist it across frames.
    ///
    /// `add_chips` receives a [`MudChipSetCtx`] and should call
    /// [`MudChipSetCtx::chip`] for each chip. The closure runs exactly once.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        state: &mut ChipSetState,
        add_chips: impl FnOnce(&mut MudChipSetCtx<'_>) -> R,
    ) -> R {
        // Move callbacks out of `self` so the ctx can hold `&mut` borrows
        // without conflicting with `&self` (config).
        let mut on_value_changed = self.on_selected_value_changed.take();
        let mut on_values_changed = self.on_selected_values_changed.take();
        let mut on_close = self.on_close.take();

        let config: &MudChipSet = &self;

        ui.horizontal_wrapped(|inner_ui| {
            let mut ctx = MudChipSetCtx {
                config,
                state,
                on_value_changed: &mut on_value_changed,
                on_values_changed: &mut on_values_changed,
                on_close: &mut on_close,
                ui: inner_ui,
            };
            add_chips(&mut ctx)
        })
            .inner
    }
}

// =============================================================================
// MudChipSetCtx
// =============================================================================

/// Context passed to [`MudChipSet::show`]'s closure.
///
/// Provides [`chip`](Self::chip) plus public access to the config and state
/// via [`config`](Self::config), [`state`](Self::state), and [`ui`](Self::ui).
pub struct MudChipSetCtx<'a> {
    /// The set's configuration.
    pub config: &'a MudChipSet,
    /// The caller-owned selection state.
    pub state: &'a mut ChipSetState,
    /// The `Ui` to render into (a `horizontal_wrapped` scope).
    pub ui: &'a mut Ui,
    // Callbacks — owned by the enclosing `show` and moved here by `&mut`.
    on_value_changed: &'a mut Option<Box<dyn FnMut(Option<String>)>>,
    on_values_changed: &'a mut Option<Box<dyn FnMut(HashSet<String>)>>,
    on_close: &'a mut Option<Box<dyn FnMut(String)>>,
}

impl<'a> MudChipSetCtx<'a> {
    /// Renders one chip with cascading defaults applied, updates the state,
    /// and fires callbacks.
    ///
    /// Returns `true` when the chip's close button was clicked **or** the
    /// selection changed this frame.
    pub fn chip(&mut self, mut chip: MudChip) -> bool {
        // 1. Cascade config → chip.
        apply_cascade(&mut chip, self.config);

        // 2. Resolve key + seed selection state.
        let key = chip_key(&chip);
        chip.selected = is_key_selected(
            self.config.selection_mode,
            self.state.selected_value.as_deref(),
            &self.state.selected_values,
            &key,
        );

        // 3. Render.
        let (resp, close_clicked) = chip.show(self.ui);

        // 4. Close handling.
        if close_clicked && !self.config.disabled {
            let changed = apply_close(
                self.config.selection_mode,
                &mut self.state.selected_value,
                &mut self.state.selected_values,
                &key,
            );
            if changed {
                self.fire_selection_changed();
            }
            if let Some(cb) = self.on_close.as_mut() {
                cb(key);
            }
            return true;
        }

        // 5. Body-click → selection change.
        if resp.clicked() && !chip.disabled && !self.config.read_only {
            let changed = match self.config.selection_mode {
                SelectionMode::Single => {
                    apply_single_selection(&mut self.state.selected_value, &key)
                }
                SelectionMode::Toggle => apply_toggle_selection(
                    &mut self.state.selected_value,
                    &key,
                    self.config.mandatory,
                ),
                SelectionMode::Multi => {
                    apply_multi_selection(&mut self.state.selected_values, &key)
                }
            };
            if changed {
                self.fire_selection_changed();
            }
            return changed;
        }

        false
    }

    /// Convenience: current single-selection value.
    pub fn selected_value(&self) -> Option<&String> {
        self.state.selected_value.as_ref()
    }

    /// Convenience: current multi-selection set.
    pub fn selected_values(&self) -> &HashSet<String> {
        &self.state.selected_values
    }

    /// Dispatches the appropriate selection-change callback (single or multi).
    fn fire_selection_changed(&mut self) {
        match self.config.selection_mode {
            SelectionMode::Multi => {
                // Clone first to avoid holding `&self.state` across the
                // callback call.
                let values = self.state.selected_values.clone();
                if let Some(cb) = self.on_values_changed.as_mut() {
                    cb(values);
                }
            }
            SelectionMode::Single | SelectionMode::Toggle => {
                let value = self.state.selected_value.clone();
                if let Some(cb) = self.on_value_changed.as_mut() {
                    cb(value);
                }
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── chip_key ───────────────────────────────────────────────────────────

    #[test]
    fn chip_key_prefers_value() {
        let chip = MudChip::new("Label").value("key");
        assert_eq!(chip_key(&chip), "key");
    }

    #[test]
    fn chip_key_falls_back_to_text() {
        let chip = MudChip::new("Label");
        assert_eq!(chip_key(&chip), "Label");
    }

    // ── is_key_selected ────────────────────────────────────────────────────

    #[test]
    fn is_key_selected_single() {
        let mut values = HashSet::new();
        values.insert("x".to_owned());
        assert!(is_key_selected(
            SelectionMode::Single,
            Some("x"),
            &values,
            "x"
        ));
        assert!(!is_key_selected(
            SelectionMode::Single,
            Some("x"),
            &values,
            "y"
        ));
        assert!(!is_key_selected(SelectionMode::Single, None, &values, "x"));
    }

    #[test]
    fn is_key_selected_multi() {
        let mut values = HashSet::new();
        values.insert("x".to_owned());
        assert!(is_key_selected(
            SelectionMode::Multi,
            None,
            &values,
            "x"
        ));
        assert!(!is_key_selected(
            SelectionMode::Multi,
            None,
            &values,
            "y"
        ));
    }

    // ── apply_single_selection ─────────────────────────────────────────────

    #[test]
    fn single_selects_from_empty() {
        let mut current = None;
        assert!(apply_single_selection(&mut current, "a"));
        assert_eq!(current.as_deref(), Some("a"));
    }

    #[test]
    fn single_replaces_previous() {
        let mut current = Some("a".to_owned());
        assert!(apply_single_selection(&mut current, "b"));
        assert_eq!(current.as_deref(), Some("b"));
    }

    #[test]
    fn single_reclick_is_noop() {
        let mut current = Some("a".to_owned());
        assert!(!apply_single_selection(&mut current, "a"));
        assert_eq!(current.as_deref(), Some("a"));
    }

    // ── apply_toggle_selection ─────────────────────────────────────────────

    #[test]
    fn toggle_selects_from_empty() {
        let mut current = None;
        assert!(apply_toggle_selection(&mut current, "a", false));
        assert_eq!(current.as_deref(), Some("a"));
    }

    #[test]
    fn toggle_clears_when_not_mandatory() {
        let mut current = Some("a".to_owned());
        assert!(apply_toggle_selection(&mut current, "a", false));
        assert!(current.is_none());
    }

    #[test]
    fn toggle_keeps_when_mandatory() {
        let mut current = Some("a".to_owned());
        assert!(!apply_toggle_selection(&mut current, "a", true));
        assert_eq!(current.as_deref(), Some("a"));
    }

    #[test]
    fn toggle_replaces_other() {
        let mut current = Some("a".to_owned());
        assert!(apply_toggle_selection(&mut current, "b", true));
        assert_eq!(current.as_deref(), Some("b"));
    }

    // ── apply_multi_selection ──────────────────────────────────────────────

    #[test]
    fn multi_inserts_then_removes() {
        let mut values = HashSet::new();
        assert!(apply_multi_selection(&mut values, "a"));
        assert!(values.contains("a"));
        assert!(apply_multi_selection(&mut values, "a"));
        assert!(values.is_empty());
    }

    #[test]
    fn multi_accumulates_distinct_keys() {
        let mut values = HashSet::new();
        apply_multi_selection(&mut values, "a");
        apply_multi_selection(&mut values, "b");
        apply_multi_selection(&mut values, "c");
        assert_eq!(values.len(), 3);
    }

    // ── apply_close ────────────────────────────────────────────────────────

    #[test]
    fn close_multi_removes_if_present() {
        let mut value = None;
        let mut values: HashSet<String> = ["a".to_owned(), "b".to_owned()].into();
        assert!(apply_close(SelectionMode::Multi, &mut value, &mut values, "a"));
        assert_eq!(values.len(), 1);
        assert!(values.contains("b"));
    }

    #[test]
    fn close_multi_noop_if_absent() {
        let mut value = None;
        let mut values: HashSet<String> = ["b".to_owned()].into();
        assert!(!apply_close(SelectionMode::Multi, &mut value, &mut values, "a"));
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn close_single_clears_if_selected() {
        let mut value = Some("a".to_owned());
        let mut values = HashSet::new();
        assert!(apply_close(SelectionMode::Single, &mut value, &mut values, "a"));
        assert!(value.is_none());
    }

    #[test]
    fn close_single_noop_if_other() {
        let mut value = Some("b".to_owned());
        let mut values = HashSet::new();
        assert!(!apply_close(SelectionMode::Single, &mut value, &mut values, "a"));
        assert_eq!(value.as_deref(), Some("b"));
    }

    // ── apply_cascade ──────────────────────────────────────────────────────

    #[test]
    fn cascade_fills_unset_fields_from_set() {
        let set = MudChipSet::new()
            .color(Color::Secondary)
            .size(Size::Large)
            .variant(Variant::Outlined)
            .check_mark(true)
            .all_closable(true);

        let mut chip = MudChip::new("x");
        apply_cascade(&mut chip, &set);

        assert_eq!(chip.color, Some(Color::Secondary));
        assert_eq!(chip.size, Some(Size::Large));
        assert_eq!(chip.variant, Some(Variant::Outlined));
        assert!(chip.show_check_mark);
        assert!(chip.closable);
    }

    #[test]
    fn cascade_preserves_chip_overrides() {
        let set = MudChipSet::new()
            .color(Color::Secondary)
            .size(Size::Small)
            .variant(Variant::Outlined)
            .all_closable(true);

        let mut chip = MudChip::new("x")
            .color(Color::Error)
            .size(Size::Large)
            .variant(Variant::Filled)
            .closable(false); // explicitly opted out

        apply_cascade(&mut chip, &set);

        assert_eq!(chip.color, Some(Color::Error));
        assert_eq!(chip.size, Some(Size::Large));
        assert_eq!(chip.variant, Some(Variant::Filled));
        assert!(
            !chip.closable,
            "chip.closable(false) must not be overridden"
        );
    }

    #[test]
    fn cascade_disabled_or_ed() {
        let set = MudChipSet::new().disabled(true);
        let mut chip = MudChip::new("x").disabled(false);
        apply_cascade(&mut chip, &set);
        assert!(chip.disabled, "set.disabled must force chip.disabled");
    }

    // ── ChipSetState ───────────────────────────────────────────────────────

    #[test]
    fn state_default_is_empty() {
        let s = ChipSetState::new();
        assert!(s.selected_value.is_none());
        assert!(s.selected_values.is_empty());
    }

    #[test]
    fn state_builders_seed_values() {
        let s = ChipSetState::new()
            .with_selected_value("a")
            .with_selected_values(["b", "c"]);
        assert_eq!(s.selected_value.as_deref(), Some("a"));
        assert!(s.selected_values.contains("b"));
        assert!(s.selected_values.contains("c"));
    }

    #[test]
    fn state_clear_empties_both() {
        let mut s = ChipSetState::new()
            .with_selected_value("a")
            .with_selected_values(["b"]);
        s.clear();
        assert!(s.selected_value.is_none());
        assert!(s.selected_values.is_empty());
    }

    #[test]
    fn state_is_selected_matches_mode() {
        let mut s = ChipSetState::new();
        s.selected_value = Some("a".to_owned());
        assert!(s.is_selected(SelectionMode::Single, "a"));
        assert!(!s.is_selected(SelectionMode::Single, "b"));

        let mut s = ChipSetState::new();
        s.selected_values.insert("a".to_owned());
        assert!(s.is_selected(SelectionMode::Multi, "a"));
        assert!(!s.is_selected(SelectionMode::Multi, "b"));
    }

    #[test]
    fn state_selected_count() {
        let s = ChipSetState::new();
        assert_eq!(s.selected_count(SelectionMode::Single), 0);
        assert_eq!(s.selected_count(SelectionMode::Multi), 0);

        let s = ChipSetState::new().with_selected_value("a");
        assert_eq!(s.selected_count(SelectionMode::Single), 1);

        let s = ChipSetState::new().with_selected_values(["a", "b", "c"]);
        assert_eq!(s.selected_count(SelectionMode::Multi), 3);
    }

    // ── MudChipSet defaults & builders ─────────────────────────────────────

    #[test]
    fn defaults_match_mudblazor() {
        let set = MudChipSet::new();
        assert_eq!(set.selection_mode, SelectionMode::Single);
        assert!(!set.mandatory);
        assert!(!set.read_only);
        assert!(!set.disabled);
        assert!(!set.all_closable);
        assert!(set.color.is_none());
        assert!(set.icon_color.is_none());
        assert!(set.selected_color.is_none());
        assert!(set.variant.is_none());
        assert!(set.size.is_none());
        assert!(set.label.is_none());
        assert!(!set.check_mark);
        assert!(set.checked_icon.is_none());
        assert!(set.close_icon.is_none());
        assert!(set.on_selected_value_changed.is_none());
        assert!(set.on_selected_values_changed.is_none());
        assert!(set.on_close.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let set = MudChipSet::new()
            .selection_mode(SelectionMode::Multi)
            .mandatory(true)
            .read_only(true)
            .disabled(true)
            .all_closable(true)
            .color(Color::Error)
            .icon_color(Color::Success)
            .selected_color(Color::Warning)
            .variant(Variant::Outlined)
            .size(Size::Large)
            .label(true)
            .check_mark(true)
            .checked_icon("C")
            .close_icon("X")
            .on_selected_value_changed(|_| {})
            .on_selected_values_changed(|_| {})
            .on_close(|_| {});

        assert_eq!(set.selection_mode, SelectionMode::Multi);
        assert!(set.mandatory);
        assert!(set.read_only);
        assert!(set.disabled);
        assert!(set.all_closable);
        assert_eq!(set.color, Some(Color::Error));
        assert_eq!(set.icon_color, Some(Color::Success));
        assert_eq!(set.selected_color, Some(Color::Warning));
        assert_eq!(set.variant, Some(Variant::Outlined));
        assert_eq!(set.size, Some(Size::Large));
        assert_eq!(set.label, Some(true));
        assert!(set.check_mark);
        assert_eq!(set.checked_icon.as_deref(), Some("C"));
        assert_eq!(set.close_icon.as_deref(), Some("X"));
        assert!(set.on_selected_value_changed.is_some());
        assert!(set.on_selected_values_changed.is_some());
        assert!(set.on_close.is_some());
    }
}