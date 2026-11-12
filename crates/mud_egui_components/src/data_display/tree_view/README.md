# MudTreeView for `mud_egui`

A recursive, selectable, expandable tree for [egui](https://github.com/emilk/egui),
ported from MudBlazor's [`MudTreeView<T>`](https://mudblazor.com/components/treeview),
`MudTreeViewItem<T>`, and `MudTreeViewItemToggleButton` (MudBlazor 8.13.0).

```rust
let mut state = TreeState::<&str>::new();

MudTreeView::new()
    .selection_mode(SelectionMode::Single)
    .show(ui, &mut state, |tree| {
        tree.item(
            TreeItem::text("documents", "Documents")
                .icon(filled::FOLDER)
                .child(TreeItem::text("readme", "README.md"))
                .child(TreeItem::text("license", "LICENSE")),
        );
        tree.item(TreeItem::text("downloads", "Downloads").icon(filled::DOWNLOAD));
    });
```

Every label renders through **`MudText`**, every glyph through **`MudIcon`**.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Auto behaviour](#auto-behaviour)
6. [Performance and memory choices](#performance--memory-choices)
7. [Testing](#testing)
8. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor parameter surface was verified against DNDocs for 8.13.0 before
this port[reference:4][reference:5][reference:6][reference:7]. The tree is
**row-template driven** in the same sense as `MudTable<T>`: you supply a
closure that builds the hierarchy with `TreeItem` values; the component
handles selection, expansion, auto-behaviour and rendering.

### Ported parameters

| MudBlazor `MudTreeView<T>` | Rust | Default |
|---|---|---|
| `AutoExpand` | `auto_expand(bool)` | `false` |
| `AutoSelectParent` | `auto_select_parent(bool)` | `true` |
| `CheckBoxColor` | `check_box_color(Color)` | `Default` |
| `CheckedIcon` | `checked_icon(s)` | `CHECK_BOX` |
| `ChildContent` | the closure to `show` | required |
| `Color` | `color(Color)` | `Primary` |
| `Comparer` | *(Rust `PartialEq`)* | — |
| `Dense` | `dense(bool)` | `false` |
| `Disabled` | `disabled(bool)` | `false` |
| `ExpandOnClick` | `expand_on_click(bool)` | `false` |
| `ExpandOnDoubleClick` | `expand_on_double_click(bool)` | `false` |
| `FilterFunc` | `filter(Fn(&T) -> bool)` | `None` |
| `Height` | `height(f32)` | `None` |
| `Hover` | `hover(bool)` | `false` |
| `IndeterminateIcon` | `indeterminate_icon(s)` | `INDETERMINATE_CHECK_BOX` |
| `MaxHeight` | `max_height(f32)` | `None` |
| `ReadOnly` | `read_only(bool)` | `false` |
| `Ripple` | `ripple(bool)` | `true` |
| `SelectedValue` | `TreeState::selected_value` | `None` |
| `SelectedValues` | `TreeState::selected_values` | `[]` |
| `SelectionMode` | `selection_mode(SelectionMode)` | `Single` |
| `TriState` | `tri_state(bool)` | `true` |
| `UncheckedIcon` | `unchecked_icon(s)` | `CHECK_BOX_OUTLINE_BLANK` |
| `Width` | `width(f32)` | `None` |

### Ported `TreeItem` parameters

| MudBlazor `MudTreeViewItem<T>` | Rust | Default |
|---|---|---|
| `BodyContent` | `body(f)` | `None` |
| `CanExpand` | `can_expand(bool)` | `true` |
| `ChildContent` | `child(item)` / `children(iter)` | none |
| `Disabled` | `disabled(bool)` | `false` |
| `EndIcon` | `end_icon(s)` | `None` |
| `EndIconColor` | `end_icon_color(Color)` | `Inherit` |
| `EndText` | `end_text(s)` | `None` |
| `EndTextTypo` | `end_text_typo(Typo)` | `Body1` |
| `ExpandButtonIcon` | `expand_button_icon(s)` | `CHEVRON_RIGHT` |
| `ExpandButtonIconColor` | `expand_button_icon_color(Color)` | `Default` |
| `Expanded` | `expanded(bool)` | `None` |
| `Icon` | `icon(s)` | `None` |
| `IconColor` | `icon_color(Color)` | `Inherit` |
| `IconExpanded` | `icon_expanded(s)` | `None` |
| `OnClick` / `OnDoubleClick` | `on_click(f)` / `on_double_click(f)` | `None` |
| `ReadOnly` | `read_only(bool)` | `false` |
| `Text` | `with_text(s)` / `text(value, s)` | `None` |
| `Value` | `new(value)` | required |
| `Visible` | `visible(bool)` | `true` |

### Deliberately not ported

- **`Items` + `ItemTemplate`** — MudBlazor's data-model mode. The Rust port
  uses the closure-driven builder (`TreeItem::child(..)`), which is the
  idiomatic egui equivalent.
- **`ServerData`** — async lazy loading. egui has no per-frame async
  runtime; the caller can implement lazy loading by swapping tree contents
  in a click handler.
- **`Content`** (full-row replacement) — `body(..)` replaces the icon + text
  + end area, which covers the common case.
- **`FilterFunc` is synchronous** — `filter(Fn(&T) -> bool)`. Async filters
  are not supported.
- **`Style` / `UserAttributes`** — no CSS strings or HTML tags in egui.

---

## Architecture

```
mud_tree_view.rs
├── MudTreeView<'a, T>          — configuration + render entry point
│   └── show(ui, state, |tree| ..) -> (Response, R)
├── DrawCfg                     — immutable per-frame config
├── render_node(..)             — recursive row painter
├── auto_select_pass(..)        — bottom-up parent/child invariant
├── collect_ancestors_of_selected(..)
├── compute_tri_state(..)
├── is_visible(..)
├── handle_row_click / handle_checkbox_click / cascade_set
└── tests                       — 24 pure-function tests

tree_state.rs                   — TreeState<T> (caller-owned)
tree_item.rs                    — TreeItem<'a, T> (node data model)
tree_ui.rs                      — TreeUi<'a, T> (collector)
tri_state_kind.rs               — pub(crate) TriStateKind
```

**Pure layer.** `compute_tri_state`, `auto_select_pass`,
`collect_ancestors_of_selected`, `is_visible`, `cascade_set` are free
functions, unit-tested without an `egui::Context`.

**State layer.** `TreeState<T>` is caller-owned — the egui equivalent of
`SelectedValue`, `SelectedValues`, and per-item `Expanded`.

**Component layer.** `show` runs the two pre-passes, then recurses through
`render_node`.

---

## API reference

### `MudTreeView<'a, T>`

| Method | Default | Mirrors | Description |
|---|---|---|---|
| `new()` | — | — | MudBlazor defaults. |
| `filter(Fn(&T) -> bool)` | `None` | `FilterFunc` | Node visibility predicate. |
| `selection_mode(SelectionMode)` | `Single` | `SelectionMode` | `Single` / `Toggle` / `Multi`. |
| `color(Color)` | `Primary` | `Color` | Selected-row accent. |
| `check_box_color(Color)` | `Default` | `CheckBoxColor` | Checkbox tint. |
| `checked_icon(s)` | `CHECK_BOX` | `CheckedIcon` | Checked glyph. |
| `unchecked_icon(s)` | `CHECK_BOX_OUTLINE_BLANK` | `UncheckedIcon` | Unchecked glyph. |
| `indeterminate_icon(s)` | `INDETERMINATE_CHECK_BOX` | `IndeterminateIcon` | Partial-check glyph. |
| `tri_state(bool)` | `true` | `TriState` | Partial-state glyphs. |
| `auto_expand(bool)` | `false` | `AutoExpand` | Expand ancestors of selection. |
| `auto_select_parent(bool)` | `true` | `AutoSelectParent` | Derived parent state. |
| `read_only(bool)` | `false` | `ReadOnly` | Freeze selection. |
| `expand_on_click(bool)` | `false` | `ExpandOnClick` | Row click expands. |
| `expand_on_double_click(bool)` | `false` | `ExpandOnDoubleClick` | Double-click expands. |
| `dense(bool)` | `false` | `Dense` | Compact rows. |
| `hover(bool)` | `false` | `Hover` | Row hover fill. |
| `ripple(bool)` | `true` | `Ripple` | Click ripple. |
| `disabled(bool)` | `false` | `Disabled` | Disable everything. |
| `height(f32)` | `None` | `Height` | Fixed height + scroll. |
| `max_height(f32)` | `None` | `MaxHeight` | Maximum height + scroll. |
| `width(f32)` | `None` | `Width` | Fixed width. |
| `expand_all(bool)` | `false` | `ExpandAllAsync` | Force-expand all nodes. |
| `collapse_all(bool)` | `false` | `CollapseAllAsync` | Force-collapse all nodes. |
| `aria_label(s)` | `None` | — | Accessible label. |
| `classes(iter)` | none | `Class` | Utility classes. |
| `flavor(StyleFlavor)` | default | — | Bootstrap / Tailwind. |
| `tag(t)` / `tag_ref::<U>()` | `None` | `Tag` | Arbitrary data. |
| `show(ui, state, \|tree\| ..)` | — | — | Returns `(Response, R)`. |

### `TreeItem<'a, T>`

| Method | Mirrors | Description |
|---|---|---|
| `new(value)` | `Value` | Required value. |
| `text(value, s)` | `Value` + `Text` | Convenience constructor. |
| `with_text(s)` | `Text` | Caption. |
| `child(item)` | `ChildContent` | Adds one child. |
| `children(iter)` | `ChildContent` | Adds several children. |
| `icon(s)` | `Icon` | Leading icon. |
| `icon_color(Color)` | `IconColor` | Default `Inherit`. |
| `icon_expanded(s)` | `IconExpanded` | Swapped when expanded. |
| `end_icon(s)` | `EndIcon` | Trailing icon. |
| `end_icon_color(Color)` | `EndIconColor` | |
| `end_text(s)` | `EndText` | Trailing text. |
| `end_text_typo(Typo)` | `EndTextTypo` | Default `Body1`. |
| `expand_button_icon(s)` | `ExpandButtonIcon` | Default `CHEVRON_RIGHT`. |
| `expand_button_icon_color(Color)` | `ExpandButtonIconColor` | Default `Default`. |
| `expanded(bool)` | `Expanded` | Seed only. |
| `can_expand(bool)` | `CanExpand` | Default `true`. |
| `disabled(bool)` | `Disabled` | |
| `read_only(bool)` | `ReadOnly` | |
| `visible(bool)` | `Visible` | |
| `body(f)` | `BodyContent` | Custom row body. |
| `on_click(f)` | `OnClick` | `FnMut(&T)`. |
| `on_double_click(f)` | `OnDoubleClick` | `FnMut(&T)`. |

### `TreeState<T>`

| Field / method | Description |
|---|---|
| `selected_value: Option<T>` | Single / toggle selection. |
| `selected_values: Vec<T>` | Multi selection. |
| `expanded: Vec<(T, bool)>` | Expansion state (seeded from `TreeItem::expanded`). |
| `last_node_rects: Vec<Rect>` | Screen-space row rectangles from the last frame. |
| `is_selected(&T, SelectionMode)` | Mode-aware membership test. |
| `is_expanded(&T)` / `ensure_expanded` / `toggle_expanded` / `clear_expanded` | Expansion helpers. |

### Geometry

| Constant | Value |
|---|---|
| `ROW_HEIGHT` | `40.0` |
| `DENSE_ROW_HEIGHT` | `32.0` |
| `INDENT_PER_DEPTH` | `24.0` |

---

## Examples

### File browser

```rust
TreeItem::text("documents", "Documents")
    .icon(filled::FOLDER)
    .child(TreeItem::text("readme", "README.md").icon(filled::DESCRIPTION))
    .child(TreeItem::text("src", "src").icon(filled::FOLDER)
        .child(TreeItem::text("main", "main.rs").icon(filled::CODE)))
```

### Multi-select with tri-state

```rust
MudTreeView::new()
    .selection_mode(SelectionMode::Multi)
    .tri_state(true)
    .auto_select_parent(true)
    .show(ui, &mut state, |tree| {
        tree.item(
            TreeItem::text("fruits", "Fruits")
                .child(TreeItem::text("apple", "Apple"))
                .child(TreeItem::text("banana", "Banana")),
        );
    });
```

Selecting both children auto-checks the parent. Unchecking one makes the
parent indeterminate.

### Filter

```rust
MudTreeView::new()
    .filter(|v: &&str| v.contains(query))
    .show(ui, &mut state, |tree| { /* … */ });
```

A branch stays visible when it or any descendant matches.

### Expand-all / collapse-all

```rust
MudTreeView::new()
    .expand_all(true)  // ExpandAllAsync equivalent
    .show(ui, &mut state, |tree| { /* … */ });
```

### Read-only, hover, dense

```rust
MudTreeView::new()
    .read_only(true)
    .hover(true)
    .dense(true)
    .show(ui, &mut state, |tree| { /* … */ });
```

### Custom row body

```rust
TreeItem::text("user", "User")
    .body(|ui, is_selected| {
        ui.add(MudText::new("User").typo(Typo::Body1));
        ui.add(MudText::new("admin").typo(Typo::Caption).color(Color::TextSecondary));
    })
```

### Scrollable tree

```rust
MudTreeView::new()
    .max_height(300.0)
    .show(ui, &mut state, |tree| { /* … */ });
```

---

## Auto behaviour

### `auto_expand`

Before rendering, `collect_ancestors_of_selected` walks the tree and
collects the value of every node that has a selected descendant. Those
values are force-expanded for the frame, so the selected item is always
visible.

### `auto_select_parent`

Before rendering, `auto_select_pass` walks the tree bottom-up and enforces:

> A parent is in `selected_values` **iff** every direct child is checked.

A click on a leaf therefore reaches the parent with a one-frame delay —
the same latency you would get from a Blazor event handler.

### `tri_state`

In Multi mode, a parent's checkbox is painted based on its **descendants**
(not its own `selected` flag):

| Descendant state | Parent checkbox |
|---|---|
| All checked | Checked |
| Some checked, some unchecked | Indeterminate |
| None checked | Unchecked |

Leaves use their own `selected_values` membership.

### Click targets

| Target | Single / Toggle | Multi |
|---|---|---|
| Expand button | Toggle expand | Toggle expand |
| Checkbox | Toggle node | Toggle whole subtree (cascade) |
| Row body | Select node | Toggle node |
| Double-click | Expand (if `expand_on_double_click`) | Same |

The row body also expands when `expand_on_click(true)`.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Tree is built once per frame from `TreeItem` values | No retained render tree. |
| `TreeState` is caller-owned | Selection / expansion survive frames. |
| Recursion is O(n) per visible node | Typical trees (<1000 nodes) render in microseconds. |
| `expanded` is a `Vec<(T, bool)>` with linear search | `T: PartialEq` suffices; no `Hash` bound. |
| Interaction IDs are path-based | `T` needs no `Hash`. |
| Pre-passes are linear | `auto_select_pass` and `collect_ancestors_of_selected` are one walk each. |
| Ripple reads `ui.input(..)` **before** `memory_mut(..)` | Avoids the re-entrant `RwLock` deadlock documented in `MudToggleGroup`. |
| All text through `MudText`, icons through `MudIcon` | Theme/font updates propagate automatically. |

Not done, on purpose: virtualisation and row-height measurement. Both need a
layout engine; the tree stays deterministic.

---

## Testing

`cargo test` runs 24 pure-function tests:

| Area | Tests |
|---|---|
| `TreeState` | empty defaults, `ensure_expanded` seeds once, `toggle_expanded` flips/creates, `is_selected` mode-aware |
| `TreeItem` | defaults, `child` / `children` order |
| Visibility | `visible(false)` hides subtree, filter matches self or descendant, filter matches leaf |
| Tri-state | leaf uses own state, parent mixed → indeterminate, parent all → checked, parent none → unchecked |
| Auto-select-parent | adds parent when all children checked, removes parent when some unchecked, disabled keeps user choice |
| Auto-expand | ancestors collected, none when nothing selected |
| Cascade | check every descendant, uncheck every descendant |
| Builders | MudBlazor defaults, every field set, tag round-trip |
| Collector | root order preserved |

Rendering (`show`) needs a live `egui::Context` and is not unit-tested;
the pure layer is the source of truth.

```bash
cargo test -p mud_egui_components mud_tree_view
```

---

## Differences from MudBlazor

- **No `Items` / `ItemTemplate`.** The closure-driven builder is the
  idiomatic egui equivalent and avoids a separate `TreeItemData<T>` type.
- **No `ServerData`.** Lazy loading would need an async runtime per frame;
  the caller can swap tree contents in a click handler instead.
- **No `Content`.** `body(..)` replaces the icon + text + end area;
  `Content` would replace the whole row, which conflicts with the built-in
  expand button and checkbox.
- **`FilterFunc` is synchronous.** `filter(Fn(&T) -> bool)`.
- **`Style` → `classes(..)`.** egui has no inline CSS strings.
- **`UserAttributes` not ported.** There is no HTML tag.
- **`Expanded` is seeded once, not two-way bound.** Setting
  `TreeItem::expanded(true)` initialises the state; subsequent clicks toggle
  it. To control expansion manually, update `TreeState::expanded` yourself.
- **`auto_select_parent` runs at render time**, not in the click handler, so
  a leaf click reaches the parent one frame later. Visually imperceptible.
- **Single public enum only.** `SelectionMode` is reused; the only new enum
  is the private `TriStateKind`, which lives in its own file.