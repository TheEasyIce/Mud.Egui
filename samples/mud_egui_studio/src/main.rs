//! # Mud.Egui.Studio
//!
//! A desktop application built on [`eframe`]/[`egui`] for building and editing
//! Mud content.
//!
//! This binary crate is deliberately thin: it only
//! 1. configures the native window ([`eframe::NativeOptions`]),
//! 2. installs the image loaders required by `egui_extras`, and
//! 3. delegates startup to [`app::App`].
//!
//! Everything else — UI state, theming, and views — lives in sibling modules.
//!
//! ## Module layout
//!
//! | Module     | Responsibility                                    |
//! |------------|---------------------------------------------------|
//! | [`app`]    | Application state and the [`eframe::App`] impl    |
//! | [`fonts`]  | Custom font registration for the egui context     |
//! | [`views`]  | Reusable UI widgets and panel layouts             |
//!
//! ## Startup order (important)
//!
//! Image loaders **must** be installed before any `MudImage` is decoded.
//! The closure passed to [`eframe::run_native`] therefore runs
//! `install_image_loaders` *first*, then `setup_custom_fonts`, and only
//! then constructs [`app::App`].

mod app;
mod fonts;
mod views;

use app::App;
use eframe::egui;

/// Application entry point.
///
/// Configures a resizable 1280×800 window titled *"Mud.Egui.Studio"* and
/// hands control to eframe's event loop.
///
/// # Errors
///
/// Returns [`eframe::Error`] if the native window or graphics context cannot
/// be created (e.g. no display server, missing GPU backend).
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // .with_min_inner_size([1280.0, 800.0])
            .with_inner_size([1280.0, 800.0])
            .with_title("Mud.Egui.Studio")
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Mud.Egui.Studio",
        options,
        Box::new(|cc| {
            // MUST come first — installs the bytes:// and file:// loaders
            // before any MudImage tries to decode.
            egui_extras::install_image_loaders(&cc.egui_ctx);

            fonts::setup_custom_fonts(&cc.egui_ctx);

            Ok(Box::new(App::default()))
        }),
    )
}