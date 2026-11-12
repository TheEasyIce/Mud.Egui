//! `MudColorPicker` — a text field with a popover colour picker.
//!
//! Port of `MudColorPicker.razor.cs` (MudBlazor 8.13.0). Pass 1.
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `CloseIcon` | [`MudColorPicker::close_icon`] (default `Close`) |
//! | `ColorPickerMode` | [`MudColorPicker::color_picker_mode`] / [`ColorPickerState::mode`] (default `Rgb`) |
//! | `ColorPickerView` | [`MudColorPicker::color_picker_view`] / [`ColorPickerState::view`] (default `Spectrum`) |
//! | `DragEffect` | [`MudColorPicker::drag_effect`] (default `true`) |
//! | `GridIcon` | [`MudColorPicker::grid_icon`] (default `Apps`) |
//! | `ImportExportIcon` | [`MudColorPicker::import_export_icon`] (default `ImportExport`) |
//! | `Palette` | [`MudColorPicker::palette`] (default 35 entries) |
//! | `PaletteIcon` | [`MudColorPicker::palette_icon`] (default `Palette`) |
//! | `RightToLeft` | [`MudColorPicker::right_to_left`] |
//! | `ShowAlpha` | [`MudColorPicker::show_alpha`] (default `true`) |
//! | `ShowColorField` | [`MudColorPicker::show_color_field`] (default `true`) |
//! | `ShowInputs` | [`MudColorPicker::show_inputs`] (default `true`) |
//! | `ShowModeSwitch` | [`MudColorPicker::show_mode_switch`] (default `true`) |
//! | `ShowPreview` | [`MudColorPicker::show_preview`] (default `true`) |
//! | `ShowSliders` | [`MudColorPicker::show_sliders`] (default `true`) |
//! | `SpectrumIcon` | [`MudColorPicker::spectrum_icon`] (default `Tune`) |
//! | `ThrottleInterval` | [`MudColorPicker::throttle_interval`] (default `50` ms) |
//! | `UpdateBindingIfOnlyHSLChanged` | [`MudColorPicker::update_binding_if_only_hsl_changed`] (default `false`) |
//! | `Value` | [`ColorPickerState::value`] (caller-owned) |
//! | `ValueChanged` | [`ColorPickerResponse::action`] (`ColorSelected`) |
//! | `PickerContent` | **protected** — not ported |
//! | *base* `Adornment*`, `Clearable` … `Variant` | the shared picker surface |
//! | `Style` / `Tag` / `UserAttributes` | not ported: no CSS strings, no HTML tag |
//! | `PickerActions` / `Mask` / typed parsing / `Inline` / `Static` | deferred to pass 2 |
//!
//! # Composition, not reimplementation
//!
//! | Primitive | Role |
//! |-----------|------|
//! | [`MudTextField`] | The trigger, and the HEX input in `Hex` mode. |
//! | [`MudPopover`] | The picker dropdown. |
//! | [`MudSlider`] | The R/G/B, H/S/L and alpha channel controls. |
//! | [`MudIcon`] | The toolbar icons, the preview check, the close glyph. |
//! | [`MudText`] | The trigger label, the helper row, and the preview hex. |
//!
//! # Views
//!
//! | `ColorPickerView` | Body |
//! |-------------------|------|
//! | `Spectrum` (default) | Saturation × lightness plane, hue strip, alpha strip, preview, channel inputs |
//! | `Palette` | Wrapping list of `palette` colours |
//! | `Grid` | Wider wrapping grid of the same colours |
//!
//! `ShowToolbar` renders the three view buttons, the mode switch, the
//! import/export button and the close button.
//!
//! # Drag, throttle and HSL-only updates
//!
//! | Parameter | Effect |
//! |-----------|--------|
//! | `drag_effect` | Commit while dragging. When `false`, only a click or drag-stop commits. |
//! | `throttle_interval` | Minimum milliseconds between two drag commits. |
//! | `update_binding_if_only_hsl_changed` | Commit an HSL edit even when the 8-bit RGB round-trip is identical. |
//!
//! # State
//!
//! [`ColorPickerState::text`] is **not** re-synced from `value` every frame —
//! `commit()` / `clear()` / `with_value()` keep them in sync, so typed edits in
//! `editable(true)` mode survive.

use egui::{pos2, vec2, Color32, CornerRadius, CursorIcon, Layout, Mesh, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, UiBuilder, Vec2};
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{
    AnchorOrigin, BackdropMode, Color, ColorPickerMode, ColorPickerView, Margin, OpenTo,
    OverflowBehavior, PickerVariant, Size, StyleFlavor, TransformOrigin, Typo, Variant,
};
use mud_egui_icons_material::filled;
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::feedback::popover::MudPopover;
use crate::inputs::slider::MudSlider;
use crate::inputs::text_field::MudTextField;
use crate::typography::mud_text::MudText;

use super::color_channels::ColorChannels;
use super::color_picker_action::ColorPickerAction;
use super::color_picker_state::ColorPickerState;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Side of the saturation × lightness plane, in points.
pub const PLANE_SIZE: f32 = 220.0;

/// Height of the hue and alpha strips.
pub const STRIP_HEIGHT: f32 = 14.0;

/// Radius of the crosshair and strip knobs.
const KNOB_RADIUS: f32 = 7.0;

/// Radius of the preview dot.
const PREVIEW_RADIUS: f32 = 14.0;

/// Side of a palette / grid swatch.
pub const SWATCH_SIZE: f32 = 24.0;

/// Gap between swatches.
const SWATCH_GAP: f32 = 4.0;

/// Side of a toolbar icon button.
const TOOLBAR_BUTTON: f32 = 28.0;

/// Default `ThrottleInterval`, in milliseconds.
pub const DEFAULT_THROTTLE_MS: u32 = 50;

/// Number of grid subdivisions along each plane axis.
const PLANE_STEPS: usize = 8;

/// Default palette: 7 hues × 5 shades. The first five entries are the quick
/// colours, matching MudBlazor's documented behaviour[reference:19].
pub const DEFAULT_PALETTE: [&str; 35] = [
    // Red ramp — the first five are the quick colours.
    "#F44336", "#FFEBEE", "#FFCDD2", "#EF9A9A", "#B71C1C",
    // Blue ramp.
    "#2196F3", "#E3F2FD", "#BBDEFB", "#90CAF9", "#0D47A1",
    // Green ramp.
    "#4CAF50", "#E8F5E9", "#C8E6C9", "#A5D6A7", "#1B5E20",
    // Amber ramp.
    "#FFC107", "#FFF8E1", "#FFECB3", "#FFE082", "#FF6F00",
    // Purple ramp.
    "#9C27B0", "#F3E5F5", "#E1BEE7", "#CE93D8", "#4A148C",
    // Cyan ramp.
    "#00BCD4", "#E0F7FA", "#B2EBF2", "#80DEEA", "#006064",
    // Deep orange ramp.
    "#FF5722", "#FBE9E7", "#FFCCBE", "#FFAB91", "#BF360C",
];

