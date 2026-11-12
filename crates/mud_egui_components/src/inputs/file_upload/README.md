# MudFileUpload for `mud_egui`

A file picker and drag-and-drop surface for
[egui](https://github.com/emilk/egui), ported from MudBlazor’s
[`MudFileUpload<T>`](https://mudblazor.com/components/fileupload) (MudBlazor 8.13.0).

```rust
let mut files: Vec<PathBuf> = Vec::new();

ui.add(
    MudFileUpload::new(&mut files)
        .accept(".png, .jpg")
        .max_file_size(2 * 1024 * 1024)
        .label("Upload images"),
);

if !files.is_empty() {
    // files is the caller-owned list.
}
```

The component itself paints no text through `MudText` (it draws its own button
label and chips), but any `activator_content` you supply should use
**`MudText`** and **`MudIcon`** like every other component.

---

## Contents

1. [Design principle: API parity, not API invention](#design-principle-api-parity-not-api-invention)
2. [Architecture](#architecture)
3. [API reference](#api-reference)
4. [Examples](#examples)
5. [Mode semantics](#mode-semantics)
6. [Validation](#validation)
7. [Performance and memory choices](#performance-and-memory-choices)
8. [Testing](#testing)
9. [Differences from MudBlazor](#differences-from-mudblazor)

---

## Design principle: API parity, not API invention

The MudBlazor 8.13.0 parameter surface was verified against DNDocs before
implementation. `MudFileUpload<T>` has **ten parameters**, and all ten are
ported.

### Ported parameters

| MudBlazor | Type | Default | Rust mapping |
|---|---|---|---|
| `Accept` | `string?` | `null` | `accept(&'a str)` |
| `ActivatorContent` | `RenderFragment?` | `null` | `activator_content(FnOnce(&mut Ui) -> Response)` |
| `AppendMultipleFiles` | `bool` | `false` | `append_multiple_files(bool)` |
| `Disabled` | `bool` | `false` | `disabled(bool)` |
| `Files` | `T?` | — | caller-owned `&mut Vec<PathBuf>` |
| `FilesChanged` | `EventCallback<T?>` | — | `FileUploadResponse::changed` |
| `Hidden` | `bool` | **`true`** | `hidden(bool)` |
| `MaxFileSize` | `long?` | `null` | `max_file_size(u64)` → `Option<u64>` |
| `MaximumFileCount` | `int` | **`10`** | `maximum_file_count(usize)` |
| `Class` | `string?` | `null` | `classes(iter)` — stored, not resolved |
| `Tag` | `object?` | `null` | `tag(t)` / `tag_ref::<U>()` |

### Deliberately not ported

| MudBlazor | Reason |
|---|---|
| `InputClass` / `InputStyle` | egui has no inline CSS strings. The internal `InputFile` element has no egui analogue. |
| `Style` / `UserAttributes` | No CSS strings, no HTML tag. |
| `Class` resolution | Stored on `base` for parity only. The component’s own button chrome comes from `Variant` / `Color` / `Size`; a utility-class frame would composite a second fill. |
| Form-validation members (`Required`, `Error`, `ErrorText`, `HelperText`, `ConversionError`, `Strict`, `Immediate`, `DebounceInterval`) | Inherited from `MudFormComponent`; no egui form system. |

### `Hidden` is the drag-and-drop switch

MudBlazor’s docs: *“Defaults to `true`. When `false`, files can be uploaded via
drag-and-drop.”*

| `hidden` | Mode |
|---|---|
| `true` (default) | Button only. Clicking opens the OS file picker. |
| `false` | Button **and** a dashed drop zone. Files can be dropped on the component. |

There is no separate `drag_and_drop` flag in MudBlazor; the pre-existing Rust
port added one, and it is removed here.

### `MaximumFileCount` is per-pick, not per-list

The docs: *“The maximum number of files retrieved during a call to
`GetMultipleFiles`. This property does not limit the total number of uploaded
files allowed; a limit should be validated manually.”* `filter_files` truncates
the picked batch; `apply_selection` does not enforce a total cap.

---

## Architecture

```
inputs/file_upload/
└── mud_file_upload.rs
    ├── constants        MIN_BUTTON_WIDTH 200, DROP_ZONE_HEIGHT 180,
    │                    CORNER_RADIUS 4, ANIM_SECONDS 0.12,
    │                    DASH_LEN 8, DASH_GAP 6
    ├── SizeMetrics      height / font_size / padding_y per Size
    ├── pure helpers     parse_extensions, matches_accept, filter_files,
    │                    apply_selection, lerp_color, contrast_text,
    │                    radius_u8
    ├── rendering        draw_button, draw_drop_zone, draw_chip,
    │                    draw_dashed_rect, draw_dashed_line
    ├── pick_files       rfd::FileDialog (blocking)
    ├── MudFileUpload    builder + show
    └── impl Widget      ui.add(MudFileUpload::new(&mut files))
```

**Pure layer.** Extension parsing, accept matching, filtering, selection
application, colour lerp and contrast are free functions over `&Path`,
`&[String]` and `Option<u64>`. None touch `egui::Context`.

**Rendering layer.** `draw_button` and `draw_drop_zone` build the two surfaces;
`draw_chip` builds the removable file pills; `draw_dashed_rect` builds the drop
border.

**Picker layer.** `pick_files` is the only OS call. It is synchronous and
blocking; the README documents the async alternative.

---

## API reference

| Method | Type | Default | Mirrors | Description |
|---|---|---|---|---|
| `new(files)` | `&'a mut Vec<PathBuf>` | — | `Files` | Binds the component to a file list. |
| `single()` | — | — | `T = IBrowserFile` | Keeps only one file; replaces on pick. |
| `accept(s)` | `&'a str` | `None` | `Accept` | Comma-separated extensions. |
| `append_multiple_files(b)` | `bool` | `false` | `AppendMultipleFiles` | Append instead of replace. |
| `disabled(b)` | `bool` | `false` | `Disabled` | Disables interaction. |
| `hidden(b)` | `bool` | `true` | `Hidden` | `false` enables the drop zone. |
| `max_file_size(bytes)` | `u64` | `None` | `MaxFileSize` | Per-file byte cap. |
| `maximum_file_count(n)` | `usize` | `10` | `MaximumFileCount` | Per-pick cap; clamped to ≥ 1. |
| `variant(v)` | `Variant` | `Filled` | *(button chrome)* | Button variant. |
| `color(c)` | `Color` | `Primary` | *(button chrome)* | Accent colour. |
| `size(s)` | `Size` | `Medium` | *(button chrome)* | Button height and font. |
| `elevation(e)` | `u8` | `0` | *(button chrome)* | Shadow depth, clamped to 24. |
| `corner_radius(r)` | `f32` | `4.0` | *(button chrome)* | Corner radius. |
| `label(s)` | `&'a str` | `"Browse files"` | *(ActivatorContent fallback)* | Button label. |
| `aria_label(s)` | `&'a str` | `None` | *(extension)* | Accessible label. |
| `activator_content(f)` | `FnOnce(&mut Ui) -> Response` | `None` | `ActivatorContent` | Custom button content. |
| `classes(iter)` | `iter of AsRef<str>` | none | `Class` | Stored on `ComponentBase`. |
| `tag(t)` / `tag_ref::<U>()` | `T: Any + Send + Sync` | `None` | `Tag` | Arbitrary user data. |
| `show(ui)` | `&mut Ui` | — | — | Returns `FileUploadResponse`. |

### `FileUploadResponse`

| Field | Type | Description |
|---|---|---|
| `response` | `Response` | The component’s `Response`. Deref target. |
| `changed` | `bool` | `true` on the frame the file list changed. |

`FileUploadResponse` implements `Deref<Target = Response>`, so `out.clicked()`,
`out.hovered()` and `out.rect` work directly.

### `Size` mapping

| `Size` | Height | Font |
|---|---|---|
| `ExtraSmall` / `Small` | 28 pt | 12 pt |
| `Medium` | 36 pt | 14 pt |
| `Large` / `ExtraLarge` | 48 pt | 16 pt |

### Constants

| Name | Value |
|---|---|
| `MIN_BUTTON_WIDTH` | `200.0` |
| `DROP_ZONE_HEIGHT` | `180.0` |
| `CORNER_RADIUS` | `4.0` |
| `ANIM_SECONDS` | `0.12` |
| `DASH_LEN` / `DASH_GAP` | `8.0` / `6.0` |

---

## Examples

### Basic button

```rust
let mut files = Vec::new();
MudFileUpload::new(&mut files)
    .label("Choose a file")
    .show(ui);
```

### Accepted types and size cap

```rust
MudFileUpload::new(&mut files)
    .accept(".png, .jpg, .jpeg")
    .max_file_size(2 * 1024 * 1024)
    .label("Upload images")
    .show(ui);
```

### Drag-and-drop zone

```rust
MudFileUpload::new(&mut files)
    .hidden(false)          // enables the drop zone
    .label("Drop files here")
    .show(ui);
```

### Append instead of replace

```rust
MudFileUpload::new(&mut files)
    .append_multiple_files(true)
    .label("Add more")
    .show(ui);
```

### Single file

```rust
MudFileUpload::new(&mut files)
    .single()
    .accept(".pdf")
    .label("Choose a PDF")
    .show(ui);
```

### Custom activator

```rust
MudFileUpload::new(&mut files)
    .activator_content(|ui| {
        ui.add(MudButton::new("Upload").start_icon(filled::UPLOAD))
    })
    .show(ui);
```

### Reacting to a change

```rust
let out = MudFileUpload::new(&mut files).show(ui);
if out.changed {
    log::info!("{} file(s) selected", files.len());
}
```

### Chip list with removable files

```rust
// After `show`, `files` is the current list. The component draws a chip per
// file with a × button; clicking it removes the entry and sets `changed`.
```

### Tag round-trip

```rust
let upload = MudFileUpload::new(&mut files).tag("avatar");
assert_eq!(upload.tag_ref::<&str>(), Some(&"avatar"));
```

---

## Mode semantics

### Button mode (`hidden = true`, default)

```
┌──────────────────────────────┐
│        Browse files          │   ← button, 36 pt tall (Medium)
└──────────────────────────────┘
   ↓ click
   rfd::FileDialog → filtered → `files`
```

- Width: `available_width().min(200).max(80)`.
- Height: `SizeMetrics.height` (28 / 36 / 48 pt).
- Fill: `Variant::Filled` uses the accent; `Outlined` and `Text` are transparent
  with an accent border or label.

### Drag-and-drop mode (`hidden = false`)

```
┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐
│   Drag files here or click     │   ← dashed border, 180 pt tall (Medium)
│   to browse                    │
└ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘
```

- The dashed border animates from grey to the accent colour while
  `i.raw.hovered_files` is non-empty.
- Dropping outside the component still fires egui’s window-wide drop event, but
  the component only accepts it when `response.contains_pointer()` is true.
- Clicking still opens the picker.

### Chips

When `hidden = true` and `files` is non-empty, the component draws one chip per
file under the button. Each chip is a two-button group: the filename and a ×.
Clicking × removes that entry from `files` and sets `changed`.

---

## Validation

`filter_files` runs three checks in order, and stops at `maximum_file_count`:

1. **Extension** — `matches_accept` compares the lowercased extension against
   the parsed `accept` list. An empty list accepts everything.
2. **Size** — if `max_file_size` is set, `std::fs::metadata(path).len()` is
   compared against it. A file that cannot be read has size `0` and passes.
3. **Limit** — the batch is truncated to `maximum_file_count`.

Files that fail are silently dropped; the returned error is always `None`,
because MudBlazor surfaces per-file failures through form validation rather than
a single error string. If you need to show the user which files were rejected,
check `files.len()` after `show` against the number of files the user picked.

---

## Performance and memory choices

| Choice | Effect |
|---|---|
| Theme fetched once per frame | One `MudTheme::fetch` and one `Color::resolve`. |
| `accept` parsed once per frame | An empty list is an early return; a non-empty list is a small `Vec<String>`. |
| Pure validators on `&Path` | Extension and size checks are allocations only when the extension is lowercased. |
| `animate_bool_with_time` | Hover and focus transitions are idle when nothing changes. |
| One `allocate_exact_size` per surface | The button and drop zone are each one rect. |
| Chips are built only when `files` is non-empty **and** `hidden` | A drop-zone upload does not pay for a chip list it cannot show. |
| `rfd::FileDialog` is synchronous | One OS call per click; no background thread or channel. See the note below. |

### The blocking file dialog

`pick_files` calls `rfd::FileDialog::pick_files()`, which blocks the UI thread
until the user closes the dialog. For a desktop app this is usually acceptable —
the dialog is modal to the window anyway — but it means `show` cannot be called
from a frame that also animates. If you need a non-blocking picker, spawn
`rfd::AsyncFileDialog` on a worker thread and feed the result back through a
channel:

```rust
let (tx, rx) = std::sync::mpsc::channel();
std::thread::spawn(move || {
    let picked = rfd::AsyncFileDialog::new().pick_files();
    let _ = tx.send(picked);
});
// On a later frame: if let Ok(Some(handles)) = rx.try_recv() { … }
```

The component itself does not do this because the channel would have to live
somewhere, and that somewhere is the caller’s state. The README documents the
pattern rather than hiding it inside the widget.

---

## Testing

`cargo test -p mud_egui_components mud_file_upload` runs 9 pure-function tests:

| Area | Tests |
|---|---|
| `parse_extensions` | comma-separated, case folding, non-dot filtering, `None` |
| `matches_accept` | case-insensitive match, extensionless files, empty list |
| `filter_files` | extension filtering, batch truncation, size cap |
| `apply_selection` | replace vs append, single keeps only the first |
| `SizeMetrics` | edge sizes fold into `Small` / `Large` |
| Builders | defaults (hidden **true**, max count **10**), every field set, `maximum_file_count(0)` clamps to 1, tag round-trip |

No test creates an `egui::Context`. `show`, `draw_button`, `draw_drop_zone`,
`draw_chip` and `pick_files` need a live `Ui` or a real file dialog and are left
to integration tests; everything they delegate to is covered above.

Suggested integration tests (`egui_kittest` or a headless `Context::run`
harness with a fake picker):

- Clicking the button in `hidden = true` mode opens the picker and updates
  `files`.
- Dropping a file on the component in `hidden = false` mode updates `files`.
- Dropping a file outside the component does **not** update `files`.
- `single()` keeps only the first picked file.
- `append_multiple_files(true)` extends the list.
- The chip × removes the corresponding entry.

```bash
cargo test -p mud_egui_components mud_file_upload
```

---

## Differences from MudBlazor

- **`Files` / `FilesChanged` become a `&mut Vec<PathBuf>` plus
  `FileUploadResponse::changed`.** The caller owns the list; `changed` is `true`
  on the frame it moved. This is the same split `MudAlert` and `MudCollapse`
  use.
- **`Hidden` is the drag-and-drop switch, with no separate flag.** Your
  pre-existing port added `drag_and_drop(bool)`; it is removed because MudBlazor
  has no such parameter. Use `.hidden(false)` to enable the drop zone.
- **`MaximumFileCount` defaults to `10`, not `100`.** The DNDocs page is
  explicit; the pre-existing default of `100` was invented. It also caps only
  one picker call, not the total list.
- **The picker is synchronous.** `rfd::FileDialog` blocks the UI thread. The
  README documents the `AsyncFileDialog` + channel alternative; the component
  does not hide it.
- **`ActivatorContent` is a closure that returns a `Response`.** This is the
  direct analogue of a `RenderFragment` that renders the clickable activator.
- **`InputClass` / `InputStyle` are not ported.** egui has no CSS strings.
- **`Style` / `UserAttributes` are not ported.** No CSS strings, no HTML tag.
- **The chip list is an extension.** MudBlazor shows files through
  `SelectedTemplate` (a render fragment), which is a `ChildContent`-style slot.
  The port draws a default chip row so the common case works without a template;
  if you need a custom row, read `files` after `show` and draw your own.
- **The button chrome is an extension.** MudBlazor’s `MudFileUpload` wraps a
  `MudButton` through `ActivatorContent`; the port exposes `Variant`, `Color`,
  `Size`, `Elevation` and `CornerRadius` so a default button exists without a
  separate `MudButton` call. Every one is marked as button chrome in the API
  table, not as a MudBlazor parameter.
- **The drop-zone height and dash pattern are constants.** MudBlazor’s SCSS
  uses the same values; they are named at the top of the file so they can be
  adjusted in one place.