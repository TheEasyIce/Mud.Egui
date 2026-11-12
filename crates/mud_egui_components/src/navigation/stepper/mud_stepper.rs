//! `MudStepper` — a wizard that guides the user through a series of steps.
//!
//! Port of `MudStepper.razor.cs` (MudBlazor 8.13.0).
//!
//! # MudBlazor parity — `MudStepper`
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `ActionContent`     | [`MudStepper::action_content`] | `None` |
//! | `ActiveIndex`       | [`StepperState::active_index`] | `0` |
//! | `ActiveIndexChanged`| [`MudStepper::on_active_index_changed`] | `None` |
//! | `CenterLabels`      | [`MudStepper::center_labels`] | `false` |
//! | `ChildContent`      | closure to [`MudStepper::show`] | required |
//! | `CompleteButtonIcon`| [`MudStepper::complete_button_icon`] | `DONE` |
//! | `CompletedContent`  | [`MudStepper::completed_content`] | `None` |
//! | `CompletedStepColor`| [`MudStepper::completed_step_color`] | `Primary` |
//! | `ConnectorTemplate` | [`MudStepper::connector_template`] | `None` |
//! | `CurrentStepColor`  | [`MudStepper::current_step_color`] | `Primary` |
//! | `ErrorStepColor`    | [`MudStepper::error_step_color`] | `Error` |
//! | `LabelTemplate`     | [`MudStepper::label_template`] | `None` |
//! | `NextButtonIcon`    | [`MudStepper::next_button_icon`] | `NAVIGATE_NEXT` |
//! | `NonLinear`         | [`MudStepper::non_linear`] | `false` |
//! | `OnPreviewInteraction` | [`MudStepper::on_preview_interaction`] | `None` |
//! | `PreviousButtonIcon`| [`MudStepper::previous_button_icon`] | `NAVIGATE_BEFORE` |
//! | `ResetButtonIcon`   | [`MudStepper::reset_button_icon`] | `FIRST_PAGE` |
//! | `Ripple`            | [`MudStepper::ripple`] | `false` |
//! | `ScrollableNavigation` | [`MudStepper::scrollable_navigation`] | `true` |
//! | `ShowResetButton`   | [`MudStepper::show_reset_button`] | `false` |
//! | `SkipButtonIcon`    | [`MudStepper::skip_button_icon`] | `SKIP_NEXT` |
//! | `StepCompleteIcon`  | [`MudStepper::step_complete_icon`] | `DONE` |
//! | `StepErrorIcon`     | [`MudStepper::step_error_icon`] | `PRIORITY_HIGH` |
//! | `TitleTemplate`     | [`MudStepper::title_template`] | `None` |
//! | `Vertical`          | [`MudStepper::vertical`] | `false` |
//!
//! ### Extensions (not in MudBlazor)
//!
//! | Extension | Description |
//! |-----------|-------------|
//! | [`StepperState`] | Caller-owned active index + per-step flags. |
//! | [`StepperResponse`] | Rich return value with `action` and `active_index`. |
//! | [`StepperAction`] | Reports what was clicked this frame. |
//! | [`StepPhase`] | Resolved visual phase, passed to templates. |
//!
//! ### Get-only properties not ported
//!
//! `ActiveStep`, `Steps`, `CanGoToNextStep`, `IsCompleted`,
//! `IsCurrentStepSkippable`, `PreviousStepEnabled`, `ShowCompleteInsteadOfNext`
//! are exposed as **pure helper functions** (`is_completed`,
//! `can_go_to_next`, `previous_step_enabled`, `is_current_skippable`,
//! `show_complete_instead_of_next`) so the caller can call them with the state
//! they already own.
//!
//! # Nav buttons
//!
//! MudBlazor renders **icon-only** action buttons by default. The port uses
//! [`MudButton`] with `start_icon(..)` + label (`"Previous"`, `"Skip"`,
//! `"Next"`, `"Complete"`, `"Reset"`) so the labels are accessible and the
//! buttons match the rest of the component library. To reproduce MudBlazor's
//! icon-only look exactly, use [`action_content`](MudStepper::action_content).
//!
//! # Text and icons
//!
//! Every label renders through [`MudText`], every glyph through [`MudIcon`],
//! every nav button through [`MudButton`].

use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Id, Rect, Response, Sense, Stroke,
    StrokeKind, Ui, Vec2,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, Size, StepAction, StyleFlavor, Typo, Variant};
use mud_egui_icons_material::filled;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::typography::mud_text::MudText;
use crate::buttons::button::mud_button::MudButton;

use super::mud_step::MudStep;
use super::step_flags::StepFlags;
use super::step_phase::StepPhase;
use super::stepper_action::StepperAction;
use super::stepper_interaction::StepperInteraction;
use super::stepper_state::StepperState;

// =============================================================================
// Constants
// =============================================================================

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Diameter of a step circle.
pub const STEP_DIAMETER: f32 = 32.0;

/// Diameter of a step circle for `Size::Small`.
pub const STEP_DIAMETER_SMALL: f32 = 24.0;

/// Diameter of a step circle for `Size::Large`.
pub const STEP_DIAMETER_LARGE: f32 = 40.0;

/// Thickness of the connector line between steps.
pub const CONNECTOR_THICKNESS: f32 = 2.0;

/// Vertical gap between the circle and its label.
const LABEL_GAP: f32 = 6.0;

/// Gap between adjacent step columns (horizontal) or rows (vertical).
const STEP_GAP: f32 = 8.0;

/// Vertical gap between the header, the content and the nav row.
const SECTION_GAP: f32 = 16.0;

/// Gap between nav buttons.
const NAV_GAP: f32 = 8.0;

/// Approximate label height when only the title is present.
const LABEL_H_TITLE_ONLY: f32 = 22.0;

/// Approximate label height when both title and secondary text are present.
const LABEL_H_WITH_SECONDARY: f32 = 40.0;

// =============================================================================
// Pure helpers
// =============================================================================

/// Circle diameter for a given [`Size`].
#[inline]
pub fn step_diameter(size: Size) -> f32 {
    match size {
        Size::Small => STEP_DIAMETER_SMALL,
        Size::Large => STEP_DIAMETER_LARGE,
        _ => STEP_DIAMETER,
    }
}

