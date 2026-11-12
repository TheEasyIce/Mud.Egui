//! `MudStep` — one step in a [`MudStepper`](super::MudStepper).
//!
//! Port of `MudStep.razor.cs` (MudBlazor 8.13.0).
//!
//! # MudBlazor parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `ChildContent`      | [`MudStep::new`] / [`MudStep::content`] | required (one of) |
//! | `Completed`         | [`MudStep::completed`] (seed) | from `StepperState` |
//! | `CompletedChanged`  | *(state is authoritative)* | — |
//! | `CompletedStepColor` | [`MudStep::completed_step_color`] | `None` (inherit) |
//! | `Disabled`          | [`MudStep::disabled`] | `false` |
//! | `DisabledChanged`   | *(rebuild next frame)* | — |
//! | `ErrorStepColor`    | [`MudStep::error_step_color`] | `None` (inherit) |
//! | `HasError`          | [`MudStep::has_error`] (seed) | from `StepperState` |
//! | `HasErrorChanged`   | *(state is authoritative)* | — |
//! | `IsActive`          | [`StepperState::active_index`] | — |
//! | `OnClick`           | [`MudStep::on_click`] | `None` |
//! | `SecondaryText`     | [`MudStep::secondary_text`] | `None` |
//! | `Skippable`         | [`MudStep::skippable`] | `false` |
//! | `Skipped`           | [`MudStep::skipped`] (seed) | from `StepperState` |
//! | `SkippedChanged`    | *(state is authoritative)* | — |
//! | `SkippedStepColor`  | [`MudStep::skipped_step_color`] | `None` (inherit) |
//! | `Title`             | [`MudStep::new`] / [`MudStep::title`] | `None` |
//!
//! ### Not ported
//!
//! * **`CompletedChanged` / `HasErrorChanged` / `SkippedChanged`** — the flags
//!   live in [`StepperState`](super::StepperState), so the caller already
//!   knows when they change.
//! * **`DisabledChanged`** — rebuild the step with `.disabled(..)` next frame.
//! * **`SetCompletedAsync` / `SetHasErrorAsync` / `SetSkippedAsync` /
//!   `SetDisabledAsync`** — use [`StepperState::set_completed`] etc. for the
//!   first three; rebuild the step for `Disabled`.
//!
//! # Seeding semantics
//!
//! `.completed(..)`, `.has_error(..)` and `.skipped(..)` are **seeds**: they
//! initialise the corresponding [`StepFlags`](super::StepFlags) on first
//! render, then the state wins. This matches MudBlazor's `@bind-…` behaviour
//! (setting the parameter initialises, and subsequent user interaction
//! overrides it).

use egui::Ui;
use mud_egui_core::enums::Color;

/// One step in a stepper.
#[must_use = "MudStep does nothing until you add it to a MudStepper"]
pub struct MudStep<'a> {
    /// Step title (`Title`).
    pub title: Option<String>,
    /// Step subtitle (`SecondaryText`).
    pub secondary_text: Option<String>,
    /// Step body (`ChildContent`). Rendered only when the step is active.
    pub content: Option<Box<dyn FnMut(&mut Ui) + 'a>>,

    /// Seed for `Completed` (ignored after the first render).
    pub completed: Option<bool>,
    /// Seed for `HasError` (ignored after the first render).
    pub has_error: Option<bool>,
    /// Seed for `Skipped` (ignored after the first render).
    pub skipped: Option<bool>,

    /// Prevents the step header from being clickable (`Disabled`).
    pub disabled: bool,
    /// Allows the user to skip this step (`Skippable`).
    pub skippable: bool,

    /// Per-step completed colour override (`CompletedStepColor`).
    pub completed_step_color: Option<Color>,
    /// Per-step error colour override (`ErrorStepColor`).
    pub error_step_color: Option<Color>,
    /// Per-step skipped colour override (`SkippedStepColor`).
    pub skipped_step_color: Option<Color>,

    /// Fires when the step header is clicked (`OnClick`).
    pub on_click: Option<Box<dyn FnMut() + 'a>>,
}

