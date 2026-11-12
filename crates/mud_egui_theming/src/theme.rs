//! The core theme engine and Dependency Injection (DI) hub for `mud_egui`.
//!
//! This module defines the `MudTheme` struct, which acts as the single source of truth
//! for the application's look and feel (Palettes, Typography, Layout, Breakpoints, and Shadows).
//!
//! # Architecture: Theme Provider Pattern
//! To avoid passing a `MudTheme` reference down through every single UI component function,
//! this module utilizes `egui::Context`'s internal memory storage (`data_mut`).
//! 1. Call `theme.sync(ctx)` once per frame in your `eframe::App::update`.
//! 2. Anywhere in your UI code, call `MudTheme::fetch(ctx)` to instantly retrieve the active theme.

use std::sync::Arc;

use egui::{Color32, Context, Id};

use crate::models::palettes::palette::Palette;
use crate::models::ui::breakpoints::Breakpoints;
use crate::models::ui::layout::LayoutProperties;
use crate::models::ui::shadow::MudShadows;
use crate::models::ui::typography::Typography;

/// The unique identifier used to store and retrieve the theme from `egui`'s temporary memory.
const THEME_ID: &str = "mud_egui_theme";

/// The central theme configuration object for the application.
///
/// This struct holds all styling data. In a typical application, you should create
/// a single instance of `MudTheme`, store it in your `eframe::App` state, and call
/// `theme.sync(ctx)` at the start of your `update` loop to apply styles globally and
/// make the theme available to all child components.
#[derive(Debug, Clone)]
pub struct MudTheme {
    /// The color palette used when the application is in Light Mode.
    pub palette_light: Palette,
    /// The color palette used when the application is in Dark Mode.
    pub palette_dark: Palette,

    /// Global typographic scales (font sizes, line heights, etc.).
    pub typography: Typography,

    /// Global geometric layout properties (drawer widths, border radii, etc.).
    pub layout: LayoutProperties,

    /// Global responsive design thresholds (sm, md, lg, etc.).
    pub breakpoints: Breakpoints,

    /// The drop-shadow elevation configurations for Light Mode.
    pub shadows_light: MudShadows,
    /// The drop-shadow elevation configurations for Dark Mode (uses glow instead of shadow).
    pub shadows_dark: MudShadows,

    /// Global state flag controlling which palette and shadows are currently active.
    /// Toggle this and the next frame will instantly switch themes across the entire UI.
    pub is_dark_mode: bool,
}

impl Default for MudTheme {
    /// Generates a standard `MudTheme` using the default MudBlazor specifications.
    fn default() -> Self {
        Self {
            palette_light: Palette::new_light(),
            palette_dark: Palette::new_dark(),
            typography: Typography::default(),
            layout: LayoutProperties::default(),
            breakpoints: Breakpoints::default(),
            shadows_light: MudShadows::new_light(),
            shadows_dark: MudShadows::new_dark(),
            is_dark_mode: false, // Default to light mode
        }
    }
}

impl MudTheme {
    /// Returns a reference to the currently active palette based on the `is_dark_mode` flag.
    pub fn current_palette(&self) -> &Palette {
        if self.is_dark_mode {
            &self.palette_dark
        } else {
            &self.palette_light
        }
    }

    /// Returns a reference to the currently active shadow set based on the `is_dark_mode` flag.
    pub fn current_shadows(&self) -> &MudShadows {
        if self.is_dark_mode {
            &self.shadows_dark
        } else {
            &self.shadows_light
        }
    }

    /// Resolves a semantic utility-class key (e.g., `"MudPalettePrimaryDarken"`) into a concrete color.
    ///
    /// # Usage
    /// This is heavily used by the `TwStyle` parsing engine. When a component has a class
    /// like `bg-primary`, the parser queries this method to get the exact `Color32` for the
    /// currently active theme.
    pub fn resolve(&self, key: &str) -> Option<Color32> {
        self.current_palette().resolve(key)
    }