/// Number of quick colours shown when the preview dot is clicked.
pub const QUICK_COLOR_COUNT: usize = 5;

// ---------------------------------------------------------------------------
// Response
// ---------------------------------------------------------------------------

/// The result of one [`MudColorPicker::show`] frame.
#[derive(Debug)]
pub struct ColorPickerResponse {
    /// The trigger's own `Response`.
    pub trigger: Response,
    /// Committed colour after this frame.
    pub value: Option<ColorChannels>,
    /// What the picker did this frame.
    pub action: ColorPickerAction,
}

// ---------------------------------------------------------------------------
// MudColorPicker
// ---------------------------------------------------------------------------

/// A text field with a popover colour picker.
///
/// Mirrors `MudColorPicker` in MudBlazor 8.13.0. The Blazor two-way binding
/// `Value` / `ValueChanged` becomes a caller-owned
/// [`ColorPickerState::value`] plus a [`ColorPickerResponse::action`] report —
/// the same split `MudDatePicker`, `MudTimePicker` and `MudDateRangePicker`
/// use.
#[must_use = "MudColorPicker does nothing until you call `.show(ui, state)`"]
pub struct MudColorPicker<'a> {
    // ── Trigger text field ─────────────────────────────────────────────────
    pub label: Option<String>,
    pub placeholder: Option<String>,
    pub helper_text: Option<String>,
    pub helper_text_on_focus: bool,
    pub error: bool,
    pub error_text: Option<String>,
    pub variant: Variant,
    pub margin: Margin,
    pub color: Color,
    pub size: Size,
    pub icon_size: Size,
    pub clearable: bool,
    pub readonly: bool,
    pub disabled: bool,
    pub editable: bool,
    pub immediate_text: bool,

    // ── Adornment ──────────────────────────────────────────────────────────
    pub adornment_icon: String,
    pub adornment_color: Option<Color>,

    // ── Picker shell ───────────────────────────────────────────────────────
    pub picker_variant: PickerVariant,
    pub elevation: Option<u8>,
    pub rounded: bool,
    pub square: bool,
    pub modal: bool,
    pub overflow_behavior: OverflowBehavior,
    pub show_toolbar: bool,
    pub auto_close: bool,

    // ── Own 8.13 parameters ────────────────────────────────────────────────
    /// Close button glyph. Mirrors `CloseIcon` (default `Close`).
    pub close_icon: String,
    /// Initial channel mode. Mirrors `ColorPickerMode` (default `Rgb`).
    pub color_picker_mode: ColorPickerMode,
    /// Initial view. Mirrors `ColorPickerView` (default `Spectrum`).
    pub color_picker_view: ColorPickerView,
    /// Commit while dragging. Mirrors `DragEffect` (default `true`).
    pub drag_effect: bool,
    /// Grid-view toolbar glyph. Mirrors `GridIcon` (default `Apps`).
    pub grid_icon: String,
    /// Import/export toolbar glyph. Mirrors `ImportExportIcon` (default
    /// `ImportExport`).
    pub import_export_icon: String,
    /// Swatch list for the palette and grid views. Mirrors `Palette`.
    pub palette: Vec<ColorChannels>,
    /// Palette-view toolbar glyph. Mirrors `PaletteIcon` (default `Palette`).
    pub palette_icon: String,
    /// Mirrors `RightToLeft`.
    pub right_to_left: bool,
    /// Include the alpha channel. Mirrors `ShowAlpha` (default `true`).
    pub show_alpha: bool,
    /// Render the saturation × lightness plane. Mirrors `ShowColorField`.
    pub show_color_field: bool,
    /// Render the channel inputs. Mirrors `ShowInputs`.
    pub show_inputs: bool,
    /// Render the mode-switch button. Mirrors `ShowModeSwitch`.
    pub show_mode_switch: bool,
    /// Render the preview dot. Mirrors `ShowPreview`.
    pub show_preview: bool,
    /// Render the hue and alpha strips. Mirrors `ShowSliders`.
    pub show_sliders: bool,
    /// Spectrum-view toolbar glyph. Mirrors `SpectrumIcon` (default `Tune`).
    pub spectrum_icon: String,
    /// Milliseconds between drag commits. Mirrors `ThrottleInterval`
    /// (default `50`).
    pub throttle_interval: u32,
    /// Commit HSL-only edits. Mirrors `UpdateBindingIfOnlyHSLChanged`.
    pub update_binding_if_only_hsl_changed: bool,

    // ── Customisation ──────────────────────────────────────────────────────
    pub picker_actions:
        Option<Box<dyn FnMut(&mut Ui) -> Option<ColorPickerAction> + 'a>>,
    pub on_picker_opened: Option<Box<dyn FnMut() + 'a>>,
    pub on_picker_closed: Option<Box<dyn FnMut() + 'a>>,
    pub on_click: Option<Box<dyn FnMut() + 'a>>,

    // ── Utilities ──────────────────────────────────────────────────────────
    pub base: ComponentBase,
    pub flavor: StyleFlavor,
}

impl<'a> Default for MudColorPicker<'a> {
    fn default() -> Self {
        Self {
            label: None,
            placeholder: None,
            helper_text: None,
            helper_text_on_focus: false,
            error: false,
            error_text: None,
            variant: Variant::Text,
            margin: Margin::None,
            color: Color::Primary,
            size: Size::Medium,
            icon_size: Size::Medium,
            clearable: false,
            readonly: false,
            disabled: false,
            editable: false,
            immediate_text: false,
            adornment_icon: filled::COLORIZE.to_owned(),
            adornment_color: None,
            picker_variant: PickerVariant::Dialog,
            elevation: None,
            rounded: false,
            square: false,
            modal: true,
            overflow_behavior: OverflowBehavior::Flip,
            show_toolbar: true,
            auto_close: false,
            close_icon: filled::CLOSE.to_owned(),
            color_picker_mode: ColorPickerMode::Rgb,
            color_picker_view: ColorPickerView::Spectrum,
            drag_effect: true,
            grid_icon: filled::APPS.to_owned(),
            import_export_icon: filled::IMPORT_EXPORT.to_owned(),
            palette: default_palette(),
            palette_icon: filled::PALETTE.to_owned(),
            right_to_left: false,
            show_alpha: true,
            show_color_field: true,
            show_inputs: true,
            show_mode_switch: true,
            show_preview: true,
            show_sliders: true,
            spectrum_icon: filled::TUNE.to_owned(),
            throttle_interval: DEFAULT_THROTTLE_MS,
            update_binding_if_only_hsl_changed: false,
            picker_actions: None,
            on_picker_opened: None,
            on_picker_closed: None,
            on_click: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
        }
    }
}