/// Resolves the visual phase of a step.
///
/// Priority: [`Error`](StepPhase::Error) > [`Skipped`](StepPhase::Skipped) >
/// [`Completed`](StepPhase::Completed) > [`Active`](StepPhase::Active) >
/// [`Pending`](StepPhase::Pending).
#[inline]
pub fn step_phase(flags: &StepFlags, is_active: bool) -> StepPhase {
    if flags.has_error {
        StepPhase::Error
    } else if flags.skipped {
        StepPhase::Skipped
    } else if flags.completed {
        StepPhase::Completed
    } else if is_active {
        StepPhase::Active
    } else {
        StepPhase::Pending
    }
}

/// Whether every step in `0..count` is completed.
#[inline]
pub fn is_completed(state: &StepperState, count: usize) -> bool {
    state.is_completed(count)
}

/// Whether the active step's `Next` button should read `Complete`.
///
/// MudBlazor's `ShowCompleteInsteadOfNext` is `true` when the active index is
/// the last step.
#[inline]
pub fn show_complete_instead_of_next(active_index: usize, count: usize) -> bool {
    count > 0 && active_index + 1 >= count
}

/// Whether the user can advance to the next step.
///
/// In linear mode (`non_linear == false`) this is `true` only when the active
/// step is already completed or skipped; in non-linear mode it is always
/// `true` (subject to `on_preview_interaction`).
#[inline]
pub fn can_go_to_next(flags: &StepFlags, non_linear: bool, is_last: bool) -> bool {
    if is_last {
        return false;
    }
    non_linear || flags.completed || flags.skipped
}

/// Whether the `Previous` button is enabled.
#[inline]
pub fn previous_step_enabled(active_index: usize) -> bool {
    active_index > 0
}

/// Whether the current step can be skipped.
#[inline]
pub fn is_current_skippable(step: &MudStep<'_>) -> bool {
    step.skippable && !step.disabled
}

/// Resolves a `Color` against the palette, falling back to `default`.
#[inline]
fn resolve_color(c: Color, palette: &Palette, default: Color32) -> Color32 {
    match c {
        Color::Default | Color::Inherit => default,
        other => other.resolve(palette).unwrap_or(default),
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
// StepView
// =============================================================================

/// A read-only snapshot of one step, passed to the `label_template` /
/// `title_template` closures.
#[derive(Debug)]
pub struct StepView<'s> {
    /// 0-based index of the step.
    pub index: usize,
    /// `MudStep.title`.
    pub title: Option<&'s str>,
    /// `MudStep.secondary_text`.
    pub secondary_text: Option<&'s str>,
    /// Whether the step is the active one.
    pub is_active: bool,
    /// Whether the step is completed.
    pub completed: bool,
    /// Whether the step has an error.
    pub has_error: bool,
    /// Whether the step was skipped.
    pub skipped: bool,
    /// Whether the step is disabled.
    pub disabled: bool,
    /// Whether the step can be skipped.
    pub skippable: bool,
    /// The resolved [`StepPhase`].
    pub phase: StepPhase,
}

// =============================================================================
// StepperUi / StepperTemplates
// =============================================================================

/// Collector passed to [`MudStepper::show`]'s closure.
pub struct StepperUi<'a> {
    pub(crate) steps: Vec<MudStep<'a>>,
}

impl<'a> StepperUi<'a> {
    /// Adds a step.
    pub fn step(&mut self, step: MudStep<'a>) {
        self.steps.push(step);
    }

    /// Adds a step with the given title and no content.
    pub fn text_step(&mut self, title: impl Into<String>) {
        self.steps.push(MudStep::new(title));
    }

    /// Number of collected steps.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Whether no steps have been collected.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

/// Mutable per-frame template bag, taken out of `self` before rendering so
/// disjoint borrows are possible.
struct StepperTemplates<'a> {
    label_template: Option<Box<dyn for<'v> FnMut(&mut Ui, &StepView<'v>) + 'a>>,
    title_template: Option<Box<dyn for<'v> FnMut(&mut Ui, &StepView<'v>) + 'a>>,
    connector_template: Option<Box<dyn FnMut(&mut Ui, bool) + 'a>>,
    action_content: Option<Box<dyn FnMut(&mut Ui, &mut StepperState) + 'a>>,
    completed_content: Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    on_preview: Option<Box<dyn FnMut(&mut StepperInteraction) + 'a>>,
    on_active_index_changed: Option<Box<dyn FnMut(usize) + 'a>>,
}

// =============================================================================
// StepperCfg
// =============================================================================

/// Immutable per-frame rendering configuration.
struct StepperCfg<'p> {
    vertical: bool,
    non_linear: bool,
    center_labels: bool,
    scrollable_navigation: bool,
    show_reset_button: bool,
    ripple: bool,
    circle_diameter: f32,
    completed_color: Color32,
    current_color: Color32,
    error_color: Color32,
    skipped_color: Color32,
    divider: Color32,
    text_primary: Color32,
    text_secondary: Color32,
    text_disabled: Color32,
    surface: Color32,
    contrast: Color32,
    complete_icon: String,
    error_icon: String,
    skip_icon: String,
    prev_icon: String,
    next_icon: String,
    complete_button_icon: String,
    reset_icon: String,
    base_id: Id,
    palette: &'p Palette,
}

// =============================================================================
// MudStepper
// =============================================================================

