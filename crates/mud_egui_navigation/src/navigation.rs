//! In-memory URL history. Port of `NavigationManager.cs` (subscriber events
//! dropped — the Router polls instead; see module docs).
//!
//! [`NavigationManager`] owns a bounded back/forward history of absolute URIs
//! under a fixed base URI, and lets callers register *veto* handlers that may
//! cancel any navigation before it is committed. It performs no rendering: the
//! [`Router`](crate::router::Router) simply compares
//! [`NavigationManager::current_uri`] with the URI it last rendered once per
//! frame.
//!
//! # Allocation notes
//!
//! * URI resolution returns a [`Cow`], so re-navigating to the current URI
//!   (a no-op) and absolute in-base URIs never allocate for the comparison.
//! * `back`/`forward` pass the history entry to veto handlers by reference;
//!   the target is only copied when at least one handler is registered.
//! * [`NavigationManager::uri_with_query`] rewrites the query in a single pass.

use crate::query;
use std::borrow::Cow;
use std::fmt;

/// Default upper bound for the number of history entries kept by
/// [`NavigationManager::default`].
pub const DEFAULT_MAX_HISTORY: usize = 100;

/// How a navigation modifies the history stack.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NavigationKind {
    /// A new entry is pushed (and any forward entries are discarded).
    #[default]
    Push,
    /// The current entry is replaced in place.
    Replace,
    /// The cursor moves one entry back.
    Back,
    /// The cursor moves one entry forward.
    Forward,
}

impl NavigationKind {
    /// The variant name as a static string (`"Push"`, `"Replace"`, `"Back"`,
    /// `"Forward"`), matching the C# enum names.
    pub fn as_str(self) -> &'static str {
        match self {
            NavigationKind::Push => "Push",
            NavigationKind::Replace => "Replace",
            NavigationKind::Back => "Back",
            NavigationKind::Forward => "Forward",
        }
    }
}

impl fmt::Display for NavigationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Veto context. Handlers mutate `is_prevented` via `prevent`.
///
/// A fresh context is created for every navigation that has at least one
/// registered handler; all handlers (up to the first one that prevents) see
/// the same instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocationChangingContext {
    /// The absolute URI the navigation is heading to.
    pub target_location: String,
    /// Whether this is a push, replace, back or forward navigation.
    pub kind: NavigationKind,
    is_prevented: bool,
}

impl LocationChangingContext {
    pub(crate) fn new(target: String, kind: NavigationKind) -> Self {
        Self {
            target_location: target,
            kind,
            is_prevented: false,
        }
    }

    /// Cancels the navigation. Remaining handlers are skipped and the history
    /// is left untouched.
    pub fn prevent(&mut self) {
        self.is_prevented = true;
    }

    /// Whether a handler has called [`prevent`](Self::prevent).
    pub fn is_prevented(&self) -> bool {
        self.is_prevented
    }
}

/// Reasons a navigation request can be rejected outright (as opposed to being
/// vetoed, which is reported as `Ok(false)`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavigateError {
    /// The URI was empty or whitespace-only.
    Empty,
    /// The URI could not be interpreted; carries the offending text.
    Malformed(String),
    /// An absolute URI that does not start with the manager's base URI;
    /// carries the offending text.
    OutsideBase(String),
    /// `navigate_to` was invoked while veto handlers were running.
    ReentrantCall,
}

impl fmt::Display for NavigateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NavigateError::Empty => write!(f, "Empty URI"),
            NavigateError::Malformed(s) => write!(f, "Malformed URI: {}", s),
            NavigateError::OutsideBase(s) => write!(f, "URI outside base: {}", s),
            NavigateError::ReentrantCall => {
                write!(f, "navigate_to called from within a changing handler")
            }
        }
    }
}
impl std::error::Error for NavigateError {}

/// Token returned by [`NavigationManager::register_veto`]; pass it to
/// [`NavigationManager::unregister_veto`] to remove the handler.
#[derive(Debug, PartialEq, Eq)]
pub struct VetoHandle {
    pub(crate) id: u64,
}

type ChangingHandler = Box<dyn FnMut(&mut LocationChangingContext)>;

/// A bounded, in-memory browser-like history under a fixed base URI.
///
/// All stored entries are absolute URIs that start with [`base_uri`](Self::base_uri).
/// The manager is intentionally not `Clone` (it owns boxed closures).
pub struct NavigationManager {
    base_uri: String,
    history: Vec<String>,
    index: usize,
    max_history: usize,
    changing_handlers: Vec<(u64, ChangingHandler)>,
    next_handler_id: u64,
    is_running_handlers: bool,
}