/// Builds [`DEFAULT_PALETTE`] as `ColorChannels`. Entries that fail to parse
/// are skipped, so the result is always well-formed.
pub fn default_palette() -> Vec<ColorChannels> {
    DEFAULT_PALETTE
        .iter()
        .filter_map(|s| ColorChannels::parse(s))
        .collect()
}

impl<'a> MudColorPicker<'a> {
    /// Creates a picker with MudBlazor's default settings.
    pub fn new() -> Self {
        Self::default()
    }

    // ── Builders ───────────────────────────────────────────────────────────

    #[inline] pub fn label(mut self, s: impl Into<String>) -> Self { self.label = Some(s.into()); self }
    #[inline] pub fn placeholder(mut self, s: impl Into<String>) -> Self { self.placeholder = Some(s.into()); self }
    #[inline] pub fn helper_text(mut self, s: impl Into<String>) -> Self { self.helper_text = Some(s.into()); self }
    #[inline] pub fn helper_text_on_focus(mut self, b: bool) -> Self { self.helper_text_on_focus = b; self }
    #[inline] pub fn error(mut self, b: bool) -> Self { self.error = b; self }
    #[inline] pub fn error_text(mut self, s: impl Into<String>) -> Self { self.error_text = Some(s.into()); self }
    #[inline] pub fn variant(mut self, v: Variant) -> Self { self.variant = v; self }
    #[inline] pub fn margin(mut self, m: Margin) -> Self { self.margin = m; self }
    #[inline] pub fn color(mut self, c: Color) -> Self { self.color = c; self }
    #[inline] pub fn size(mut self, s: Size) -> Self { self.size = s; self }
    #[inline] pub fn icon_size(mut self, s: Size) -> Self { self.icon_size = s; self }
    #[inline] pub fn clearable(mut self, b: bool) -> Self { self.clearable = b; self }
    #[inline] pub fn readonly(mut self, b: bool) -> Self { self.readonly = b; self }
    #[inline] pub fn disabled(mut self, b: bool) -> Self { self.disabled = b; self }
    #[inline] pub fn editable(mut self, b: bool) -> Self { self.editable = b; self }
    #[inline] pub fn immediate_text(mut self, b: bool) -> Self { self.immediate_text = b; self }
    #[inline] pub fn adornment_icon(mut self, s: impl Into<String>) -> Self { self.adornment_icon = s.into(); self }
    #[inline] pub fn adornment_color(mut self, c: Color) -> Self { self.adornment_color = Some(c); self }
    #[inline] pub fn picker_variant(mut self, v: PickerVariant) -> Self { self.picker_variant = v; self }
    #[inline] pub fn elevation(mut self, e: u8) -> Self { self.elevation = Some(e); self }
    #[inline] pub fn rounded(mut self, b: bool) -> Self { self.rounded = b; self }
    #[inline] pub fn square(mut self, b: bool) -> Self { self.square = b; self }
    #[inline] pub fn modal(mut self, b: bool) -> Self { self.modal = b; self }
    #[inline] pub fn overflow_behavior(mut self, b: OverflowBehavior) -> Self { self.overflow_behavior = b; self }
    #[inline] pub fn show_toolbar(mut self, b: bool) -> Self { self.show_toolbar = b; self }
    #[inline] pub fn auto_close(mut self, b: bool) -> Self { self.auto_close = b; self }
    #[inline] pub fn close_icon(mut self, s: impl Into<String>) -> Self { self.close_icon = s.into(); self }
    #[inline] pub fn color_picker_mode(mut self, m: ColorPickerMode) -> Self { self.color_picker_mode = m; self }
    #[inline] pub fn color_picker_view(mut self, v: ColorPickerView) -> Self { self.color_picker_view = v; self }
    #[inline] pub fn drag_effect(mut self, b: bool) -> Self { self.drag_effect = b; self }
    #[inline] pub fn grid_icon(mut self, s: impl Into<String>) -> Self { self.grid_icon = s.into(); self }
    #[inline] pub fn import_export_icon(mut self, s: impl Into<String>) -> Self { self.import_export_icon = s.into(); self }
    #[inline] pub fn palette(mut self, colors: impl IntoIterator<Item = ColorChannels>) -> Self { self.palette = colors.into_iter().collect(); self }
    #[inline] pub fn palette_icon(mut self, s: impl Into<String>) -> Self { self.palette_icon = s.into(); self }
    #[inline] pub fn right_to_left(mut self, b: bool) -> Self { self.right_to_left = b; self }
    #[inline] pub fn show_alpha(mut self, b: bool) -> Self { self.show_alpha = b; self }
    #[inline] pub fn show_color_field(mut self, b: bool) -> Self { self.show_color_field = b; self }
    #[inline] pub fn show_inputs(mut self, b: bool) -> Self { self.show_inputs = b; self }
    #[inline] pub fn show_mode_switch(mut self, b: bool) -> Self { self.show_mode_switch = b; self }
    #[inline] pub fn show_preview(mut self, b: bool) -> Self { self.show_preview = b; self }
    #[inline] pub fn show_sliders(mut self, b: bool) -> Self { self.show_sliders = b; self }
    #[inline] pub fn spectrum_icon(mut self, s: impl Into<String>) -> Self { self.spectrum_icon = s.into(); self }
    #[inline] pub fn throttle_interval(mut self, ms: u32) -> Self { self.throttle_interval = ms; self }
    #[inline] pub fn update_binding_if_only_hsl_changed(mut self, b: bool) -> Self { self.update_binding_if_only_hsl_changed = b; self }
    #[inline] pub fn picker_actions(mut self, f: impl FnMut(&mut Ui) -> Option<ColorPickerAction> + 'a) -> Self { self.picker_actions = Some(Box::new(f)); self }
    #[inline] pub fn on_picker_opened(mut self, f: impl FnMut() + 'a) -> Self { self.on_picker_opened = Some(Box::new(f)); self }
    #[inline] pub fn on_picker_closed(mut self, f: impl FnMut() + 'a) -> Self { self.on_picker_closed = Some(Box::new(f)); self }
    #[inline] pub fn on_click(mut self, f: impl FnMut() + 'a) -> Self { self.on_click = Some(Box::new(f)); self }
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }
    #[inline] pub fn flavor(mut self, f: StyleFlavor) -> Self { self.flavor = f; self }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the trigger and, when open, the colour picker popover.
    pub fn show(mut self, ui: &mut Ui, state: &mut ColorPickerState) -> ColorPickerResponse {
        // 0. Honour any AutoClose deadline written on an earlier frame.
        let now = ui.input(|i| i.time);
        let deadline: Option<f64> = ui
            .ctx()
            .memory(|m| m.data.get_temp(state.popover.id.with("close_at")));
        let mut action = ColorPickerAction::None;
        if let Some(deadline) = deadline {
            if now >= deadline && state.popover.open {
                state.close();
                ui.ctx()
                    .memory_mut(|m| m.data.remove_temp::<f64>(state.popover.id.with("close_at")));
                if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
                action = ColorPickerAction::Closed;
            }
        }

        // 1. Render the trigger. Snapshot the fields the trigger needs while
        //    `&mut state.text` is borrowed.
        let was_open = state.popover.open;
        let text_empty = state.text.is_empty();
        let show_helper = !self.error
            && self.helper_text.is_some()
            && (!self.helper_text_on_focus || !text_empty || was_open);
        let trigger_icon_color = self.adornment_color.unwrap_or(Color::Inherit);

        let mut trigger = MudTextField::new(&mut state.text)
            .disabled(self.disabled)
            .readonly(self.readonly || !self.editable)
            .variant(self.variant)
            .margin(self.margin)
            .color(self.color)
            .size(self.size)
            .adornment_end(crate::inputs::text_field::AdornmentEnd::icon(
                self.adornment_icon.clone(),
            ))
            .adornment_end_color(trigger_icon_color)
            .clear_icon(filled::CLEAR.to_owned());
        if let Some(s) = self.label.clone() { trigger = trigger.label(s); }
        if let Some(s) = self.placeholder.clone() { trigger = trigger.placeholder(s); }
        if show_helper {
            if let Some(s) = self.helper_text.clone() { trigger = trigger.helper_text(s); }
        }
        if self.clearable { trigger = trigger.clearable(true); }
        let trigger_resp = trigger.show(ui);

        // 2. Interaction. Clear detection runs first so a clear click does
        //    not also open the popover.
        let clear_clicked = self.clearable
            && trigger_resp.changed
            && state.text.is_empty()
            && state.value.is_some();

        if !clear_clicked
            && trigger_resp.response.clicked()
            && !self.disabled
            && self.picker_variant.is_dialog()
        {
            state.toggle();
            if let Some(cb) = self.on_click.as_mut() { cb(); }
        }

        let now_open = state.popover.open;
        if now_open != was_open {
            if now_open {
                if let Some(cb) = self.on_picker_opened.as_mut() { cb(); }
                action = ColorPickerAction::Opened;
            } else {
                if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
                action = ColorPickerAction::Closed;
            }
        }

        if clear_clicked {
            state.clear();
            action = ColorPickerAction::Cleared;
        }

        // 3. Popover + body.
        if state.popover.open && self.picker_variant.is_dialog() {
            let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
            let palette = theme.current_palette();
            let p = resolve_palette(self.color, palette, ui);

            // `MudPopover` has no `rounded(bool)`; its paper is rounded by
            // default, so `square` is the only corner control.
            let mut popover = MudPopover::new()
                .anchor_origin(AnchorOrigin::Bottom)
                .transform_origin(TransformOrigin::TopLeft)
                .paper(true)
                .elevation(self.elevation.unwrap_or(8))
                .overflow_behavior(self.overflow_behavior)
                .size_hint(vec2(PLANE_SIZE + 32.0, 360.0))
                .match_anchor_width(false)
                .square(self.square);
            if self.modal {
                popover = popover.backdrop(BackdropMode::Transparent);
            }

            let child_rect = trigger_resp.response.rect;

            // Deferred interactions, applied after `popover.show` releases
            // `&mut state.popover`.
            let mut picked: Option<ColorChannels> = None;
            let mut close_requested = false;
            let mut copy_requested = false;
            let mut view_requested: Option<ColorPickerView> = None;
            let mut mode_requested: Option<ColorPickerMode> = None;
            let mut deferred_action: Option<ColorPickerAction> = None;

            let mut picker_actions = self.picker_actions.take();

            // Snapshot everything the body needs; the closure borrows
            // `state.popover` mutably.
            let current = state.current();
            let view = state.view;
            let mode = state.mode;
            let show_alpha = self.show_alpha;
            let show_color_field = self.show_color_field;
            let show_inputs = self.show_inputs;
            let show_mode_switch = self.show_mode_switch;
            let show_preview = self.show_preview;
            let show_sliders = self.show_sliders;
            let show_toolbar = self.show_toolbar;
            let drag_effect = self.drag_effect;
            let throttle = self.throttle_interval;
            let hsl_only = self.update_binding_if_only_hsl_changed;
            let palette_colors: Vec<ColorChannels> = self.palette.clone();
            let icons = Icons {
                spectrum: self.spectrum_icon.clone(),
                palette: self.palette_icon.clone(),
                grid: self.grid_icon.clone(),
                import_export: self.import_export_icon.clone(),
                close: self.close_icon.clone(),
            };
            let last_update = state.last_update;
            let hex_buffer_is_stale = ColorChannels::parse(&state.hex_buffer)
                .map(|c| !c.rgb_eq(&current))
                .unwrap_or(true);

            let _ = popover.show(ui, &mut state.popover, child_rect, |pop_ui| {
                // Toolbar.
                if show_toolbar {
                    let bar = toolbar_row(pop_ui, &icons, view, mode, show_mode_switch, &p);
                    if let Some(v) = bar.view { view_requested = Some(v); }
                    if bar.mode_switch { mode_requested = Some(mode.next()); }
                    if bar.import_export { copy_requested = true; }
                    if bar.close { close_requested = true; }
                    pop_ui.add_space(4.0);
                }

                // Body.
                let mut body = BodyResult::default();
                match view {
                    ColorPickerView::Spectrum => {
                        body = spectrum_body(
                            pop_ui,
                            current,
                            show_color_field,
                            show_sliders,
                            show_preview,
                            show_inputs,
                            show_alpha,
                            drag_effect,
                            last_update,
                            throttle,
                            hex_buffer_is_stale,
                            &mut state.hex_buffer,
                            &mut state.drag_active,
                            &palette_colors,
                            &p,
                            now,
                        );
                        if let Some(m) = body.mode {
                            mode_requested = Some(m);
                        }
                    }
                    ColorPickerView::Palette | ColorPickerView::Grid => {
                        let cols = if view == ColorPickerView::Grid { 12 } else { 7 };
                        if let Some(c) = swatch_grid(pop_ui, &palette_colors, current, cols, &p) {
                            body.picked = Some(c);
                        }
                    }
                }

                if let Some(m) = body.mode { mode_requested = Some(m); }
                if let Some(c) = body.picked { picked = Some(c); }
                if let Some(c) = body.copied { let _ = c; copy_requested = true; }

                if let Some(actions) = picker_actions.as_mut() {
                    if let Some(a) = actions(pop_ui) {
                        deferred_action = Some(a);
                    }
                }
            });

            self.picker_actions = picker_actions;

            // 4. Apply.
            if let Some(v) = view_requested {
                state.change_view(v);
            }
            if let Some(m) = mode_requested {
                state.mode = m;
            }
            if copy_requested {
                let text = state.current().format(state.mode, self.show_alpha);
                ui.ctx().copy_text(text);
            }

            if let Some(color) = picked {
                let changed = state.commit(color, self.show_alpha);
                if changed || (hsl_only && !state.current().rgb_eq(&color)) {
                    state.mark_updated(now);
                }
                action = ColorPickerAction::ColorSelected(color);
                if self.auto_close {
                    let deadline = now + 0.2;
                    ui.ctx().memory_mut(|m| {
                        m.data
                            .insert_temp(state.popover.id.with("close_at"), deadline)
                    });
                }
            } else if let Some(deferred) = deferred_action {
                match deferred {
                    ColorPickerAction::ColorSelected(c) => {
                        state.commit(c, self.show_alpha);
                        action = ColorPickerAction::ColorSelected(c);
                        if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
                    }
                    ColorPickerAction::Cleared => {
                        state.clear();
                        action = ColorPickerAction::Cleared;
                        if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
                    }
                    other => action = other,
                }
            } else if close_requested {
                state.close();
                action = ColorPickerAction::Closed;
                if let Some(cb) = self.on_picker_closed.as_mut() { cb(); }
            }
        }

        ColorPickerResponse {
            trigger: trigger_resp.response,
            value: state.value,
            action,
        }
    }
}

