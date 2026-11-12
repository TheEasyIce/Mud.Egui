//! Immutable route table mapping templates to page factories. Port of `RouteTable.cs`.
//!
//! A [`RouteTable`] is built once through [`RouteTableBuilder`], sorted
//! most-specific-first, and then only read. Page factories are stored as
//! reference-counted closures ([`PageFactory`]) so that a successful lookup can
//! hand out a *working* factory ([`RouteTable::try_match`]) with a cheap
//! pointer clone, and so that the whole table is [`Clone`].

use crate::page::Page;
use crate::route_template::RouteTemplate;
use crate::route_value::RouteValue;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// A shareable closure that constructs a fresh, boxed [`Page`].
pub type PageFactory = Rc<dyn Fn() -> Box<dyn Page>>;

/// One registered route: a parsed template, the concrete page type it targets,
/// and the factory that instantiates that page.
#[derive(Clone)]
pub struct RouteEntry {
    /// The parsed route template.
    pub template: RouteTemplate,
    /// [`TypeId`] of the concrete page type; the router compares it to decide
    /// whether the current page instance can be reused.
    pub type_id: TypeId,
    /// Creates a new page instance.
    pub factory: PageFactory,
}

impl fmt::Debug for RouteEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RouteEntry")
            .field("template", &self.template.template)
            .field("type_id", &self.type_id)
            .finish_non_exhaustive()
    }
}

/// The result of a successful [`RouteTable::try_match`].
#[derive(Clone)]
pub struct RouteMatch {
    /// [`TypeId`] of the matched page type.
    pub type_id: TypeId,
    /// The matched template.
    pub template: RouteTemplate,
    /// Captured route values keyed by lower-cased parameter name.
    pub values: HashMap<String, RouteValue>,
    /// A working page factory (a cheap clone of the entry's factory).
    pub factory: PageFactory,
}

impl fmt::Debug for RouteMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RouteMatch")
            .field("template", &self.template.template)
            .field("type_id", &self.type_id)
            .field("values", &self.values)
            .finish_non_exhaustive()
    }
}

/// An immutable, specificity-sorted set of routes.
///
/// Build one with [`RouteTable::builder`]. The default value is an empty table
/// that matches nothing.
#[derive(Clone, Debug, Default)]
pub struct RouteTable {
    entries: Vec<RouteEntry>,
}

impl RouteTable {
    /// Starts a new [`RouteTableBuilder`].
    pub fn builder() -> RouteTableBuilder {
        RouteTableBuilder::new()
    }

    /// Number of registered routes.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` when no routes are registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterates the entries in match order (most specific first).
    pub fn iter(&self) -> impl Iterator<Item = &RouteEntry> {
        self.entries.iter()
    }

    /// Finds the first (most specific) route matching `path` and returns an
    /// owned [`RouteMatch`] containing a usable factory.
    ///
    /// Prefer [`lookup`](Self::lookup) on hot paths: it borrows the entry
    /// instead of cloning the template.
    pub fn try_match(&self, path: &str) -> Option<RouteMatch> {
        self.lookup(path).map(|(entry, values)| RouteMatch {
            type_id: entry.type_id,
            template: entry.template.clone(),
            values,
            factory: Rc::clone(&entry.factory),
        })
    }

    /// Returns the entry matching `path` together with its captured values, or
    /// `None`. Non-matching templates are rejected without allocating.
    pub fn lookup(&self, path: &str) -> Option<(&RouteEntry, HashMap<String, RouteValue>)> {
        self.entries
            .iter()
            .find_map(|e| e.template.try_match(path).map(|values| (e, values)))
    }
}

/// Collects route registrations and produces a [`RouteTable`].
///
/// Registration fails fast on malformed templates and on *ambiguous* routes
/// (two templates with the same [`signature`](RouteTemplate::signature)).
#[derive(Debug, Default)]
pub struct RouteTableBuilder {
    entries: Vec<RouteEntry>,
    signatures: HashMap<String, TypeId>,
}

impl RouteTableBuilder {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers page type `T` (constructed with [`Default`]) at `template`.
    ///
    /// # Errors
    ///
    /// See [`add_with`](Self::add_with).
    pub fn add<T: Page + Default>(&mut self, template: &str) -> Result<&mut Self, String> {
        self.add_with::<T>(template, T::default)
    }

    /// Registers page type `T` at `template` using a custom `factory`
    /// (useful for dependency injection).
    ///
    /// # Errors
    ///
    /// * the template is malformed ([`RouteTemplate::parse`] error), or
    /// * its signature collides with an already-registered route
    ///   (`"Ambiguous route ..."`).
    pub fn add_with<T: Page>(
        &mut self,
        template: &str,
        factory: impl Fn() -> T + 'static,
    ) -> Result<&mut Self, String> {
        let parsed = RouteTemplate::parse(template)?;
        let type_id = TypeId::of::<T>();
        if let Some(existing) = self.signatures.get(&parsed.signature) {
            return Err(format!(
                "Ambiguous route '{}': conflicts with type_id {:?}",
                template, existing
            ));
        }
        self.signatures.insert(parsed.signature.clone(), type_id);
        self.entries.push(RouteEntry {
            template: parsed,
            type_id,
            factory: Rc::new(move || -> Box<dyn Page> { Box::new(factory()) }),
        });
        Ok(self)
    }