impl Default for NavigationManager {
    /// Base `/`, initial location `/`, [`DEFAULT_MAX_HISTORY`] entries.
    fn default() -> Self {
        Self::new("/", "/", DEFAULT_MAX_HISTORY)
    }
}

impl fmt::Debug for NavigationManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NavigationManager")
            .field("base_uri", &self.base_uri)
            .field("history", &self.history)
            .field("index", &self.index)
            .field("max_history", &self.max_history)
            .field("veto_handlers", &self.changing_handlers.len())
            .finish()
    }
}

impl NavigationManager {
    /// Creates a manager positioned at `initial_uri`.
    ///
    /// * `base_uri` gets a trailing `/` appended if missing.
    /// * `initial_uri` may be base-relative (`"/home"`, `"home"`) or absolute
    ///   under the base; if it cannot be resolved the base URI itself is used.
    /// * `max_history` is clamped to at least 1; the oldest entries are dropped
    ///   once it is exceeded.
    pub fn new(initial_uri: &str, base_uri: &str, max_history: usize) -> Self {
        let base = if base_uri.ends_with('/') {
            base_uri.to_string()
        } else {
            format!("{}/", base_uri)
        };
        let mut mgr = Self {
            base_uri: base,
            history: Vec::new(),
            index: 0,
            max_history: max_history.max(1),
            changing_handlers: Vec::new(),
            next_handler_id: 0,
            is_running_handlers: false,
        };
        let resolved = mgr
            .resolve_within_base(initial_uri)
            .map(Cow::into_owned)
            .unwrap_or_else(|_| mgr.base_uri.clone());
        mgr.history.push(resolved);
        mgr
    }

    /// The normalised base URI (always ends with `/`).
    pub fn base_uri(&self) -> &str {
        &self.base_uri
    }

    /// The absolute URI at the history cursor.
    pub fn current_uri(&self) -> &str {
        self.history
            .get(self.index)
            .map(String::as_str)
            .unwrap_or(self.base_uri.as_str())
    }

    /// The current URI relative to the base, without allocating: no leading
    /// slash, query and fragment included (e.g. `orders/42?x=1`).
    pub fn current_relative(&self) -> &str {
        self.current_uri()
            .strip_prefix(self.base_uri.as_str())
            .unwrap_or("")
    }

    /// Whether [`back`](Self::back) would move the cursor.
    pub fn can_go_back(&self) -> bool {
        self.index > 0
    }

    /// Whether [`forward`](Self::forward) would move the cursor.
    pub fn can_go_forward(&self) -> bool {
        self.index + 1 < self.history.len()
    }

    /// Number of entries currently stored.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Zero-based position of the cursor within the history.
    pub fn history_index(&self) -> usize {
        self.index
    }

    /// All stored entries, oldest first.
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Maximum number of retained entries.
    pub fn max_history(&self) -> usize {
        self.max_history
    }

    /// Number of currently registered veto handlers.
    pub fn veto_count(&self) -> usize {
        self.changing_handlers.len()
    }

    /// Navigates to `uri`, pushing a new history entry or (when `replace`) replacing
    /// the current one.
    ///
    /// `uri` may be base-relative (`"/orders/1"`, `"orders/1"`) or absolute under
    /// the base URI. Navigating to the URI that is already current is a no-op that
    /// returns `Ok(true)` without invoking veto handlers.
    ///
    /// Returns `Ok(true)` if the location is (now) the target, `Ok(false)` if a
    /// veto handler prevented the navigation.
    ///
    /// # Errors
    ///
    /// * [`NavigateError::ReentrantCall`] when called while handlers are running,
    /// * [`NavigateError::Empty`] for an empty/whitespace URI,
    /// * [`NavigateError::OutsideBase`] for an absolute URI outside the base.
    pub fn navigate_to(&mut self, uri: &str, replace: bool) -> Result<bool, NavigateError> {
        if self.is_running_handlers {
            return Err(NavigateError::ReentrantCall);
        }
        let resolved = self.resolve_within_base(uri)?;
        if resolved == self.current_uri() {
            return Ok(true);
        }
        let target = resolved.into_owned();
        let kind = if replace {
            NavigationKind::Replace
        } else {
            NavigationKind::Push
        };
        if !run_handlers(
            &mut self.changing_handlers,
            &mut self.is_running_handlers,
            &target,
            kind,
        ) {
            return Ok(false);
        }
        if replace {
            if let Some(slot) = self.history.get_mut(self.index) {
                *slot = target;
            }
        } else {
            self.push_entry(target);
        }
        Ok(true)
    }