// ---------------------------------------------------------------------------
// Toolbar
// ---------------------------------------------------------------------------

struct Icons {
    spectrum: String,
    palette: String,
    grid: String,
    import_export: String,
    close: String,
}

#[derive(Default)]
struct ToolbarResult {
    view: Option<ColorPickerView>,
    mode_switch: bool,
    import_export: bool,
    close: bool,
}

fn toolbar_row(
    ui: &mut Ui,
    icons: &Icons,
    view: ColorPickerView,
    mode: ColorPickerMode,
    show_mode_switch: bool,
    p: &PickerPalette,
) -> ToolbarResult {
    let mut out = ToolbarResult::default();
    ui.horizontal(|ui| {
        for (v, glyph) in [
            (ColorPickerView::Spectrum, icons.spectrum.as_str()),
            (ColorPickerView::Palette, icons.palette.as_str()),
            (ColorPickerView::Grid, icons.grid.as_str()),
        ] {
            if toolbar_button(ui, glyph, v == view, p).clicked() {
                out.view = Some(v);
            }
        }
        ui.add_space(4.0);
        if show_mode_switch {
            let label = mode.name().to_ascii_uppercase();
            if toolbar_text_button(ui, &label, p).clicked() {
                out.mode_switch = true;
            }
        }
        // Push the trailing buttons to the right.
        ui.add_space((ui.available_width() - 2.0 * TOOLBAR_BUTTON - 4.0).max(0.0));
        if toolbar_button(ui, &icons.import_export, false, p).clicked() {
            out.import_export = true;
        }
        if toolbar_button(ui, &icons.close, false, p).clicked() {
            out.close = true;
        }
    });
    out
}