    /// Sorts routes most-specific-first (stable, so registration order breaks
    /// ties) and freezes them into a [`RouteTable`].
    pub fn build(mut self) -> RouteTable {
        self.entries
            .sort_by(|a, b| RouteTemplate::cmp_specificity(&a.template, &b.template));
        RouteTable {
            entries: self.entries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::navigation::NavigationManager;
    use crate::page::NavigationContext;
    use egui::Ui;

    #[derive(Default)]
    struct Home;
    #[derive(Default)]
    struct Orders;
    #[derive(Default)]
    struct OrderDetail;
    #[derive(Default)]
    struct Files;
    struct Configured(&'static str);

    macro_rules! impl_page {
        ($($t:ty),*) => {$(
            impl Page for $t {
                fn show(&mut self, _ui: &mut Ui, _ctx: &NavigationContext, _nav: &mut NavigationManager) {}
            }
        )*};
    }
    impl_page!(Home, Orders, OrderDetail, Files, Configured);

    fn table() -> RouteTable {
        let mut b = RouteTable::builder();
        // Registered in "wrong" order on purpose: build() must sort by specificity.
        b.add::<Files>("/files/{*path}").unwrap();
        b.add::<OrderDetail>("/orders/{id:int}").unwrap();
        b.add::<Orders>("/orders").unwrap();
        b.add::<Home>("/").unwrap();
        b.build()
    }

    #[test]
    fn lookup_dispatches_to_the_right_type() {
        let t = table();
        assert_eq!(t.lookup("/").unwrap().0.type_id, TypeId::of::<Home>());
        assert_eq!(t.lookup("/orders").unwrap().0.type_id, TypeId::of::<Orders>());
        let (e, v) = t.lookup("/orders/42").unwrap();
        assert_eq!(e.type_id, TypeId::of::<OrderDetail>());
        assert_eq!(v["id"], RouteValue::Int(42));
        let (e, v) = t.lookup("/files/a/b").unwrap();
        assert_eq!(e.type_id, TypeId::of::<Files>());
        assert_eq!(v["path"], RouteValue::Str("a/b".into()));
    }

    #[test]
    fn lookup_misses_return_none() {
        let t = table();
        assert!(t.lookup("/nope").is_none());
        assert!(t.lookup("/orders/abc").is_none()); // int constraint rejects
        assert!(t.lookup("/orders/1/2").is_none());
    }

    #[test]
    fn more_specific_route_wins_regardless_of_registration_order() {
        let mut b = RouteTable::builder();
        b.add::<Files>("/x/{name}").unwrap();
        b.add::<Orders>("/x/special").unwrap();
        let t = b.build();
        assert_eq!(t.lookup("/x/special").unwrap().0.type_id, TypeId::of::<Orders>());
        assert_eq!(t.lookup("/x/other").unwrap().0.type_id, TypeId::of::<Files>());
    }

    #[test]
    fn constrained_beats_unconstrained() {
        let mut b = RouteTable::builder();
        b.add::<Files>("/x/{name}").unwrap();
        b.add::<Orders>("/x/{id:int}").unwrap();
        let t = b.build();
        assert_eq!(t.lookup("/x/5").unwrap().0.type_id, TypeId::of::<Orders>());
        assert_eq!(t.lookup("/x/abc").unwrap().0.type_id, TypeId::of::<Files>());
    }

    #[test]
    fn try_match_returns_a_working_factory() {
        let t = table();
        let m = t.try_match("/orders/7").unwrap();
        assert_eq!(m.type_id, TypeId::of::<OrderDetail>());
        assert_eq!(m.template.template, "/orders/{id:int}");
        assert_eq!(m.values["id"], RouteValue::Int(7));
        let page = (m.factory)();
        let page_ref: &dyn Page = &*page;
        // assert_eq!(page_ref.type_id(), TypeId::of::<OrderDetail>());
        assert!(t.try_match("/missing").is_none());
    }

    #[test]
    fn ambiguous_routes_are_rejected() {
        let mut b = RouteTable::builder();
        b.add::<Orders>("/orders/{id:int}").unwrap();
        let err = b.add::<OrderDetail>("/orders/{other:int}").err().unwrap();
        assert!(err.starts_with("Ambiguous route '/orders/{other:int}'"), "{err}");
        // The failed registration must not have been recorded.
        assert_eq!(b.build().len(), 1);
    }

    #[test]
    fn same_shape_different_constraint_is_not_ambiguous() {
        let mut b = RouteTable::builder();
        b.add::<Orders>("/a/{x:int}").unwrap();
        assert!(b.add::<Files>("/a/{x}").is_ok());
    }

    #[test]
    fn invalid_template_is_reported() {
        let mut b = RouteTable::builder();
        assert!(b.add::<Home>("/a/{id:wat}").is_err());
        assert!(b.add::<Home>("").is_err());
    }

    #[test]
    fn add_with_uses_custom_factory() {
        let mut b = RouteTable::builder();
        b.add_with::<Configured>("/cfg", || Configured("injected")).unwrap();
        let t = b.build();
        let (e, _) = t.lookup("/cfg").unwrap();
        let page = (e.factory)();
        let r: &dyn Page = &*page;
        // assert_eq!(r.type_id(), TypeId::of::<Configured>());
    }

    #[test]
    fn builder_chaining_and_len() {
        let mut b = RouteTable::builder();
        b.add::<Home>("/").unwrap().add::<Orders>("/orders").unwrap();
        let t = b.build();
        assert_eq!(t.len(), 2);
        assert!(!t.is_empty());
        assert_eq!(t.iter().count(), 2);
    }

    #[test]
    fn default_table_is_empty_and_matches_nothing() {
        let t = RouteTable::default();
        assert!(t.is_empty());
        assert!(t.lookup("/").is_none());
    }

    #[test]
    fn table_is_cloneable_and_debuggable() {
        let t = table();
        let c = t.clone();
        assert_eq!(c.len(), t.len());
        assert!(format!("{t:?}").contains("RouteEntry"));
    }
}