/// A wizard that guides the user through a series of steps.
#[must_use = "MudStepper does nothing until you call `.show(ui, state, |s| ..)`"]
pub struct MudStepper<'a> {
    // ── Behavior ───────────────────────────────────────────────────────────
    /// Allows users to move between steps arbitrarily (`NonLinear`).
    pub non_linear: bool,
    /// Renders steps vertically (`Vertical`).
    pub vertical: bool,
    /// Centers labels below each circle (`CenterLabels`, horizontal only).
    pub center_labels: bool,
    /// Shows a scroll bar for the step header if needed (`ScrollableNavigation`).
    pub scrollable_navigation: bool,
    /// Shows a button to reset to the first step (`ShowResetButton`).
    pub show_reset_button: bool,
    /// Shows a ripple on click (`Ripple`, only when `non_linear`).
    pub ripple: bool,

    // ── Appearance ─────────────────────────────────────────────────────────
    /// Colour of completed steps (`CompletedStepColor`).
    pub completed_step_color: Color,
    /// Colour of the current step (`CurrentStepColor`).
    pub current_step_color: Color,
    /// Colour of steps with errors (`ErrorStepColor`).
    pub error_step_color: Color,

    /// Icon shown on completed steps (`StepCompleteIcon`).
    pub step_complete_icon: String,
    /// Icon shown on steps with errors (`StepErrorIcon`).
    pub step_error_icon: String,
    /// Icon shown on skipped steps (`SkipButtonIcon`).
    pub skip_button_icon: String,

    /// Icon of the previous button (`PreviousButtonIcon`).
    pub previous_button_icon: String,
    /// Icon of the next button (`NextButtonIcon`).
    pub next_button_icon: String,
    /// Icon of the complete button (`CompleteButtonIcon`).
    pub complete_button_icon: String,
    /// Icon of the reset button (`ResetButtonIcon`).
    pub reset_button_icon: String,

    // ── Templates ──────────────────────────────────────────────────────────
    /// Custom circle renderer for each step (`LabelTemplate`).
    pub label_template: Option<Box<dyn for<'v> FnMut(&mut Ui, &StepView<'v>) + 'a>>,
    /// Custom title-area renderer for each step (`TitleTemplate`).
    pub title_template: Option<Box<dyn for<'v> FnMut(&mut Ui, &StepView<'v>) + 'a>>,
    /// Custom connector renderer (`ConnectorTemplate`). Receives `is_complete`.
    pub connector_template: Option<Box<dyn FnMut(&mut Ui, bool) + 'a>>,
    /// Custom action row (`ActionContent`). Receives the state by `&mut`.
    pub action_content: Option<Box<dyn FnMut(&mut Ui, &mut StepperState) + 'a>>,
    /// Shown instead of the active step when every step is completed
    /// (`CompletedContent`).
    pub completed_content: Option<Box<dyn FnMut(&mut Ui) + 'a>>,

    // ── Callbacks ──────────────────────────────────────────────────────────
    /// Fires when the active index changes (`ActiveIndexChanged`).
    pub on_active_index_changed: Option<Box<dyn FnMut(usize) + 'a>>,
    /// Fires before a step change and may cancel it (`OnPreviewInteraction`).
    pub on_preview_interaction: Option<Box<dyn FnMut(&mut StepperInteraction) + 'a>>,

    // ── Utilities ──────────────────────────────────────────────────────────
    /// Utility classes on the wrapper.
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

impl<'a> Default for MudStepper<'a> {
    fn default() -> Self {
        Self {
            non_linear: false,
            vertical: false,
            center_labels: false,
            scrollable_navigation: true,
            show_reset_button: false,
            ripple: false,
            completed_step_color: Color::Primary,
            current_step_color: Color::Primary,
            error_step_color: Color::Error,
            step_complete_icon: filled::DONE.to_owned(),
            step_error_icon: filled::PRIORITY_HIGH.to_owned(),
            skip_button_icon: filled::SKIP_NEXT.to_owned(),
            previous_button_icon: filled::NAVIGATE_BEFORE.to_owned(),
            next_button_icon: filled::NAVIGATE_NEXT.to_owned(),
            complete_button_icon: filled::DONE.to_owned(),
            reset_button_icon: filled::FIRST_PAGE.to_owned(),
            label_template: None,
            title_template: None,
            connector_template: None,
            action_content: None,
            completed_content: None,
            on_active_index_changed: None,
            on_preview_interaction: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
        }
    }
}

impl<'a> MudStepper<'a> {
    /// Creates a stepper with MudBlazor defaults.
    pub fn new() -> Self {
        Self::default()
    }

    // ── Behavior builders ──────────────────────────────────────────────────

    #[inline]
    pub fn non_linear(mut self, b: bool) -> Self {
        self.non_linear = b;
        self
    }

    #[inline]
    pub fn vertical(mut self, b: bool) -> Self {
        self.vertical = b;
        self
    }

    #[inline]
    pub fn center_labels(mut self, b: bool) -> Self {
        self.center_labels = b;
        self
    }

    #[inline]
    pub fn scrollable_navigation(mut self, b: bool) -> Self {
        self.scrollable_navigation = b;
        self
    }

    #[inline]
    pub fn show_reset_button(mut self, b: bool) -> Self {
        self.show_reset_button = b;
        self
    }

    #[inline]
    pub fn ripple(mut self, b: bool) -> Self {
        self.ripple = b;
        self
    }

    // ── Appearance builders ────────────────────────────────────────────────

    #[inline]
    pub fn completed_step_color(mut self, c: Color) -> Self {
        self.completed_step_color = c;
        self
    }

    #[inline]
    pub fn current_step_color(mut self, c: Color) -> Self {
        self.current_step_color = c;
        self
    }

    #[inline]
    pub fn error_step_color(mut self, c: Color) -> Self {
        self.error_step_color = c;
        self
    }

    #[inline]
    pub fn step_complete_icon(mut self, s: impl Into<String>) -> Self {
        self.step_complete_icon = s.into();
        self
    }

    #[inline]
    pub fn step_error_icon(mut self, s: impl Into<String>) -> Self {
        self.step_error_icon = s.into();
        self
    }

    #[inline]
    pub fn skip_button_icon(mut self, s: impl Into<String>) -> Self {
        self.skip_button_icon = s.into();
        self
    }

    #[inline]
    pub fn previous_button_icon(mut self, s: impl Into<String>) -> Self {
        self.previous_button_icon = s.into();
        self
    }

    #[inline]
    pub fn next_button_icon(mut self, s: impl Into<String>) -> Self {
        self.next_button_icon = s.into();
        self
    }

    #[inline]
    pub fn complete_button_icon(mut self, s: impl Into<String>) -> Self {
        self.complete_button_icon = s.into();
        self
    }

    #[inline]
    pub fn reset_button_icon(mut self, s: impl Into<String>) -> Self {
        self.reset_button_icon = s.into();
        self
    }

    // ── Template builders ──────────────────────────────────────────────────

