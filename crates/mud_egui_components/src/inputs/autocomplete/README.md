# MudAutocomplete for `mud_egui`

A text field with a filtered dropdown of candidates, single-value
selection only, for [egui](https://github.com/emilk/egui). Ported from
MudBlazor's `MudAutocomplete<T>` (MudBlazor 8.13.0).

```rust
let mut state = AutocompleteState::<Fruit>::new("fruit");

// Refresh candidates when the search text changes.
if state.text_changed_since_search() {
    state.candidates = fruits
        .iter()
        .filter(|f| f.name.to_lowercase().contains(&state.search_text.to_lowercase()))
        .cloned()
        .collect();
    state.mark_searched();
}

MudAutocomplete::new()
    .label("Fruit")
    .tostring_func(|f| f.name.clone())
    .show(ui, &mut state);
```

The trigger renders through **`MudTextField`** (so its label, placeholder,
helper, error, variant, margin, size, and search-icon adornment come from the
same widget). Candidate rows render text through **`MudText`**; any custom
per-item icons or badges go through `MudIcon` / `MudBadge` inside
`item_template`.

---

## Design principle: caller-owned async, not per-frame `await`

MudBlazor's `SearchFunc` returns `Task<IEnumerable<T>>`. egui is
immediate-mode with no async runtime per frame. The port therefore
**delegates the search to the caller**:

| MudBlazor concept | Rust equivalent |
|-------------------|-----------------|
| `SearchFunc: Func<string, Task<IEnumerable<T>>>` | `state.candidates: Vec<T>` — the caller populates it. |
| Debounce via `DebounceInterval` | `state.text_changed_since_search()` + `mark_searched()`. |
| Async channel-based search | Push results into `candidates` from a channel reader; the component never knows. |
| `ShowProgressIndicator` | Caller can render a spinner above the popover when their channel is empty but still awaiting. |

This is documented in the module header of `mud_autocomplete.rs` and matches
the caller-owned-state pattern every other component in this batch follows
(`MudTable`, `MudTreeView`, `MudChipSet`, `MudNavMenu`, `MudPagination`,
`MudStepper`, `MudTabs`, `MudSelect`).

### Composition, not reimplementation

`MudAutocomplete` reuses the same two primitives as `MudSelect`:

| Primitive | Role |
|-----------|------|
| [`MudTextField`](../../inputs/text_field/README.md) | The editable trigger. |
| [`MudPopover`](../../overlays/popover/README.md) | The candidate dropdown. |

Only new code is the **candidate list** (rows + keyboard focus) and the
**commit / coerce / reset logic** — both pure helpers tested without
`egui::Context`.

---

## Pass 1 scope and pass 2 deferral

### Ported `MudAutocomplete` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Value` | `AutocompleteState::selected` (`Option<T>`) | `None` |
| `Text` | `AutocompleteState::search_text` | `""` |
| `ToStringFunc` | `tostring_func(f)` | `None` |
| `SearchFunc` | *(caller-owned candidates)* | — |
| `Label` | `label(s)` | `None` |
| `Placeholder` | `placeholder(s)` | `None` |
| `HelperText` | `helper_text(s)` | `None` |
| `Error` | `error(bool)` | `false` |
| `ErrorText` | `error_text(s)` | `None` |
| `Required` | `required(bool)` | `false` (visual only) |
| `Disabled` | `disabled(bool)` | `false` |
| `ReadOnly` | `readonly(bool)` | `false` |
| `Clearable` | `clearable(bool)` | `true` |
| `Variant` | `variant(Variant)` | `Text` |
| `Margin` | `margin(Margin)` | `None` |
| `Color` | `color(Color)` | `Primary` |
| `Size` | `size(Size)` | `Medium` |
| `Dense` | `dense(bool)` | `false` |
| `Adornment` + `AdornmentIcon` + `AdornmentColor` | `adornment_icon(s)` / `adornment_color(Color)` | `SEARCH` / `None` |
| `AnchorOrigin` | `anchor_origin(AnchorOrigin)` | `None` |
| `TransformOrigin` | `transform_origin(TransformOrigin)` | `None` |
| `Position` | `position(Position)` | `None` |
| `MaxItems` | `max_items(usize)` | `None` |
| `MinCharacters` | `min_characters(usize)` | `0` |
| `OpenOnFocus` | `open_on_focus(bool)` | `true` |
| `ResetValueOnEmptyText` | `reset_value_on_empty_text(bool)` | `false` |
| `NoItemsTemplate` | `no_items_template(f)` | `None` |
| `AfterItemsTemplate` | `after_items_template(f)` | `None` |
| `ItemTemplate` | `item_template(f)` | `None` |
| `OnOpen` | `on_open(f)` | `None` |
| `OnClose` | `on_close(f)` | `None` |

### Deferred to pass 2

| Deferred parameter | Why pass 2 |
|--------------------|------------|
| `CoerceValue` / `CoerceValueDelay` | Blur coercion (typed text exactly matching one candidate). Straightforward, deferred for pass-1 simplicity. |
| `DebounceInterval` | Overlaps with the caller-owned search model; the caller uses `text_changed_since_search` and their own timing. |
| `Strict` | Reverting on typed-but-not-coerced text — depends on CoerceValue. |
| `SelectValueOnTab` | One keybinding; deferred. |
| `ShowProgressIndicator` / `ProgressIndicatorColor` / `ProgressIndicatorInPopoverTemplate` | Depends on async; the caller renders their own spinner. |
| `MoreItemsTemplate` | "Showing N of M" footer — needs `TotalItems` count which async callers may not have. |
| `KeyDownPreventDefault` / `KeyUpPreventDefault` | Browser-specific. |
| `HelperTextOnFocus` | Helper row behaviour. |
| `ShrinkLabel` / `DisableUnderLine` / `AriaLabel` | Cosmetic / accessibility refinements. |
| `AfterItemsTemplate` with parameter / `NoItemsTemplate` with parameter | The pass-1 forms take no argument. |
| `ReturnedItemsCount` | Available as `state.candidates.len()`. |

### Not ported

- **`Class` / `Style` / `Tag` / `UserAttributes`** — no CSS strings or HTML
  tags in egui.
- **`InputClass` / `OuterClass`** — CSS string variants, same reason.

---

## Architecture

```
inputs/autocomplete/
├── autocomplete_action.rs  — AutocompleteAction<T>
├── autocomplete_state.rs   — AutocompleteState<T>
├── mud_autocomplete.rs     — MudAutocomplete + pure helpers + tests
└── mod.rs
```

**Pure layer.** `row_height`, `display_text_of`, `next_focused_index`,
`prev_focused_index`, `commit_selection`, `position_to_origins` — free
functions, unit-tested without `egui::Context`.

**State layer.** `AutocompleteState<T>` holds the search text, the
caller-populated candidates, the committed selection, and the popover state.
`text_changed_since_search` and `mark_searched` provide the debounce
primitive.

**Component layer.** `show` renders the trigger (`MudTextField`), then handles
open/close + keyboard navigation + candidate clicks, then delegates popover
positioning to `MudPopover`.

---

## API reference — `MudAutocomplete<T>`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default autocomplete. |
| `tostring_func(f)` | `Fn(&T) -> String` | `None` | Candidate display text. `"?"` when absent. |
| `error(bool)` / `error_text(s)` | `bool` / `impl Into<String>` | `false` / `None` | Error visual only. |
| `required(bool)` | `bool` | `false` | Visual only. |
| `disabled(bool)` / `readonly(bool)` | `bool` | `false` | Interaction flags. |
| `clearable(bool)` | `bool` | `true` | Shows the clear button. |
| `open_on_focus(bool)` | `bool` | `true` | Auto-open on focus. |
| `reset_value_on_empty_text(bool)` | `bool` | `false` | Clear selection when text is cleared. |
| `min_characters(usize)` | `usize` | `0` | Min text before the dropdown opens. |
| `max_items(usize)` | `usize` | `None` | Max candidates shown. |
| `label(s)` / `placeholder(s)` / `helper_text(s)` | `impl Into<String>` | `None` | Trigger text slots. |
| `variant(Variant)` / `margin(Margin)` / `color(Color)` / `size(Size)` | enum | `Text` / `None` / `Primary` / `Medium` | Trigger appearance. |
| `dense(bool)` | `bool` | `false` | Compact candidate rows. |
| `adornment_icon(s)` / `adornment_color(Color)` | `impl Into<String>` / `Color` | `SEARCH` / `None` | Search icon and colour. |
| `position(Position)` / `anchor_origin(..)` / `transform_origin(..)` | placement | `None` | Popover placement. |
| `right_to_left(bool)` | `bool` | `false` | RTL flag. |
| `max_height(f32)` / `popover_class(s)` | `f32` / `impl Into<String>` | `300.0` / `None` | Popover box. |
| `item_template(f)` | `FnMut(&mut Ui, &T)` | `None` | Custom candidate renderer. |
| `no_items_template(f)` | `FnMut(&mut Ui)` | `None` | Custom "no results" renderer. |
| `after_items_template(f)` | `FnMut(&mut Ui)` | `None` | Footer under the candidate list. |
| `on_open(f)` / `on_close(f)` | `FnMut()` | `None` | Open/close callbacks. |
| `classes(iter)` / `flavor(StyleFlavor)` | — | — | Utilities. |
| `show(ui, state)` | `(&mut Ui, &mut AutocompleteState<T>)` | — | Returns `AutocompleteResponse<T>`. |

### `AutocompleteState<T>`

| Field / method | Description |
|----------------|-------------|
| `search_text: String` | Current text (edited by the `TextEdit`). |
| `selected: Option<T>` | Committed selection. |
| `candidates: Vec<T>` | Caller-owned candidate list. |
| `popover: PopoverState` | Open/closed + stable id. |
| `focused_candidate: Option<usize>` | Keyboard-focused index. |
| `last_searched_text: String` | Snapshot for `text_changed_since_search`. |
| `new(id_salt)` / `with_selected_text(v, s)` | Constructors. |
| `is_open()` / `open()` / `close()` / `toggle()` | Popover control. |
| `select(v, s)` / `clear()` | Programmatic commit / reset. |
| `text_changed_since_search()` / `mark_searched()` | Debounce primitive. |
| `is_selected(&T)` | Membership check. |

### `AutocompleteResponse<T>`

| Field | Type | Description |
|-------|------|-------------|
| `trigger` | `egui::Response` | Trigger response. |
| `selected` | `Option<T>` | Selection after this frame. |
| `search_text` | `String` | Search text after this frame. |
| `action` | `AutocompleteAction<T>` | What happened this frame. |

### `AutocompleteAction<T>`

| Variant | Description |
|---------|-------------|
| `None` | No interaction. |
| `Opened` / `Closed` | Dropdown opened / closed. |
| `Selected { value }` | A candidate was committed. |
| `Cleared` | Selection cleared. |

Helpers: `changed_selection()`, `toggled_open()`.

---

## Examples

### Basic autocomplete

```rust
let mut state = AutocompleteState::<String>::new("search");

if state.text_changed_since_search() {
    state.candidates = all_fruits
        .iter()
        .filter(|f| f.to_lowercase().contains(&state.search_text.to_lowercase()))
        .cloned()
        .collect();
    state.mark_searched();
}

MudAutocomplete::new()
    .label("Fruit")
    .tostring_func(|s| s.clone())
    .show(ui, &mut state);
```

### Async search via channel

```rust
// In your app struct:
//   search_tx: Sender<String>
//   search_rx: Receiver<Vec<Fruit>>
//   last_dispatched: String
//
// Each frame:
if state.text_changed_since_search() {
    // Dispatch the search (non-blocking).
    search_tx.try_send(state.search_text.clone()).ok();
    state.mark_searched();
}
// Drain any results that have arrived.
while let Ok(results) = search_rx.try_recv() {
    state.candidates = results;
    ui.ctx().request_repaint();
}

MudAutocomplete::new()
    .label("Fruit")
    .tostring_func(|f| f.name.clone())
    .show(ui, &mut state);
```

### With item template (icon + secondary text)

```rust
MudAutocomplete::new()
    .tostring_func(|f: &Fruit| f.name.clone())
    .item_template(|ui, fruit| {
        ui.add(MudIcon::new(filled::ECO));
        ui.add(MudText::new(fruit.name.clone()));
        ui.add(MudText::new(fruit.origin.clone()).font_size(11.0));
    })
    .show(ui, &mut state);
```

### No-results template

```rust
MudAutocomplete::new()
    .no_items_template(|ui| {
        ui.add(MudText::new("No fruits matched your search"));
    })
    .show(ui, &mut state);
```

### Open on focus off (click to open)

```rust
MudAutocomplete::new()
    .open_on_focus(false)
    .show(ui, &mut state);
```

### Reset on empty text

```rust
MudAutocomplete::new()
    .reset_value_on_empty_text(true)
    .show(ui, &mut state);
// Clearing the text and blurring → `state.selected = None`.
```

### Max items and dense

```rust
MudAutocomplete::new()
    .max_items(5)
    .dense(true)
    .show(ui, &mut state);
```

### Min characters

```rust
MudAutocomplete::new()
    .min_characters(2)
    .show(ui, &mut state);
// The dropdown stays closed until at least 2 characters are typed.
```

### Read the outcome

```rust
let resp = MudAutocomplete::new().show(ui, &mut state);
match resp.action {
    AutocompleteAction::Selected { value } => log::info!("selected {value:?}"),
    AutocompleteAction::Cleared => log::info!("cleared"),
    AutocompleteAction::Opened => log::info!("opened"),
    AutocompleteAction::Closed => log::info!("closed"),
    AutocompleteAction::None => {}
}
```

### Programmatic set / clear

```rust
state.select(my_fruit.clone(), my_fruit.name.clone());
state.clear();
```

### Focus a candidate programmatically

```rust
state.focused_candidate = Some(0);
```

---

## Keyboard navigation

While the trigger has focus **and** the dropdown is open:

| Key | Action |
|-----|--------|
| `↓` | Focus next candidate (clamps at last) |
| `↑` | Focus previous candidate (clamps at first) |
| `Enter` | Commit the focused candidate |
| `Escape` | Close the dropdown |

Blur (Tab / click outside) closes the dropdown and reverts `search_text` to
the selected item's display text (or empty when nothing is selected).

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `display_text_of` / `next_focused_index` / `prev_focused_index` / `commit_selection` are pure | Unit-tested without `egui::Context`. |
| `AutocompleteState<T>` owns `candidates` | No per-frame allocation for candidates once populated. |
| `text_changed_since_search` + `mark_searched` | Prevents repeated filter runs on every frame. |
| Row rendering is index-based | Avoids `iter()` + mutation conflict; clone-on-click. |
| `item_template` is `take()`n from `self` before the render loop | No borrow conflict with `state.candidates` reads. |
| `with_alpha` uses `f32` multiplier | Matches `MudChip` / `MudSelect`. |
| `MudTextField` / `MudPopover` reuse | Zero duplication of trigger chrome or dropdown positioning. |
| `MudIcon` reuse across rows | Free thanks to the transform cache. |
| Focus fill + hover fill both checked | Keyboard focus is visible even when the mouse isn't moving. |

---

## Testing

`cargo test` runs 15+ pure-function tests:

| Area | Tests |
|------|-------|
| `row_height` | dense / normal |
| `display_text_of` | with / without `tostring_func` |
| `next_focused_index` / `prev_focused_index` | empty list, clamping, wrap-free behaviour |
| `commit_selection` | writes text, sets selected, closes popover, clears focused |
| `position_to_origins` | bottom / top / left / right |
| Defaults | every `MudAutocomplete` field matches MudBlazor |
| Builders | every field set |
| `AutocompleteState` | open / close / toggle, select / clear, text-changed helper |
| `AutocompleteAction` | `changed_selection` / `toggled_open` |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the
source of truth.

```bash
cargo test -p mud_egui_components mud_autocomplete
cargo test -p mud_egui_components autocomplete_state
cargo test -p mud_egui_components autocomplete_action
```

---

## Differences from MudBlazor

- **`SearchFunc` → caller-owned `candidates`.** MudBlazor's async `Task<IEnumerable<T>>` is not a fit for immediate-mode egui. The port exposes `text_changed_since_search` and `mark_searched` so callers can debounce and populate candidates themselves — sync filter, cached result, or channel read from an async task.
- **`CoerceValue` / `CoerceValueDelay` / `Strict` / `SelectValueOnTab` / `ShowProgressIndicator` deferred.** See the deferral table.
- **`ReturnedItemsCount` → `state.candidates.len()`.** No separate parameter needed.
- **`ResetValueOnEmptyText` is a real behaviour.** Blurring with empty text clears the selection (when enabled) — this is implemented.
- **`tostring_func` optional with `"?"` fallback.** MudBlazor requires either a `ToStringFunc` or `T: ToString`. The Rust port doesn't force `T: Display`; instead, absent `tostring_func` shows `"?"` in both the trigger and candidate rows (visible marker, no panic).
- **`ItemTemplate` / `NoItemsTemplate` / `AfterItemsTemplate` take `&mut Ui`.** MudBlazor render fragments do the same; no change.
- **`Class` / `Style` / `Tag` / `UserAttributes` not ported.** No CSS strings or HTML tags in egui.