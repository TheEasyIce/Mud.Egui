//! The `RouteHost` equivalent. Renders the current page inside the current
//! layout, re-dispatching on URI change.
//!
//! # Frame loop
//!
//! Call [`Router::render`] once per frame. It compares
//! [`NavigationManager::current_uri`] with the URI it dispatched last (a plain
//! `&str` comparison — no allocation in the steady state). Only when they
//! differ does it resolve the route, build a fresh [`NavigationContext`], run
//! the lifecycle hooks and swap/reuse the page. Every frame it then draws the
//! page (inside the layout, if any) with a *borrowed* context — the context is
//! never cloned per frame.

use crate::navigation::NavigationManager;
use crate::page::{Layout, NavigationContext, Page};
use crate::query;
use crate::route_table::{PageFactory, RouteTable};
use crate::route_template::RouteTemplate;
use egui::{RichText, Ui};
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// Resolves the current URI to a [`Page`] and renders it inside an optional
/// [`Layout`].
///
/// Configure with the builder-style methods ([`with_layout`](Self::with_layout),
/// [`reuse_pages`](Self::reuse_pages), [`not_found_with`](Self::not_found_with)),
/// then call [`render`](Self::render) every frame.
pub struct Router {
    routes: RouteTable,
    layout: Option<Box<dyn Layout>>,
    reuse_pages: bool,

    not_found: Option<(TypeId, PageFactory)>,

    current_page: Option<Box<dyn Page>>,
    current_page_type: Option<TypeId>,
    current_route: Option<RouteTemplate>,
    current_ctx: Option<NavigationContext>,
    last_uri: String,
}

impl Default for Router {
    /// A router with an empty [`RouteTable`] (everything is "not found").
    fn default() -> Self {
        Self::new(RouteTable::default())
    }
}

impl fmt::Debug for Router {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Router")
            .field("routes", &self.routes.len())
            .field("has_layout", &self.layout.is_some())
            .field("reuse_pages", &self.reuse_pages)
            .field("current_route", &self.current_route.as_ref().map(|r| &r.template))
            .field("last_uri", &self.last_uri)
            .finish_non_exhaustive()
    }
}

impl Router {
    /// Creates a router over `routes`. Page reuse is enabled by default.
    pub fn new(routes: RouteTable) -> Self {
        Self {
            routes,
            layout: None,
            reuse_pages: true,
            not_found: None,
            current_page: None,
            current_page_type: None,
            current_route: None,
            current_ctx: None,
            last_uri: String::new(),
        }
    }

    /// Wraps every routed page in `layout`. The layout instance lives as long
    /// as the router and receives `on_navigated_*` hooks on every navigation.
    pub fn with_layout(mut self, layout: Box<dyn Layout>) -> Self {
        self.layout = Some(layout);
        self
    }

    /// Enables (default) or disables page-instance reuse.
    ///
    /// When enabled, navigating between URIs that resolve to the *same page
    /// type* keeps the existing instance and calls
    /// `bind(ctx, reset_missing = true)` on it instead of constructing a new
    /// one.
    pub fn reuse_pages(mut self, b: bool) -> Self {
        self.reuse_pages = b;
        self
    }

    /// Uses `factory` to build the page shown when no route matches.
    pub fn not_found_with<T: Page>(mut self, factory: impl Fn() -> T + 'static) -> Self {
        self.not_found = Some((
            TypeId::of::<T>(),
            Rc::new(move || -> Box<dyn Page> { Box::new(factory()) }),
        ));
        self
    }

    /// Uses `T::default()` to build the page shown when no route matches.
    pub fn not_found_default<T: Page + Default>(self) -> Self {
        self.not_found_with::<T>(T::default)
    }

    /// The context of the page currently shown, if any navigation has been
    /// processed yet.
    pub fn current_context(&self) -> Option<&NavigationContext> {
        self.current_ctx.as_ref()
    }

    /// The template of the matched route (`None` for the not-found page or
    /// before the first dispatch).
    pub fn current_route(&self) -> Option<&RouteTemplate> {
        self.current_route.as_ref()
    }

    /// [`TypeId`] of the page currently shown.
    pub fn current_page_type(&self) -> Option<TypeId> {
        self.current_page_type
    }

