# MudList for `mud_egui`

A scrollable, selectable vertical list for [egui](https://github.com/emilk/egui),
ported from MudBlazor's [`MudList<T>`](https://mudblazor.com/components/list),
`MudListItem<T>`, and `MudListSubheader<T>` (MudBlazor 8.13.0).

```rust
let mut state = ListState::<&str>::new();

MudList::new()
    .selection_mode(SelectionMode::Single)
    .show(ui, &mut state, |list| {
        list.subheader("Inbox");
        list.item(MudListItem::new("starred").with_text("Starred").icon(filled::STAR));
        list.text_item("sent", "Sent");
        list.text_item("drafts", "Drafts");
    });
```

Every visible string is rendered through **`MudText`**, and every icon through
**`MudIcon`**, so the list inherits the surrounding typography and theme
automatically.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Selection and expansion](#selection-and-expansion)
6. [Performance and memory choices](#performance-and-memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor parameter surface was checked against the published API docs and
the component sources before this port.

### `MudList<T>`

| MudBlazor | Rust method | Default |
|---|---|---|
| `CheckBoxColor` | `checkbox_color(Color)` | `Color::Default` |
| `CheckedIcon` | `checked_icon(s)` | `CHECK_BOX` |
| `ChildContent` | closure to `show` | required |
| `Color` | `color(Color)` | `Primary` |
| `Comparer` | *(Rust `T: PartialEq`)* | — |
| `Dense` | `dense(bool)` | `false` |
| `Disabled` | `disabled(bool)` | `false` |
| `Gutters` | `gutters(bool)` | `true` |
| `Padding` | `padding(bool)` | `false` |
| `ReadOnly` | `readonly(bool)` | `false` |
| `SelectedValue` | `ListState::selected_value` | `None` |
| `SelectedValues` | `ListState::selected_values` | `[]` |
| `SelectionMode` | `selection_mode(SelectionMode)` | `Single` |
| `UncheckedIcon` | `unchecked_icon(s)` | `CHECK_BOX_OUTLINE_BLANK` |
| `Class` (base) | `classes(iter)` | none |
| `Style` (base) | *(use `classes`)* | — |
| `Tag` (base) | `tag(t)` / `tag_ref::<U>()` | `None` |
| `UserAttributes` (base) | **not ported** (no HTML tag) | — |

### `MudListItem<T>`

| MudBlazor | Rust method | Default |
|---|---|---|
| `AvatarContent` | `avatar(f)` | `None` |
| `ChildContent` | `child(f)` | `None` |
| `Dense` | `dense(bool)` | inherit |
| `Disabled` | `disabled(bool)` | `false` |
| `ExpandIconColor` | `expand_icon_color(Color)` | `Default` |
| `ExpandLessIcon` | `expand_less_icon(s)` | `EXPAND_LESS` |
| `ExpandMoreIcon` | `expand_more_icon(s)` | `EXPAND_MORE` |
| `Expanded` | `expanded(bool)` | `None` (uncontrolled) |
| `Gutters` | `gutters(bool)` | inherit |
| `Href` | `href(s)` | `None` |
| `Icon` | `icon(s)` | `None` |
| `IconColor` | `icon_color(Color)` | `Inherit` |
| `IconSize` | `icon_size(Size)` | `Medium` |
| `Inset` | `inset(bool)` | `false` |
| `NestedList` | `nested(f)` | `None` |
| `OnClick` | `on_click(f)` | `None` |
| `OnClickPreventDefault` | `on_click_prevent_default(bool)` | `false` |
| `Ripple` | `ripple(bool)` | `true` |
| `SecondaryText` | `secondary_text(s)` | `None` |
| `Target` | `target(s)` | `None` |
| `Text` | `text(s)` / `with_text(s)` | `None` |
| `Value` | `MudListItem::new(value)` | required |

### `MudListSubheader<T>`

| MudBlazor | Rust method | Default |
|---|---|---|
| `ChildContent` | `MudListSubheader::new(text)` | required |
| `Gutters` | `gutters(bool)` | inherit |
| `Inset` | `inset(bool)` | `false` |

### Deliberately not ported

- **`Comparer`** — MudBlazor allows a custom `IEqualityComparer<T>`. In Rust,
  implement `PartialEq` on your value type; the list uses `T: PartialEq`
  everywhere.
- **`Href` / `Target` / `ForceLoad`** — stored as fields for parity, but egui
  has no browser router. Clicking fires `on_click(&value)`; the caller
  navigates. `Target` and `ForceLoad` are unused.
- **`UserAttributes`** — there is no HTML tag to splat attributes onto.
- **`Style`** — egui has no inline CSS strings; use `classes(..)`.

---

## Architecture

```
mud_list.rs
├── MudList<'a, T>         — list configuration + render entry point
│   └── show(ui, state, |list| ..) -> (Response, R)
├── MudListItem<'a, T>     — per-item config (text, icon, avatar, nested, …)
├── MudListSubheader       — non-interactive header row
├── ListUi<'a, T>          — collector passed to the show closure
├── ListState<T>           — caller-owned selection + expansion
├── apply_selection(..)    — pure selection state machine
├── geometry helpers       — item_height, subheader_height, indent_px,
│                            icon_px, avatar_px
└── render helpers         — draw_item, draw_subheader, paint_checkbox,
                             start_ripple, paint_ripple

list_entry.rs              — private ListEntry enum (own file per project rule)
```

**Pure layer.** `apply_selection` and all geometry helpers are free functions
covered by unit tests, exactly as in `MudDataGrid` and `MudGrid`.

**State layer.** `ListState<T>` is caller-owned. It is the egui equivalent of
MudBlazor's `SelectedValue`, `SelectedValues`, and `Expanded` bindings.

**Component layer.** `show` resolves the theme and wrapper frame, calls the
pure layer, allocates one row per item, and paints. It never measures text to
place rows, which keeps layout deterministic.

---

## API reference

### `MudList<'a, T>`

| Method | Default | Mirrors | Description |
|---|---|---|---|
| `new()` | — | — | MudBlazor defaults. |
| `selection_mode(SelectionMode)` | `Single` | `SelectionMode` | `Single` / `Toggle` / `Multi`. |
| `color(Color)` | `Primary` | `Color` | Accent for selected items. |
| `checkbox_color(Color)` | `Default` | `CheckBoxColor` | Checkbox accent in `Multi`. |
| `checked_icon(s)` | `CHECK_BOX` | `CheckedIcon` | Checked glyph. |
| `unchecked_icon(s)` | `CHECK_BOX_OUTLINE_BLANK` | `UncheckedIcon` | Unchecked glyph. |
| `dense(bool)` | `false` | `Dense` | Compact rows. |
| `disabled(bool)` | `false` | `Disabled` | Disables every item. |
| `readonly(bool)` | `false` | `ReadOnly` | Prevents selection; keeps expansion. |
| `gutters(bool)` | `true` | `Gutters` | Side padding on items. |
| `padding(bool)` | `false` | `Padding` | Vertical padding around the list. |
| `classes(iter)` | none | `Class` | Utility classes on the wrapper. |
| `flavor(StyleFlavor)` | default | — | Bootstrap or Tailwind. |
| `tag(t)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary user data. |
| `show(ui, state, \|list\| ..)` | — | — | Draws and returns `(Response, R)`. |

### `MudListItem<'a, T>`

| Method | Mirrors | Description |
|---|---|---|
| `new(value)` | `Value` | Required value. |
| `text(value, s)` | `Text` | Convenience constructor. |
| `with_text(s)` | `Text` | Primary caption. |
| `secondary_text(s)` | `SecondaryText` | Small caption under the title. |
| `icon(s)` | `Icon` | Leading Material SVG path. |
| `icon_color(Color)` | `IconColor` | Default `Inherit`. |
| `icon_size(Size)` | `IconSize` | Default `Medium`. |
| `avatar(f)` | `AvatarContent` | Overrides `icon`. |
| `child(f)` | `ChildContent` | Overrides `text` + `secondary_text`. |
| `nested(f)` | `NestedList` | Expandable child items. |
| `expanded(bool)` | `Expanded` | Initial expansion state. |
| `dense(bool)` | `Dense` | Overrides the list. |
| `disabled(bool)` | `Disabled` | Disables this item. |
| `inset(bool)` | `Inset` | Extra left indent. |
| `gutters(bool)` | `Gutters` | Overrides the list. |
| `href(s)` | `Href` | Stored for parity; navigate in `on_click`. |
| `target(s)` | `Target` | Stored; unused. |
| `force_load(bool)` | `ForceLoad` | Stored; unused. |
| `ripple(bool)` | `Ripple` | Default `true`. |
| `on_click(f)` | `OnClick` | `FnMut(&T)`. |
| `on_click_prevent_default(bool)` | `OnClickPreventDefault` | Skips selection/expansion. |
| `id(s)` | — | Stable key for expansion and ripple. |
| `expand_less_icon(s)` | `ExpandLessIcon` | Default `EXPAND_LESS`. |
| `expand_more_icon(s)` | `ExpandMoreIcon` | Default `EXPAND_MORE`. |
| `expand_icon_color(Color)` | `ExpandIconColor` | Default `Default`. |

### `ListState<T>`

| Field / method | Description |
|---|---|
| `selected_value: Option<T>` | Single / toggle selection. |
| `selected_values: Vec<T>` | Multi selection. |
| `expanded: Vec<(String, bool)>` | Expansion state. |
| `is_selected(&T, SelectionMode)` | Mode-aware membership test. |
| `is_expanded(&str)` | `Option<bool>`; `None` if uninitialised. |
| `ensure_expanded(&str, bool)` | Seeds the initial value once. |
| `toggle_expanded(&str)` | Flips the state. |
| `clear_expanded()` | Drops all expansion entries. |

### Geometry constants

| Name | Value |
|---|---|
| `ITEM_HEIGHT` | `48.0` |
| `DENSE_ITEM_HEIGHT` | `32.0` |
| `SUBHEADER_HEIGHT` | `36.0` |
| `DENSE_SUBHEADER_HEIGHT` | `28.0` |
| `GUTTER_PX` | `12.0` |
| `INSET_PX` | `16.0` |
| `INDENT_PER_DEPTH` | `16.0` |

---

## Examples

### Single selection (default)

```rust
let mut state = ListState::<&str>::new();

MudList::new().show(ui, &mut state, |list| {
    list.text_item("inbox", "Inbox");
    list.text_item("sent", "Sent");
    list.text_item("drafts", "Drafts");
});

if let Some(folder) = state.selected_value {
    // …
}
```

### Multi selection with checkboxes

```rust
MudList::new()
    .selection_mode(SelectionMode::Multi)
    .checkbox_color(Color::Success)
    .show(ui, &mut state, |list| {
        for tag in ["urgent", "work", "personal"] {
            list.text_item(tag, tag);
        }
    });

// state.selected_values holds every checked tag.
```

### Toggle selection (deselectable)

```rust
MudList::new()
    .selection_mode(SelectionMode::Toggle)
    .show(ui, &mut state, |list| {
        list.text_item("active", "Active only");
    });
```

Clicking the selected item clears `selected_value`.

### Icons and secondary text

```rust
MudList::new().show(ui, &mut state, |list| {
    list.item(
        MudListItem::new("starred")
            .with_text("Starred")
            .secondary_text("12 messages")
            .icon(filled::STAR)
            .icon_color(Color::Warning),
    );
    list.item(
        MudListItem::new("archive")
            .with_text("Archive")
            .icon(filled::ARCHIVE),
    );
});
```

### Avatars

```rust
list.item(
    MudListItem::new("ada")
        .with_text("Ada Lovelace")
        .avatar(|ui| {
            ui.add(MudAvatar::new().text("AL").size(Size::Small));
        }),
);
```

`avatar` overrides `icon`, matching MudBlazor.

### Custom row content

```rust
list.item(
    MudListItem::new("custom").child(|ui| {
        ui.add(MudText::new("Custom").typo(Typo::Body1));
        ui.add(MudText::new("content").typo(Typo::Caption));
    }),
);
```

`child` overrides `text` and `secondary_text`.

### Nested expandable items

```rust
list.item(
    MudListItem::new("projects")
        .with_text("Projects")
        .icon(filled::FOLDER)
        .nested(|nested| {
            nested.text_item("alpha", "Alpha");
            nested.text_item("beta", "Beta");
        }),
);
```

Expansion is remembered in `state.expanded`; seed it with `.expanded(true)`
to start open.

### Subheaders

```rust
list.subheader("Work");
list.text_item("inbox", "Inbox");
list.text_item("sent", "Sent");

list.subheader("Personal");
list.text_item("home", "Home");
```

### Dense list

```rust
MudList::new().dense(true).show(ui, &mut state, |list| { /* … */ });
```

Or per item with `.dense(true)`.

### Disabled and read-only

```rust
MudList::new()
    .readonly(true)   // expansion still works
    .show(ui, &mut state, |list| { /* … */ });

MudList::new()
    .disabled(true)   // nothing responds
    .show(ui, &mut state, |list| { /* … */ });
```

### Gutters and inset

```rust
MudList::new()
    .gutters(false)
    .show(ui, &mut state, |list| {
        list.item(MudListItem::new(1).with_text("Flush"));
        list.item(MudListItem::new(2).with_text("Inset").inset(true));
    });
```

### Click callback and tag

```rust
let list = MudList::new()
    .tag("sidebar")
    .show(ui, &mut state, |list| {
        list.item(
            MudListItem::new("logout")
                .with_text("Log out")
                .on_click(|value| println!("clicked {value:?}")),
        );
    });

assert_eq!(list.0.widget_info, list.0.widget_info); // response is available
assert_eq!(/* tag */ "sidebar", "sidebar");
```

---

## Selection and expansion

### Selection

| Mode | Click unselected | Click selected |
|---|---|---|
| `Single` | selects it | no change |
| `Toggle` | selects it | clears the selection |
| `Multi` | adds it to `selected_values` | removes it |

`Single` and `Toggle` write `state.selected_value`; `Multi` writes
`state.selected_values`. The two fields are kept mutually exclusive by
`apply_selection`. This mirrors MudBlazor's note that `SelectedValue` and
`SelectedValues` are read/written only in their respective modes.

`readonly(true)` disables selection entirely while keeping expansion.
`disabled(true)` (list) or `.disabled(true)` (item) disables everything,
including expansion and ripple.

### Expansion

Expansion state lives in `ListState::expanded`, a `Vec<(String, bool)>`
keyed by the item's `id` (or its positional path when `id` is not set).

- `.expanded(true)` **seeds** the initial state once. After that, clicks
  toggle it. This matches MudBlazor's remark that setting `Expanded` without
  two-way binding initialises the state.
- To control expansion manually, read `state.is_expanded(key)` before
  rendering and set `state.expanded` yourself; clicks will still toggle it
  unless you intercept the item's `on_click`.

### `OnClickPreventDefault`

When `on_click_prevent_default(true)` is set, clicking the row only fires
`on_click`. Neither selection nor expansion changes. This mirrors MudBlazor's
`OnClickPreventDefault`.

---

## Performance and memory choices

| Choice | Effect |
|--------|--------|
| `MudListItem` is built once per frame from borrowed closures | No retained render tree. |
| Rows are allocated with `allocate_exact_size` + absolute child `Ui`s | No per-row layout pass; row height is known before painting. |
| `apply_selection` is a pure state machine | Testable without an `egui::Context`. |
| Expansion state is `Vec<(String, bool)>` | No `HashMap` hashing; items are few. |
| Ripple is memory-backed and keyed by `ui.scope_id().with(path)` | No per-item state on the list struct. |
| `start_ripple` reads `ui.input(..)` **before** `memory_mut(..)` | Avoids the re-entrant `RwLock` deadlock fixed in `MudToggleGroup`. |
| Strings are moved into the render path, not cloned per frame | The item is single-use per frame. |
| All text goes through `MudText`; all icons through `MudIcon` | Theme/font updates propagate automatically; no duplicated style logic. |

Not done, on purpose: a row-height cache and a text-measurement cache. Both
would need invalidation rules that belong to a layout engine; the list stays
deterministic instead.

---

## Testing

`cargo test` runs the module's tests:

| Area | Tests |
|------|-------|
| `apply_selection` | single select/reclick, single replace, toggle deselect, toggle replace, multi membership and order |
| `is_selected` | mode-aware membership |
| Expansion | `ensure_expanded` seeds once, `toggle_expanded` flips/creates, `clear_expanded` empties |
| Geometry | item/subheader heights per density, indent scaling, icon px mapping, avatar px per density |
| Defaults | `MudList` matches MudBlazor, `MudListItem` defaults, `MudListSubheader` defaults |
| Builders | every `MudListItem` builder sets its field |
| Collector | order and subheader insertion |
| Tag | round-trips by type |

Rendering itself (`show`) needs a live `egui::Context` and is not unit-tested;
the pure layer is the source of truth for selection, expansion, and geometry.

```bash
cargo test -p mud_egui_components mud_list
```

---

## Differences from MudBlazor

- **No `Comparer` parameter.** Implement `PartialEq` on your value type; the
  list uses `T: PartialEq` for membership tests.
- **`SelectedValue` and `SelectedValues` are unified in `ListState<T>`.**
  The caller owns both fields and passes `&mut ListState` to `show`; the list
  writes the field that matches `selection_mode`.
- **`Href` / `Target` / `ForceLoad` are stored, not acted on.** egui has no
  browser router; clicking fires `on_click(&value)` and the caller navigates.
  `Target` and `ForceLoad` have no meaningful egui equivalent and are unused.
- **`Style` is `classes(..)`.** egui has no inline CSS strings; utility
  classes are resolved through the Bootstrap or Tailwind flavour.
- **`UserAttributes` not ported.** There is no HTML tag.
- **`Expanded` is seeded, not hard-controlled.** Setting `.expanded(true)`
  initialises the state on first render; subsequent clicks toggle it, matching
  MudBlazor's non-two-way behaviour. To fully control it, update
  `state.expanded` yourself each frame.
- **Selection modes are `SelectionMode::{Single, Toggle, Multi}`** — the
  existing `mud_egui_core::enums::SelectionMode`, which already matches
  MudBlazor's `SingleSelection` / `ToggleSelection` / `MultiSelection`.
- **No new public enums.** The only enum in the module is the private
  `ListEntry` collector, kept in its own file per project convention.