impl<'a> Default for MudStep<'a> {
    fn default() -> Self {
        Self {
            title: None,
            secondary_text: None,
            content: None,
            completed: None,
            has_error: None,
            skipped: None,
            disabled: false,
            skippable: false,
            completed_step_color: None,
            error_step_color: None,
            skipped_step_color: None,
            on_click: None,
        }
    }
}

impl<'a> MudStep<'a> {
    /// Creates a step with the given title and no content.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            ..Default::default()
        }
    }

    /// Creates a title-less step with content only.
    pub fn content(f: impl FnMut(&mut Ui) + 'a) -> Self {
        Self {
            content: Some(Box::new(f)),
            ..Default::default()
        }
    }

    // ── Builders ───────────────────────────────────────────────────────────

    #[inline]
    pub fn title(mut self, s: impl Into<String>) -> Self {
        self.title = Some(s.into());
        self
    }

    #[inline]
    pub fn secondary_text(mut self, s: impl Into<String>) -> Self {
        self.secondary_text = Some(s.into());
        self
    }

    #[inline]
    pub fn child(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self {
        self.content = Some(Box::new(f));
        self
    }

    /// Seeds `Completed` (ignored after the first render).
    #[inline]
    pub fn completed(mut self, b: bool) -> Self {
        self.completed = Some(b);
        self
    }

    /// Seeds `HasError` (ignored after the first render).
    #[inline]
    pub fn has_error(mut self, b: bool) -> Self {
        self.has_error = Some(b);
        self
    }

    /// Seeds `Skipped` (ignored after the first render).
    #[inline]
    pub fn skipped(mut self, b: bool) -> Self {
        self.skipped = Some(b);
        self
    }

    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    #[inline]
    pub fn skippable(mut self, b: bool) -> Self {
        self.skippable = b;
        self
    }

    #[inline]
    pub fn completed_step_color(mut self, c: Color) -> Self {
        self.completed_step_color = Some(c);
        self
    }

    #[inline]
    pub fn error_step_color(mut self, c: Color) -> Self {
        self.error_step_color = Some(c);
        self
    }

    #[inline]
    pub fn skipped_step_color(mut self, c: Color) -> Self {
        self.skipped_step_color = Some(c);
        self
    }

    #[inline]
    pub fn on_click(mut self, f: impl FnMut() + 'a) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_title_only() {
        let s = MudStep::new("Account");
        assert_eq!(s.title.as_deref(), Some("Account"));
        assert!(s.secondary_text.is_none());
        assert!(s.content.is_none());
        assert!(s.completed.is_none());
        assert!(!s.disabled && !s.skippable);
    }

    #[test]
    fn content_constructor_has_no_title() {
        let s = MudStep::content(|_| {});
        assert!(s.title.is_none());
        assert!(s.content.is_some());
    }

    #[test]
    fn builders_set_every_field() {
        let s = MudStep::new("T")
            .title("T2")
            .secondary_text("Sub")
            .child(|_| {})
            .completed(true)
            .has_error(true)
            .skipped(true)
            .disabled(true)
            .skippable(true)
            .completed_step_color(Color::Success)
            .error_step_color(Color::Error)
            .skipped_step_color(Color::Warning)
            .on_click(|| {});
        assert_eq!(s.title.as_deref(), Some("T2"));
        assert_eq!(s.secondary_text.as_deref(), Some("Sub"));
        assert!(s.content.is_some());
        assert_eq!(s.completed, Some(true));
        assert_eq!(s.has_error, Some(true));
        assert_eq!(s.skipped, Some(true));
        assert!(s.disabled && s.skippable);
        assert_eq!(s.completed_step_color, Some(Color::Success));
        assert_eq!(s.error_step_color, Some(Color::Error));
        assert_eq!(s.skipped_step_color, Some(Color::Warning));
        assert!(s.on_click.is_some());
    }
}