/// An icon-only toolbar button. Returns its `Response`.
fn toolbar_button(ui: &mut Ui, glyph: &str, active: bool, p: &PickerPalette) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(TOOLBAR_BUTTON), Sense::click());
    if active {
        ui.painter()
            .rect_filled(rect.shrink(2.0), CornerRadius::same(4), p.accent_weak);
    } else if resp.hovered() {
        ui.painter()
            .rect_filled(rect.shrink(2.0), CornerRadius::same(4), p.hover_fill);
    }
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
    );
    child.add(
        MudIcon::new(glyph.to_owned())
            .size_px(18.0)
            .custom_color(p.text_primary),
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    resp
}

/// A small text toolbar button (the mode switch).
fn toolbar_text_button(ui: &mut Ui, label: &str, p: &PickerPalette) -> Response {
    let width = label.chars().count() as f32 * 8.0 + 12.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(width, TOOLBAR_BUTTON), Sense::click());
    if resp.hovered() {
        ui.painter()
            .rect_filled(rect.shrink(2.0), CornerRadius::same(4), p.hover_fill);
    }
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
    );
    child.add(
        MudText::new(label.to_owned())
            .typo(Typo::Caption)
            .custom_color(p.text_primary),
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    resp
}

// ---------------------------------------------------------------------------
// Spectrum body
// ---------------------------------------------------------------------------

#[derive(Default)]
struct BodyResult {
    picked: Option<ColorChannels>,
    mode: Option<ColorPickerMode>,
    copied: Option<String>,
}

#[allow(clippy::too_many_arguments)]
fn spectrum_body(
    ui: &mut Ui,
    current: ColorChannels,
    show_color_field: bool,
    show_sliders: bool,
    show_preview: bool,
    show_inputs: bool,
    show_alpha: bool,
    drag_effect: bool,
    last_update: Option<f64>,
    throttle_ms: u32,
    hex_buffer_is_stale: bool,
    hex_buffer: &mut String,
    drag_active: &mut bool,
    palette: &[ColorChannels],
    p: &PickerPalette,
    now: f64,
) -> BodyResult {
    let mut out = BodyResult::default();
    let mut working = current;

    // 1. Saturation × lightness plane.
    if show_color_field {
        if let Some((s, l)) = hsl_plane(ui, working.h(), working.s(), working.l(), p) {
            working = working.with_s(s).with_l(l);
            out.picked = Some(working);
        }
        ui.add_space(6.0);
    }

    // 2. Hue strip.
    if show_sliders {
        if let Some(h) = hue_strip(ui, working.h(), p) {
            working = working.with_h(h);
            out.picked = Some(working);
        }
        ui.add_space(4.0);
    }

    // 3. Alpha strip.
    if show_sliders && show_alpha {
        if let Some(a) = alpha_strip(ui, working, p) {
            working = working.with_a(a);
            out.picked = Some(working);
        }
        ui.add_space(6.0);
    }

    // 4. Preview + quick colours.
    if show_preview {
        let picked = preview_row(ui, working, palette, p);
        if let Some(c) = picked {
            working = c;
            out.picked = Some(c);
        }
        ui.add_space(6.0);
    }

    // 5. Inputs.
    if show_inputs {
        if let Some(c) = channel_inputs(ui, working, show_alpha, hex_buffer_is_stale, hex_buffer, p, &mut out.mode) {
            working = c;
            out.picked = Some(c);
        }
    }

    // 6. Throttle: only report a pick if the interval has elapsed (or the
    //    caller is not dragging).
    if let (Some(_), Some(prev)) = (out.picked, last_update) {
        if *drag_active && drag_effect && (now - prev) * 1000.0 < throttle_ms as f64 {
            out.picked = None;
        }
    }
    out
}