    /// Moves one entry back. Returns `Ok(false)` when there is nothing to go
    /// back to, when handlers are currently running, or when a handler vetoes.
    pub fn back(&mut self) -> Result<bool, NavigateError> {
        if !self.can_go_back() || self.is_running_handlers {
            return Ok(false);
        }
        Ok(self.move_to(self.index - 1, NavigationKind::Back))
    }

    /// Moves one entry forward. Returns `Ok(false)` when there is nothing to
    /// go forward to, when handlers are currently running, or when a handler
    /// vetoes.
    pub fn forward(&mut self) -> Result<bool, NavigateError> {
        if !self.can_go_forward() || self.is_running_handlers {
            return Ok(false);
        }
        Ok(self.move_to(self.index + 1, NavigationKind::Forward))
    }

    /// Register a handler that runs before every navigation. Use the returned
    /// handle with `unregister_veto` when done (Rust has no `IDisposable`
    /// auto-cleanup; manual is the idiomatic alternative).
    ///
    /// Handlers run in registration order and stop at the first one that calls
    /// [`LocationChangingContext::prevent`].
    pub fn register_veto(
        &mut self,
        f: impl FnMut(&mut LocationChangingContext) + 'static,
    ) -> VetoHandle {
        let id = self.next_handler_id;
        self.next_handler_id += 1;
        self.changing_handlers.push((id, Box::new(f)));
        VetoHandle { id }
    }

    /// Removes the handler identified by `handle`. Unknown handles are ignored.
    pub fn unregister_veto(&mut self, handle: VetoHandle) {
        self.changing_handlers.retain(|(id, _)| *id != handle.id);
    }

    /// Converts a relative URI to an absolute one under `base_uri`.
    /// Falls back to the base URI itself when `relative` cannot be resolved.
    pub fn to_absolute(&self, relative: &str) -> String {
        self.resolve_within_base(relative)
            .map(Cow::into_owned)
            .unwrap_or_else(|_| self.base_uri.clone())
    }

    /// Converts an absolute URI to a base-relative path (no leading slash,
    /// matching the C# contract). Returns empty on foreign URIs.
    pub fn to_base_relative(&self, absolute: &str) -> String {
        match self.resolve_within_base(absolute) {
            Ok(resolved) => resolved
                .strip_prefix(self.base_uri.as_str())
                .unwrap_or("")
                .to_string(),
            Err(_) => String::new(),
        }
    }

    /// Returns the current URI with the given query parameter set (or removed
    /// if `None`). Path and fragment are preserved; the name is matched
    /// case-insensitively. Does not navigate.
    pub fn uri_with_query(&self, name: &str, value: Option<&str>) -> String {
        let (path, query_text, frag) = query::split_uri(self.current_uri());
        format!(
            "{}{}{}",
            path,
            query::with_parameter(query_text, name, value),
            frag
        )
    }

    fn move_to(&mut self, new_index: usize, kind: NavigationKind) -> bool {
        let Some(target) = self.history.get(new_index) else {
            return false;
        };
        // Disjoint field borrows: `target` borrows `history`, handlers borrow the rest.
        if !run_handlers(
            &mut self.changing_handlers,
            &mut self.is_running_handlers,
            target,
            kind,
        ) {
            return false;
        }
        self.index = new_index;
        true
    }

    fn push_entry(&mut self, target: String) {
        self.history.truncate(self.index + 1);
        self.history.push(target);
        self.index = self.history.len() - 1;
        if self.history.len() > self.max_history {
            let excess = self.history.len() - self.max_history;
            self.history.drain(..excess);
            self.index = self.index.saturating_sub(excess);
        }
    }