    #[inline]
    pub fn label_template(
        mut self,
        f: impl for<'v> FnMut(&mut Ui, &StepView<'v>) + 'a,
    ) -> Self {
        self.label_template = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn title_template(
        mut self,
        f: impl for<'v> FnMut(&mut Ui, &StepView<'v>) + 'a,
    ) -> Self {
        self.title_template = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn connector_template(mut self, f: impl FnMut(&mut Ui, bool) + 'a) -> Self {
        self.connector_template = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn action_content(
        mut self,
        f: impl FnMut(&mut Ui, &mut StepperState) + 'a,
    ) -> Self {
        self.action_content = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn completed_content(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self {
        self.completed_content = Some(Box::new(f));
        self
    }

    // ── Callback builders ──────────────────────────────────────────────────

    #[inline]
    pub fn on_active_index_changed(mut self, f: impl FnMut(usize) + 'a) -> Self {
        self.on_active_index_changed = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn on_preview_interaction(
        mut self,
        f: impl FnMut(&mut StepperInteraction) + 'a,
    ) -> Self {
        self.on_preview_interaction = Some(Box::new(f));
        self
    }

    // ── Utilities ──────────────────────────────────────────────────────────

    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    #[inline]
    pub fn flavor(mut self, f: StyleFlavor) -> Self {
        self.flavor = f;
        self
    }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Draws the stepper.
    ///
    /// Returns `(StepperResponse, user_result)`. `StepperResponse` carries the
    /// wrapper response, the resulting `active_index`, and a
    /// [`StepperAction`] describing what was clicked this frame.
    pub fn show<'s, R>(
        mut self,
        ui: &mut Ui,
        state: &mut StepperState,
        add_steps: impl FnOnce(&mut StepperUi<'s>) -> R,
    ) -> (StepperResponse, R)
    where
        's: 'a,
    {
        // 1. Collect steps.
        let mut collector = StepperUi { steps: Vec::new() };
        let user_result = add_steps(&mut collector);
        let mut steps = collector.steps;
        let count = steps.len();

        // 2. Seed state.
        for (i, step) in steps.iter().enumerate() {
            state.ensure_step(
                i,
                step.completed.unwrap_or(false),
                step.has_error.unwrap_or(false),
                step.skipped.unwrap_or(false),
            );
        }
        if count == 0 {
            let response = ui.allocate_response(Vec2::ZERO, Sense::hover());
            return (
                StepperResponse {
                    response,
                    active_index: 0,
                    action: StepperAction::None,
                },
                user_result,
            );
        }
        state.active_index = state.active_index.min(count - 1);

        // 3. Theme + frame.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);

        self.tw_state.dark = ui.visuals().dark_mode;
        let frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => self
                .tw_style
                .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
        };

        // 4. Resolve config.
        let cfg = StepperCfg {
            vertical: self.vertical,
            non_linear: self.non_linear,
            center_labels: self.center_labels,
            scrollable_navigation: self.scrollable_navigation,
            show_reset_button: self.show_reset_button,
            ripple: self.ripple,
            circle_diameter: step_diameter(Size::Medium),
            completed_color: resolve_color(
                self.completed_step_color,
                palette,
                palette.primary.into(),
            ),
            current_color: resolve_color(self.current_step_color, palette, palette.primary.into()),
            error_color: resolve_color(self.error_step_color, palette, palette.error.into()),
            skipped_color: palette.text_secondary.into(),
            divider: palette.divider.into(),
            text_primary: palette.text_primary.into(),
            text_secondary: palette.text_secondary.into(),
            text_disabled: palette.text_disabled.into(),
            surface: palette.surface.into(),
            contrast: palette.primary_contrast_text.into(),
            complete_icon: self.step_complete_icon.clone(),
            error_icon: self.step_error_icon.clone(),
            skip_icon: self.skip_button_icon.clone(),
            prev_icon: self.previous_button_icon.clone(),
            next_icon: self.next_button_icon.clone(),
            complete_button_icon: self.complete_button_icon.clone(),
            reset_icon: self.reset_button_icon.clone(),
            base_id: ui.scope_id().with("mud_stepper"),
            palette,
        };

        // 5. Take templates + callbacks out of `self`.
        let mut tpl = StepperTemplates {
            label_template: self.label_template.take(),
            title_template: self.title_template.take(),
            connector_template: self.connector_template.take(),
            action_content: self.action_content.take(),
            completed_content: self.completed_content.take(),
            on_preview: self.on_preview_interaction.take(),
            on_active_index_changed: self.on_active_index_changed.take(),
        };

        let mut action = StepperAction::None;

        // 6. Render.
        let inner = frame.show(ui, |ui| {
            ui.vertical(|ui| {
                // 6a. Header.
                //
                // The closure captures `&mut steps`, `&mut state`, `&mut tpl`,
                // `&mut action`, so it is `FnMut` (needs `&mut header` to be
                // called) and must be declared `mut`. We pass `&mut header` to
                // `ScrollArea::show` so the closure is not moved into the
                // if-branch; the else-branch can still call it.
                let mut header = |ui: &mut Ui| {
                    if cfg.vertical {
                        render_vertical_header(ui, &mut steps, state, &cfg, &mut tpl, &mut action);
                    } else {
                        render_horizontal_header(ui, &mut steps, state, &cfg, &mut tpl, &mut action);
                    }
                };
                if cfg.vertical && cfg.scrollable_navigation {
                    egui::ScrollArea::vertical()
                        .max_height(240.0)
                        .auto_shrink([false, false])
                        .show(ui, &mut header);
                } else {
                    header(ui);
                }

                ui.add_space(SECTION_GAP);

                // 6b. Content.
                let completed = is_completed(state, count);
                if completed {
                    if let Some(content) = tpl.completed_content.as_mut() {
                        content(ui);
                    } else if let Some(content) = steps[state.active_index].content.as_mut() {
                        content(ui);
                    }
                } else if let Some(content) = steps[state.active_index].content.as_mut() {
                    content(ui);
                }

                ui.add_space(SECTION_GAP);

                // 6c. Nav row.
                if let Some(action_content) = tpl.action_content.as_mut() {
                    action_content(ui, state);
                } else {
                    render_nav(ui, &mut steps, state, &cfg, &mut tpl, &mut action, completed);
                }
            })
                .response
        });

        (
            StepperResponse {
                response: inner.response,
                active_index: state.active_index,
                action,
            },
            user_result,
        )
    }
}

// =============================================================================
// Response
// =============================================================================

/// The result of one [`MudStepper::show`] frame.
#[derive(Debug)]
pub struct StepperResponse {
    /// The wrapper's own `egui::Response`.
    pub response: Response,
    /// The active index **after** this frame's click was applied.
    pub active_index: usize,
    /// What the user clicked this frame.
    pub action: StepperAction,
}

// =============================================================================
// Rendering — helpers
// =============================================================================

fn step_view<'s, 'a>(
    index: usize,
    step: &'s MudStep<'a>,
    flags: StepFlags,
    is_active: bool,
) -> StepView<'s> {
    StepView {
        index,
        title: step.title.as_deref(),
        secondary_text: step.secondary_text.as_deref(),
        is_active,
        completed: flags.completed,
        has_error: flags.has_error,
        skipped: flags.skipped,
        disabled: step.disabled,
        skippable: step.skippable,
        phase: step_phase(&flags, is_active),
    }
}

fn phase_color(phase: StepPhase, cfg: &StepperCfg<'_>) -> Color32 {
    match phase {
        StepPhase::Pending => cfg.surface,
        StepPhase::Active => cfg.current_color,
        StepPhase::Completed => cfg.completed_color,
        StepPhase::Error => cfg.error_color,
        StepPhase::Skipped => cfg.skipped_color,
    }
}

fn phase_icon<'c>(phase: StepPhase, cfg: &'c StepperCfg<'_>) -> Option<&'c str> {
    match phase {
        StepPhase::Completed => Some(&cfg.complete_icon),
        StepPhase::Error => Some(&cfg.error_icon),
        StepPhase::Skipped => Some(&cfg.skip_icon),
        _ => None,
    }
}

/// Paints one circle (or dispatches to `label_template`).
#[allow(clippy::too_many_arguments)]
fn render_circle(
    ui: &mut Ui,
    circle_rect: Rect,
    index: usize,
    phase: StepPhase,
    is_active: bool,
    disabled: bool,
    cfg: &StepperCfg<'_>,
    tpl: &mut StepperTemplates<'_>,
    view: &StepView<'_>,
    clicked: bool,
) {
    let r = circle_rect.width() * 0.5;
    let fill = phase_color(phase, cfg);
    let stroke = if phase == StepPhase::Pending {
        Stroke::new(CONNECTOR_THICKNESS, cfg.divider)
    } else {
        Stroke::NONE
    };

    ui.painter().circle_filled(circle_rect.center(), r, fill);
    if stroke != Stroke::NONE {
        ui.painter().circle_stroke(circle_rect.center(), r, stroke);
    }

    if is_active && !disabled {
        ui.painter().circle_stroke(
            circle_rect.center(),
            r + 3.0,
            Stroke::new(2.0, with_alpha(cfg.current_color, 0.5)),
        );
    }
    if clicked && cfg.ripple {
        ui.painter().circle_stroke(
            circle_rect.center(),
            r + 6.0,
            Stroke::new(1.5, with_alpha(cfg.current_color, 0.3)),
        );
    }

    if let Some(template) = tpl.label_template.as_mut() {
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(circle_rect)
                .layout(egui::Layout::centered_and_justified(
                    egui::Direction::TopDown,
                )),
        );
        template(&mut child, view);
        return;
    }

    if let Some(icon) = phase_icon(phase, cfg) {
        let icon_px = r * 1.1;
        let icon_rect = Rect::from_center_size(circle_rect.center(), Vec2::splat(icon_px));
        ui.put(
            icon_rect,
            MudIcon::new(icon.to_owned())
                .size_px(icon_px)
                .custom_color(cfg.contrast),
        );
    } else {
        let text_color = if phase == StepPhase::Pending {
            if disabled { cfg.text_disabled } else { cfg.text_secondary }
        } else {
            cfg.contrast
        };
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(circle_rect)
                .layout(egui::Layout::centered_and_justified(
                    egui::Direction::TopDown,
                )),
        );
        child.add(
            MudText::new((index + 1).to_string())
                .font_size(r * 0.9)
                .bold(true)
                .custom_color(text_color)
                .inline(true),
        );
    }
}