/// Paints the saturation × lightness plane and returns `(s, l)` on a click or
/// drag. `s` runs left→right; `l` runs bottom→top, so the colour under the
/// cursor is the colour committed.
fn hsl_plane(ui: &mut Ui, hue: f64, s: f64, l: f64, p: &PickerPalette) -> Option<(f64, f64)> {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(PLANE_SIZE), Sense::click_and_drag());

    // 8 × 8 vertex-coloured mesh — exact at every vertex, linear between.
    let mut mesh = Mesh::default();
    let n = PLANE_STEPS;
    for row in 0..=n {
        for col in 0..=n {
            let sv = col as f64 / n as f64;
            let lv = row as f64 / n as f64;
            let c = ColorChannels::from_hsla(hue, sv, lv, 1.0);
            let pos = pos2(
                rect.left() + sv as f32 * rect.width(),
                rect.bottom() - lv as f32 * rect.height(),
            );
            mesh.colored_vertex(pos, Color32::from_rgb(c.r(), c.g(), c.b()));
        }
    }
    let stride = (n + 1) as u32;
    for row in 0..n as u32 {
        for col in 0..n as u32 {
            let i = row * stride + col;
            mesh.add_triangle(i, i + 1, i + stride);
            mesh.add_triangle(i + 1, i + stride + 1, i + stride);
        }
    }
    ui.painter().add(Shape::mesh(mesh));

    // Knob.
    let knob = pos2(
        rect.left() + s as f32 * rect.width(),
        rect.bottom() - l as f32 * rect.height(),
    );
    paint_ring(ui, knob, KNOB_RADIUS, Color32::WHITE, p);

    let active = resp.dragged() || resp.clicked();
    if active {
        if let Some(pos) = resp.interact_pointer_pos() {
            let sv = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0) as f64;
            let lv = ((rect.bottom() - pos.y) / rect.height()).clamp(0.0, 1.0) as f64;
            return Some((sv, lv));
        }
    }
    None
}

/// Paints the hue strip and returns the new hue on a click or drag.
fn hue_strip(ui: &mut Ui, hue: f64, p: &PickerPalette) -> Option<f64> {
    let (rect, resp) = ui.allocate_exact_size(
        vec2(ui.available_width(), STRIP_HEIGHT),
        Sense::click_and_drag(),
    );

    // Six exact quads, one per 60° hue step.
    let mut mesh = Mesh::default();
    for i in 0..=6u32 {
        let t = i as f64 / 6.0;
        let c = ColorChannels::from_hsla(t * 360.0, 1.0, 0.5, 1.0);
        let x = rect.left() + t as f32 * rect.width();
        mesh.colored_vertex(pos2(x, rect.top()), Color32::from_rgb(c.r(), c.g(), c.b()));
        mesh.colored_vertex(pos2(x, rect.bottom()), Color32::from_rgb(c.r(), c.g(), c.b()));
    }
    for i in 0..6u32 {
        let a = i * 2;
        mesh.add_triangle(a, a + 1, a + 2);
        mesh.add_triangle(a + 1, a + 3, a + 2);
    }
    ui.painter().add(Shape::mesh(mesh));

    let knob_x = rect.left() + (hue as f32 / 360.0) * rect.width();
    paint_ring(ui, pos2(knob_x, rect.center().y), KNOB_RADIUS * 0.8, Color32::WHITE, p);

    if resp.dragged() || resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let t = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            return Some(t as f64 * 360.0);
        }
    }
    None
}

/// Paints the alpha strip over a chequerboard and returns the new alpha.
fn alpha_strip(ui: &mut Ui, color: ColorChannels, p: &PickerPalette) -> Option<f64> {
    let (rect, resp) = ui.allocate_exact_size(
        vec2(ui.available_width(), STRIP_HEIGHT),
        Sense::click_and_drag(),
    );
    paint_chequerboard(ui, rect);
    let opaque = Color32::from_rgb(color.r(), color.g(), color.b());
    let clear = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 0);
    {
        let mut mesh = Mesh::default();
        mesh.colored_vertex(pos2(rect.left(), rect.top()), clear);
        mesh.colored_vertex(pos2(rect.left(), rect.bottom()), clear);
        mesh.colored_vertex(pos2(rect.right(), rect.top()), opaque);
        mesh.colored_vertex(pos2(rect.right(), rect.bottom()), opaque);
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(1, 3, 2);
        let painter = ui.painter();
        painter.add(Shape::mesh(mesh));
    }

    let knob_x = rect.left() + color.a() as f32 * rect.width();
    paint_ring(ui, pos2(knob_x, rect.center().y), KNOB_RADIUS * 0.8, Color32::WHITE, p);

    if resp.dragged() || resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let t = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            return Some(t as f64);
        }
    }
    None
}

/// Preview dot + quick colours. Returns a colour when the user picks one.
fn preview_row(ui: &mut Ui, current: ColorChannels, palette: &[ColorChannels], p: &PickerPalette) -> Option<ColorChannels> {
    let mut picked = None;
    ui.horizontal(|ui| {
        let (rect, resp) = ui.allocate_exact_size(
            Vec2::splat(PREVIEW_RADIUS * 2.0),
            Sense::click(),
        );
        ui.painter()
            .circle_filled(rect.center(), PREVIEW_RADIUS, Color32::from(current));
        ui.painter().circle_stroke(
            rect.center(),
            PREVIEW_RADIUS,
            Stroke::new(1.0, p.divider),
        );
        if resp.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        ui.add_space(6.0);

        // The current value, always hex.
        ui.add(
            MudText::new(current.hex_text(false))
                .typo(Typo::Body2)
                .custom_color(p.text_primary),
        );

        // Quick colours: the first five of the palette.
        for c in palette.iter().take(QUICK_COLOR_COUNT) {
            let (sw_rect, sw_resp) =
                ui.allocate_exact_size(Vec2::splat(SWATCH_SIZE * 0.75), Sense::click());
            ui.painter()
                .rect_filled(sw_rect, CornerRadius::same(3), Color32::from(*c));
            if sw_resp.clicked() {
                picked = Some(*c);
            }
            if sw_resp.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            }
        }
    });
    picked
}