    /// Synchronizes this theme with the global `egui::Context` and installs it for child components.
    ///
    /// This method does two things:
    /// 1. Maps our semantic tokens (e.g., `palette.surface`) into `egui`'s native
    ///    interaction-based `Visuals` system, applying it to *all* style scopes via `all_styles_mut`.
    /// 2. Calls `install(ctx)` to place a shared `Arc` of this theme into memory.
    ///
    /// # Usage
    /// Call this once per frame at the very beginning of `eframe::App::update`.
    pub fn sync(&self, ctx: &Context) {
        let palette = self.current_palette();
        let is_dark = self.is_dark_mode;

        // NEW EGUI API: `all_styles_mut` modifies the base style AND any active
        // scoped styles, ensuring our theme overrides apply everywhere consistently.
        ctx.all_styles_mut(|style| {
            // --------------------------------------------------------------------
            // Global Visual Properties
            // --------------------------------------------------------------------
            style.visuals.dark_mode = is_dark;
            style.visuals.override_text_color = Some(palette.text_primary.into());

            // --------------------------------------------------------------------
            // Widget Interaction States
            // --------------------------------------------------------------------

            // Non-interactive: Standard static elements (e.g., labels, standard frames)
            style.visuals.widgets.noninteractive.bg_fill = palette.surface.into();

            // Note: We use `Color32::from()` explicitly here because `Stroke::new` uses generics
            // for its color argument, which causes type-inference issues with `.into()`.
            style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, Color32::from(palette.divider));
            style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, Color32::from(palette.text_primary));

            // Inactive: Interactive elements (like buttons) that are NOT currently being hovered/clicked.
            style.visuals.widgets.inactive.bg_fill = palette.action_disabled_background.into();
            style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, Color32::from(palette.text_primary));

            // Hovered: Interactive elements when the mouse is over them.
            style.visuals.widgets.hovered.bg_fill = palette.action_disabled.into();
            style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, Color32::from(palette.text_primary));

            // Active: Interactive elements when they are being clicked/pressed. (Brand colors pop here)
            style.visuals.widgets.active.bg_fill = palette.primary.into();
            style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, Color32::from(palette.primary_contrast_text));

            // --------------------------------------------------------------------
            // Base Containers
            // --------------------------------------------------------------------

            // Panel fill is the absolute bottom layer (e.g., CentralPanel).
            style.visuals.panel_fill = palette.background.into();

            // Window fill is for floating/draggable egui windows and dialogs.
            style.visuals.window_fill = palette.surface.into();
            style.visuals.window_stroke = egui::Stroke::new(1.0, Color32::from(palette.divider));

            // --------------------------------------------------------------------
            // Typography (Fonts & Sizes)
            // --------------------------------------------------------------------

            // Maps the MudTheme Typography configuration to egui's built-in TextStyles.
            // This ensures standard widgets (ui.label, ui.heading, ui.button) use the fonts.

            let body_family = to_egui_family(self.typography.body2.font_family);
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(self.typography.body2.font_size, body_family),
            );

            let button_family = to_egui_family(self.typography.button.font_family);
            style.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::new(self.typography.button.font_size, button_family),
            );

            let heading_family = to_egui_family(self.typography.h1.font_family);
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::new(self.typography.h1.font_size, heading_family),
            );

            let mono_family = to_egui_family("Monospace");
            style.text_styles.insert(
                egui::TextStyle::Monospace,
                egui::FontId::new(14.0, mono_family),
            );
        });

        // Store the theme in Context memory so widgets can retrieve it dynamically
        self.install(ctx);
    }

    /// Stores a shared `Arc` copy of the theme in the `egui` context.
    ///
    /// By using `Arc`, we avoid expensive clones of the entire theme struct every frame.
    /// `insert_temp` ensures this data is cleared if it isn't refreshed every frame,
    /// keeping memory clean.
    #[allow(deprecated)]
    pub fn install(&self, ctx: &Context) {
        let shared = Arc::new(self.clone());
        ctx.data_mut(|d| d.insert_temp(Id::new(THEME_ID), shared));
    }

    /// Retrieves the theme installed by `sync`/`install` from the `egui` context.
    ///
    /// # Usage
    /// Inside any component's `show` or `render` method, you can grab the theme without
    /// needing it passed down through function arguments:
    ///
    /// ```ignore
    /// let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
    /// let lookup = |key: &str| theme.resolve(key); // Ready to pass to TwStyle!
    /// ```
    #[allow(deprecated)]
    pub fn fetch(ctx: &Context) -> Option<Arc<MudTheme>> {
        ctx.data(|d| d.get_temp::<Arc<MudTheme>>(Id::new(THEME_ID)))
    }
}

/// Helper to safely map a string font name to an egui `FontFamily`.
///
/// This handles egui's built-in families (Proportional, Monospace) and automatically
/// falls back to a `Named` family for custom fonts loaded by the app.
pub fn to_egui_family(name: &str) -> egui::FontFamily {
    match name {
        "Proportional" => egui::FontFamily::Proportional,
        "Monospace" => egui::FontFamily::Monospace,
        _ => egui::FontFamily::Name(std::sync::Arc::from(name)),
    }
}