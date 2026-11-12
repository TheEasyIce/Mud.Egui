Here is the complete guide for the Theming system. It has been placed in a single code block so you can easily copy it into your project's documentation.

```markdown
# Theming for Mud.Egui

Welcome to the Theming system for Mud.Egui. This crate (`mud_egui_theming`) is the direct Rust/egui port of the C# `Mud.Avalonia.Theming` system.

## 🚨 The Paradigm Shift: Avalonia vs. egui

If you are coming from Avalonia, WPF, or web development, you need to unlearn the concept of a `ResourceDictionary` and `ThemeProvider` controls. 

**In Avalonia (Retained Mode):**
1. A `ThemeProvider` control listens to the OS theme.
2. It generates a massive dictionary of 500+ XAML resources.
3. It broadcasts an event (`GlobalThemeChanged`).
4. Controls with `DynamicResource` bindings update themselves.

**In egui (Immediate Mode):**
1. You hold a `MudTheme` struct in your app state.
2. Every single frame (60 FPS), you call `theme.sync(ctx)`.
3. This overwrites egui's internal `Visuals` (colors, rounding, strokes).
4. Widgets read colors directly from the theme struct.

Because of this, **we completely deleted** the following C# files:
*   `ThemeProvider.cs`
*   `ThemeBroadcaster.cs`
*   `MudThemeResourceFactory.cs`
*   `MudThemeResourceKeyTable.cs`
*   `ThemeInjectionMode.cs`
*   All `*ResourceDefinition.cs` files

They are simply not needed.

---

## Getting Started

To use the theme, initialize it in your application state and sync it every frame.

```rust
use mud_egui_theming::theme::MudTheme;

struct MyApp {
    theme: MudTheme,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            theme: MudTheme::default(), // Starts in Light mode
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Sync the theme to egui's global styling every frame
        self.theme.sync(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello Mud.Egui");
            
            if ui.button("Toggle Dark Mode").clicked() {
                self.theme.is_dark_mode = !self.theme.is_dark_mode;
            }
        });
    }
}
```

---

## Core Concepts

### 1. `MudTheme` (The Aggregator)
The `MudTheme` struct holds everything: your light and dark palettes, typography, layout dimensions, shadows, and breakpoints.

It also holds the global `is_dark_mode` boolean. Setting this to `true` will instantly swap all colors to the dark palette on the next `sync()` call.

### 2. `Palette` (Semantic Colors)
The `Palette` struct holds semantic colors instead of raw hex codes. You don't ask for "Blue", you ask for `theme.current_palette().primary`.

It comes with two default palettes:
*   `Palette::new_light()`: Ivory backgrounds, ink text, deep clay primary.
*   `Palette::new_dark()`: Charcoal backgrounds, ivory text, warm clay primary.

**Computed Variants:**
Just like C#, colors have computed variants:
*   `palette.primary_darken()`
*   `palette.primary_lighten()`
*   `palette.primary.with_alpha(0.5)`

### 3. `MudColor` (Color Math)
Because egui uses `Color32` (which lacks HSL math and hex parsing), we provide `MudColor`.
*   Parses Hex: `MudColor::parse("#B85533")`
*   Converts directly to egui: `let color: Color32 = mud_color.into();`

### 4. `Typography`
We ported the 14-tier Material Design type scale. It holds `FontVariant` structs (H1-H6, Body1/2, Subtitle1/2, etc.).

### 5. `LayoutProperties`
Holds the standard MudBlazor dimensions:
*   `appbar_height` (64.0)
*   `drawer_width_left` (240.0)
*   `drawer_mini_width_left` (56.0)
*   `default_border_radius` (4.0)

### 6. `MudShadows`
Provides 25 levels of elevation (`0` through `24`), mapped to `egui::epaint::Shadow`. Light mode uses black alpha shadows; Dark mode uses ivory alpha shadows to ensure visibility on dark surfaces.

### 7. `Breakpoints`
Used for responsive design. Instead of checking raw pixel widths, you check the bucket.
*   `Xs`, `Sm`, `Md`, `Lg`, `Xl`, `Xxl`.

---

## Using the Theme in Widgets

Because egui is immediate mode, widgets can easily read the global theme from the `Context`.

### Accessing the Theme
We provide a `fetch` method that safely retrieves the shared theme from `egui`'s memory.

```rust
use mud_egui_theming::theme::MudTheme;

fn my_custom_widget(ui: &mut egui::Ui) {
    // 1. Fetch the theme
    let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
    let palette = theme.current_palette();

    // 2. Use the colors
    let text_color = palette.primary_contrast_text.into();
    let bg_color = palette.primary.into();

    ui.colored_label(text_color, "Hello, Colored Text!");
}
```

### Resolving Utility Keys
If you are using the `bs_*` or `tw_*` utility parsers (like `bg-primary` or `border-dashed`), they output string keys like `"MudPalettePrimary"`.

Use `theme.resolve(key)` to turn that string into a real `Color32`.

```rust
// Inside your TwStyle / BsStyle build_frame method:
let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();

let color = match bg_class {
    TwBg::Literal(c) => c,
    TwBg::Resource { key, alpha } => {
        // Resolves "MudPalettePrimary" to a Color32
        let base_color = theme.resolve(&key).unwrap_or(egui::Color32::TRANSPARENT);
        // apply opacity...
        base_color
    }
};
```

### Checking Breakpoints
To build responsive layouts, check the current screen width against the theme's breakpoints.

```rust
fn responsive_widget(ui: &mut egui::Ui) {
    let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
    let screen_width = ui.ctx().screen_rect().width();
    
    let bucket = theme.breakpoints.get_bucket(screen_width);

    match bucket {
        BreakpointBucket::Xs | BreakpointBucket::Sm => {
            ui.label("Mobile View");
        }
        _ => {
            ui.label("Desktop View");
        }
    }
}
```

### Applying Shadows
Use the active shadow set to give elevation to panels or popups.

```rust
let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
let shadow = theme.current_shadows().get(4); // Get elevation level 4

// Apply to an egui Frame
egui::Frame::window(ui.style())
    .shadow(shadow)
    .show(ui, |ui| {
        ui.label("Elevated Card");
    });
```

---

## Troubleshooting

| Symptom | Likely Cause / Fix |
|---|---|
| **Theme colors don't apply** | Did you call `self.theme.sync(ctx)` at the top of your `update()` method? |
| **Dark mode doesn't work** | Ensure `theme.is_dark_mode = true` is set, and that you called `sync(ctx)` afterwards. |
| **Colors are transparent** | Ensure `MudColor` parsing didn't fail. Hex codes should include `#` or be valid lengths (3, 4, 6, 8). |
| **Text is invisible on buttons** | You are probably using `palette.primary` for text when you should be using `palette.primary_contrast_text`. |
| **Compile error on `style`** | In newer egui versions, `ctx.style()` is gone. Use `ctx.all_styles_mut(|style| { ... })` inside `sync()`. |
```