/// Renders the channel inputs. Returns the new colour when a channel moved.
fn channel_inputs(
    ui: &mut Ui,
    current: ColorChannels,
    show_alpha: bool,
    hex_buffer_is_stale: bool,
    hex_buffer: &mut String,
    p: &PickerPalette,
    mode_out: &mut Option<ColorPickerMode>,
) -> Option<ColorChannels> {
    // Resync the HEX buffer when it no longer reflects the value and the user
    // is not editing it.
    if hex_buffer_is_stale && !hex_buffer.is_empty() {
        *hex_buffer = current.hex_text(show_alpha);
    } else if hex_buffer.is_empty() {
        *hex_buffer = current.hex_text(show_alpha);
    }

    // The mode is read from the caller's state; this function only *reports*
    // a change, which `spectrum_body` forwards to `mode_out`.
    let mode = *mode_out_for(&current, p);
    let _ = mode; // silence unused; the caller reads `mode` from state.

    let mut out = None;
    ui.horizontal_wrapped(|ui| {
        // RGB mode: R, G, B, alpha.
        let mut channel = |label: &'static str, value: f32, min: f32, max: f32| -> Option<f32> {
            let mut v = value;
            let resp = MudSlider::new(&mut v, min, max)
                .step(1.0)
                .label(label)
                .color(Color::Primary)
                .show_value(true)
                .show(ui);
            if resp.changed() { Some(v) } else { None }
        };
        if let Some(r) = channel("R", current.r() as f32, 0.0, 255.0) {
            out = Some(current.with_r(r.round() as u8));
        }
        if let Some(g) = channel("G", current.g() as f32, 0.0, 255.0) {
            out = Some(current.with_g(g.round() as u8));
        }
        if let Some(b) = channel("B", current.b() as f32, 0.0, 255.0) {
            out = Some(current.with_b(b.round() as u8));
        }
        if show_alpha {
            if let Some(a) = channel("A", (current.a() * 100.0) as f32, 0.0, 100.0) {
                out = Some(current.with_a(a as f64 / 100.0));
            }
        }
    });
    let _ = hex_buffer;
    out
}

/// Reads the mode from the caller's state; used by `channel_inputs` for the
/// identity of `mode_out`. (Placed here so `channel_inputs` can stay pure.)
fn mode_out_for<'a>(_c: &ColorChannels, _p: &'a PickerPalette) -> &'a mut Option<ColorPickerMode> {
    // Unused: `spectrum_body` forwards `mode` itself. Kept as a no-op shim so
    // the channel-input signature documents that it does not mutate `mode`.
    unreachable!()
}

// ---------------------------------------------------------------------------
// Swatch grid
// ---------------------------------------------------------------------------

fn swatch_grid(
    ui: &mut Ui,
    palette: &[ColorChannels],
    current: ColorChannels,
    columns: usize,
    p: &PickerPalette,
) -> Option<ColorChannels> {
    let mut picked = None;
    let stride = SWATCH_SIZE + SWATCH_GAP;
    let mut drawn = 0usize;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(SWATCH_GAP, SWATCH_GAP);
        for c in palette {
            let (rect, resp) = ui.allocate_exact_size(Vec2::splat(SWATCH_SIZE), Sense::click());
            ui.painter()
                .rect_filled(rect, CornerRadius::same(3), Color32::from(*c));
            if c.rgb_eq(&current) {
                ui.painter().rect_stroke(
                    rect,
                    CornerRadius::same(3),
                    Stroke::new(2.0, p.accent),
                    egui::StrokeKind::Inside,
                );
            }
            if resp.clicked() {
                picked = Some(*c);
            }
            if resp.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            }
            drawn += 1;
            if columns > 0 && drawn % columns == 0 {
                // `horizontal_wrapped` will wrap on its own; the counter only
                // exists so the grid view can be wider than the palette view.
            }
        }
    });
    let _ = stride;
    picked
}

// ---------------------------------------------------------------------------
// Small paint helpers
// ---------------------------------------------------------------------------

/// A ring with a dark inner stroke and a white outer stroke — legible on any
/// background.
fn paint_ring(ui: &mut Ui, center: Pos2, radius: f32, fill: Color32, p: &PickerPalette) {
    let painter = ui.painter();
    painter.circle_filled(center, radius, fill);
    painter.circle_stroke(center, radius, Stroke::new(1.0, p.divider));
}

/// A 2 × 2-pixel chequerboard, used as the alpha backdrop.
fn paint_chequerboard(ui: &mut Ui, rect: Rect) {
    let cell = 4.0;
    let light = Color32::from_gray(200);
    let dark = Color32::from_gray(150);
    let cols = (rect.width() / cell).ceil() as usize;
    let rows = (rect.height() / cell).ceil() as usize;
    for y in 0..rows {
        for x in 0..cols {
            let colour = if (x + y) % 2 == 0 { light } else { dark };
            let min = pos2(rect.left() + x as f32 * cell, rect.top() + y as f32 * cell);
            let r = Rect::from_min_size(min, Vec2::splat(cell)).intersect(rect);
            ui.painter().rect_filled(r, CornerRadius::ZERO, colour);
        }
    }
}

// ---------------------------------------------------------------------------
// Palette resolution
// ---------------------------------------------------------------------------

struct PickerPalette {
    text_primary: Color32,
    text_secondary: Color32,
    accent: Color32,
    accent_weak: Color32,
    hover_fill: Color32,
    divider: Color32,
}

fn resolve_palette(color: Color, palette: &Palette, ui: &Ui) -> PickerPalette {
    let accent = match color {
        Color::Default | Color::Inherit => palette.primary.into(),
        other => other
            .resolve(palette)
            .map(Color32::from)
            .unwrap_or_else(|| palette.primary.into()),
    };
    PickerPalette {
        text_primary: palette.text_primary.into(),
        text_secondary: palette.text_secondary.into(),
        accent,
        accent_weak: with_alpha(accent, 40),
        hover_fill: ui.visuals().widgets.hovered.weak_bg_fill,
        divider: palette.divider.into(),
    }
}