/// Paints the title / secondary text block (or dispatches to `title_template`).
fn render_label(
    ui: &mut Ui,
    rect: Rect,
    centered: bool,
    view: &StepView<'_>,
    cfg: &StepperCfg<'_>,
    tpl: &mut StepperTemplates<'_>,
) {
    if let Some(template) = tpl.title_template.as_mut() {
        let mut child = ui.new_child(
            egui::UiBuilder::new().max_rect(rect).layout(if centered {
                egui::Layout::top_down(egui::Align::Center)
            } else {
                egui::Layout::top_down(egui::Align::Min)
            }),
        );
        template(&mut child, view);
        return;
    }

    let layout = if centered {
        egui::Layout::top_down(egui::Align::Center)
    } else {
        egui::Layout::top_down(egui::Align::Min)
    };
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(layout));
    let text_color = if view.disabled {
        cfg.text_disabled
    } else if view.is_active {
        cfg.current_color
    } else if view.completed {
        cfg.completed_color
    } else {
        cfg.text_primary
    };
    if let Some(title) = view.title {
        child.add(
            MudText::new(title.to_owned())
                .typo(Typo::Body2)
                .bold(view.is_active)
                .custom_color(text_color)
                .no_wrap(true),
        );
    }
    if let Some(sec) = view.secondary_text {
        child.add(
            MudText::new(sec.to_owned())
                .typo(Typo::Caption)
                .custom_color(cfg.text_secondary)
                .no_wrap(true),
        );
    }
}

// =============================================================================
// Rendering — horizontal
// =============================================================================