    /// Renders the current route. Call once per frame from `App::ui`.
    pub fn render(&mut self, ui: &mut Ui, nav: &mut NavigationManager) {
        // 1. Dispatch on URI change (frame-start, before rendering).
        self.sync(nav);

        // 2. Render current page inside current layout, borrowing the context.
        let fallback;
        let ctx: &NavigationContext = match self.current_ctx.as_ref() {
            Some(c) => c,
            None => {
                fallback = build_ctx(nav, self.current_route.as_ref());
                &fallback
            }
        };
        let Some(page) = self.current_page.as_mut() else {
            Self::render_not_found(ui);
            return;
        };
        match self.layout.as_mut() {
            Some(layout) => layout.show(ui, ctx, &mut |ui| page.show(ui, ctx, nav)),
            None => page.show(ui, ctx, nav),
        }
    }

    /// Re-dispatches if (and only if) the navigation URI changed since the last
    /// call. Allocation-free when nothing changed.
    pub(crate) fn sync(&mut self, nav: &NavigationManager) {
        if nav.current_uri() != self.last_uri {
            self.dispatch(nav);
            self.last_uri.clear();
            self.last_uri.push_str(nav.current_uri());
        }
    }

    fn dispatch(&mut self, nav: &NavigationManager) {
        let (path_raw, query_text) = query::split_path_query(nav.current_relative());
        let path = normalize_path(path_raw);
        let query_map = query::first_value_map(query_text);

        // Look up the route. Only cheap handles (Rc clone, TypeId) leave the match.
        let (source, type_id, new_route, route_values) = match self.routes.lookup(&path) {
            Some((entry, values)) => (
                Source::Factory(Rc::clone(&entry.factory)),
                entry.type_id,
                Some(entry.template.clone()),
                values,
            ),
            None => match &self.not_found {
                Some((tid, factory)) => (
                    Source::Factory(Rc::clone(factory)),
                    *tid,
                    None,
                    HashMap::new(),
                ),
                None => (
                    Source::Default,
                    TypeId::of::<DefaultNotFoundPage>(),
                    None,
                    HashMap::new(),
                ),
            },
        };

        let ctx = NavigationContext {
            location: nav.current_uri().to_string(),
            path,
            route_values,
            query: query_map,
        };

        let has_page = self.current_page.is_some();
        let reuse_page = self.reuse_pages && has_page && self.current_page_type == Some(type_id);

        // Leaving hooks: layout first, then page. Skipped on the very first
        // dispatch, when there is nothing to leave.
        if has_page {
            if let Some(layout) = self.layout.as_mut() {
                layout.on_navigated_from();
            }
            if let Some(old) = self.current_page.as_mut() {
                old.on_navigated_from();
            }
        }

        let mut reused = false;
        if reuse_page {
            // Rebind on the existing instance, then fire hooks in order. The
            // factory is deliberately not invoked.
            if let Some(page) = self.current_page.as_mut() {
                page.bind(&ctx, true);
                page.on_navigated_to(&ctx);
                reused = true;
            }
        }
        if !reused {
            let mut page: Box<dyn Page> = match source {
                Source::Factory(f) => f(),
                Source::Default => Box::new(DefaultNotFoundPage::new(ctx.path.clone())),
            };
            page.bind(&ctx, false);
            page.on_navigated_to(&ctx);
            self.current_page = Some(page);
            self.current_page_type = Some(type_id);
        }
        self.current_route = new_route;

        if let Some(layout) = self.layout.as_mut() {
            layout.on_navigated_to(&ctx);
        }

        self.current_ctx = Some(ctx);
    }

    fn render_not_found(ui: &mut Ui) {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new("No route matched.")
                    .size(18.0)
                    .color(ui.visuals().weak_text_color()),
            );
        });
    }

    /// Drops the current page + layout, firing `on_navigated_from` on both.
    /// Call from `App`'s shutdown path if the app cares about cleanup hooks.
    /// Calling it again is a harmless no-op.
    pub fn shutdown(&mut self) {
        if let Some(layout) = self.layout.as_mut() {
            layout.on_navigated_from();
        }
        if let Some(page) = self.current_page.as_mut() {
            page.on_navigated_from();
        }
        self.current_page = None;
        self.current_page_type = None;
        self.current_route = None;
        self.current_ctx = None;
        self.layout = None;
    }
}

/// Where a freshly needed page instance comes from.
enum Source {
    Factory(PageFactory),
    Default,
}

/// Built-in fallback page shown when no route (and no custom not-found page)
/// matches. Its `bind` keeps the displayed path current when it is reused.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct DefaultNotFoundPage {
    path: String,
}

impl DefaultNotFoundPage {
    fn new(path: String) -> Self {
        Self { path }
    }
}

impl Page for DefaultNotFoundPage {
    fn bind(&mut self, ctx: &NavigationContext, _reset_missing: bool) {
        self.path.clear();
        self.path.push_str(&ctx.path);
    }

