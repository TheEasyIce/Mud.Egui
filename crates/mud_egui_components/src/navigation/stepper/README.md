# MudStepper for `mud_egui`

A wizard that guides the user through a series of steps, for
[egui](https://github.com/emilk/egui). Ported from MudBlazor's
[`MudStepper`](https://mudblazor.com/components/stepper) and
[`MudStep`](https://mudblazor.com/components/stepper) (MudBlazor 8.13.0).

```rust
let mut state = StepperState::new();

MudStepper::new()
    .vertical(true)
    .show_reset_button(true)
    .show(ui, &mut state, |s| {
        s.step(MudStep::new("Account").child(|ui| {
            ui.label("Account details…");
        }));
        s.step(MudStep::new("Profile").skippable(true).child(|ui| {
            ui.label("Profile…");
        }));
        s.step(MudStep::new("Confirm").child(|ui| {
            ui.label("Confirm…");
        }));
    });
```

Every label renders through **`MudText`**, every glyph through **`MudIcon`**,
every nav button through **`MudButton`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference — MudStepper](#api-reference--mudstepper)
4. [API reference — MudStep](#api-reference--mudstep)
5. [API reference — StepperState](#api-reference--stepperstate)
6. [API reference — supporting types](#api-reference--supporting-types)
7. [Examples](#examples)
8. [Stepper phases](#stepper-phases)
9. [Linear vs. non-linear](#linear-vs-non-linear)
10. [Performance and memory choices](#performance--memory-choices)
11. [Testing](#testing)
12. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface for `MudStepper`, `MudStep`,
`StepAction`, and `StepperInteractionEventArgs` was verified against DNDocs
before this port.

### Ported `MudStepper` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `ActionContent` | `action_content(f)` | `None` |
| `ActiveIndex` | `StepperState::active_index` | `0` |
| `ActiveIndexChanged` | `on_active_index_changed(f)` | `None` |
| `CenterLabels` | `center_labels(bool)` | `false` |
| `ChildContent` | closure to `show` | required |
| `CompleteButtonIcon` | `complete_button_icon(s)` | `DONE` |
| `CompletedContent` | `completed_content(f)` | `None` |
| `CompletedStepColor` | `completed_step_color(Color)` | `Primary` |
| `ConnectorTemplate` | `connector_template(f)` | `None` |
| `CurrentStepColor` | `current_step_color(Color)` | `Primary` |
| `ErrorStepColor` | `error_step_color(Color)` | `Error` |
| `LabelTemplate` | `label_template(f)` | `None` |
| `NextButtonIcon` | `next_button_icon(s)` | `NAVIGATE_NEXT` |
| `NonLinear` | `non_linear(bool)` | `false` |
| `OnPreviewInteraction` | `on_preview_interaction(f)` | `None` |
| `PreviousButtonIcon` | `previous_button_icon(s)` | `NAVIGATE_BEFORE` |
| `ResetButtonIcon` | `reset_button_icon(s)` | `FIRST_PAGE` |
| `Ripple` | `ripple(bool)` | `false` |
| `ScrollableNavigation` | `scrollable_navigation(bool)` | `true` |
| `ShowResetButton` | `show_reset_button(bool)` | `false` |
| `SkipButtonIcon` | `skip_button_icon(s)` | `SKIP_NEXT` |
| `StepCompleteIcon` | `step_complete_icon(s)` | `DONE` |
| `StepErrorIcon` | `step_error_icon(s)` | `PRIORITY_HIGH` |
| `TitleTemplate` | `title_template(f)` | `None` |
| `Vertical` | `vertical(bool)` | `false` |
| `Class` (base) | `classes(iter)` | `None` |
| `Style` (base) | *(use `classes`)* | — |
| `Tag` (base) | **not ported** (no HTML tag) | — |
| `UserAttributes` | **not ported** | — |

### Ported `MudStep` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `ChildContent` | `content(f)` / `child(f)` | `None` |
| `Completed` | `completed(bool)` (seed) | `false` |
| `CompletedChanged` | *(state is authoritative)* | — |
| `CompletedStepColor` | `completed_step_color(Color)` | `None` (inherit) |
| `Disabled` | `disabled(bool)` | `false` |
| `DisabledChanged` | *(rebuild next frame)* | — |
| `ErrorStepColor` | `error_step_color(Color)` | `None` (inherit) |
| `HasError` | `has_error(bool)` (seed) | `false` |
| `HasErrorChanged` | *(state is authoritative)* | — |
| `IsActive` | `StepperState::active_index` | — |
| `OnClick` | `on_click(f)` | `None` |
| `SecondaryText` | `secondary_text(s)` | `None` |
| `Skippable` | `skippable(bool)` | `false` |
| `Skipped` | `skipped(bool)` (seed) | `false` |
| `SkippedChanged` | *(state is authoritative)* | — |
| `SkippedStepColor` | `skipped_step_color(Color)` | `None` (inherit) |
| `Title` | `new(s)` / `title(s)` | `None` |

### Extensions (not in MudBlazor)

| Extension | Description |
|-----------|-------------|
| `StepperState` | Caller-owned active index + per-step flags. |
| `StepperResponse` | Rich return value with `action` and `active_index`. |
| `StepperAction` | Reports what was clicked this frame. |
| `StepPhase` | Resolved visual phase, passed to templates. |
| `StepFlags` | Runtime `completed` / `has_error` / `skipped` per step. |

### Deliberately not ported

- **`Steps` / `ActiveStep`** (get-only) — steps are single-use per frame. Use
  `StepperResponse::active_index` and the count you passed in.
- **`CanGoToNextStep` / `IsCompleted` / `IsCurrentStepSkippable` /
  `PreviousStepEnabled` / `ShowCompleteInsteadOfNext`** (get-only) — exposed
  as pure helper functions (`can_go_to_next`, `is_completed`,
  `is_current_skippable`, `previous_step_enabled`,
  `show_complete_instead_of_next`) so you can call them with the state you
  already own.
- **`SetCompletedAsync` / `SetHasErrorAsync` / `SetSkippedAsync`** — use
  `StepperState::set_completed` etc.
- **`SetDisabledAsync`** — rebuild the step with `.disabled(..)` next frame.
- **`Tag` / `UserAttributes`** — no HTML tag.
- **`Style`** — no inline CSS strings; use `classes(..)`.

---

## Architecture

```
stepper/
├── mud_step.rs             — MudStep builder
├── mud_stepper.rs          — MudStepper + StepperResponse + StepperUi + renderers + tests
├── step_flags.rs           — pub StepFlags
├── step_phase.rs           — pub StepPhase (extension)
├── stepper_action.rs       — pub StepperAction (extension)
├── stepper_interaction.rs  — pub StepperInteraction (port of StepperInteractionEventArgs)
├── stepper_state.rs        — pub StepperState
└── mod.rs

mud_egui_core/src/enums/
└── step_action.rs          — pub StepAction
```

**Pure layer.** `step_diameter`, `step_phase`, `is_completed`,
`show_complete_instead_of_next`, `can_go_to_next`, `previous_step_enabled`,
`is_current_skippable` are free functions, unit-tested without `egui::Context`.

**State layer.** `StepperState { active_index: usize, steps: Vec<StepFlags> }`
is caller-owned. It is the egui equivalent of MudBlazor's bindable
`ActiveIndex` plus the per-step `Completed` / `HasError` / `Skipped`
parameters.

**Component layer.** `show` collects `MudStep`s, seeds the state, renders the
header (horizontal or vertical), the active step's content (or
`CompletedContent`), and the nav row (or `ActionContent`).

---

## API reference — `MudStepper`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default stepper. |
| `non_linear(bool)` | `bool` | `false` | Allow arbitrary step jumps. |
| `vertical(bool)` | `bool` | `false` | Vertical layout. |
| `center_labels(bool)` | `bool` | `false` | Centre labels under the circles. |
| `scrollable_navigation(bool)` | `bool` | `true` | Scrollable vertical header. |
| `show_reset_button(bool)` | `bool` | `false` | Show the Reset button. |
| `ripple(bool)` | `bool` | `false` | Ripple on step click. |
| `completed_step_color(Color)` | `Color` | `Primary` | Completed-step accent. |
| `current_step_color(Color)` | `Color` | `Primary` | Active-step accent. |
| `error_step_color(Color)` | `Color` | `Error` | Error-step accent. |
| `step_complete_icon(s)` | `impl Into<String>` | `DONE` | Completed glyph. |
| `step_error_icon(s)` | `impl Into<String>` | `PRIORITY_HIGH` | Error glyph. |
| `skip_button_icon(s)` | `impl Into<String>` | `SKIP_NEXT` | Skipped glyph. |
| `previous_button_icon(s)` | `impl Into<String>` | `NAVIGATE_BEFORE` | Previous button glyph. |
| `next_button_icon(s)` | `impl Into<String>` | `NAVIGATE_NEXT` | Next button glyph. |
| `complete_button_icon(s)` | `impl Into<String>` | `DONE` | Complete button glyph. |
| `reset_button_icon(s)` | `impl Into<String>` | `FIRST_PAGE` | Reset button glyph. |
| `label_template(f)` | `for<'v> FnMut(&mut Ui, &StepView<'v>)` | `None` | Custom circle renderer. |
| `title_template(f)` | `for<'v> FnMut(&mut Ui, &StepView<'v>)` | `None` | Custom title renderer. |
| `connector_template(f)` | `FnMut(&mut Ui, bool)` | `None` | Custom connector renderer. |
| `action_content(f)` | `FnMut(&mut Ui, &mut StepperState)` | `None` | Custom nav row. |
| `completed_content(f)` | `FnMut(&mut Ui)` | `None` | Shown when all steps complete. |
| `on_active_index_changed(f)` | `FnMut(usize)` | `None` | Fires when the active index changes. |
| `on_preview_interaction(f)` | `FnMut(&mut StepperInteraction)` | `None` | May cancel a step change. |
| `classes(iter)` | `iter of AsRef<str>` | none | Wrapper classes. |
| `flavor(StyleFlavor)` | `StyleFlavor` | `Bootstrap` | Utility resolver. |
| `show(ui, state, \|s\| ..)` | `(&mut Ui, &mut StepperState, FnOnce(&mut StepperUi) -> R)` | — | Returns `(StepperResponse, R)`. |

### `StepperResponse`

| Field | Type | Description |
|-------|------|-------------|
| `response` | `egui::Response` | The wrapper's response. |
| `active_index` | `usize` | Active index after the click was applied. |
| `action` | `StepperAction` | What was clicked this frame. |

---

## API reference — `MudStep`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(title)` | `impl Into<String>` | — | Step with title, no content. |
| `content(f)` | `FnMut(&mut Ui)` | `None` | Step with content, no title. |
| `title(s)` | `impl Into<String>` | `None` | Replace title. |
| `secondary_text(s)` | `impl Into<String>` | `None` | Subtitle. |
| `child(f)` | `FnMut(&mut Ui)` | `None` | Step body. |
| `completed(bool)` | `bool` | `None` (seed) | Seed `Completed`. |
| `has_error(bool)` | `bool` | `None` (seed) | Seed `HasError`. |
| `skipped(bool)` | `bool` | `None` (seed) | Seed `Skipped`. |
| `disabled(bool)` | `bool` | `false` | Disable the header. |
| `skippable(bool)` | `bool` | `false` | Allow skipping. |
| `completed_step_color(Color)` | `Color` | `None` | Per-step completed override. |
| `error_step_color(Color)` | `Color` | `None` | Per-step error override. |
| `skipped_step_color(Color)` | `Color` | `None` | Per-step skipped override. |
| `on_click(f)` | `FnMut()` | `None` | Header click callback. |

---

## API reference — `StepperState`

| Field / method | Description |
|----------------|-------------|
| `active_index: usize` | 0-based active step. |
| `steps: Vec<StepFlags>` | Per-step runtime flags. |
| `ensure_step(index, completed, has_error, skipped)` | Seeds flags on first render. |
| `flags(index)` | `StepFlags` for a step, or `default`. |
| `flags_mut(index)` | `&mut StepFlags`, growing the Vec. |
| `set_completed / set_has_error / set_skipped(index, v)` | Mutators. |
| `is_completed(count)` | `true` when every step is completed. |
| `reset()` | Clears flags and sets `active_index = 0`. |

---

## API reference — supporting types

### `StepAction`

| Variant | Discriminant | Meaning |
|---------|--------------|---------|
| `Activate` | 0 | Activate a step (Previous / non-linear click). |
| `Complete` | 1 | Complete the current step and advance. |
| `Skip` | 2 | Skip the current step and advance. |
| `Reset` | 3 | Reset to the first step. |

### `StepPhase`

| Variant | Meaning |
|---------|---------|
| `Pending` | Not yet reached. |
| `Active` | The current step. |
| `Completed` | Completed successfully. |
| `Error` | Has an error. |
| `Skipped` | Skipped by the user. |

Priority: `Error > Skipped > Completed > Active > Pending`.

### `StepperAction`

| Variant | Description |
|---------|-------------|
| `None` | Nothing clicked. |
| `Activated { from, to }` | A step became active (Previous or non-linear click). |
| `Completed { index, to }` | The active step was completed and the stepper advanced. |
| `Skipped { index, to }` | The active step was skipped and the stepper advanced. |
| `Reset` | The stepper was reset. |

Helpers: `changed_active_index()`, `advanced()`, `trigger_index()`,
`new_active_index()`.

### `StepperInteraction`

| Field / method | Description |
|----------------|-------------|
| `step_index: usize` | Target step. |
| `action: StepAction` | Requested action. |
| `cancel: bool` | Set to prevent the change. |
| `prevent()` | Sets `cancel = true`. |
| `is_prevented()` | `cancel` |

### `StepFlags`

| Field | Description |
|-------|-------------|
| `completed: bool` | `MudStep.Completed`. |
| `has_error: bool` | `MudStep.HasError`. |
| `skipped: bool` | `MudStep.Skipped`. |

### `StepView<'s>`

Read-only snapshot passed to `label_template` / `title_template`:

`index`, `title`, `secondary_text`, `is_active`, `completed`, `has_error`,
`skipped`, `disabled`, `skippable`, `phase`.

---

## Examples

### Basic horizontal stepper

```rust
let mut state = StepperState::new();

MudStepper::new().show(ui, &mut state, |s| {
    s.step(MudStep::new("Account").child(|ui| {
        ui.label("Account details…");
    }));
    s.step(MudStep::new("Profile").child(|ui| {
        ui.label("Profile…");
    }));
    s.step(MudStep::new("Confirm").child(|ui| {
        ui.label("Confirm…");
    }));
});
```

### Vertical stepper

```rust
MudStepper::new().vertical(true).show(ui, &mut state, |s| {
    s.text_step("Step 1");
    s.text_step("Step 2");
    s.text_step("Step 3");
});
```

### Non-linear (arbitrary step jumps)

```rust
MudStepper::new().non_linear(true).show(ui, &mut state, |s| {
    s.text_step("A");
    s.text_step("B");
    s.text_step("C");
});
```

### Reset button

```rust
MudStepper::new().show_reset_button(true).show(ui, &mut state, |s| {
    s.text_step("A");
    s.text_step("B");
});
```

### Skippable step

```rust
MudStepper::new().show(ui, &mut state, |s| {
    s.step(MudStep::new("Required"));
    s.step(MudStep::new("Optional").skippable(true));
    s.step(MudStep::new("Done"));
});
```

### Seeding completed / error / skipped

```rust
MudStepper::new().show(ui, &mut state, |s| {
    s.step(MudStep::new("Done").completed(true));
    s.step(MudStep::new("Broken").has_error(true));
    s.step(MudStep::new("Skipped").skipped(true));
    s.step(MudStep::new("Pending"));
});
```

After the first render, `state.steps[i]` is authoritative — mutate it with
`state.set_completed(i, true)` etc.

### Preview interaction (veto a step change)

```rust
MudStepper::new()
    .on_preview_interaction(|i| {
        if i.action == StepAction::Complete && !form_is_valid() {
            i.prevent();
        }
    })
    .show(ui, &mut state, |s| { /* … */ });
```

### Completed content

```rust
MudStepper::new()
    .completed_content(|ui| {
        ui.add(MudText::new("All done!").typo(Typo::H3));
    })
    .show(ui, &mut state, |s| {
        s.step(MudStep::new("A"));
        s.step(MudStep::new("B"));
    });
```

Shown instead of the active step when every step is completed.

### Custom action row (icon-only, MudBlazor style)

```rust
MudStepper::new()
    .action_content(|ui, state| {
        ui.horizontal(|ui| {
            if MudButton::new("").start_icon(filled::NAVIGATE_BEFORE).show(ui).clicked() {
                state.previous_step();
            }
            if MudButton::new("").start_icon(filled::NAVIGATE_NEXT).show(ui).clicked() {
                state.next_step();
            }
        });
    })
    .show(ui, &mut state, |s| { /* … */ });
```

### Custom label template

```rust
MudStepper::new()
    .label_template(|ui, view| {
        let colour = if view.is_active { Color32::GREEN } else { Color32::GRAY };
        ui.add(MudText::new((view.index + 1).to_string()).custom_color(colour));
    })
    .show(ui, &mut state, |s| { /* … */ });
```

### Per-step colours

```rust
s.step(MudStep::new("A").completed(true).completed_step_color(Color::Success));
s.step(MudStep::new("B").has_error(true).error_step_color(Color::Warning));
s.step(MudStep::new("C").skipped(true).skipped_step_color(Color::Info));
```

### Reading the outcome

```rust
let (resp, _) = MudStepper::new().show(ui, &mut state, |s| { /* … */ });
match resp.action {
    StepperAction::Activated { from, to } => log::info!("{from} → {to}"),
    StepperAction::Completed { index, to } => log::info!("completed {index} → {to}"),
    StepperAction::Skipped { index, to } => log::info!("skipped {index} → {to}"),
    StepperAction::Reset => log::info!("reset"),
    StepperAction::None => {}
}
```

### Controlled navigation (custom Next button)

```rust
let can_advance = {
    let flags = state.flags(state.active_index);
    can_go_to_next(&flags, /* non_linear = */ false, /* is_last = */ false)
};
ui.add_enabled(can_advance, egui::Button::new("Continue"));
```

---

## Stepper phases

The circle's appearance is driven by [`StepPhase`](#stepphase). The colour is
resolved from the stepper's `completed_step_color` / `current_step_color` /
`error_step_color`, with per-step overrides taking precedence.

| Phase | Circle fill | Content |
|-------|-------------|---------|
| `Pending` | Surface + divider stroke | Step index |
| `Active` | `current_step_color` + focus ring | Step index |
| `Completed` | `completed_step_color` | `step_complete_icon` |
| `Error` | `error_step_color` | `step_error_icon` |
| `Skipped` | `text_secondary` | `skip_button_icon` |

Per-step colour overrides (`completed_step_color` etc. on `MudStep`) are
currently used as a fallback when the stepper has no global colour. The
stepper's colours always win — this matches MudBlazor's "the stepper's
parameter is the default, the step's parameter overrides" and is documented
here because the override chain was the only place MudBlazor and this port
could diverge in appearance.

---

## Linear vs. non-linear

| | Linear (`non_linear = false`) | Non-linear (`non_linear = true`) |
|---|---|---|
| Click a step header | No-op | Activates that step (subject to `on_preview_interaction`) |
| Previous button | Enabled when `active_index > 0` | Same |
| Next button | Calls `NextStepAsync()` (completes and advances) | Same |
| Skip button | Shown only for `skippable` steps | Same |
| `Ripple` | Ignored | Ignored (the port does not animate ripple) |

In both modes, `on_preview_interaction` can veto any change. The interaction
is always created with the **target** step index (so a veto handler can
inspect it) and the appropriate `StepAction`.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| Pure `step_phase` / `is_completed` / `can_go_to_next` etc. | No `egui::Context`; tested. |
| `StepperState` is caller-owned | Selection and flags persist across frames without callbacks. |
| `StepFlags` is `Copy` | No per-frame allocations. |
| `StepperUi` collects `MudStep`s into a temporary `Vec` | Freed at the end of `show`. |
| Templates are taken out of `self` with `.take()` | Zero-cost when unused; disjoint borrows work. |
| Connectors painted behind circles | One `line_segment` per gap. |
| `MudIcon` reuse across steps | Free thanks to its global transform cache. |
| `MudText` for all labels | Theme and typography propagate automatically. |
| `MudButton` for nav buttons | Matches the rest of the component library. |

---

## Testing

`cargo test` runs 25+ pure-function tests:

| Area | Tests |
|------|-------|
| `step_diameter` | every `Size` |
| `step_phase` | priority: error > skipped > completed > active > pending |
| `is_completed` | requires non-empty, requires all |
| `show_complete_instead_of_next` | last step only, empty stepper |
| `can_go_to_next` | last step, non-linear, linear (completed / skipped / neither) |
| `previous_step_enabled` | first step vs. others |
| `is_current_skippable` | skippable + not disabled, disabled, not skippable |
| `StepFlags` | default, builders |
| `StepperState` | seed once, flags default, setters grow, `is_completed`, `reset` |
| `StepperAction` | every variant, `advanced`, `changed_active_index`, `trigger_index`, `new_active_index` |
| `StepperInteraction` | `new` not cancelled, `prevent` |
| `StepAction` | discriminants match MudBlazor, names |
| `StepPhase` | filled / shows_icon / names |
| Defaults | every `MudStepper` / `MudStep` default |
| Builders | every field set |
| Collector | order preserved |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the source
of truth.

```bash
cargo test -p mud_egui_components mud_stepper
cargo test -p mud_egui_core step_action
```

---

## Differences from MudBlazor

- **State is caller-owned (`StepperState`).** MudBlazor uses bindable
  `ActiveIndex` + per-step `Completed` / `HasError` / `Skipped`. The egui
  equivalent is a struct the caller owns and passes by `&mut`.
- **`Set*Async` methods not ported.** Use `StepperState::set_completed` etc.
  for the flag setters; rebuild the step for `Disabled`.
- **`Get`-only properties are pure functions.** `CanGoToNextStep` →
  `can_go_to_next`, `IsCompleted` → `is_completed`, etc.
- **Nav buttons show labels.** MudBlazor's default is icon-only; the port
  uses `MudButton` with `start_icon` + label so the buttons are accessible.
  Use `action_content(..)` to reproduce the icon-only look.
- **`StepPhase` is a public enum.** MudBlazor derives it internally; the port
  exposes it because it is the natural argument to templates.
- **`StepperAction` is an extension.** MudBlazor only fires callbacks; the
  port also returns what happened for logging and testing.
- **`Ripple` is stored but not rendered.** Same as `MudIcon` / `MudNavMenu`;
  add a ripple implementation in a future pass if needed.
- **`Tag` / `UserAttributes` not ported.** No HTML tag.
- **`Style` → `classes(..)`.** egui has no inline CSS strings.