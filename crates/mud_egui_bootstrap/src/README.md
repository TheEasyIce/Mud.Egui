# Bootstrap Utilities for Mud.Egui

Bootstrap-style utility classes for egui widgets. Style controls with class names like `bg-primary`, `p-3` and `rounded-pill`, resolved against your active theme.

## Contents

- [Getting Started](#getting-started)
- [Utilities](#utilities)
- [Scales](#scales)
- [Specificity Rules](#specificity-rules)
- [Troubleshooting](#troubleshooting)

## Getting Started

egui is an immediate-mode library, so there is no persistent widget object and no `IsEnabled`-style property. Instead, each component stores a style cache (`BsStyle`) next to its `ComponentBase`. The cache resolves class names once and reuses the result on later frames.

```rust
use mud_egui_components::foundation::base::component_base::ComponentBase;
use mud_egui_bootstrap::bs_style::BsStyle;

pub struct MyCard {
    base: ComponentBase,
    style: BsStyle,
}

impl MyCard {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new().classes(["bg-primary", "p-3", "rounded-pill"]),
            style: BsStyle::default(),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Resolve the classes into an egui::Frame.
        // 14.0 is the rem size in points.
        let frame = self.style.build_frame(&self.base, ui, 14.0);

        // Draw inside the frame.
        frame.show(ui, |ui| {
            ui.label("Hello Bootstrap");
        });
    }
}
```

**The flow:** declare classes on `ComponentBase` → resolve with `BsStyle::build_frame` → draw inside the returned `egui::Frame`.

Keep `BsStyle` as a field on your component. Creating it inside `ui()` throws the cache away every frame.

## Utilities

| Utility | Controls | Example classes |
| --- | --- | --- |
| [BsBackground](#bsbackground) | `Frame.fill` | `bg-primary`, `bg-body`, `bg-primary-hover` |
| [BsBorder](#bsborder) | `Frame.stroke` | `border-dashed-2`, `border-0` |
| [BsMargin / BsPadding](#bsmargin--bspadding) | `Frame.outer_margin` / `Frame.inner_margin` | `m-3`, `px-2`, `mt-n2`, `mx-auto` |
| [BsRounded](#bsrounded) | `Frame.corner_radius` | `rounded-3`, `rounded-pill`, `rounded-top` |

### BsBackground

Binds `Frame.fill` to a theme palette color.

- **Syntax:** `bg-primary`, `bg-body`, `bg-transparent`, `bg-white`, `bg-black`
- **Hover:** `bg-primary-hover` maps to `MudPalettePrimaryHoverBrush`
- **Aliases:** `bg-body` maps to `MudPaletteBackground`
- **Rule:** the last `bg-*` class written wins

### BsBorder

Sets `Frame.stroke`.

- **Styles:** `border-solid`, `border-dashed`, `border-dotted`
- **Width:** `border-0` to `border-5`
- **Combined:** `border-dashed-2` (dashed style, 2px width)

### BsMargin & BsPadding

Sets `Frame.outer_margin` (margin) and `Frame.inner_margin` (padding).

- **Syntax:** `m-3` (margin), `p-3` (padding), `mx-2` (horizontal), `mt-0` (top)
- **Negative margins:** `mt-n2` (that is, `-0.5rem`)
- **Auto margins:** `mx-auto`, `ms-auto`, `me-auto`

> **Note:** egui has no auto margins. For `*-auto` classes the cache returns an `egui::Align`, which you must apply to your layout yourself.

### BsRounded

Sets `Frame.corner_radius`.

- **Sizes:** `rounded-0` to `rounded-5`, `rounded-pill`, `rounded-circle`
- **Sides:** `rounded-top`, `rounded-end`, `rounded-bottom`, `rounded-start`

## Scales

All values are multiples of the `rem` size you pass to `build_frame` (Bootstrap's usual base is `14.0` here).

### Spacing (margin & padding)

| Step | `0` | `1` | `2` | `3` | `4` | `5` |
| --- | --- | --- | --- | --- | --- | --- |
| rem | 0 | 0.25 | 0.5 | 1 | 1.5 | 3 |

### Corner radius

| Step | `0` | `1` | `2` (default) | `3` | `4` | `5` | `pill` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| rem | 0 | 0.25 | 0.375 | 0.5 | 1 | 2 | 50 |

## Specificity Rules

When several classes touch the same property, the more specific one wins, regardless of order.

| Utility | Order (lowest → highest) |
| --- | --- |
| Margin / padding | `m` < `mx`/`my` < `mt`/`mb`/`ms`/`me` |
| Rounded | `rounded*` < `top` < `end` < `bottom` < `start` |
| Background | last `bg-*` written wins |

Example: `p-0 pt-3` gives zero padding on three sides and `1rem` on top.

## Troubleshooting

| Symptom | Likely cause / fix |
| --- | --- |
| **Classes do nothing** | Pass the `ComponentBase` to `build_frame` and draw inside the returned `Frame`. |
| **`mx-auto` doesn't center** | `auto` returns an `egui::Align`. Apply it to the layout, for example with `ui.with_layout(egui::Layout::right_to_left(...), ...)`. |
| **Padding is huge** | Check the `rem` value you passed. At `16.0`, `p-3` is 16px; at `14.0` it is 14px. A mistaken value such as `48.0` multiplies every scale. |
| **Wrong colors** | The resolver returns raw keys such as `"MudPalettePrimary"`. Make sure your theme lookup can resolve them. |