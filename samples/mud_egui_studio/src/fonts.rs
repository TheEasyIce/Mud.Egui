//! Custom font registration.
//!
//! egui ships with a small default font set. To use the project's design
//! fonts we register two additional [`FontData`] entries and expose each as
//! a named [`FontFamily`] that can be referenced from styles and widgets:
//!
//! | Family name      | Font file                                                     |
//! |------------------|---------------------------------------------------------------|
//! | `"Google Sans"`  | `assets/fonts/Google_Sans/GoogleSans-VariableFont_GRAD,opsz,wght.ttf` |
//! | `"JetBrains Mono"` | `assets/fonts/JetBrains_Mono/static/JetBrainsMono-Regular.ttf` |
//!
//! Fonts are embedded at compile time with [`include_bytes!`], so no runtime
//! file I/O or path resolution is required.

use std::sync::Arc;

use eframe::egui;
use egui::{FontData, FontDefinitions, FontFamily};

/// Registers the project's custom fonts on the given egui context.
///
/// Call this once, during app startup, **after** any image loaders have been
/// installed and **before** the first frame is drawn. Calling it again on the
/// same context simply replaces the previous definitions.
///
/// # Examples
///
/// ```no_run
/// # use eframe::egui;
/// # fn setup_custom_fonts(_: &egui::Context) {}
/// let ctx = egui::Context::default();
/// setup_custom_fonts(&ctx);
/// ```
///
/// # Panics
///
/// Panics at **compile time** if any of the embedded font files listed in
/// this module's documentation are missing or were renamed — [`include_bytes!`]
/// resolves paths relative to this source file (`src/fonts.rs`), so the
/// `../assets/…` prefix is required.
pub fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "google_sans".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "assets/fonts/Google_Sans/GoogleSans-VariableFont_GRAD,opsz,wght.ttf"
        ))),
    );

    fonts.font_data.insert(
        "jetbrains_mono".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "assets/fonts/JetBrains_Mono/static/JetBrainsMono-Regular.ttf"
        ))),
    );

    fonts.families.insert(
        FontFamily::Name(Arc::from("Google Sans")),
        vec!["google_sans".to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name(Arc::from("JetBrains Mono")),
        vec!["jetbrains_mono".to_owned()],
    );

    ctx.set_fonts(fonts);
}