    fn show(&mut self, ui: &mut Ui, _ctx: &NavigationContext, _nav: &mut NavigationManager) {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new(format!("No route matches '{}'.", self.path))
                    .size(18.0)
                    .color(ui.visuals().weak_text_color()),
            );
        });
    }
}

/// Rebuilds a context for the manager's current URI (fallback only).
fn build_ctx(nav: &NavigationManager, route_template: Option<&RouteTemplate>) -> NavigationContext {
    let (path_raw, query_text) = query::split_path_query(nav.current_relative());
    let path = normalize_path(path_raw);
    let route_values = route_template
        .and_then(|t| t.try_match(&path))
        .unwrap_or_default();
    NavigationContext {
        location: nav.current_uri().to_string(),
        path,
        route_values,
        query: query::first_value_map(query_text),
    }
}

/// Ensures exactly one leading `/`.
fn normalize_path(path: &str) -> String {
    let trimmed = path.trim_start_matches('/');
    let mut out = String::with_capacity(trimmed.len() + 1);
    out.push('/');
    out.push_str(trimmed);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;
    use std::cell::{Cell, RefCell};

    type Log = Rc<RefCell<Vec<String>>>;

    fn push(log: &Log, s: impl Into<String>) {
        log.borrow_mut().push(s.into());
    }
    fn take(log: &Log) -> Vec<String> {
        std::mem::take(&mut *log.borrow_mut())
    }

    struct OrderPage {
        log: Log,
        id: i64,
    }
    impl Page for OrderPage {
        fn bind(&mut self, ctx: &NavigationContext, reset_missing: bool) {
            self.id = ctx.route_i64("id").unwrap_or(-1);
            push(&self.log, format!("bind:{}:{}", self.id, reset_missing));
        }
        fn on_navigated_to(&mut self, ctx: &NavigationContext) {
            push(&self.log, format!("to:{}", ctx.path));
        }
        fn on_navigated_from(&mut self) {
            push(&self.log, "from");
        }
        fn show(&mut self, _: &mut Ui, _: &NavigationContext, _: &mut NavigationManager) {}
    }

    struct HomePage {
        log: Log,
    }
    impl Page for HomePage {
        fn bind(&mut self, _: &NavigationContext, _: bool) {
            push(&self.log, "home:bind");
        }
        fn on_navigated_to(&mut self, _: &NavigationContext) {
            push(&self.log, "home:to");
        }
        fn on_navigated_from(&mut self) {
            push(&self.log, "home:from");
        }
        fn show(&mut self, _: &mut Ui, _: &NavigationContext, _: &mut NavigationManager) {}
    }

    /// A page that only implements `show` (all hooks defaulted).
    #[derive(Default)]
    struct PlainPage;
    impl Page for PlainPage {
        fn show(&mut self, _: &mut Ui, _: &NavigationContext, _: &mut NavigationManager) {}
    }

    struct CustomNotFound;
    impl Page for CustomNotFound {
        fn show(&mut self, _: &mut Ui, _: &NavigationContext, _: &mut NavigationManager) {}
    }

    struct TestLayout {
        log: Log,
    }
    impl Layout for TestLayout {
        fn on_navigated_to(&mut self, ctx: &NavigationContext) {
            push(&self.log, format!("L:to:{}", ctx.path));
        }
        fn on_navigated_from(&mut self) {
            push(&self.log, "L:from");
        }
        fn show(&mut self, ui: &mut Ui, _: &NavigationContext, body: &mut dyn FnMut(&mut Ui)) {
            body(ui);
        }
    }

    struct Fixture {
        router: Router,
        nav: NavigationManager,
        log: Log,
        created: Rc<Cell<usize>>,
    }

    fn fixture_with(initial: &str, configure: impl FnOnce(Router, &Log) -> Router) -> Fixture {
        let log: Log = Rc::default();
        let created = Rc::new(Cell::new(0usize));
        let mut b = RouteTable::builder();
        {
            let (l, c) = (Rc::clone(&log), Rc::clone(&created));
            b.add_with::<OrderPage>("/orders/{id:int}", move || {
                c.set(c.get() + 1);
                OrderPage { log: Rc::clone(&l), id: 0 }
            })
                .unwrap();
        }
        {
            let l = Rc::clone(&log);
            b.add_with::<HomePage>("/", move || HomePage { log: Rc::clone(&l) })
                .unwrap();
        }
        b.add::<PlainPage>("/plain").unwrap();
        let router = configure(Router::new(b.build()), &log);
        let nav = NavigationManager::new(initial, "/app/", 20);
        Fixture { router, nav, log, created }
    }

    fn fixture(initial: &str) -> Fixture {
        fixture_with(initial, |r, _| r)
    }

    fn not_found_path(r: &Router) -> String {
        let page: &dyn Page = &**r.current_page.as_ref().expect("page");
        let any: &dyn Any = page;
        any.downcast_ref::<DefaultNotFoundPage>()
            .expect("default not-found page")
            .path
            .clone()
    }

    // ── dispatch & lifecycle ──

    #[test]
    fn first_dispatch_creates_binds_and_activates() {
        let mut f = fixture("/orders/42");
        f.router.sync(&f.nav);
        assert_eq!(take(&f.log), vec!["bind:42:false", "to:/orders/42"]);
        assert_eq!(f.created.get(), 1);
        assert_eq!(f.router.current_page_type(), Some(TypeId::of::<OrderPage>()));
        let ctx = f.router.current_context().unwrap();
        assert_eq!(ctx.route_i64("id"), Some(42));
        assert_eq!(ctx.path, "/orders/42");
        assert_eq!(ctx.location, "/app/orders/42");
        assert_eq!(f.router.current_route().unwrap().template, "/orders/{id:int}");
    }

    #[test]
    fn same_page_type_is_reused_and_rebound() {
        let mut f = fixture("/orders/42");
        f.router.sync(&f.nav);
        take(&f.log);

        f.nav.navigate_to("/orders/43", false).unwrap();
        f.router.sync(&f.nav);
        assert_eq!(take(&f.log), vec!["from", "bind:43:true", "to:/orders/43"]);
        assert_eq!(f.created.get(), 1, "factory must not run when the page is reused");
        assert_eq!(f.router.current_context().unwrap().route_i64("id"), Some(43));
    }

    #[test]
    fn different_page_type_replaces_the_page() {
        let mut f = fixture("/orders/42");
        f.router.sync(&f.nav);
        take(&f.log);

        f.nav.navigate_to("/", false).unwrap();
        f.router.sync(&f.nav);
        assert_eq!(take(&f.log), vec!["from", "home:bind", "home:to"]);
        assert_eq!(f.router.current_page_type(), Some(TypeId::of::<HomePage>()));
        assert!(f.router.current_route().unwrap().is_match("/"));
    }

    #[test]
    fn reuse_can_be_disabled() {
        let mut f = fixture_with("/orders/1", |r, _| r.reuse_pages(false));
        f.router.sync(&f.nav);
        take(&f.log);
        f.nav.navigate_to("/orders/2", false).unwrap();
        f.router.sync(&f.nav);
        assert_eq!(take(&f.log), vec!["from", "bind:2:false", "to:/orders/2"]);
        assert_eq!(f.created.get(), 2);
    }

    #[test]
    fn sync_is_idempotent_for_an_unchanged_uri() {
        let mut f = fixture("/orders/1");
        f.router.sync(&f.nav);
        take(&f.log);
        f.router.sync(&f.nav);
        f.router.sync(&f.nav);
        assert!(take(&f.log).is_empty());
        assert_eq!(f.created.get(), 1);
    }

    #[test]
    fn query_only_change_redispatches_and_reuses() {
        let mut f = fixture("/orders/1?tab=a");
        f.router.sync(&f.nav);
        take(&f.log);
        f.nav.navigate_to("/orders/1?tab=b", false).unwrap();
        f.router.sync(&f.nav);
        assert_eq!(take(&f.log), vec!["from", "bind:1:true", "to:/orders/1"]);
        assert_eq!(f.router.current_context().unwrap().query("tab"), Some("b"));
    }

    #[test]
    fn back_navigation_redispatches() {
        let mut f = fixture("/orders/1");
        f.router.sync(&f.nav);
        f.nav.navigate_to("/orders/2", false).unwrap();
        f.router.sync(&f.nav);
        take(&f.log);
        f.nav.back().unwrap();
        f.router.sync(&f.nav);
        assert_eq!(take(&f.log), vec!["from", "bind:1:true", "to:/orders/1"]);
    }

    #[test]
    fn page_with_only_show_works() {
        let mut f = fixture("/plain");
        f.router.sync(&f.nav);
        assert_eq!(f.router.current_page_type(), Some(TypeId::of::<PlainPage>()));
    }

    // ── context building ──

    #[test]
    fn context_carries_path_query_and_location() {
        let mut f = fixture("/orders/7?Filter=x&filter=y&Page=2#frag");
        f.router.sync(&f.nav);
        let c = f.router.current_context().unwrap();
        assert_eq!(c.path, "/orders/7");
        assert_eq!(c.location, "/app/orders/7?Filter=x&filter=y&Page=2#frag");
        assert_eq!(c.query("FILTER"), Some("x")); // first value wins
        assert_eq!(c.query_parse::<u8>("page"), Some(2));
    }

    #[test]
    fn redundant_slashes_in_the_uri_are_ignored() {
        let mut f = fixture("//orders//9/");
        f.router.sync(&f.nav);
        assert_eq!(f.router.current_context().unwrap().route_i64("id"), Some(9));
    }

    // ── not found ──

    #[test]
    fn unmatched_uri_shows_default_not_found_with_current_path() {
        let mut f = fixture("/nope/x");
        f.router.sync(&f.nav);
        assert_eq!(f.router.current_page_type(), Some(TypeId::of::<DefaultNotFoundPage>()));
        assert!(f.router.current_route().is_none());
        assert_eq!(not_found_path(&f.router), "/nope/x");
    }

    #[test]
    fn reused_default_not_found_page_tracks_the_new_path() {
        let mut f = fixture("/nope/x");
        f.router.sync(&f.nav);
        f.nav.navigate_to("/other", false).unwrap();
        f.router.sync(&f.nav);
        assert_eq!(not_found_path(&f.router), "/other");
    }

    #[test]
    fn custom_not_found_page_is_used() {
        let mut f = fixture_with("/zzz", |r, _| r.not_found_with(|| CustomNotFound));
        f.router.sync(&f.nav);
        assert_eq!(f.router.current_page_type(), Some(TypeId::of::<CustomNotFound>()));
    }

    #[test]
    fn not_found_default_uses_default_ctor() {
        let mut f = fixture_with("/zzz", |r, _| r.not_found_default::<PlainPage>());
        f.router.sync(&f.nav);
        assert_eq!(f.router.current_page_type(), Some(TypeId::of::<PlainPage>()));
    }

    #[test]
    fn empty_router_treats_everything_as_not_found() {
        let mut r = Router::default();
        let nav = NavigationManager::new("/", "/app/", 5);
        r.sync(&nav);
        assert_eq!(r.current_page_type(), Some(TypeId::of::<DefaultNotFoundPage>()));
        assert_eq!(not_found_path(&r), "/");
    }

    // ── layout ──

    #[test]
    fn layout_hooks_wrap_page_hooks() {
        let mut f = fixture_with("/orders/42", |r, log| {
            r.with_layout(Box::new(TestLayout { log: Rc::clone(log) }))
        });
        f.router.sync(&f.nav);
        // First dispatch: nothing to leave, so no `from` hooks.
        assert_eq!(
            take(&f.log),
            vec!["bind:42:false", "to:/orders/42", "L:to:/orders/42"]
        );

        f.nav.navigate_to("/orders/43", false).unwrap();
        f.router.sync(&f.nav);
        assert_eq!(
            take(&f.log),
            vec!["L:from", "from", "bind:43:true", "to:/orders/43", "L:to:/orders/43"]
        );
    }

    // ── shutdown ──

    #[test]
    fn shutdown_fires_from_hooks_once_and_clears_state() {
        let mut f = fixture_with("/orders/42", |r, log| {
            r.with_layout(Box::new(TestLayout { log: Rc::clone(log) }))
        });
        f.router.sync(&f.nav);
        take(&f.log);

        f.router.shutdown();
        assert_eq!(take(&f.log), vec!["L:from", "from"]);
        assert!(f.router.current_page_type().is_none());
        assert!(f.router.current_context().is_none());

        f.router.shutdown();
        assert!(take(&f.log).is_empty(), "second shutdown must be a no-op");
    }

    // ── helpers ──

    #[test]
    fn normalize_path_forces_one_leading_slash() {
        assert_eq!(normalize_path(""), "/");
        assert_eq!(normalize_path("a/b"), "/a/b");
        assert_eq!(normalize_path("///a"), "/a");
    }

    #[test]
    fn build_ctx_fallback_matches_template() {
        let nav = NavigationManager::new("/orders/5?x=1", "/app/", 5);
        let tpl = RouteTemplate::parse("/orders/{id:int}").unwrap();
        let c = build_ctx(&nav, Some(&tpl));
        assert_eq!(c.route_i64("id"), Some(5));
        assert_eq!(c.query("x"), Some("1"));
        assert!(build_ctx(&nav, None).route_values.is_empty());
    }

    #[test]
    fn router_debug_does_not_panic() {
        let f = fixture("/");
        assert!(format!("{:?}", f.router).contains("Router"));
    }
}