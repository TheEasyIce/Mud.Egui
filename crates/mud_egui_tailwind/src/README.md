# Tailwind Utilities for Mud.Egui

Tailwind-style utility classes for egui widgets. Supports arbitrary values, opacity modifiers and interactive variants.

## Contents

- [Getting Started](#getting-started)
- [Utilities](#utilities)
- [Spacing Scale](#spacing-scale)
- [Specificity Rules](#specificity-rules)
- [Troubleshooting](#troubleshooting)

## Getting Started

Tailwind has variants such as `hover:` and `dark:`, so you **must** pass the current interaction and theme state into the resolver.

```rust
use mud_egui_components::foundation::base::component_base::ComponentBase;
use mud_egui_tailwind::tw_style::TwStyle;

pub struct MyCard {
    base: ComponentBase,
    style: TwStyle,
    hovered: bool, // result from the previous frame
}

impl MyCard {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new().classes([
                "bg-primary/90",
                "hover:bg-primary-dark",
                "p-4",
                "rounded-lg",
            ]),
            style: TwStyle::default(),
            hovered: false,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let is_dark = ui.visuals().dark_mode;

        // Resolve the classes into an egui::Frame (16.0 is the rem size).
        let frame = self
            .style
            .build_frame(&self.base, ui, self.hovered, is_dark, 16.0);

        let out = frame.show(ui, |ui| {
            ui.label("Hello Tailwind");
        });

        // Remember hover state for the next frame.
        self.hovered = out.response.hovered();
    }
}
```

**Why remember hover from the previous frame?** The frame's real rect is only known after it is drawn, and the style is needed before drawing. Reading the last frame's `response.hovered()` is accurate and avoids testing the whole available area (`ui.max_rect()`), which would report a hover for anything inside the parent.

## Utilities

| Utility | Controls | Example classes |
| --- | --- | --- |
| [TwBackground](#twbackground) | `Frame.fill` | `bg-primary/50`, `bg-[#f00]`, `hover:bg-primary` |
| [TwBorder](#twborder) | `Frame.stroke` | `border border-dashed`, `border-[3px]` |
| [TwMargin / TwPadding](#twmargin--twpadding) | outer / inner margin | `m-4`, `-mt-2`, `p-[1.5rem]` |
| [TwRounded](#twrounded) | `Frame.corner_radius` | `rounded-lg`, `rounded-tl`, `rounded-[8px]` |

### TwBackground

- **Palette:** `bg-primary`, `bg-appbar-background`
- **Aliases:** `bg-primary-foreground` (maps to `MudPalettePrimaryText`), `bg-primary-dark`, `bg-primary-light`
- **Arbitrary hex:** `bg-[#f00]`, `bg-[#ff000080]` (CSS RGBA order)
- **Opacity:** `bg-primary/50` (50%), `bg-white/[0.2]`
- **Variants (stackable):** `hover:bg-primary`, `dark:bg-surface`, `focus:bg-active`

### TwBorder

Width and style are separate classes, as in Tailwind.

- **Styles:** `border-solid`, `border-dashed`, `border-dotted`, `border-double`, `border-none`
- **Widths:** `border` (1px), `border-2` (2px), `border-[3px]`, `border-[0.125rem]`
- **Example:** `border border-dashed`

### TwMargin & TwPadding

- **Spacing:** `m-4` (1rem), `p-2.5`, `mt-px` (1 pixel)
- **Negative margins:** `-mt-2`, `-mx-4`
- **Arbitrary values:** `m-[12px]`, `p-[1.5rem]`, `mt-[12]` (unitless means px)
- **Auto margins:** `mx-auto`, `ml-auto`, `mr-auto` (returns an `egui::Align`)

### TwRounded

- **Sizes:** `rounded-none`, `rounded-sm`, `rounded-md`, `rounded-lg`, `rounded-xl`, `rounded-2xl`, `rounded-full`
- **Corners:** `rounded-t` (top), `rounded-tl` (top-left), `rounded-ss` (start-start), and so on
- **Arbitrary:** `rounded-[8px]`

## Spacing Scale

A number `n` equals `n * 0.25rem`.

| Class | Value |
| --- | --- |
| `p-px` | 1px |
| `p-1` | 0.25rem |
| `p-4` | 1rem |
| `p-8` | 2rem |

## Specificity Rules

When several classes touch the same property, the more specific one wins, regardless of order.

| Utility | Order (lowest → highest) |
| --- | --- |
| Margin / padding | `m` < `mx`/`my` < `ms`/`me` < `mt`/`mr`/`mb`/`ml` |
| Rounded | `rounded` < `s` < `e` < `t` < `r` < `b` < `l` < `ss` < `se` < `ee` < `es` < `tl` < `tr` < `br` < `bl` |

Rounded classes follow Tailwind CSS order and are resolved per corner.

## Troubleshooting

| Symptom | Likely cause / fix |
| --- | --- |
| **`hover:` classes never trigger** | Pass the hover flag to `build_frame`. Use the previous frame's `response.hovered()`, as in the example above. |
| **Opacity looks wrong** | Your theme lookup should return an unmultiplied `egui::Color32` so the parser can change the alpha channel. |
| **`mx-auto` doesn't center** | `auto` returns an `egui::Align`. Apply it to the UI layout yourself. |
| **Arbitrary value fails to parse** | Check for typos. `[12px]` is valid; `[12px` (missing bracket) is not. |
| **`dark:` classes ignored** | Pass `ui.visuals().dark_mode` (or your own theme flag) as `is_dark`. |