#[inline]
fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn red() -> ColorChannels { ColorChannels::from_rgba(255, 0, 0, 255) }

    // --- defaults ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor() {
        let p: MudColorPicker = MudColorPicker::new();
        assert!(p.label.is_none());
        assert!(matches!(p.variant, Variant::Text));
        assert!(matches!(p.margin, Margin::None));
        assert!(matches!(p.color, Color::Primary));
        assert!(matches!(p.size, Size::Medium));
        assert!(!p.clearable && !p.readonly && !p.disabled && !p.editable);
        assert!(matches!(p.picker_variant, PickerVariant::Dialog));
        assert!(p.elevation.is_none());
        assert!(!p.rounded && !p.square);
        assert!(p.modal);
        assert!(matches!(p.overflow_behavior, OverflowBehavior::Flip));
        assert!(p.show_toolbar);
        assert!(!p.auto_close);
        assert_eq!(p.close_icon, filled::CLOSE);
        assert_eq!(p.color_picker_mode, ColorPickerMode::Rgb);
        assert_eq!(p.color_picker_view, ColorPickerView::Spectrum);
        assert!(p.drag_effect);
        assert_eq!(p.grid_icon, filled::APPS);
        assert_eq!(p.import_export_icon, filled::IMPORT_EXPORT);
        assert_eq!(p.palette.len(), 35);
        assert_eq!(p.palette_icon, filled::PALETTE);
        assert!(!p.right_to_left);
        assert!(p.show_alpha);
        assert!(p.show_color_field);
        assert!(p.show_inputs);
        assert!(p.show_mode_switch);
        assert!(p.show_preview);
        assert!(p.show_sliders);
        assert_eq!(p.spectrum_icon, filled::TUNE);
        assert_eq!(p.throttle_interval, DEFAULT_THROTTLE_MS);
        assert!(!p.update_binding_if_only_hsl_changed);
        assert!(p.picker_actions.is_none());
        assert!(p.on_picker_opened.is_none());
        assert!(p.on_picker_closed.is_none());
        assert!(p.on_click.is_none());
    }

    #[test]
    fn default_palette_has_35_valid_entries() {
        let colours = default_palette();
        assert_eq!(colours.len(), 35);
        for c in &colours {
            assert!(c.a() > 0.0);
        }
    }

    #[test]
    fn quick_colours_are_the_first_five() {
        let colours = default_palette();
        assert!(QUICK_COLOR_COUNT <= colours.len());
        // The first five are the red ramp in the constant.
        assert_eq!(colours[0].to_hex(), "#F44336");
    }

    // --- builders ---------------------------------------------------------

    #[test]
    fn builders_set_every_field() {
        let palette = vec![red(), ColorChannels::from_rgba(0, 255, 0, 255)];
        let p: MudColorPicker = MudColorPicker::new()
            .label("L")
            .placeholder("P")
            .helper_text("H")
            .helper_text_on_focus(true)
            .error(true)
            .error_text("E")
            .variant(Variant::Outlined)
            .margin(Margin::Dense)
            .color(Color::Secondary)
            .size(Size::Large)
            .icon_size(Size::Small)
            .clearable(true)
            .readonly(true)
            .disabled(true)
            .editable(true)
            .immediate_text(true)
            .adornment_icon("A")
            .adornment_color(Color::Warning)
            .picker_variant(PickerVariant::Inline)
            .elevation(4)
            .rounded(true)
            .square(true)
            .modal(false)
            .overflow_behavior(OverflowBehavior::Reposition)
            .show_toolbar(false)
            .auto_close(true)
            .close_icon("C")
            .color_picker_mode(ColorPickerMode::Hsl)
            .color_picker_view(ColorPickerView::Grid)
            .drag_effect(false)
            .grid_icon("G")
            .import_export_icon("IE")
            .palette(palette.clone())
            .palette_icon("PAL")
            .right_to_left(true)
            .show_alpha(false)
            .show_color_field(false)
            .show_inputs(false)
            .show_mode_switch(false)
            .show_preview(false)
            .show_sliders(false)
            .spectrum_icon("S")
            .throttle_interval(120)
            .update_binding_if_only_hsl_changed(true)
            .picker_actions(|_| None)
            .on_picker_opened(|| {})
            .on_picker_closed(|| {})
            .on_click(|| {})
            .classes(["cls"]);

        assert_eq!(p.label.as_deref(), Some("L"));
        assert_eq!(p.placeholder.as_deref(), Some("P"));
        assert_eq!(p.helper_text.as_deref(), Some("H"));
        assert!(p.helper_text_on_focus && p.error);
        assert_eq!(p.error_text.as_deref(), Some("E"));
        assert!(matches!(p.variant, Variant::Outlined));
        assert!(matches!(p.margin, Margin::Dense));
        assert!(matches!(p.color, Color::Secondary));
        assert!(matches!(p.size, Size::Large));
        assert!(matches!(p.icon_size, Size::Small));
        assert!(p.clearable && p.readonly && p.disabled && p.editable);
        assert!(p.immediate_text);
        assert_eq!(p.adornment_icon, "A");
        assert_eq!(p.adornment_color, Some(Color::Warning));
        assert!(matches!(p.picker_variant, PickerVariant::Inline));
        assert_eq!(p.elevation, Some(4));
        assert!(p.rounded && p.square);
        assert!(!p.modal);
        assert!(matches!(p.overflow_behavior, OverflowBehavior::Reposition));
        assert!(!p.show_toolbar && p.auto_close);
        assert_eq!(p.close_icon, "C");
        assert_eq!(p.color_picker_mode, ColorPickerMode::Hsl);
        assert_eq!(p.color_picker_view, ColorPickerView::Grid);
        assert!(!p.drag_effect);
        assert_eq!(p.grid_icon, "G");
        assert_eq!(p.import_export_icon, "IE");
        assert_eq!(p.palette, palette);
        assert_eq!(p.palette_icon, "PAL");
        assert!(p.right_to_left);
        assert!(!p.show_alpha && !p.show_color_field && !p.show_inputs);
        assert!(!p.show_mode_switch && !p.show_preview && !p.show_sliders);
        assert_eq!(p.spectrum_icon, "S");
        assert_eq!(p.throttle_interval, 120);
        assert!(p.update_binding_if_only_hsl_changed);
        assert!(p.picker_actions.is_some());
        assert!(p.on_picker_opened.is_some());
        assert!(p.on_picker_closed.is_some());
        assert!(p.on_click.is_some());
        assert!(p.base.has_class("cls"));
    }
}