    fn resolve_within_base<'a>(&self, uri: &'a str) -> Result<Cow<'a, str>, NavigateError> {
        let uri = uri.trim();
        if uri.is_empty() {
            return Err(NavigateError::Empty);
        }
        if let Some(rest) = uri.strip_prefix('/') {
            return Ok(Cow::Owned(format!("{}{}", self.base_uri, rest)));
        }
        if has_scheme(uri) {
            if uri.starts_with(self.base_uri.as_str()) {
                return Ok(Cow::Borrowed(uri));
            }
            return Err(NavigateError::OutsideBase(uri.to_string()));
        }
        Ok(Cow::Owned(format!("{}{}", self.base_uri, uri)))
    }
}

/// Runs veto handlers in order against a fresh context. Returns `true` when the
/// navigation may proceed. The `running` flag is cleared by an RAII guard so it
/// cannot get stuck if a handler panics.
fn run_handlers(
    handlers: &mut [(u64, ChangingHandler)],
    running: &mut bool,
    target: &str,
    kind: NavigationKind,
) -> bool {
    if handlers.is_empty() {
        return true;
    }

    struct ClearOnDrop<'a>(&'a mut bool);
    impl Drop for ClearOnDrop<'_> {
        fn drop(&mut self) {
            *self.0 = false;
        }
    }

    *running = true;
    let _guard = ClearOnDrop(running);
    let mut ctx = LocationChangingContext::new(target.to_string(), kind);
    for (_, handler) in handlers.iter_mut() {
        handler(&mut ctx);
        if ctx.is_prevented() {
            break;
        }
    }
    !ctx.is_prevented()
}