fn render_horizontal_header<'a>(
    ui: &mut Ui,
    steps: &mut [MudStep<'a>],
    state: &mut StepperState,
    cfg: &StepperCfg<'_>,
    tpl: &mut StepperTemplates<'a>,
    action: &mut StepperAction,
) {
    let n = steps.len();
    if n == 0 {
        return;
    }
    let available = ui.available_width();
    let col_w = available / n as f32;
    let label_h = steps
        .iter()
        .map(|s| {
            if s.secondary_text.is_some() {
                LABEL_H_WITH_SECONDARY
            } else {
                LABEL_H_TITLE_ONLY
            }
        })
        .fold(LABEL_H_TITLE_ONLY, f32::max);
    let header_h = cfg.circle_diameter + LABEL_GAP + label_h;

    let (rect, _) = ui.allocate_exact_size(vec2(available, header_h), Sense::hover());
    let r = cfg.circle_diameter * 0.5;
    let circle_y = rect.top() + r;

    // Connectors first.
    for i in 0..n.saturating_sub(1) {
        let x1 = rect.left() + col_w * (i as f32 + 0.5) + r;
        let x2 = rect.left() + col_w * (i as f32 + 1.5) - r;
        if let Some(template) = tpl.connector_template.as_mut() {
            let (line_rect, _) = ui.allocate_exact_size(
                vec2((x2 - x1).max(0.0), CONNECTOR_THICKNESS),
                Sense::hover(),
            );
            let _ = line_rect;
            template(ui, state.flags(i).completed);
        } else {
            let color = if state.flags(i).completed {
                cfg.completed_color
            } else {
                cfg.divider
            };
            ui.painter().line_segment(
                [pos2(x1, circle_y), pos2(x2, circle_y)],
                Stroke::new(CONNECTOR_THICKNESS, color),
            );
        }
    }

    // Steps.
    for i in 0..n {
        let col_left = rect.left() + col_w * i as f32;
        let cx = col_left + col_w * 0.5;
        let circle_rect = Rect::from_center_size(pos2(cx, circle_y), Vec2::splat(cfg.circle_diameter));
        let flags = state.flags(i);
        let is_active = i == state.active_index;
        let phase = step_phase(&flags, is_active);
        let step = &mut steps[i];

        let sense = if step.disabled || !cfg.non_linear {
            Sense::hover()
        } else {
            Sense::click()
        };
        let resp = ui.interact(circle_rect, cfg.base_id.with(("step", i)), sense);
        if resp.hovered() && cfg.non_linear && !step.disabled {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        let clicked = resp.clicked() && cfg.non_linear && !step.disabled;
        if clicked {
            if let Some(cb) = step.on_click.as_mut() {
                cb();
            }
            if i != state.active_index {
                if let Some(cb) = tpl.on_preview.as_mut() {
                    let mut interaction = StepperInteraction::new(i, StepAction::Activate);
                    cb(&mut interaction);
                    if !interaction.cancel {
                        let from = state.active_index;
                        state.active_index = i;
                        if let Some(cb) = tpl.on_active_index_changed.as_mut() {
                            cb(i);
                        }
                        *action = StepperAction::Activated { from, to: i };
                    }
                } else {
                    let from = state.active_index;
                    state.active_index = i;
                    if let Some(cb) = tpl.on_active_index_changed.as_mut() {
                        cb(i);
                    }
                    *action = StepperAction::Activated { from, to: i };
                }
            }
        }

        let view = step_view(i, step, flags, is_active);
        render_circle(ui, circle_rect, i, phase, is_active, step.disabled, cfg, tpl, &view, clicked);

        let label_rect = Rect::from_min_size(
            pos2(col_left + 4.0, circle_rect.bottom() + LABEL_GAP),
            vec2(col_w - 8.0, label_h),
        );
        render_label(ui, label_rect, cfg.center_labels, &view, cfg, tpl);
    }
}

// =============================================================================
// Rendering — vertical
// =============================================================================

fn render_vertical_header<'a>(
    ui: &mut Ui,
    steps: &mut [MudStep<'a>],
    state: &mut StepperState,
    cfg: &StepperCfg<'_>,
    tpl: &mut StepperTemplates<'a>,
    action: &mut StepperAction,
) {
    let n = steps.len();
    if n == 0 {
        return;
    }
    let r = cfg.circle_diameter * 0.5;
    let label_w = (ui.available_width() - cfg.circle_diameter - STEP_GAP).max(20.0);

    for i in 0..n {
        let row_h = cfg.circle_diameter.max(LABEL_H_WITH_SECONDARY) + STEP_GAP;
        let (row_rect, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::hover());
        let circle_rect = Rect::from_center_size(
            pos2(row_rect.left() + r, row_rect.top() + r),
            Vec2::splat(cfg.circle_diameter),
        );
        let flags = state.flags(i);
        let is_active = i == state.active_index;
        let phase = step_phase(&flags, is_active);
        let step = &mut steps[i];

        // Vertical connector to the next step (before this row's circle so it
        // sits behind).
        if i + 1 < n {
            let y1 = circle_rect.bottom();
            let y2 = row_rect.bottom() + r;
            let x = circle_rect.center().x;
            let color = if flags.completed {
                cfg.completed_color
            } else {
                cfg.divider
            };
            if let Some(template) = tpl.connector_template.as_mut() {
                template(ui, flags.completed);
            } else {
                ui.painter().line_segment(
                    [pos2(x, y1), pos2(x, y2)],
                    Stroke::new(CONNECTOR_THICKNESS, color),
                );
            }
        }

        let sense = if step.disabled || !cfg.non_linear {
            Sense::hover()
        } else {
            Sense::click()
        };
        let resp = ui.interact(circle_rect, cfg.base_id.with(("vstep", i)), sense);
        if resp.hovered() && cfg.non_linear && !step.disabled {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        let clicked = resp.clicked() && cfg.non_linear && !step.disabled;
        if clicked {
            if let Some(cb) = step.on_click.as_mut() {
                cb();
            }
            if i != state.active_index {
                let allowed = if let Some(cb) = tpl.on_preview.as_mut() {
                    let mut interaction = StepperInteraction::new(i, StepAction::Activate);
                    cb(&mut interaction);
                    !interaction.cancel
                } else {
                    true
                };
                if allowed {
                    let from = state.active_index;
                    state.active_index = i;
                    if let Some(cb) = tpl.on_active_index_changed.as_mut() {
                        cb(i);
                    }
                    *action = StepperAction::Activated { from, to: i };
                }
            }
        }

        let view = step_view(i, step, flags, is_active);
        render_circle(ui, circle_rect, i, phase, is_active, step.disabled, cfg, tpl, &view, clicked);

        let label_rect = Rect::from_min_size(
            pos2(circle_rect.right() + STEP_GAP, row_rect.top()),
            vec2(label_w, row_h),
        );
        render_label(ui, label_rect, false, &view, cfg, tpl);
    }
}

// =============================================================================
// Rendering — nav row
// =============================================================================

fn render_nav<'a>(
    ui: &mut Ui,
    steps: &mut [MudStep<'a>],
    state: &mut StepperState,
    cfg: &StepperCfg<'_>,
    tpl: &mut StepperTemplates<'a>,
    action: &mut StepperAction,
    completed: bool,
) {
    let count = steps.len();
    let active = state.active_index;
    let flags = state.flags(active);
    let step = &mut steps[active];

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = NAV_GAP;

        // Previous.
        let prev_enabled = previous_step_enabled(active);
        if MudButton::new("Previous")
            .variant(Variant::Text)
            .start_icon(cfg.prev_icon.clone())
            .disabled(!prev_enabled)
            .show(ui)
            .clicked()
        {
            let target = active - 1;
            let allowed = if let Some(cb) = tpl.on_preview.as_mut() {
                let mut interaction = StepperInteraction::new(target, StepAction::Activate);
                cb(&mut interaction);
                !interaction.cancel
            } else {
                true
            };
            if allowed {
                state.active_index = target;
                if let Some(cb) = tpl.on_active_index_changed.as_mut() {
                    cb(target);
                }
                *action = StepperAction::Activated {
                    from: active,
                    to: target,
                };
            }
        }

        // Skip (only when the current step is skippable and not completed).
        if is_current_skippable(step) && !completed {
            if MudButton::new("Skip")
                .variant(Variant::Text)
                .start_icon(cfg.skip_icon.clone())
                .show(ui)
                .clicked()
            {
                let allowed = if let Some(cb) = tpl.on_preview.as_mut() {
                    let mut interaction = StepperInteraction::new(active, StepAction::Skip);
                    cb(&mut interaction);
                    !interaction.cancel
                } else {
                    true
                };
                if allowed {
                    state.flags_mut(active).skipped = true;
                    let to = (active + 1).min(count - 1);
                    state.active_index = to;
                    if let Some(cb) = tpl.on_active_index_changed.as_mut() {
                        cb(to);
                    }
                    *action = StepperAction::Skipped { index: active, to };
                }
            }
        }

        // Next or Complete.
        if !completed {
            let is_last = show_complete_instead_of_next(active, count);
            let enabled = is_last || can_go_to_next(&flags, cfg.non_linear, is_last) || true;
            let _ = enabled;
            let (label, icon, color) = if is_last {
                ("Complete", cfg.complete_button_icon.clone(), cfg.current_color)
            } else {
                ("Next", cfg.next_icon.clone(), cfg.current_color)
            };
            if MudButton::new(label)
                .variant(Variant::Filled)
                .color(Color::Custom(color))
                .start_icon(icon)
                .show(ui)
                .clicked()
            {
                let allowed = if let Some(cb) = tpl.on_preview.as_mut() {
                    let mut interaction = StepperInteraction::new(active, StepAction::Complete);
                    cb(&mut interaction);
                    !interaction.cancel
                } else {
                    true
                };
                if allowed {
                    state.flags_mut(active).completed = true;
                    let to = if is_last { active } else { active + 1 };
                    state.active_index = to;
                    if let Some(cb) = tpl.on_active_index_changed.as_mut() {
                        cb(to);
                    }
                    *action = StepperAction::Completed { index: active, to };
                }
            }
        }

        // Reset.
        if cfg.show_reset_button {
            if MudButton::new("Reset")
                .variant(Variant::Text)
                .start_icon(cfg.reset_icon.clone())
                .show(ui)
                .clicked()
            {
                let allowed = if let Some(cb) = tpl.on_preview.as_mut() {
                    let mut interaction = StepperInteraction::new(0, StepAction::Reset);
                    cb(&mut interaction);
                    !interaction.cancel
                } else {
                    true
                };
                if allowed {
                    state.reset();
                    if let Some(cb) = tpl.on_active_index_changed.as_mut() {
                        cb(0);
                    }
                    *action = StepperAction::Reset;
                }
            }
        }
    });
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── step_diameter ──────────────────────────────────────────────────────

    #[test]
    fn step_diameter_maps_sizes() {
        assert_eq!(step_diameter(Size::Small), STEP_DIAMETER_SMALL);
        assert_eq!(step_diameter(Size::Medium), STEP_DIAMETER);
        assert_eq!(step_diameter(Size::Large), STEP_DIAMETER_LARGE);
        assert_eq!(step_diameter(Size::ExtraSmall), STEP_DIAMETER);
        assert_eq!(step_diameter(Size::ExtraLarge), STEP_DIAMETER);
    }

    // ── step_phase ─────────────────────────────────────────────────────────

    fn flags(completed: bool, has_error: bool, skipped: bool) -> StepFlags {
        StepFlags {
            completed,
            has_error,
            skipped,
        }
    }

    #[test]
    fn phase_error_wins_over_everything() {
        assert_eq!(step_phase(&flags(true, true, true), true), StepPhase::Error);
    }

    #[test]
    fn phase_skipped_wins_over_completed_and_active() {
        assert_eq!(step_phase(&flags(true, false, true), true), StepPhase::Skipped);
    }

    #[test]
    fn phase_completed_wins_over_active() {
        assert_eq!(step_phase(&flags(true, false, false), true), StepPhase::Completed);
    }

    #[test]
    fn phase_active_when_only_active() {
        assert_eq!(step_phase(&flags(false, false, false), true), StepPhase::Active);
    }

    #[test]
    fn phase_pending_when_inactive_and_empty() {
        assert_eq!(step_phase(&flags(false, false, false), false), StepPhase::Pending);
    }

    // ── is_completed ───────────────────────────────────────────────────────

    #[test]
    fn is_completed_requires_non_empty() {
        let s = StepperState::new();
        assert!(!is_completed(&s, 0));
    }

    #[test]
    fn is_completed_requires_all() {
        let mut s = StepperState::new();
        s.set_completed(0, true);
        assert!(!is_completed(&s, 2));
        s.set_completed(1, true);
        assert!(is_completed(&s, 2));
    }

    // ── show_complete_instead_of_next ──────────────────────────────────────

    #[test]
    fn complete_instead_of_next_only_on_last_step() {
        assert!(!show_complete_instead_of_next(0, 3));
        assert!(!show_complete_instead_of_next(1, 3));
        assert!(show_complete_instead_of_next(2, 3));
        assert!(!show_complete_instead_of_next(0, 0));
    }

    // ── can_go_to_next ─────────────────────────────────────────────────────

    #[test]
    fn cannot_advance_from_last_step() {
        assert!(!can_go_to_next(&flags(true, false, false), true, true));
    }

    #[test]
    fn non_linear_always_allows_advance() {
        assert!(can_go_to_next(&flags(false, false, false), true, false));
    }

    #[test]
    fn linear_requires_completed_or_skipped() {
        assert!(!can_go_to_next(&flags(false, false, false), false, false));
        assert!(can_go_to_next(&flags(true, false, false), false, false));
        assert!(can_go_to_next(&flags(false, false, true), false, false));
    }

    // ── previous_step_enabled ──────────────────────────────────────────────

    #[test]
    fn previous_enabled_only_after_first_step() {
        assert!(!previous_step_enabled(0));
        assert!(previous_step_enabled(1));
        assert!(previous_step_enabled(99));
    }

    // ── is_current_skippable ───────────────────────────────────────────────

    #[test]
    fn current_skippable_requires_flag_and_not_disabled() {
        let a: MudStep<'static> = MudStep::new("a").skippable(true);
        let b: MudStep<'static> = MudStep::new("b").skippable(true).disabled(true);
        let c: MudStep<'static> = MudStep::new("c");
        assert!(is_current_skippable(&a));
        assert!(!is_current_skippable(&b));
        assert!(!is_current_skippable(&c));
    }

    // ── Defaults ───────────────────────────────────────────────────────────

    #[test]
    fn stepper_defaults_match_mudblazor() {
        let s: MudStepper<'static> = MudStepper::new();
        assert!(!s.non_linear);
        assert!(!s.vertical);
        assert!(!s.center_labels);
        assert!(s.scrollable_navigation);
        assert!(!s.show_reset_button);
        assert!(!s.ripple);
        assert!(matches!(s.completed_step_color, Color::Primary));
        assert!(matches!(s.current_step_color, Color::Primary));
        assert!(matches!(s.error_step_color, Color::Error));
        assert_eq!(s.step_complete_icon, filled::DONE);
        assert_eq!(s.step_error_icon, filled::PRIORITY_HIGH);
        assert_eq!(s.skip_button_icon, filled::SKIP_NEXT);
        assert_eq!(s.previous_button_icon, filled::NAVIGATE_BEFORE);
        assert_eq!(s.next_button_icon, filled::NAVIGATE_NEXT);
        assert_eq!(s.complete_button_icon, filled::DONE);
        assert_eq!(s.reset_button_icon, filled::FIRST_PAGE);
        assert!(s.label_template.is_none());
        assert!(s.title_template.is_none());
        assert!(s.connector_template.is_none());
        assert!(s.action_content.is_none());
        assert!(s.completed_content.is_none());
        assert!(s.on_active_index_changed.is_none());
        assert!(s.on_preview_interaction.is_none());
    }

    #[test]
    fn stepper_builders_set_every_field() {
        let s: MudStepper<'static> = MudStepper::new()
            .non_linear(true)
            .vertical(true)
            .center_labels(true)
            .scrollable_navigation(false)
            .show_reset_button(true)
            .ripple(true)
            .completed_step_color(Color::Success)
            .current_step_color(Color::Info)
            .error_step_color(Color::Error)
            .step_complete_icon("C")
            .step_error_icon("E")
            .skip_button_icon("S")
            .previous_button_icon("P")
            .next_button_icon("N")
            .complete_button_icon("D")
            .reset_button_icon("R")
            .label_template(|_, _| {})
            .title_template(|_, _| {})
            .connector_template(|_, _| {})
            .action_content(|_, _| {})
            .completed_content(|_| {})
            .on_active_index_changed(|_| {})
            .on_preview_interaction(|_| {})
            .classes(["my-stepper"]);

        assert!(s.non_linear && s.vertical && s.center_labels);
        assert!(!s.scrollable_navigation && s.show_reset_button && s.ripple);
        assert!(matches!(s.completed_step_color, Color::Success));
        assert!(matches!(s.current_step_color, Color::Info));
        assert_eq!(s.step_complete_icon, "C");
        assert_eq!(s.step_error_icon, "E");
        assert_eq!(s.skip_button_icon, "S");
        assert_eq!(s.previous_button_icon, "P");
        assert_eq!(s.next_button_icon, "N");
        assert_eq!(s.complete_button_icon, "D");
        assert_eq!(s.reset_button_icon, "R");
        assert!(s.label_template.is_some());
        assert!(s.title_template.is_some());
        assert!(s.connector_template.is_some());
        assert!(s.action_content.is_some());
        assert!(s.completed_content.is_some());
        assert!(s.on_active_index_changed.is_some());
        assert!(s.on_preview_interaction.is_some());
        assert!(s.base.has_class("my-stepper"));
    }

    // ── StepperUi ──────────────────────────────────────────────────────────

    #[test]
    fn collector_preserves_order() {
        let mut c: StepperUi<'static> = StepperUi { steps: Vec::new() };
        c.text_step("A");
        c.step(MudStep::new("B"));
        assert_eq!(c.len(), 2);
        assert_eq!(c.steps[0].title.as_deref(), Some("A"));
        assert_eq!(c.steps[1].title.as_deref(), Some("B"));
    }

    // ── StepperAction helpers (smoke) ──────────────────────────────────────

    #[test]
    fn action_helpers_agree_with_variants() {
        assert_eq!(StepperAction::None.new_active_index(), None);
        assert_eq!(StepperAction::Reset.new_active_index(), Some(0));
        assert!(StepperAction::Completed { index: 0, to: 1 }.advanced());
        assert!(!StepperAction::Activated { from: 2, to: 1 }.advanced());
    }
}