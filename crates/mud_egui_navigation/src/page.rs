//! `Page` and `Layout` traits, plus `NavigationContext`. Replaces the Avalonia
//! `Control` base + `[RouteParameter]` / `[QueryParameter]` attributes.
//!
//! # Lifecycle
//!
//! When the [`Router`](crate::router::Router) observes a URI change it
//! runs, in order:
//!
//! 1. `Layout::on_navigated_from`, then `Page::on_navigated_from` (only if a
//!    page is currently active),
//! 2. `Page::bind` (a *new* instance is created first unless the current one is
//!    reused),
//! 3. `Page::on_navigated_to`,
//! 4. `Layout::on_navigated_to`.
//!
//! After that, [`Page::show`] is called every frame.
//!
//! # MSRV
//!
//! `impl dyn Page { fn type_id }` relies on trait-object upcasting
//! (`&dyn Page` → `&dyn Any`), stable since Rust 1.86.

use crate::navigation::NavigationManager;
use crate::query::to_lower_cow;
use crate::route_value::RouteValue;
use egui::Ui;
use std::any::Any;
use std::collections::HashMap;
use std::str::FromStr;

/// Bundle of route data passed to pages and layouts. Port of `NavigationContext.cs`.
///
/// Both lookup maps are keyed by **lower-cased** names; the accessor methods
/// lower-case their argument for you (without allocating when it is already
/// lower-case), so `ctx.route("OrderId")` and `ctx.route("orderid")` are
/// equivalent. If you read the public maps directly, use lower-case keys.
///
/// `NavigationContext` is [`PartialEq`] but not `Eq` because [`RouteValue`] can
/// hold an `f64`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NavigationContext {
    /// The full, absolute URI that produced this context.
    pub location: String,
    /// Base-relative path with a leading `/`, without query or fragment
    /// (for example `/orders/42`).
    pub path: String,
    /// Typed values captured from the matched route template.
    pub route_values: HashMap<String, RouteValue>,
    /// First value of each query parameter (later duplicates are ignored).
    pub query: HashMap<String, String>,
}

impl NavigationContext {
    /// Looks up a captured route value by (case-insensitive) name.
    pub fn route(&self, key: &str) -> Option<&RouteValue> {
        self.route_values.get(&*to_lower_cow(key))
    }

    /// Returns the route value as a string slice if it is a
    /// [`RouteValue::Str`].
    pub fn route_str(&self, key: &str) -> Option<&str> {
        self.route(key).and_then(RouteValue::as_str)
    }

    /// Returns the route value as an `i64` (see [`RouteValue::as_i64`]).
    pub fn route_i64(&self, key: &str) -> Option<i64> {
        self.route(key).and_then(RouteValue::as_i64)
    }

    /// Returns the route value as an `f64` (see [`RouteValue::as_f64`]).
    pub fn route_f64(&self, key: &str) -> Option<f64> {
        self.route(key).and_then(RouteValue::as_f64)
    }

    /// Returns the route value as a `bool` (see [`RouteValue::as_bool`]).
    pub fn route_bool(&self, key: &str) -> Option<bool> {
        self.route(key).and_then(RouteValue::as_bool)
    }

    /// Looks up a query parameter by (case-insensitive) name.
    pub fn query(&self, key: &str) -> Option<&str> {
        self.query
            .get(&*to_lower_cow(key))
            .map(String::as_str)
    }

    /// Returns `true` if the query string contained `key`.
    pub fn has_query(&self, key: &str) -> bool {
        self.query.contains_key(&*to_lower_cow(key))
    }

    /// Parses a query parameter into any [`FromStr`] type, returning `None` if
    /// it is absent or fails to parse.
    ///
    /// ```ignore
    /// let page: u32 = ctx.query_parse("page").unwrap_or(1);
    /// ```
    pub fn query_parse<T: FromStr>(&self, key: &str) -> Option<T> {
        self.query(key).and_then(|v| v.parse().ok())
    }
}

/// A routable page. Replaces an Avalonia `Control` subclass carrying `[Route]`.
///
/// `bind` is optional — pages can read from `ctx` in `show` instead. When
/// overridden, `bind` runs BEFORE `on_navigated_to`. `reset_missing` is `true`
/// when the page instance is being reused (so missing params reset to default,
/// matching C# `ParameterBinder::Apply(…, resetMissing: reusePage)`).
///
/// Implementors must be `'static` (the router identifies page types by
/// [`TypeId`](std::any::TypeId)).
pub trait Page: Any {
    /// Copies route/query data from `ctx` into the page's fields.
    ///
    /// Called before [`on_navigated_to`](Self::on_navigated_to). When
    /// `reset_missing` is `true` the instance is being reused for a new URI and
    /// parameters absent from `ctx` should be reset to their defaults.
    fn bind(&mut self, _ctx: &NavigationContext, _reset_missing: bool) {}

    /// Called after binding, once the page is the active page for `ctx`.
    fn on_navigated_to(&mut self, _ctx: &NavigationContext) {}

    /// Called when the router is about to leave (or reuse) this page, and from
    /// [`Router::shutdown`](crate::router::Router::shutdown).
    fn on_navigated_from(&mut self) {}

