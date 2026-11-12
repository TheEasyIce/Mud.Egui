//! Root application state and per-frame driver.
//!
//! [`App`] is the single entry point eframe drives. It owns three
//! collaborators, each with a single job:
//!
//! | Field    | Type                  | Responsibility                                       |
//! |----------|-----------------------|------------------------------------------------------|
//! | `theme`  | [`MudTheme`]          | Design tokens pushed into the egui context each frame |
//! | `router` | [`Router`]            | Resolves the current path to a [`views`] page         |
//! | `nav`    | [`NavigationManager`] | History stack + navigation guards                     |
//!
//! ## Navigation flow
//!
//! Widgets do **not** call [`NavigationManager`] directly. Instead they push
//! the target path into egui temp data under [`PENDING_NAV`]; [`App::ui`]
//! drains that key on the next frame. This keeps navigation decisions out of
//! widget code and lets guards veto a transition *before* the router
//! re-renders.
//!
//! See [`views`] for the pages that participate in this protocol.

use eframe::egui;
use egui::ThemePreference;
use mud_egui_navigation::navigation::NavigationManager;
use mud_egui_navigation::route_table::RouteTable;
use mud_egui_navigation::router::Router;
use mud_egui_theming::theme::MudTheme;
use crate::views::layouts::MainLayout;
use crate::views::pages::{HomePage, NotFoundPage};
use crate::views::pages::layouts::{ContainerPage, GridPage};

/// egui temp-data key used to defer a navigation request to the next frame.
///
/// Producer (any widget):
///
/// ```ignore
/// ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new(PENDING_NAV), "/about".to_string()));
/// ```
///
/// Consumer: [`App::ui`], once per frame.
const PENDING_NAV: &str = "pending_nav";

/// Root application state.
///
/// Constructed via [`App::default`]; `eframe` calls [`App::ui`] every frame.
pub struct App {
    /// Design tokens synced into the egui context at the top of each frame.
    theme: MudTheme,

    /// Resolves the current path to a rendered page.
    router: Router,

    /// Owns the URL history stack and navigation guards.
    nav: NavigationManager,
}

/// Registers every route the app knows about.
///
/// Returned as a `Result` (rather than panicking here) so:
/// * the failure mode is decided by the caller — [`App::default`] panics,
/// * the function stays testable in isolation.
///
/// # Errors
///
/// Returns `Err` if any path is malformed or registered twice.
fn build_routes() -> Result<RouteTable, String> {
    let mut builder = RouteTable::builder();
    builder.add::<HomePage>("/")?;
    builder.add::<ContainerPage>("/docs/layout-containers/mudcontainer")?;
    builder.add::<GridPage>("/docs/layout-containers/mudgrid")?;
    Ok(builder.build())
}

impl Default for App {
    /// Builds the app from its static route table.
    ///
    /// # Panics
    ///
    /// Panics if [`build_routes`] fails. Routes are compile-time constants, so
    /// a failure here is a programmer error — it can only fire during
    /// development, never in a shipped build. If routes ever become dynamic
    /// (e.g. plugins), change this to return `Result` and let `main` handle
    /// the error.
    fn default() -> Self {
        let routes = build_routes().expect("route table must be valid");

        Self {
            theme: MudTheme::default(),
            router: Router::new(routes)
                .with_layout(Box::new(MainLayout::default()))
                .not_found_default::<NotFoundPage>(),
            nav: NavigationManager::new("/", "app://mud.egui.com/", 100),
        }
    }
}

impl eframe::App for App {
    /// Per-frame driver. Ordered by dependency:
    ///
    /// 1. **Theme** — push design tokens into the context so style, fonts,
    ///    and spacing take effect *this* frame.
    /// 2. **Navigation** — drain [`PENDING_NAV`] and hand the target to
    ///    [`NavigationManager::navigate_to`]. Guard vetoes (`Ok(false)`) and
    ///    errors are reported to stderr but do not abort the frame.
    /// 3. **Render** — [`Router`] resolves the current path and draws the
    ///    matching page, falling back to [`NotFoundPage`].
    ///
    /// # Notes
    ///
    /// * Dark mode is currently hard-coded — see the inline `TODO`.
    /// * `_frame` is unused; kept because the trait signature requires it.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // 1. Theme ------------------------------------------------------
        self.theme.sync(ui.ctx());
        ui.ctx().set_theme(ThemePreference::Light);
        // TODO: expose dark/light as a real setting (`App::dark_mode` field
        // or config file) instead of overriding every frame.
        self.theme.is_dark_mode = false;

        // 2. Navigation -------------------------------------------------
        let pending: Option<String> = ui
            .ctx()
            .data_mut(|d| d.remove_temp::<String>(egui::Id::new(PENDING_NAV)));

        if let Some(target) = pending {
            match self.nav.navigate_to(&target, false) {
                Ok(true) => { /* navigation committed */ }
                Ok(false) => { /* vetoed by a navigation guard */ }
                Err(err) => eprintln!("nav error for {target:?}: {err}"),
            }
        }

        // 3. Render -----------------------------------------------------
        self.router.render(ui, &mut self.nav);
    }
}