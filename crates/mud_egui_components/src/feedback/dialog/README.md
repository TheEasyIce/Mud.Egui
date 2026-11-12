# MudDialog for `mud_egui`

A modal overlay that presents information, a choice, or input, for
[egui](https://github.com/emilk/egui). Ported from MudBlazor's
[`MudDialog`](https://mudblazor.com/components/dialog) (MudBlazor 8.13.0).

```rust
let mut dialog = DialogState::new("confirm-delete");

if ui.button("Delete").clicked() {
    dialog.open();
}

let resp = MudDialog::new()
    .title("Delete item?")
    .options(DialogOptions::new().with_close_button(true))
    .show_with_actions(
        ui,
        &mut dialog,
        |ui, _ctl| {
            ui.add(MudText::body2("This cannot be undone."));
        },
        |ui, ctl| {
            if ui.add(MudButton::new("Cancel")).clicked() {
                ctl.cancel();
            }
            if ui.add(MudButton::new("Delete").color(Color::Error)).clicked() {
                ctl.close_with(DialogResult::ok_with(item_id));
            }
        },
    );

if let Some(result) = resp.result {
    if result.is_ok() {
        delete(result.data::<u64>().copied());
    }
}
```

The dialog paints only the surface; every string inside is yours to render
through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Layout and interaction](#layout-and-interaction)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against DNDocs before
implementation. `MudDialog` has **eleven own parameters**, and the supporting
`DialogOptions` / `DialogResult` / `IMudDialogInstance` contracts are covered
too.

### `MudDialog` parameters

| MudBlazor | Rust mapping | Default |
|---|---|---|
| `ActionsClass` | `actions_class(impl Into<String>)` — **stored**, not resolved | `None` |
| `ContentClass` | `content_class(impl Into<String>)` — **stored** | `None` |
| `ContentStyle` | `content_style(impl Into<String>)` — **stored** | `None` |
| `DefaultFocus` | `default_focus(DefaultFocus)` | `Element` |
| `DialogActions` | `actions` argument of `show_with_actions` | none |
| `DialogContent` | `content` argument of `show` / `show_with_actions` | required |
| `Gutters` | `gutters(bool)` | `true` |
| `IsVisible` | `DialogState::open` | `false` |
| `IsVisibleChanged` | `DialogResponse::opened` / `closed` | — |
| `OnBackdropClick` | `on_backdrop_click(f)` | `None` |
| `OnKeyDown` / `OnKeyUp` | **deferred** — browser `KeyboardEventArgs` | — |
| `Options` | `options(DialogOptions)` / `DialogState::options` | `DialogOptions::default()` |
| `Title` | `title(impl Into<String>)` / `DialogState::title` | `None` |
| `TitleClass` | `title_class(impl Into<String>)` — **stored** | `None` |
| `TitleContent` | `title_content(FnMut(&mut Ui))` | `None` |
| `Class` | `classes(iter)` — **stored** | `None` |
| `Style` / `Tag` / `UserAttributes` | **not ported** | — |

### `DialogOptions` parameters

| MudBlazor | Rust field | Default |
|---|---|---|
| `BackdropClick` | `backdrop_click` | `true` |
| `BackgroundClass` | `background_class` — **stored** | `None` |
| `CloseButton` | `close_button` | `false` |
| `CloseOnEscapeKey` | `close_on_escape_key` | `true` |
| `FullScreen` | `full_screen` | `false` |
| `FullWidth` | `full_width` | `false` |
| `MaxWidth` | `max_width: Option<MaxWidth>` | `None` |
| `NoHeader` | `no_header` | `false` |
| `Position` | `position: Option<DialogPosition>` | `None` → `Center` |

### `DialogResult` surface

| MudBlazor | Rust |
|---|---|
| `Ok()` | `DialogResult::ok()` |
| `Ok<T>(T)` | `DialogResult::ok_with(data)` |
| `Cancel()` | `DialogResult::cancel()` |
| `.Canceled` | `is_canceled()` |
| `.Data` | `data::<T>()` / `take_data::<T>()` |

### `IMudDialogInstance` surface

| MudBlazor | Rust |
|---|---|
| `Close()` | `DialogControl::close()` |
| `Cancel()` | `DialogControl::cancel()` |
| `Close(DialogResult)` | `DialogControl::close_with(result)` |
| `Close<T>(T)` | `DialogControl::close_with(DialogResult::ok_with(t))` |
| `SetTitleAsync(string?)` | `DialogControl::set_title(Option<String>)` |
| `SetOptionsAsync(DialogOptions)` | `DialogControl::set_options(DialogOptions)` |
| `CancelAll()` | **not ported** — no parent-dialog stack |

### `IDialogService` surface

| MudBlazor | Rust |
|---|---|
| `ShowAsync<T>()` | `DialogState::open()` + `MudDialog::show` |
| `Close(IDialogReference)` | `DialogState::close()` |
| `Close(IDialogReference, DialogResult)` | `DialogState::close_with(result)` |
| `GetReturnValueAsync<T>()` | `DialogResponse::result` / `DialogState::take_result()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` / `UserAttributes` | No inline CSS strings, no HTML tag. |
| `ActionsClass` / `ContentClass` / `ContentStyle` / `TitleClass` / `BackgroundClass` resolution | Stored for parity. The panel owns fill, radius and shadow; resolving a second utility frame would composite a second surface. |
| `OnKeyDown` / `OnKeyUp` | Browser `KeyboardEventArgs` have no egui equivalent. `Escape` is honoured through `DialogOptions::close_on_escape_key`. |
| `CancelAll()` | Needs a parent-dialog stack; a caller-owned state has no such stack. |
| `FocusTrap` / focus cycling | egui has no built-in focus trap. `DefaultFocus` requests focus on the opening frame; Tab cycling is the host app's responsibility. |

### Enums introduced

Two public enums, each in its own file (Rule 2):

| Enum | File | Discriminants |
|---|---|---|
| `DialogPosition` | `mud_egui_core::enums::dialog_position` | `Center = 0` … `Custom = 9` |
| `DefaultFocus` | `mud_egui_core::enums::default_focus` | `None = 0`, `Element = 1`, `FirstChild = 2`, `LastChild = 3` |

`MaxWidth` is reused from `mud_egui_core::enums`.

---

## Architecture

```
feedback/dialog/
├── dialog_result.rs   DialogResult (own file, Rule 2 companion)
├── dialog_options.rs  DialogOptions
├── dialog_state.rs    DialogState (caller-owned)
├── dialog_control.rs  DialogControl (the IMudDialogInstance analogue)
├── mud_dialog.rs      MudDialog + DialogResponse + pure helpers + tests
└── mod.rs

mud_egui_core/src/enums/
├── default_focus.rs    DefaultFocus
└── dialog_position.rs  DialogPosition
```

**Pure layer.** `position_origin`, `resolve_panel_size`, `dialog_max_width_px`,
`elevation_shadow` are free functions over `Rect`, `Vec2` and `DialogOptions`.
`DialogPosition::fractions`, `DefaultFocus::requests_focus`,
`DialogOptions::effective_position` and every `DialogResult` accessor are pure
methods. None touch `egui::Context`.

**State layer.** `DialogState` holds the open flag, the title/options
overrides, the closing result and a stable `Id`. It is the egui equivalent of
`IDialogService`'s bookkeeping plus the inline `IsVisible` binding.

**Control layer.** `DialogControl` is the `IMudDialogInstance` analogue. The
content and actions closures receive `&mut DialogControl` and call `close` /
`cancel` / `close_with` / `set_title` / `set_options`. Requests are deferred and
applied after the panel is drawn, so the closures never borrow `DialogState`.

**Component layer.** `show` paints the backdrop at `Order::Middle`, positions
and paints the panel at `Order::Foreground`, runs the title / content /
actions, caches the measured size, and applies the deferred requests.

---

## API reference

### `MudDialog<'a>`

| Method | Type | Default | Description |
|---|---|---|---|
| `new()` | — | — | Default dialog. |
| `title(s)` | `impl Into<String>` | `None` | Header text. |
| `title_content(f)` | `FnMut(&mut Ui)` | `None` | Custom header body; overrides `title`. |
| `default_focus(f)` | `DefaultFocus` | `Element` | Which element claims focus on open. |
| `options(o)` | `DialogOptions` | `DialogOptions::default()` | Initial options. |
| `gutters(b)` | `bool` | `true` | 24 pt side padding. |
| `actions_class(s)` / `content_class(s)` / `content_style(s)` / `title_class(s)` | `impl Into<String>` | `None` | **Stored**, not resolved. |
| `color(c)` | `Color` | `Default` | Accent; the panel inherits the theme surface. |
| `on_backdrop_click(f)` | `FnMut()` | `None` | Replaces the default backdrop-close. |
| `classes(iter)` | `iter of AsRef<str>` | none | **Stored**. |
| `show(ui, state, content)` | — | — | Content only; no actions row. |
| `show_with_actions(ui, state, content, actions)` | — | — | Content + actions row. |

`content` is `FnOnce(&mut Ui, &mut DialogControl) -> R`; `actions` is
`FnOnce(&mut Ui, &mut DialogControl)`.

### `DialogResponse<R>`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The panel's response; zero-sized when closed. |
| `result` | `Option<DialogResult>` | The closing frame's result. `Some` only on the closing frame. |
| `opened` | `bool` | `true` on the opening frame. |
| `closed` | `bool` | `true` on the closing frame. |
| `content` | `Option<R>` | The content closure's value. |

### `DialogState`

| Field / method | Description |
|---|---|
| `open: bool` | Whether the dialog is open. |
| `id: Id` | Stable identity. |
| `title: Option<String>` | Title override (wins over the builder's). |
| `options: DialogOptions` | Options override. |
| `result: Option<DialogResult>` | The closing result. |
| `new(id_salt)` / `opened(id_salt)` | Constructors. |
| `is_open()` / `open()` / `open_with(title, options)` | Open. |
| `close()` / `cancel()` / `close_with(result)` | Close. |
| `set_title(..)` / `set_options(..)` | Overrides. |
| `take_result()` | Drain the result. |

### `DialogControl`

| Method | Description |
|---|---|
| `close()` | `DialogResult::ok()`. |
| `cancel()` | `DialogResult::cancel()`. |
| `close_with(result)` | Custom result. |
| `is_close_requested()` | Whether a close is already pending. |
| `set_title(Option<String>)` / `set_options(DialogOptions)` | Overrides. |
| `options` / `title` (fields) | The values in force this frame. |

### `DialogOptions`

| Field / method | Description |
|---|---|
| `backdrop_click` / `close_button` / `close_on_escape_key` | Booleans. |
| `full_screen` / `full_width` / `no_header` | Layout switches. |
| `max_width: Option<MaxWidth>` | Width cap. |
| `position: Option<DialogPosition>` | Where it sits. |
| `background_class: Option<String>` | Stored. |
| `new()` / `default()` | MudBlazor defaults. |
| `with_*(..)` | Chainable setters. |
| `effective_position()` | `position` or `Center`. |

### `DialogResult`

| Method | Description |
|---|---|
| `ok()` / `ok_with(data)` / `cancel()` | Constructors. |
| `is_ok()` / `is_canceled()` / `has_data()` | Predicates. |
| `data::<T>()` / `take_data::<T>()` | Payload access. |
| `Default` | `cancel()`. |

### `DialogPosition` (`mud_egui_core::enums`)

| Variant | Discriminant | Fractions |
|---|---|---|
| `Center` | `0` | `(0.5, 0.5)` |
| `CenterLeft` | `1` | `(0.0, 0.5)` |
| `CenterRight` | `2` | `(1.0, 0.5)` |
| `TopCenter` | `3` | `(0.5, 0.0)` |
| `TopLeft` | `4` | `(0.0, 0.0)` |
| `TopRight` | `5` | `(1.0, 0.0)` |
| `BottomCenter` | `6` | `(0.5, 1.0)` |
| `BottomLeft` | `7` | `(0.0, 1.0)` |
| `BottomRight` | `8` | `(1.0, 1.0)` |
| `Custom` | `9` | `(0.0, 0.0)` |

### `DefaultFocus` (`mud_egui_core::enums`)

| Variant | Discriminant | Requests focus |
|---|---|---|
| `None` | `0` | no |
| `Element` | `1` | yes |
| `FirstChild` | `2` | yes |
| `LastChild` | `3` | yes |

### Constants

| Name | Value |
|---|---|
| `PADDING_X` | `24.0` |
| `PADDING_BOTTOM` | `20.0` |
| `TITLE_ROW_HEIGHT` | `48.0` |
| `MIN_DIALOG_WIDTH` | `280.0` |
| `MIN_DIALOG_HEIGHT` | `120.0` |
| `MAX_SCREEN_FRACTION` | `0.9` |

---

## Examples

### Basic confirmation dialog

```rust
let mut dialog = DialogState::new("confirm");
if ui.button("Delete").clicked() { dialog.open(); }

let resp = MudDialog::new()
    .title("Delete item?")
    .options(DialogOptions::new().with_close_button(true))
    .show_with_actions(ui, &mut dialog,
        |ui, _| { ui.add(MudText::body2("This cannot be undone.")); },
        |ui, ctl| {
            if ui.add(MudButton::new("Cancel")).clicked() { ctl.cancel(); }
            if ui.add(MudButton::new("Delete")).clicked() { ctl.close(); }
        });

if resp.closed {
    log::info!("canceled = {}", resp.result.as_ref().is_some_and(|r| r.is_canceled()));
}
```

### Returning a value

```rust
let resp = MudDialog::new().title("Rename").show_with_actions(ui, &mut dialog,
    |ui, ctl| {
        ui.add(MudTextField::new(&mut buffer, &mut touched).label("New name"));
        // `ctl` is available if the content needs to close.
        let _ = ctl;
    },
    |ui, ctl| {
        if ui.add(MudButton::new("Save")).clicked() {
            ctl.close_with(DialogResult::ok_with(buffer.clone()));
        }
    });

if let Some(result) = resp.result {
    if let Some(name) = result.data::<String>() {
        rename(name.clone());
    }
}
```

### Full-width dialog capped at `Md`

```rust
MudDialog::new()
    .title("Settings")
    .options(DialogOptions::new()
        .with_full_width(true)
        .with_max_width(MaxWidth::Md)
        .with_close_button(true))
    .show(ui, &mut dialog, |ui, _| { /* … */ });
```

### Full screen

```rust
MudDialog::new()
    .options(DialogOptions::new().with_full_screen(true))
    .show(ui, &mut dialog, |ui, _| { /* … */ });
```

### Top-centered, no header

```rust
MudDialog::new()
    .options(DialogOptions::new()
        .with_position(DialogPosition::TopCenter)
        .with_no_header(true))
    .show(ui, &mut dialog, |ui, _| { /* … */ });
```

### Backdrop click fires a callback instead of closing

```rust
MudDialog::new()
    .on_backdrop_click(|| log::info!("clicked outside"))
    .options(DialogOptions::new().with_backdrop_click(true))
    .show(ui, &mut dialog, |ui, _| { /* … */ });
```

### Programmatic open / close

```rust
dialog.open();                 // IsVisible = true
dialog.close();                // Close(DialogResult.Ok())
dialog.cancel();               // Close(DialogResult.Cancel())
dialog.close_with(DialogResult::ok_with(42u32));
dialog.set_title(Some("Renamed".into()));   // SetTitleAsync
dialog.set_options(DialogOptions::new().with_close_button(true)); // SetOptionsAsync
```

### Reading the result later

```rust
// If you closed from outside `show`:
if let Some(result) = dialog.take_result() {
    log::info!("closed: {result:?}");
}
```

### Title as a custom body

```rust
MudDialog::new()
    .title_content(|ui| {
        ui.horizontal(|ui| {
            ui.add(MudIcon::new(filled::WARNING).color(Color::Warning));
            ui.add(MudText::h6("Careful"));
        });
    })
    .show(ui, &mut dialog, |ui, _| { /* … */ });
```

---

## Layout and interaction

### Anatomy

```
┌───────────────────────────────────────┐  ← backdrop (Order::Middle, 96 α black)
│  ┌─────────────────────────────────┐  │  ← panel (Order::Foreground)
│  │ [ title ]                 [ ✕ ] │  │     TITLE_ROW_HEIGHT = 48
│  ├─────────────────────────────────┤  │
│  │ content                         │  │     PADDING_X = 24 (gutters)
│  │                                 │  │
│  ├─────────────────────────────────┤  │
│  │                     [ actions ] │  │     PADDING_BOTTOM = 20
│  └─────────────────────────────────┘  │
└───────────────────────────────────────┘
```

### Sizing rules

| Condition | Width | Height |
|---|---|---|
| `full_screen` | screen width | screen height |
| `full_width` | `min(90 % screen, max_width_px)` | measured, clamped |
| default | `clamp(measured, MIN_DIALOG_WIDTH, min(90 % screen, max_width_px))` | `clamp(measured, MIN_DIALOG_HEIGHT, 90 % screen)` |

The panel is positioned inside a `screen.shrink(24)` rect using
`DialogPosition::fractions()`.

### Interaction

| Input | Effect |
|---|---|
| Click outside the panel, `backdrop_click(true)`, no `on_backdrop_click` | `Cancel` |
| Click outside the panel, `backdrop_click(true)`, `on_backdrop_click` set | The callback fires; the dialog stays open |
| Click the ✕, `close_button(true)` | `Cancel` |
| `Escape`, `close_on_escape_key(true)` | `Cancel` |
| Content calls `ctl.close()` / `ctl.cancel()` / `ctl.close_with(..)` | Applied after the panel is drawn |
| Content calls `ctl.set_title(..)` / `ctl.set_options(..)` | Applied after the panel is drawn |

### Focus

`DefaultFocus::{Element, FirstChild, LastChild}` request focus on the opening
frame via `Response::request_focus()`. `None` skips it. egui has no focus
trap, so Tab cycling inside the dialog is the host application's job.

### Size cache

The panel is positioned from the *previous* frame's measured size (or
`DEFAULT_DIALOG_SIZE` on the first open frame), then the measured size is
written back. This is the same one-frame-lag pattern `MudPopover` uses; it is
invisible in practice because a dialog is opened by a click and settles within
a frame or two.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| `DialogState` is the only persistent allocation | One `Id`, one `Option<String>`, one `DialogOptions`. |
| `DialogOptions` is `Clone`, not `Copy` | It owns `Option<String>` for the class names; cloning it per frame is four pointer copies. |
| Actions closure is boxed **only when present** | `show` allocates nothing for the actions row; `show_with_actions` allocates one `Box`. |
| Size cache in `egui::Memory` | Two `get_temp` / `insert_temp` per frame; no caller-side bookkeeping. |
| Backdrop is one `rect_filled` | No shape tree, no `Frame`. |
| Panel is one `egui::Frame` | One fill, one corner radius, one shadow. |
| Deferred control requests | `DialogState` is never borrowed while the panel is drawn, so the content can call `close` / `set_title` freely. |
| `MudText` / `MudIcon` reuse | Theme and typography propagate; the icon cache absorbs repeated ✕ glyphs. |
| Pure geometry helpers | Position and size resolution are arithmetic; testable without a `Context`. |

---

## Testing

`cargo test -p mud_egui_components mud_dialog` runs the module's tests:

| Area | Tests |
|---|---|
| `DialogPosition` | default, all ten discriminants, every fraction pair, unique names, `Display` |
| `DefaultFocus` | default, discriminants, `requests_focus`, unique names, `Display` |
| `DialogResult` | `ok` / `cancel` / `ok_with`, `has_data`, `data::<T>` / `take_data::<T>`, wrong type, `Debug` hides payload, `Default` is `Cancel` |
| `DialogOptions` | defaults, every `with_*` setter, `effective_position`, clone independence |
| `DialogControl` | no initial requests, `close` / `cancel` / `close_with`, `set_title` / `set_options`, draining, last-close-wins |
| `DialogState` | construction, `open` requests focus once, `open_with`, `close` / `cancel` / `close_with`, `take_result`, `open` clears a stale result, overrides, stable id |
| `position_origin` | center, all four corners, all four edges, `Custom`, oversized clamp |
| `resolve_panel_size` | natural fit, tiny clamp-up, `max_width` cap, `full_width`, `full_width` + `max_width`, `full_screen`, height cap, zero-size screen |
| `dialog_max_width_px` | every bucket |
| `elevation_shadow` | zero, clamp above 24 |
| Builders | defaults, every field set |

Rendering needs a live `egui::Context`; the pure layer is the source of truth.

```bash
cargo test -p mud_egui_core dialog_position default_focus
cargo test -p mud_egui_components dialog_result
cargo test -p mud_egui_components dialog_options
cargo test -p mud_egui_components dialog_state
cargo test -p mud_egui_components dialog_control
cargo test -p mud_egui_components mud_dialog
```

---

## Differences from MudBlazor

- **`IDialogService.ShowAsync<T>` → `DialogState` + `MudDialog::show`.** egui
  has no reflection and no per-frame async runtime, so a dialog is a widget
  whose state the caller owns — the same model `MudDatePicker`,
  `MudColorPicker` and every other component in this workspace use.
- **`IMudDialogInstance` → `&mut DialogControl`.** The content and actions
  closures receive a handle with `close` / `cancel` / `close_with` /
  `set_title` / `set_options`. Requests are deferred and applied after the
  panel is drawn, so the closures never borrow `DialogState`.
- **`DialogResult.Data` is `Box<dyn Any + Send + Sync>`.** `Ok<T>(T)` maps to
  `ok_with(data)`; read it back with `data::<T>()` or `take_data::<T>()`.
  `DialogResult` is therefore not `PartialEq`.
- **`DialogOptions` is a `Clone` struct, not a `Copy` one.** It owns an
  `Option<String>` for `background_class`; cloning it per frame is four
  pointer copies.
- **Class strings are stored, not resolved.** `ActionsClass`, `ContentClass`,
  `ContentStyle`, `TitleClass` and `BackgroundClass` are CSS strings with no
  egui equivalent. The panel owns fill, corner radius and shadow; resolving a
  second utility frame would composite a second surface.
- **`OnKeyDown` / `OnKeyUp` are deferred.** Browser `KeyboardEventArgs` have
  no egui equivalent. `Escape` is honoured through
  `DialogOptions::close_on_escape_key`.
- **`CancelAll()` is not ported.** It needs a parent-dialog stack; a
  caller-owned state has no such stack.
- **No focus trap.** `DefaultFocus` requests focus on the opening frame;
  Tab cycling inside the dialog is the host application's job.
- **`Style` / `Tag` / `UserAttributes` are not ported.** No CSS strings, no
  HTML tag.
- **The panel size is cached for one frame.** The first open frame positions
  from `DEFAULT_DIALOG_SIZE` (560 × 320) and the second from the measured
  size. Same one-frame lag `MudPopover` documents; invisible in practice.
- **The backdrop is always dimmed.** MudBlazor's `BackdropClick` only controls
  dismissal; there is no parameter to remove the dim. The port matches at
  96 α.