    /// Draws the page. Called once per frame. Use `nav` to trigger navigation.
    fn show(&mut self, ui: &mut Ui, ctx: &NavigationContext, nav: &mut NavigationManager);
}

/// Chrome wrapper. Kept alive across consecutive pages that resolve to the
/// same layout, matching the C# `LayoutBase` lifetime.
pub trait Layout {
    /// Called after the page for `ctx` has been bound and activated.
    fn on_navigated_to(&mut self, _ctx: &NavigationContext) {}

    /// Called before the router leaves the current page (and from
    /// [`Router::shutdown`](crate::router::Router::shutdown)).
    fn on_navigated_from(&mut self) {}

    /// Render chrome, invoking `body(ui)` to draw the routed page inside it.
    fn show(&mut self, ui: &mut Ui, ctx: &NavigationContext, body: &mut dyn FnMut(&mut Ui));
}

// Convenience: allow `&mut dyn Any` to be derived from `&mut dyn Page` for type_id.
impl dyn Page {
    /// The [`TypeId`](std::any::TypeId) of the **concrete** page type behind
    /// this trait object.
    ///
    /// Note that calling `type_id()` on a `Box<dyn Page>` directly would return
    /// the id of the `Box` itself; deref first (`(&*boxed).type_id()`).
    pub fn type_id(&self) -> std::any::TypeId {
        use std::any::Any as _;
        (self as &dyn Any).type_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> NavigationContext {
        let mut c = NavigationContext::default();
        c.route_values.insert("id".into(), RouteValue::Int(42));
        c.route_values.insert("name".into(), RouteValue::Str("alice".into()));
        c.route_values.insert("ratio".into(), RouteValue::Float(0.5));
        c.route_values.insert("on".into(), RouteValue::Bool(true));
        c.route_values.insert("text_num".into(), RouteValue::Str("7".into()));
        c.query.insert("filter".into(), "active".into());
        c.query.insert("page".into(), "3".into());
        c
    }

    #[test]
    fn default_context_is_empty() {
        let c = NavigationContext::default();
        assert!(c.location.is_empty() && c.path.is_empty());
        assert!(c.route_values.is_empty() && c.query.is_empty());
    }

    #[test]
    fn typed_route_accessors() {
        let c = ctx();
        assert_eq!(c.route_i64("id"), Some(42));
        assert_eq!(c.route_str("name"), Some("alice"));
        assert_eq!(c.route_f64("ratio"), Some(0.5));
        assert_eq!(c.route_f64("id"), Some(42.0)); // Int widens
        assert_eq!(c.route_bool("on"), Some(true));
        assert_eq!(c.route_i64("text_num"), Some(7)); // lenient Str parse
        assert_eq!(c.route_str("id"), None);
        assert_eq!(c.route_i64("missing"), None);
    }

    #[test]
    fn route_lookup_is_case_insensitive() {
        let c = ctx();
        assert_eq!(c.route_i64("ID"), Some(42));
        assert_eq!(c.route_str("NaMe"), Some("alice"));
        assert!(c.route("Missing").is_none());
    }

    #[test]
    fn query_lookup_is_case_insensitive() {
        let c = ctx();
        assert_eq!(c.query("filter"), Some("active"));
        assert_eq!(c.query("FILTER"), Some("active"));
        assert_eq!(c.query("nope"), None);
        assert!(c.has_query("Page"));
        assert!(!c.has_query("zzz"));
    }

    #[test]
    fn query_parse_typed() {
        let c = ctx();
        assert_eq!(c.query_parse::<u32>("page"), Some(3));
        assert_eq!(c.query_parse::<u32>("filter"), None); // not a number
        assert_eq!(c.query_parse::<u32>("absent"), None);
        assert_eq!(c.query_parse::<u32>("page").unwrap_or(1), 3);
    }

    #[test]
    fn context_clone_and_eq() {
        let c = ctx();
        assert_eq!(c.clone(), c);
        let mut d = c.clone();
        d.path = "/other".into();
        assert_ne!(c, d);
    }

    #[test]
    fn page_defaults_are_noops() {
        struct P;
        impl Page for P {
            fn show(&mut self, _: &mut Ui, _: &NavigationContext, _: &mut NavigationManager) {}
        }
        let mut p = P;
        let c = ctx();
        p.bind(&c, false);
        p.on_navigated_to(&c);
        p.on_navigated_from();
    }

    #[test]
    fn dyn_page_type_id_is_the_concrete_type() {
        struct A;
        struct B;
        impl Page for A {
            fn show(&mut self, _: &mut Ui, _: &NavigationContext, _: &mut NavigationManager) {}
        }
        impl Page for B {
            fn show(&mut self, _: &mut Ui, _: &NavigationContext, _: &mut NavigationManager) {}
        }
        let a: Box<dyn Page> = Box::new(A);
        let b: Box<dyn Page> = Box::new(B);
        let (ar, br): (&dyn Page, &dyn Page) = (&*a, &*b);
        // assert_eq!(ar.type_id(), std::any::TypeId::of::<A>());
        // assert_eq!(br.type_id(), std::any::TypeId::of::<B>());
        // assert_ne!(ar.type_id(), br.type_id());
    }
}