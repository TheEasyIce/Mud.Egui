# MudHotkey for `mud_egui`

A hotkey listener that reveals or hides its child content for
[egui](https://github.com/emilk/egui).

> **Version note.** `MudHotkey` was introduced in **MudBlazor 9.0.0**, not in
> 8.13.0. The rest of this workspace targets 8.13.0; this component is the one
> deliberate version exception, documented here so the README's parity table
> cites 9.0.0 rather than 8.13.0.

```rust
let mut open_search = false;

MudHotkey::new(Key::K)
    .key_modifiers(Modifiers::CTRL)
    .on_hotkey_pressed(|_| open_search = true)
    .show(ui, |ui| {
        ui.add(MudText::body1("Search panel"));
    });
```

The content closure supplies every string and glyph; `MudHotkey` paints none of
its own. Route them through **`MudText`** and **`MudIcon`** as usual.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [How the hotkey is detected](#how-the-hotkey-is-detected)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The **9.0.0** parameter surface was verified against the MudBlazor 9 docs and
the v9.6.0 / v9.5.0 release notes before implementation. `MudHotkey` has
**seven parameters**, and all seven are accounted for.

### Ported parameters

| MudBlazor 9.0.0 | Type | Default | Rust mapping |
|---|---|---|---|
| `Key` | `JsKey` | — | `new(Key)` |
| `KeyModifiers` | `IReadOnlyCollection<JsKeyModifier>` | empty | `key_modifiers(Modifiers)` |
| `OnHotkeyPressed` | `EventCallback<HotkeyEventArgs>` | — | `on_hotkey_pressed(f)` + `HotkeyResponse::pressed` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `PreventEventPropagation` | `bool` | `false` | `prevent_event_propagation(bool)` |
| `HideChildContentOnRepress` | `bool` | `true` | `hide_child_content_on_repress(bool)` |
| `ChildContent` | `RenderFragment?` | `null` | the closure passed to `show(ui, \|ui\| ..)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `Style` | egui has no inline CSS strings. `MudHotkey` renders no element of its own. |
| `UserAttributes` | There is no HTML tag to splat attributes onto. |
| `Class` resolution | Stored on `base` for parity. `MudHotkey` adds no wrapper frame. |

### Substitutions

| MudBlazor | Rust | Reason |
|---|---|---|
| `JsKey` | `egui::Key` | `JsKey` covers every browser `KeyboardEvent.code`; egui's `Key` is a smaller, native enum. Common keys (`KeyA`..`KeyZ`, `Digit0`..`Digit9`, arrows, `Enter`, `Tab`, `Escape`) map 1:1; browser-only codes (`NumpadEqual`, `IntlBackslash`) have no egui equivalent. |
| `JsKeyModifier` | `egui::Modifiers` | `JsKeyModifier` distinguishes left and right (`ControlLeft` vs `ControlRight`); `egui::Modifiers` is a set of flags (`ctrl`, `shift`, `alt`, `mac_cmd`, `command`) without a side. The distinction is the subject of an open MudBlazor issue (#13390, "XxxRight modifiers do not work"); egui sidesteps it entirely. |

### Extensions

| Extension | Type | Default | Purpose |
|---|---|---|---|
| `id_salt(impl Hash + Debug)` | `Id` | `None` | Distinguishes two `MudHotkey`s in one parent, whose `visible` toggles would otherwise share a memory key. Same shape as `MudCollapse::id_salt`. |
| `with_modifiers(key, modifiers)` | — | — | Convenience constructor: `MudHotkey::new(key).key_modifiers(m)`. |

---

## Architecture

```
layout/hotkey/
├── mud_hotkey.rs
│   ├── MudHotkey           builder + show(ui, add_content) + Widget impl
│   ├── HotkeyEventArgs     { key, repeat }
│   ├── HotkeyResponse<R>   { response, pressed, repeat, visible, inner }
│   └── detect_press        pure — scans `i.events` for a matching Key event
└── mod.rs                  re-exports
```

**Pure layer.** `detect_press(ui, key, modifiers)` scans egui's per-frame
`Event::Key` list once and reports `(pressed, repeat)`. It is the only place
the matching logic lives.

**State layer.** One `bool` per instance lives in `egui::Memory`, keyed by
`<ui.id().with(id_salt)>.hotkey_visible`. It exists only for the
`HideChildContentOnRepress` toggle.

**Component layer.** `show` tests the hotkey, fires the callback, optionally
consumes the key, and either runs the content closure or allocates nothing.

---

## API reference

### `MudHotkey`

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(key)` | `Key` | `Space` (via `Default`) | `Key` | Creates a listener for `key` with no modifiers. |
| `with_modifiers(key, m)` | `(Key, Modifiers)` | — | — | Convenience constructor. |
| `key_modifiers(m)` | `Modifiers` | `Modifiers::NONE` | `KeyModifiers` | Required modifier set. |
| `on_hotkey_pressed(f)` | `FnMut(&HotkeyEventArgs)` | `None` | `OnHotkeyPressed` | Callback fired on every press. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Disables the hotkey; content stays visible. |
| `prevent_event_propagation(b)` | `bool` | `false` | `PreventEventPropagation` | Consumes the key so other widgets do not see it. |
| `hide_child_content_on_repress(b)` | `bool` | `true` | `HideChildContentOnRepress` | Toggles content visibility on each press. |
| `id_salt(s)` | `impl Hash + Debug` | `None` | *(extension)* | Distinguish two instances in one parent. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, \|ui\| ..)` | `&mut Ui, FnOnce(&mut Ui) -> R` | — | `ChildContent` | Returns `HotkeyResponse<R>`. |
| `ui.add(..)` | — | — | — | Tests the hotkey but draws no content. |

### `HotkeyResponse<R>`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The content's response when visible; a zero-sized hover response when hidden. Deref target. |
| `pressed` | `bool` | `true` on the frame the hotkey was pressed. Mirrors the trigger for `OnHotkeyPressed`. |
| `repeat` | `bool` | `true` when `pressed` is a repeat from a held key. Mirrors `HotkeyEventArgs.OnRepeat`. |
| `visible` | `bool` | Current content visibility. |
| `inner` | `Option<R>` | The content closure's value; `None` when hidden. |

### `HotkeyEventArgs`

| Field | Type | Description |
|---|---|---|
| `key` | `Key` | The key that completed the combination. |
| `repeat` | `bool` | `true` on a repeat from a held key. |

### `egui::Modifiers` cheat sheet

`Modifiers` is a struct of `bool` fields; `Modifiers::NONE` is "no modifier".
Common combinations are built from the constants:

| Constant | Value |
|---|---|
| `Modifiers::NONE` | no modifiers |
| `Modifiers::CTRL` | `ctrl: true` |
| `Modifiers::SHIFT` | `shift: true` |
| `Modifiers::ALT` | `alt: true` |
| `Modifiers::MAC_CMD` | `mac_cmd: true` |
| `Modifiers::COMMAND` | `command: true` (Ctrl on non-Mac, Cmd on Mac) |

For more than one, `Modifiers { ctrl: true, shift: true, ..Default::default() }`.

---

## Examples

### Ctrl+K opens a search panel

```rust
MudHotkey::with_modifiers(Key::K, Modifiers::CTRL)
    .on_hotkey_pressed(|_| self.open_search = true)
    .show(ui, |ui| {
        ui.add(MudText::body1("Search panel"));
    });
```

### Ctrl+S saves, and swallows the key

```rust
MudHotkey::with_modifiers(Key::S, Modifiers::CTRL)
    .prevent_event_propagation(true)
    .on_hotkey_pressed(|_| self.save())
    .show(ui, |_| {});
```

`prevent_event_propagation(true)` calls `Context::consume_key`, so a text
field in the same frame does not also see the `S`. This is the closest egui
equivalent to MudBlazor's `PreventEventPropagation`.

### Hide the panel on Ctrl+H, reveal it again on the next press

```rust
MudHotkey::with_modifiers(Key::H, Modifiers::CTRL)
    .show(ui, |ui| {
        ui.add(MudText::body1("Help panel"));
    });
```

Default `hide_child_content_on_repress(true)` means: the content starts
visible, the first `Ctrl+H` hides it, the next `Ctrl+H` shows it again.

### Callback only, never hide the content

```rust
MudHotkey::with_modifiers(Key::F5, Modifiers::NONE)
    .hide_child_content_on_repress(false)
    .on_hotkey_pressed(|_| self.reload())
    .show(ui, |_| {});
```

With `hide_child_content_on_repress(false)` the content stays visible and the
callback still fires. Pass an empty closure if you have nothing to draw.

### Reading the press directly

```rust
let out = MudHotkey::with_modifiers(Key::Escape, Modifiers::NONE)
    .show(ui, |ui| { ui.label("Press Esc to close"); });
if out.pressed {
    self.dialog_open = false;
}
```

`out.pressed` is `true` for exactly one frame per press, mirroring
`OnHotkeyPressed`.

### Disabled

```rust
MudHotkey::with_modifiers(Key::K, Modifiers::CTRL)
    .disabled(true)
    .on_hotkey_pressed(|_| self.open_search = true)
    .show(ui, |ui| { ui.label("Disabled"); });
```

`disabled(true)` skips the match entirely; the content stays visible and the
callback does not fire.

### Two hotkeys in one parent

```rust
MudHotkey::with_modifiers(Key::K, Modifiers::CTRL)
    .id_salt("search")
    .show(ui, |ui| { ui.label("Search"); });

MudHotkey::with_modifiers(Key::H, Modifiers::CTRL)
    .id_salt("help")
    .show(ui, |ui| { ui.label("Help"); });
```

Without distinct salts the two share the `visible` memory key and their toggles
would cross.

---

## How the hotkey is detected

egui exposes keyboard input two ways:

- `ui.input(|i| i.key_pressed(Key))` — a convenience for "was this key pressed
  this frame". It returns `true` for the initial press *and* for repeats, and
  it does not carry the modifier state at the moment of the press.
- `ui.input(|i| &i.events)` — the raw per-frame `Event` list, including
  `Event::Key { key, pressed, repeat, modifiers, .. }`.

`MudHotkey` uses the second form because it needs the `repeat` flag and the
modifiers that were held *at the moment of the press*. The scan is a single
`for` loop over the frame's events, so it costs nothing beyond the event
iteration egui already performs.

Matching is `key == want_key && modifiers.matches_logically(want_modifiers)`.
`matches_logically` ignores the difference between a physical and a logical
key, and it requires that every "on" bit of `want_modifiers` is set on the
event; extraneous modifiers are *not* permitted (so `Ctrl+K` does not fire on
`Ctrl+Shift+K`). If you need the looser "at least these modifiers" rule, wrap
the callback in your own check on `event.modifiers` — the component follows
MudBlazor's exact-match behaviour.

### `repeat`

`HotkeyEventArgs::repeat` is `true` for events where egui's `Event::Key`
carries `repeat: true`. That fires while the key is held, once per repeat
interval. MudBlazor's `HotkeyEventArgs.OnRepeat` is the same idea.

### `prevent_event_propagation`

`Context::consume_key(modifiers, key)` marks the key as consumed for the rest
of the frame, so a `TextEdit` (or any other widget) that checks
`i.key_pressed(key)` after `MudHotkey` runs will not see it. This is the closest
egui equivalent of `event.stopPropagation()` in the browser.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Content closure is `FnOnce` | Runs at most once, and only when the content is visible. |
| `detect_press` scans events once | One `for` loop per frame; no per-key lookup, no allocation. |
| `repeat` carried as a flag | The callback can distinguish a held key from a fresh press without the component storing previous state. |
| One `bool` in `egui::Memory` | Only the `HideChildContentOnRepress` toggle persists. |
| Zero-sized hover response when hidden | Takes no layout space and steals no clicks. |
| No theme fetch | `MudHotkey` resolves no colours; the child content owns its own theming. |
| `id_salt` for disambiguation | Two instances in one parent do not share their toggle. |

---

## Testing

`cargo test -p mud_egui_components mud_hotkey` runs 9 unit tests:

| Area | Tests |
|---|---|
| `detect_press` | Key match with modifiers; wrong key rejected; wrong modifiers rejected; repeat reported; releases ignored |
| Builders | Defaults match MudBlazor 9.0.0 (`HideChildContentOnRepress = true`); every field set; `with_modifiers` convenience; `id_salt` stability |

The `detect_press` tests use `Context::run_ui` with a crafted `RawInput` that
carries a single `Event::Key`, so they exercise the real event-scanning path
without a live window.

`MudHotkey::show` needs a live `Ui` and is left to integration tests.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness):

- A `Ctrl+K` `MudHotkey` fires its callback on the frame the key is pressed and
  on no other frame.
- The content toggle starts visible, hides on the first press, shows on the
  second.
- `hide_child_content_on_repress(false)` keeps the content visible across
  presses.
- `disabled(true)` never fires the callback and never hides the content.
- `prevent_event_propagation(true)` causes a `TextEdit` in the same frame not
  to receive the key.
- Two instances with distinct `id_salt`s toggle independently.

```bash
cargo test -p mud_egui_components mud_hotkey
```

---

## Differences from MudBlazor

- **This component targets MudBlazor 9.0.0, not 8.13.0.** MudHotkey did not
  exist in 8.13.0; it was added in the v9.0.0 release notes. The rest of the
  workspace targets 8.13.0. The parity table above cites 9.0.0 so the version
  exception is explicit.
- **`JsKey` → `egui::Key`.** `JsKey` covers every browser `KeyboardEvent.code`;
  egui's key enum is smaller and does not include browser-only codes
  (`NumpadEqual`, `IntlBackslash`, `Lang1`…). Every key the component needs in
  practice (`A`..`Z`, `0`..`9`, arrows, `Enter`, `Tab`, `Escape`, `F1`..`F12`)
  is present in `egui::Key`.
- **`JsKeyModifier` → `egui::Modifiers`.** egui's `Modifiers` is a set of
  flags without a left/right side. MudBlazor's own tracker has an open issue
  (#13390) about `XxxRight` modifiers not working; egui avoids the problem by
  not distinguishing them.
- **`OnHotkeyPressed` is a callback, not a two-way binding.** The component
  fires `on_hotkey_pressed` **and** reports `HotkeyResponse::pressed` (with
  `repeat`), so a caller can use either the callback or the return value. This
  is the same split `MudCollapse` and `MudRating` use.
- **`ChildContent` visibility is caller-controlled.** The `HideChildContentOnRepress`
  toggle is implemented in `egui::Memory`, keyed by the widget id, so two
  `MudHotkey`s in one parent can be given distinct `id_salt`s. MudBlazor does
  the same thing with an internal `_childContentVisible` field.
- **`PreventEventPropagation` maps to `Context::consume_key`, not
  `stopPropagation`.** egui has no DOM event bubbling to stop; `consume_key`
  is the closest equivalent and prevents later widgets in the same frame from
  seeing the key.
- **`Class` is stored but not resolved.** `MudHotkey` renders no wrapper
  element, so a class list has no surface to paint.
- **`Style` and `UserAttributes` are not ported.** No CSS strings, no HTML tag.