/// `true` when `uri` starts with a URI scheme followed by `://`
/// (`https://…`, `app://…`). A `://` that appears later — e.g. inside a query
/// value like `page?next=http://x` — does **not** make the URI absolute.
fn has_scheme(uri: &str) -> bool {
    match uri.find("://") {
        Some(i) if i > 0 => uri[..i]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.')),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::rc::Rc;

    const ROOT: &str = "https://app.test/";

    fn mgr() -> NavigationManager {
        NavigationManager::new("/", "https://app.test/", 10)
    }

    fn url(rel: &str) -> String {
        format!("{ROOT}{rel}")
    }

    // ── construction ──

    #[test]
    fn initial_state() {
        let m = mgr();
        assert_eq!(m.current_uri(), ROOT);
        assert_eq!(m.base_uri(), ROOT);
        assert_eq!(m.history_len(), 1);
        assert_eq!(m.history_index(), 0);
        assert!(!m.can_go_back());
        assert!(!m.can_go_forward());
        assert_eq!(m.current_relative(), "");
    }

    #[test]
    fn base_uri_gets_trailing_slash() {
        let m = NavigationManager::new("/x", "https://app.test", 5);
        assert_eq!(m.base_uri(), ROOT);
        assert_eq!(m.current_uri(), url("x"));
    }

    #[test]
    fn unresolvable_initial_uri_falls_back_to_base() {
        assert_eq!(NavigationManager::new("", ROOT, 5).current_uri(), ROOT);
        assert_eq!(
            NavigationManager::new("https://other.test/x", ROOT, 5).current_uri(),
            ROOT
        );
    }

    #[test]
    fn default_manager() {
        let m = NavigationManager::default();
        assert_eq!(m.base_uri(), "/");
        assert_eq!(m.current_uri(), "/");
        assert_eq!(m.max_history(), DEFAULT_MAX_HISTORY);
    }

    #[test]
    fn max_history_is_clamped_to_one() {
        let mut m = NavigationManager::new("/", ROOT, 0);
        assert_eq!(m.max_history(), 1);
        m.navigate_to("/a", false).unwrap();
        assert_eq!(m.history_len(), 1);
        assert_eq!(m.current_uri(), url("a"));
        assert!(!m.can_go_back());
    }

    // ── push / replace / back / forward ──

    #[test]
    fn push_back_forward() {
        let mut m = mgr();
        assert_eq!(m.navigate_to("/a", false).unwrap(), true);
        assert_eq!(m.navigate_to("/b", false).unwrap(), true);
        assert_eq!(m.current_uri(), url("b"));
        assert!(m.can_go_back() && !m.can_go_forward());

        assert_eq!(m.back().unwrap(), true);
        assert_eq!(m.current_uri(), url("a"));
        assert!(m.can_go_back() && m.can_go_forward());

        assert_eq!(m.forward().unwrap(), true);
        assert_eq!(m.current_uri(), url("b"));
    }

    #[test]
    fn back_and_forward_at_the_edges_return_false() {
        let mut m = mgr();
        assert_eq!(m.back().unwrap(), false);
        assert_eq!(m.forward().unwrap(), false);
        assert_eq!(m.history_len(), 1);
    }

    #[test]
    fn replace_overwrites_current_entry() {
        let mut m = mgr();
        m.navigate_to("/a", false).unwrap();
        m.navigate_to("/b", true).unwrap();
        assert_eq!(m.history_len(), 2);
        assert_eq!(m.current_uri(), url("b"));
        m.back().unwrap();
        assert_eq!(m.current_uri(), ROOT);
    }

    #[test]
    fn pushing_after_back_discards_forward_entries() {
        let mut m = mgr();
        m.navigate_to("/a", false).unwrap();
        m.navigate_to("/b", false).unwrap();
        m.back().unwrap();
        m.navigate_to("/c", false).unwrap();
        assert!(!m.can_go_forward());
        assert_eq!(m.history(), &[ROOT.to_string(), url("a"), url("c")]);
    }

    #[test]
    fn navigating_to_the_current_uri_is_a_noop() {
        let mut m = mgr();
        m.navigate_to("/a", false).unwrap();
        assert_eq!(m.navigate_to("/a", false).unwrap(), true);
        assert_eq!(m.navigate_to(&url("a"), false).unwrap(), true);
        assert_eq!(m.history_len(), 2);
    }

    #[test]
    fn history_is_bounded_and_cursor_stays_on_the_newest() {
        let mut m = NavigationManager::new("/", ROOT, 3);
        for p in ["/a", "/b", "/c", "/d"] {
            m.navigate_to(p, false).unwrap();
        }
        assert_eq!(m.history(), &[url("b"), url("c"), url("d")]);
        assert_eq!(m.history_index(), 2);
        assert_eq!(m.current_uri(), url("d"));
        m.back().unwrap();
        m.back().unwrap();
        assert_eq!(m.current_uri(), url("b"));
        assert!(!m.can_go_back());
    }

    // ── URI resolution / edge cases ──

    #[test]
    fn relative_and_absolute_inputs_resolve_under_base() {
        let mut m = mgr();
        m.navigate_to("c/d", false).unwrap();
        assert_eq!(m.current_uri(), url("c/d"));
        m.navigate_to("/e", false).unwrap();
        assert_eq!(m.current_uri(), url("e"));
        m.navigate_to(&url("f"), false).unwrap();
        assert_eq!(m.current_uri(), url("f"));
        m.navigate_to("  /g  ", false).unwrap();
        assert_eq!(m.current_uri(), url("g"));
    }

    #[test]
    fn invalid_targets_are_errors_and_leave_history_alone() {
        let mut m = mgr();
        assert_eq!(m.navigate_to("", false), Err(NavigateError::Empty));
        assert_eq!(m.navigate_to("   ", false), Err(NavigateError::Empty));
        assert_eq!(
            m.navigate_to("https://evil.test/x", false),
            Err(NavigateError::OutsideBase("https://evil.test/x".into()))
        );
        assert_eq!(m.history_len(), 1);
    }

    #[test]
    fn query_value_containing_a_url_is_not_mistaken_for_an_absolute_uri() {
        let mut m = mgr();
        m.navigate_to("login?next=http://x.test/", false).unwrap();
        assert_eq!(m.current_uri(), url("login?next=http://x.test/"));
        m.navigate_to("/r?u=https://y.test/", false).unwrap();
        assert_eq!(m.current_uri(), url("r?u=https://y.test/"));
    }

    #[test]
    fn has_scheme_cases() {
        assert!(has_scheme("https://a"));
        assert!(has_scheme("app+x-y.z://a"));
        assert!(!has_scheme("://a"));
        assert!(!has_scheme("a/b://c"));
        assert!(!has_scheme("page?u=http://x"));
        assert!(!has_scheme("plain"));
    }

    #[test]
    fn to_absolute_and_to_base_relative() {
        let m = mgr();
        assert_eq!(m.to_absolute("/a/b"), url("a/b"));
        assert_eq!(m.to_absolute("a/b"), url("a/b"));
        assert_eq!(m.to_absolute(""), ROOT);
        assert_eq!(m.to_absolute("https://evil.test/"), ROOT);

        assert_eq!(m.to_base_relative(&url("a/b?x=1")), "a/b?x=1");
        assert_eq!(m.to_base_relative("/a/b"), "a/b");
        assert_eq!(m.to_base_relative(ROOT), "");
        assert_eq!(m.to_base_relative("https://evil.test/a"), "");
        assert_eq!(m.to_base_relative(""), "");
    }

    #[test]
    fn current_relative_includes_query_and_fragment() {
        let mut m = mgr();
        m.navigate_to("/orders/4?x=1#top", false).unwrap();
        assert_eq!(m.current_relative(), "orders/4?x=1#top");
    }

    // ── query manipulation ──

    #[test]
    fn uri_with_query_sets_replaces_and_removes() {
        let mut m = mgr();
        m.navigate_to("/list?a=1#top", false).unwrap();
        assert_eq!(m.uri_with_query("page", Some("2")), url("list?a=1&page=2#top"));
        // The replaced pair adopts the spelling passed by the caller.
        assert_eq!(m.uri_with_query("A", Some("9")), url("list?A=9#top"));
        assert_eq!(m.uri_with_query("a", None), url("list#top"));
        assert_eq!(m.uri_with_query("missing", None), url("list?a=1#top"));
        // It must not navigate.
        assert_eq!(m.current_uri(), url("list?a=1#top"));
    }

    #[test]
    fn uri_with_query_encodes_values() {
        let m = mgr();
        assert_eq!(m.uri_with_query("q", Some("a b&c")), url("?q=a+b%26c"));
    }

    // ── veto handlers ──

    #[test]
    fn veto_blocks_push_and_replace() {
        let mut m = mgr();
        let h = m.register_veto(|c| c.prevent());
        assert_eq!(m.navigate_to("/a", false).unwrap(), false);
        assert_eq!(m.navigate_to("/a", true).unwrap(), false);
        assert_eq!(m.current_uri(), ROOT);
        assert_eq!(m.history_len(), 1);

        m.unregister_veto(h);
        assert_eq!(m.veto_count(), 0);
        assert_eq!(m.navigate_to("/a", false).unwrap(), true);
    }

    #[test]
    fn veto_blocks_back_and_forward() {
        let mut m = mgr();
        m.navigate_to("/a", false).unwrap();
        m.navigate_to("/b", false).unwrap();
        m.back().unwrap();

        let h = m.register_veto(|c| c.prevent());
        assert_eq!(m.back().unwrap(), false);
        assert_eq!(m.forward().unwrap(), false);
        assert_eq!(m.current_uri(), url("a"));
        m.unregister_veto(h);
        assert_eq!(m.forward().unwrap(), true);
        assert_eq!(m.current_uri(), url("b"));
    }

    #[test]
    fn veto_context_reports_target_and_kind() {
        let mut m = mgr();
        let seen: Rc<RefCell<Vec<(String, NavigationKind)>>> = Rc::default();
        let sink = Rc::clone(&seen);
        m.register_veto(move |c| sink.borrow_mut().push((c.target_location.clone(), c.kind)));

        m.navigate_to("/a", false).unwrap();
        m.navigate_to("/b", true).unwrap();
        m.back().unwrap();
        m.forward().unwrap();

        assert_eq!(
            *seen.borrow(),
            vec![
                (url("a"), NavigationKind::Push),
                (url("b"), NavigationKind::Replace),
                (ROOT.to_string(), NavigationKind::Back),
                (url("b"), NavigationKind::Forward),
            ]
        );
    }

    #[test]
    fn handlers_run_in_order_and_stop_at_first_veto() {
        let mut m = mgr();
        let order: Rc<RefCell<Vec<&'static str>>> = Rc::default();
        let (o1, o2, o3) = (order.clone(), order.clone(), order.clone());
        m.register_veto(move |_| o1.borrow_mut().push("first"));
        m.register_veto(move |c| {
            o2.borrow_mut().push("second");
            c.prevent();
        });
        m.register_veto(move |_| o3.borrow_mut().push("third"));

        assert_eq!(m.navigate_to("/a", false).unwrap(), false);
        assert_eq!(*order.borrow(), vec!["first", "second"]);
    }

    #[test]
    fn handlers_are_not_invoked_for_noop_navigation() {
        let mut m = mgr();
        let calls = Rc::new(Cell::new(0));
        let c = Rc::clone(&calls);
        m.register_veto(move |_| c.set(c.get() + 1));
        m.navigate_to("/", false).unwrap(); // already current
        assert_eq!(calls.get(), 0);
        m.navigate_to("/a", false).unwrap();
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn handlers_survive_and_unregister_only_removes_the_target() {
        let mut m = mgr();
        let calls = Rc::new(Cell::new(0));
        let (a, b) = (Rc::clone(&calls), Rc::clone(&calls));
        let h1 = m.register_veto(move |_| a.set(a.get() + 1));
        let _h2 = m.register_veto(move |_| b.set(b.get() + 10));
        m.navigate_to("/a", false).unwrap();
        assert_eq!(calls.get(), 11);
        assert_eq!(m.veto_count(), 2); // restored after running

        m.unregister_veto(h1);
        m.navigate_to("/b", false).unwrap();
        assert_eq!(calls.get(), 21);
        assert_eq!(m.veto_count(), 1);
    }

    #[test]
    fn handler_can_decide_based_on_target() {
        let mut m = mgr();
        m.register_veto(|c| {
            if c.target_location.ends_with("/admin") {
                c.prevent();
            }
        });
        assert_eq!(m.navigate_to("/admin", false).unwrap(), false);
        assert_eq!(m.navigate_to("/public", false).unwrap(), true);
    }

    // ── re-entrancy ──

    #[test]
    fn reentrant_navigate_is_rejected() {
        let mut m = mgr();
        // Safe Rust cannot call back into the manager from inside a handler
        // (it is mutably borrowed), so simulate the "handlers running" state.
        m.is_running_handlers = true;
        assert_eq!(m.navigate_to("/a", false), Err(NavigateError::ReentrantCall));
        assert_eq!(m.navigate_to("/a", true), Err(NavigateError::ReentrantCall));
        assert_eq!(m.history_len(), 1);
    }

    #[test]
    fn back_and_forward_are_inert_while_handlers_run() {
        let mut m = mgr();
        m.navigate_to("/a", false).unwrap();
        m.back().unwrap();
        m.is_running_handlers = true;
        assert_eq!(m.forward().unwrap(), false);
        m.is_running_handlers = false;
        m.forward().unwrap();
        m.is_running_handlers = true;
        assert_eq!(m.back().unwrap(), false);
    }

    #[test]
    fn running_flag_is_cleared_after_handlers_return() {
        let mut m = mgr();
        m.register_veto(|_| {});
        m.navigate_to("/a", false).unwrap();
        assert!(!m.is_running_handlers);
        assert_eq!(m.navigate_to("/b", false).unwrap(), true);
    }

    #[test]
    fn panicking_handler_does_not_wedge_the_manager() {
        let mut m = mgr();
        let h = m.register_veto(|_| panic!("boom"));
        let r = catch_unwind(AssertUnwindSafe(|| m.navigate_to("/a", false)));
        assert!(r.is_err());
        assert!(!m.is_running_handlers);
        assert_eq!(m.veto_count(), 1); // handlers were not lost
        assert_eq!(m.current_uri(), ROOT); // history untouched
        m.unregister_veto(h);
        assert_eq!(m.navigate_to("/a", false).unwrap(), true);
    }

    // ── small types ──

    #[test]
    fn navigation_kind_strings_and_default() {
        assert_eq!(NavigationKind::Push.as_str(), "Push");
        assert_eq!(NavigationKind::Replace.as_str(), "Replace");
        assert_eq!(NavigationKind::Back.as_str(), "Back");
        assert_eq!(NavigationKind::Forward.to_string(), "Forward");
        assert_eq!(NavigationKind::default(), NavigationKind::Push);
    }

    #[test]
    fn navigate_error_display() {
        assert_eq!(NavigateError::Empty.to_string(), "Empty URI");
        assert_eq!(NavigateError::Malformed("x".into()).to_string(), "Malformed URI: x");
        assert_eq!(NavigateError::OutsideBase("y".into()).to_string(), "URI outside base: y");
        assert_eq!(
            NavigateError::ReentrantCall.to_string(),
            "navigate_to called from within a changing handler"
        );
        let _: &dyn std::error::Error = &NavigateError::Empty;
    }

    #[test]
    fn changing_context_basics() {
        let mut c = LocationChangingContext::new("x".into(), NavigationKind::Push);
        assert!(!c.is_prevented());
        c.prevent();
        assert!(c.is_prevented());
        assert_eq!(c.clone(), c);
    }

    #[test]
    fn debug_output_mentions_state() {
        let m = mgr();
        let s = format!("{m:?}");
        assert!(s.contains("NavigationManager") && s.contains("veto_handlers"));
    }
}