# MudSelect for `mud_egui`

A dropdown that shows one item at a time (or several in multi-selection mode),
for [egui](https://github.com/emilk/egui). Ported from MudBlazor's
`MudSelect<T>` / `MudSelectItem<T>` (MudBlazor 8.13.0).

```rust
let mut state = SelectState::<&str>::new("fruit").with_selected(["apple"]);

let (resp, _) = MudSelect::new()
    .label("Fruit")
    .variant(Variant::Outlined)
    .show(ui, &mut state, |s| {
        s.text_item("apple", "Apple");
        s.text_item("banana", "Banana");
        s.text_item("cherry", "Cherry");
    });
```

Trigger label / placeholder / helper / error render through **`MudText`**
(via `MudTextField`). List rows render text through **`MudText`**, icons and
checkboxes through **`MudIcon`**.

---

## Design principle: composition, not reimplementation

`MudSelect` is **built from two primitives** rather than a fresh paint path:

| Primitive | Role |
|-----------|------|
| [`MudTextField`](../../inputs/text_field/README.md) with `readonly(true)` | The closed trigger. Gives `Variant`, `Margin`, `Size`, `Color`, `Label`, `Placeholder`, `HelperText`, `Error`, `ErrorText`, and the adornment icon in one parameter sweep. |
| [`MudPopover`](../../overlays/popover/README.md) | The dropdown. Anchor attachment, flip-when-overflowing, outside-click, Escape dismissal all delegated. |

Only new code is the **list body** (rows, checkboxes, select-all) and the
**selection state machine** — and both are pure helpers tested without an
`egui::Context`.

This keeps the port honest: any change to the trigger's visual language flows
through automatically, and any change to popover positioning flows through too.

---

## Pass 1 scope and pass 2 deferral

### Ported parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Value` / `SelectedValues` | `SelectState::selected` (`Vec<T>`) | required |
| `ToStringFunc` | `tostring_func(f)` | `None` |
| `Label` | `label(s)` | `None` |
| `Placeholder` | `placeholder(s)` | `None` |
| `HelperText` | `helper_text(s)` | `None` |
| `Error` | `error(bool)` | `false` |
| `ErrorText` | `error_text(s)` | `None` |
| `Required` | `required(bool)` | `false` (visual only) |
| `Disabled` | `disabled(bool)` | `false` |
| `ReadOnly` | `readonly(bool)` | `false` |
| `Clearable` | `clearable(bool)` | `false` |
| `MultiSelection` | `multi_selection(bool)` | `false` |
| `SelectAll` | `select_all(bool)` | `false` |
| `Strict` | `strict(bool)` | `false` |
| `LockScroll` | `lock_scroll(bool)` | `false` (stored, no-op) |
| `Variant` | `variant(Variant)` | `Text` |
| `Margin` | `margin(Margin)` | `None` |
| `Color` | `color(Color)` | `Primary` |
| `Size` | `size(Size)` | `Medium` |
| `FullWidth` | `full_width(bool)` | `false` |
| `DisableUnderLine` | `disable_underline(bool)` | `false` |
| `IconSize` | `icon_size(Size)` | `Medium` |
| `AdornmentColor` | `adornment_color(Color)` | `None` |
| `OpenIcon` | `open_icon(s)` | `ARROW_DROP_DOWN` |
| `CloseIcon` | `close_icon(s)` | `ARROW_DROP_UP` |
| `AnchorOrigin` | `anchor_origin(AnchorOrigin)` | `None` |
| `TransformOrigin` | `transform_origin(TransformOrigin)` | `None` |
| `Position` | `position(Position)` | `None` |
| `MaxHeight` | `max_height(f32)` | `300.0` |
| `PopoverClass` | `popover_class(s)` | `None` |
| `Dense` | `dense(bool)` | `false` |
| `OnOpen` | `on_open(f)` | `None` |
| `OnClose` | `on_close(f)` | `None` |
| `OnClearButtonClick` | `on_clear_button_click(f)` | `None` |
| `SelectedValuesChanged` | `on_selected_values_changed(f)` | `None` |
| `ChildContent` | `add_items` closure of `show` | required |

### Ported `MudSelectItem` parameters

| MudBlazor | Rust method | Default |
|-----------|-------------|---------|
| `Value` | `new(value)` / `text(value, s)` | required |
| `Text` | `text(s)` / `with_text(s)` | `None` |
| `Icon` | `icon(s)` | `None` |
| `Disabled` | `disabled(bool)` | `false` |
| `ChildContent` | `child(f)` | `None` |

### Deferred to pass 2

The following MudBlazor parameters are **deferred** and documented here so
the deferral is visible:

| Deferred parameter | Why pass 2 |
|--------------------|------------|
| `Counter` / `MaxLength` / `Pattern` | Form-level validation display |
| `For` / `Validation` / `OnlyValidateIfDirty` | Form validation model |
| `RequiredError` | Same as `Required` (pass 1 ships visual-only) |
| `Converter` / `Culture` / `Format` | Value formatting outside the `ToStringFunc` path |
| `Delimiter` / `MultiSelectionTextFunc` | Multi-selection text customisation |
| `AutoFocus` / `Immediate` / `InputMode` | Input internals |
| `KeyDownPreventDefault` / `KeyUpPreventDefault` | Input internals |
| `TextUpdateSuppression` | Browser-specific |
| `HelperTextOnFocus` | Helper row behaviour |
| `ShrinkLabel` | Floating label behaviour |
| `InputClass` / `OuterClass` | CSS strings, no egui equivalent |
| `CheckedIcon` / `IndeterminateIcon` | Default Material glyphs suffice for pass 1 |
| `ListClass` | Item list container classes |
| `AdornmentAriaLabel` | Accessibility refinement |
| `SelectRangeAsync` / `SelectOption` methods | Programmatic navigation; write to `SelectState::selected` directly |

### Not ported

- **`Class` (base)** — accepted via `classes(..)` on the trigger.
- **`Style` / `Tag` / `UserAttributes`** — no CSS strings or HTML tag in egui.

---

## Architecture

```
inputs/select/
├── select_action.rs   — SelectAction<T> (extension)
├── select_item.rs     — SelectItem<T>
├── select_state.rs    — SelectState<T> + SelectUi<T>
├── mud_select.rs      — MudSelect + pure helpers + tests
└── mod.rs
```

**Pure layer.** `compute_display_text`, `apply_selection`,
`apply_select_all`, `all_enabled_selected`, `some_enabled_selected`,
`select_all_icon`, `item_checkbox_icon`, `trigger_icon`, `row_height`,
`position_to_origins` — free functions, unit-tested without `egui::Context`.

**State layer.** `SelectState<T> { selected: Vec<T>, popover: PopoverState }` —
the egui equivalent of MudBlazor's `SelectedValues` binding plus the popover
state that backs the dropdown.

**Component layer.** `show` computes the display text, renders the trigger
(`MudTextField`), handles trigger clicks, then delegates to `MudPopover` for
the list. List row rendering + selection state machine live in
`render_item_row` and the pure helpers.

---

## API reference — `MudSelect<T>`

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default select. |
| `tostring_func(f)` | `Fn(&T) -> String` | `None` | Value-to-display-text mapping. |
| `error(bool)` / `error_text(s)` | `bool` / `impl Into<String>` | `false` / `None` | Error visual only. |
| `required(bool)` | `bool` | `false` | Visual only (see README). |
| `disabled(bool)` / `readonly(bool)` | `bool` | `false` | Interaction flags. |
| `clearable(bool)` | `bool` | `false` | Shows a clear button on the trigger. |
| `multi_selection(bool)` | `bool` | `false` | Multi-value selection. |
| `select_all(bool)` | `bool` | `false` | "Select all" tri-state row (multi only). |
| `strict(bool)` | `bool` | `false` | Refuses to display unmapped values. |
| `lock_scroll(bool)` | `bool` | `false` | Stored; no-op in egui. |
| `label(s)` / `placeholder(s)` / `helper_text(s)` | `impl Into<String>` | `None` | Trigger text slots. |
| `variant(Variant)` / `margin(Margin)` / `color(Color)` / `size(Size)` | enum | `Text` / `None` / `Primary` / `Medium` | Trigger appearance. |
| `full_width(bool)` / `disable_underline(bool)` / `icon_size(Size)` | `bool` / `Size` | `false` / `false` / `Medium` | Additional trigger flags. |
| `adornment_color(Color)` / `open_icon(s)` / `close_icon(s)` | `Color` / `impl Into<String>` | `None` / `ARROW_DROP_DOWN` / `ARROW_DROP_UP` | Adornment. |
| `position(Position)` / `anchor_origin(..)` / `transform_origin(..)` | placement | `None` | Popover placement. |
| `right_to_left(bool)` | `bool` | `false` | RTL flag. |
| `max_height(f32)` / `popover_class(s)` | `f32` / `impl Into<String>` | `300.0` / `None` | List box. |
| `dense(bool)` | `bool` | `false` | Compact list rows. |
| `on_open(f)` / `on_close(f)` / `on_clear_button_click(f)` | `FnMut()` | `None` | Callbacks. |
| `on_selected_values_changed(f)` | `FnMut(&[T])` | `None` | Selection-change callback. |
| `classes(iter)` / `flavor(StyleFlavor)` | — | — | Utilities. |
| `show(ui, state, \|s\| ..)` | `(&mut Ui, &mut SelectState<T>, FnOnce(&mut SelectUi<T>) -> R)` | — | Returns `(SelectResponse<T>, R)`. |

### `SelectResponse<T>`

| Field | Type | Description |
|-------|------|-------------|
| `trigger` | `egui::Response` | Trigger's response. |
| `selected` | `Vec<T>` | Selection **after** this frame's click. |
| `action` | `SelectAction<T>` | What happened this frame. |

### `SelectState<T>`

| Field / method | Description |
|----------------|-------------|
| `selected: Vec<T>` | Current selection. |
| `popover: PopoverState` | Open/closed + stable id. |
| `new(id_salt)` / `with_selected(iter)` | Constructor / seed. |
| `is_open()` / `open()` / `close()` / `toggle()` | Popover control. |
| `is_selected(&T)` | Membership check. |

### `SelectItem<T>`

| Method | Description |
|--------|-------------|
| `new(value)` | Value with no text. |
| `text(value, s)` | Value + display text. |
| `with_text(s)` / `icon(s)` / `disabled(b)` / `child(f)` | Builders. |

### `SelectAction<T>`

| Variant | Description |
|---------|-------------|
| `None` | No interaction. |
| `Opened` / `Closed` | Dropdown opened / closed. |
| `Selected { value, multi }` | Item selected. |
| `Deselected { value }` | Item deselected (multi only). |
| `Cleared` | Clear button pressed. |
| `SelectAll { select }` | Select-all toggled. |

Helpers: `changed_selection()`, `toggled_open()`.

---

## Examples

### Single selection

```rust
let mut state = SelectState::<u32>::new("age").with_selected([18]);
MudSelect::new()
    .label("Age")
    .show(ui, &mut state, |s| {
        for n in [18, 25, 30, 40, 55] {
            s.text_item(n, format!("{n}"));
        }
    });
```

### Outlined variant

```rust
MudSelect::new()
    .label("Country")
    .variant(Variant::Outlined)
    .show(ui, &mut state, |s| {
        s.text_item("us", "United States");
        s.text_item("gb", "United Kingdom");
    });
```

### Multi-selection with select-all

```rust
let mut state = SelectState::<&str>::new("tags").with_selected(["rust"]);
MudSelect::new()
    .label("Tags")
    .multi_selection(true)
    .select_all(true)
    .show(ui, &mut state, |s| {
        s.text_item("rust", "Rust");
        s.text_item("go", "Go");
        s.text_item("zig", "Zig");
    });
```

### Clearable

```rust
MudSelect::new()
    .clearable(true)
    .show(ui, &mut state, |s| { /* items */ });
```

### With `tostring_func` (no item text)

```rust
MudSelect::new()
    .tostring_func(|v: &u32| format!("{v}"))
    .show(ui, &mut state, |s| {
        for n in 0..10 { s.value(n); }
    });
```

### With icons

```rust
MudSelect::new().show(ui, &mut state, |s| {
    s.item(SelectItem::text("home", "Home").icon(filled::HOME));
    s.item(SelectItem::text("work", "Work").icon(filled::WORK));
});
```

### Dense

```rust
MudSelect::new()
    .dense(true)
    .show(ui, &mut state, |s| { /* items */ });
```

### Above the anchor

```rust
MudSelect::new()
    .position(Position::Top)
    .show(ui, &mut state, |s| { /* items */ });
```

### Callbacks

```rust
MudSelect::new()
    .on_open(|| log::info!("opened"))
    .on_close(|| log::info!("closed"))
    .on_selected_values_changed(|sel| log::info!("selection: {sel:?}"))
    .show(ui, &mut state, |s| { /* items */ });
```

### Custom value type (non-`Copy`)

```rust
#[derive(Clone, PartialEq)]
struct Fruit { id: u32, name: String }

let mut state = SelectState::<Fruit>::new("fruit");
MudSelect::new()
    .tostring_func(|f| f.name.clone())
    .show(ui, &mut state, |s| {
        s.item(SelectItem::text(
            Fruit { id: 1, name: "Apple".into() }, "Apple"
        ));
        s.item(SelectItem::text(
            Fruit { id: 2, name: "Banana".into() }, "Banana"
        ));
    });
```

### Read the outcome

```rust
let (resp, _) = MudSelect::new().show(ui, &mut state, |s| { /* items */ });
match resp.action {
    SelectAction::Selected { value, multi } => log::info!("selected {value:?} (multi: {multi})"),
    SelectAction::Deselected { value } => log::info!("deselected {value:?}"),
    SelectAction::Cleared => log::info!("cleared"),
    SelectAction::SelectAll { select } => log::info!("select-all: {select}"),
    SelectAction::Opened => log::info!("opened"),
    SelectAction::Closed => log::info!("closed"),
    SelectAction::None => {}
}
```

---

## Trigger display text

`compute_display_text` derives the trigger string from the current selection
using this precedence per value:

1. Matching item's `text` (looked up by `PartialEq` value match).
2. `tostring_func(&value)`.
3. `"?"` (a visible marker; the value was bound but not rendered).

Multiple selections are joined with `MULTI_DELIMITER = ", "`. All three paths
are unit-tested (`compute_display_text_*`).

---

## Select-all behaviour

When `multi_selection(true)` and `select_all(true)`, a select-all row is
rendered above the item list. Its checkbox is tri-state:

| State | Glyph |
|-------|-------|
| All enabled items selected | `CHECK_BOX` |
| Some enabled items selected | `INDETERMINATE_CHECK_BOX` |
| None selected | `CHECK_BOX_OUTLINE_BLANK` |

Clicking the row:
- **Not all selected** → selects every non-disabled item.
- **All selected** → clears the selection entirely.

Disabled items are skipped by `apply_select_all(.., true)`. The matching
predicates (`all_enabled_selected`, `some_enabled_selected`) also ignore
disabled items.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `compute_display_text` / `apply_selection` / `apply_select_all` are pure | Unit-tested without `egui::Context`. |
| `SelectState` is `Clone` (not `Copy`) when `T` is `Clone` | Matches `T`'s bounds. |
| Display text is a **local** `String` | No per-frame allocation on `state`; dropped when the trigger returns. |
| Trigger reuses `MudTextField` | Zero duplication of variant / margin / size logic. |
| List reuses `MudPopover` | Zero duplication of anchor / flip / dismiss. |
| Only the trigger's response is inspected for clicks | The field unions its adornment response, so adornment clicks register. |
| List rows use `allocate_exact_size` + child `Ui`s | No per-row layout pass. |
| `MudIcon` reuse across rows | Free thanks to its transform cache. |
| `MudText` for all labels | Theme and typography propagate automatically. |
| `selected_snapshot` is cloned once before the popover renders | Cheap if `T: Copy`; uses `T: Clone` otherwise. |

Not done, on purpose: virtualising the item list. MudBlazor renders every
visible item; the port does the same. For hundreds of items, wrap the
collector in a filter that pushes fewer items.

---

## Testing

`cargo test` runs 30+ pure-function tests:

| Area | Tests |
|------|-------|
| `row_height` | dense / normal |
| `trigger_icon` | open / closed |
| `item_text_for` | text match, no-text item, missing value |
| `compute_display_text` | empty, item-text priority, `tostring_func` fallback, unknown → `?`, delimiter join |
| `apply_selection` | single replace, single re-click noop, multi add/remove |
| `apply_select_all` | adds enabled, skips disabled, clears |
| `all_enabled_selected` / `some_enabled_selected` | full / partial / empty |
| `select_all_icon` | tri-state mapping |
| `item_checkbox_icon` | selected / not |
| `position_to_origins` | bottom / top / left / right / start under RTL |
| Defaults | every `MudSelect` field matches MudBlazor |
| Builders | every field set |
| `SelectState` | open / close / toggle / is_selected |
| `SelectAction` | `changed_selection` / `toggled_open` |
| `SelectItem` | constructors, builders |
| Collector | preserves order |

Rendering (`show`) needs a live `egui::Context`; the pure layer is the
source of truth.

```bash
cargo test -p mud_egui_components mud_select
```

---

## Differences from MudBlazor

- **Trigger is a `MudTextField` in `readonly(true)` mode.** MudBlazor uses
  the same composition (`MudTextField` as the input). The Rust port matches.
- **`Value` / `SelectedValues` unified into `SelectState::selected: Vec<T>`.**
  MudBlazor has both a scalar `Value` and a `SelectedValues` collection;
  the port uses the collection always and lets single-selection mode leave
  at most one value. This matches the underlying storage in MudBlazor.
- **`strict` is stored but not enforced in pass 1.** MudBlazor hides the
  display text when the value is not in the item list. The port's
  `compute_display_text` currently falls back to `tostring_func` / `?` for
  missing items. Full `strict` behaviour is a one-line guard deferred to
  pass 2.
- **`LockScroll` is a no-op.** egui `Area`s do not participate in scroll
  containers.
- **`required` is visual only.** No form-level validation.
- **`Adornment` restricted to `End`.** MudBlazor's `Adornment.Start` is not
  exposed because the select trigger is a single field; use
  `MudTextField::adornment_end(..)` directly if you need a custom leading
  adornment.
- **`Clearable` is single-select only.** Multi-selection clears via
  `select_all(false)`. MudBlazor's `Clearable` is also single-select only.
- **Form validation parameters deferred.** See the "Deferred" table.
- **`Style` / `Tag` / `UserAttributes` not ported.** No CSS strings or HTML
  tags.