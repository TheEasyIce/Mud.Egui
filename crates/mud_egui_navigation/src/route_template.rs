//! Parsed, immutable route template. Port of `RouteTemplate.cs`.
//!
//! Grammar matches the C# version: `/orders/{id:int}`, `{id?}`, `{id:int?}`,
//! `{*path}`. `guid` and `datetime` constraints accept any string of the right
//! *shape* (no `uuid`/`chrono` dependency in this port).
//!
//! # Matching without allocating
//!
//! Matching walks the request path lazily with a slice-based segment iterator
//! (`PathSegments`) and compares/validates each segment as a `&str`
//! (percent-decoding yields a borrowed [`Cow`] unless an escape is present).
//! A candidate template is first validated with **zero heap allocations**; the
//! captured values are only materialised (a second, identical walk) once the
//! template is known to match. Scanning a table of N templates therefore
//! allocates only for the one that wins.

use crate::query::percent_decode;
use crate::route_value::RouteValue;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;

/// A parsed route template such as `/orders/{id:int}/items/{*rest}`.
///
/// # Grammar
///
/// | Syntax            | Meaning                                              |
/// |-------------------|------------------------------------------------------|
/// | `orders`          | literal segment, matched ASCII-case-insensitively    |
/// | `{id}`            | required parameter, captured as [`RouteValue::Str`]  |
/// | `{id:int}`        | parameter with a type constraint                     |
/// | `{id?}`           | optional parameter (must not precede required ones)  |
/// | `{id:int?}`       | optional constrained parameter                       |
/// | `{*path}`         | catch-all; must be last; captures the `/`-joined tail |
///
/// Constraints: `int`, `long`, `double`, `float`, `decimal`, `bool`, `guid`,
/// `datetime`. Parameter names are case-insensitive and are stored
/// lower-cased in the captured map.
///
/// ```ignore
/// let t = RouteTemplate::parse("/orders/{id:int}")?;
/// let values = t.try_match("/orders/42").unwrap();
/// assert_eq!(values["id"], RouteValue::Int(42));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RouteTemplate {
    /// Canonical display form: leading `/`, normalised constraint names
    /// (e.g. `/orders/{id:int}`).
    pub template: String,
    /// Shape-only fingerprint used to detect ambiguous registrations. Literals
    /// are lower-cased and parameter names are erased, so `/a/{x:int}` and
    /// `/a/{y:int}` share a signature.
    pub signature: String,
    segments: Vec<Segment>,
}

impl Default for RouteTemplate {
    /// The root template `/`, which matches only an empty path.
    fn default() -> Self {
        Self {
            template: "/".to_string(),
            signature: String::new(),
            segments: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Segment {
    kind: SegmentKind,
    /// Literal text, or the parameter name as written.
    text: String,
    /// Lower-cased parameter name (the key in the captured map).
    key: String,
    constraint: Constraint,
    optional: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SegmentKind {
    Literal,
    Parameter,
    CatchAll,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
enum Constraint {
    #[default]
    None,
    Int,
    Long,
    Double,
    Float,
    Decimal,
    Bool,
    Guid,
    DateTime,
}

impl Constraint {
    const NAMED: [Constraint; 8] = [
        Constraint::Int,
        Constraint::Long,
        Constraint::Double,
        Constraint::Float,
        Constraint::Decimal,
        Constraint::Bool,
        Constraint::Guid,
        Constraint::DateTime,
    ];

    fn name(self) -> &'static str {
        match self {
            Constraint::None => "",
            Constraint::Int => "int",
            Constraint::Long => "long",
            Constraint::Double => "double",
            Constraint::Float => "float",
            Constraint::Decimal => "decimal",
            Constraint::Bool => "bool",
            Constraint::Guid => "guid",
            Constraint::DateTime => "datetime",
        }
    }

    /// Case-insensitive lookup; the empty string means "no constraint".
    fn from_name(text: &str) -> Option<Self> {
        if text.is_empty() {
            return Some(Constraint::None);
        }
        Self::NAMED
            .iter()
            .copied()
            .find(|c| c.name().eq_ignore_ascii_case(text))
    }

    /// Allocation-free validity check.
    fn accepts(self, s: &str) -> bool {
        match self {
            Constraint::None => true,
            Constraint::Int | Constraint::Long => s.parse::<i64>().is_ok(),
            Constraint::Float | Constraint::Double | Constraint::Decimal => {
                s.parse::<f64>().is_ok()
            }
            Constraint::Bool => s.parse::<bool>().is_ok(),
            Constraint::Guid => is_uuid_format(s),
            Constraint::DateTime => is_datetime_like(s),
        }
    }

    /// Converts an already-decoded segment. `None` means "rejected".
    fn convert(self, s: &str) -> Option<RouteValue> {
        match self {
            Constraint::None => Some(RouteValue::Str(s.to_string())),
            Constraint::Int | Constraint::Long => s.parse().ok().map(RouteValue::Int),
            Constraint::Float | Constraint::Double | Constraint::Decimal => {
                s.parse().ok().map(RouteValue::Float)
            }
            Constraint::Bool => s.parse().ok().map(RouteValue::Bool),
            Constraint::Guid => {
                if is_uuid_format(s) {
                    Some(RouteValue::Str(s.to_ascii_lowercase()))
                } else {
                    None
                }
            }
            Constraint::DateTime => {
                if is_datetime_like(s) {
                    Some(RouteValue::Str(s.to_string()))
                } else {
                    None
                }
            }
        }
    }
}

impl RouteTemplate {
    /// Parses and validates a template.
    ///
    /// Leading/trailing whitespace and redundant slashes are ignored.
    ///
    /// # Errors
    ///
    /// Returns a human-readable message when the template is empty, mixes
    /// literal and parameter syntax in one segment (`x{id}`), has an invalid
    /// or duplicated parameter name, uses an unknown constraint, puts a
    /// catch-all anywhere but last (or makes it optional/constrained), or has
    /// a required segment after an optional one.
    pub fn parse(template: &str) -> Result<Self, String> {
        let template = template.trim();
        if template.is_empty() {
            return Err("Empty route template".into());
        }
        let mut segments = Vec::new();
        for part in template.split('/').filter(|s| !s.is_empty()) {
            segments.push(parse_segment(part, template)?);
        }
        validate_ordering(&segments, template)?;

        let display: Vec<String> = segments.iter().map(Segment::to_display).collect();
        let sig: Vec<String> = segments.iter().map(Segment::to_signature).collect();
        Ok(Self {
            template: format!("/{}", display.join("/")),
            signature: sig.join("/"),
            segments,
        })
    }

    /// Matches `path` (query and fragment must already be removed) and returns
    /// the captured values, keyed by lower-cased parameter name.
    ///
    /// Returns `None` without allocating when the path does not match. Empty
    /// path segments (`//`) are ignored and path segments are
    /// percent-decoded (`+` is **not** treated as a space in paths).
    pub fn try_match(&self, path: &str) -> Option<HashMap<String, RouteValue>> {
        if !self.walk(path, &mut |_, _| {}) {
            return None;
        }
        let mut captured: HashMap<String, RouteValue> = HashMap::new();
        self.walk(path, &mut |seg, value| {
            let converted = match seg.constraint {
                Constraint::None => Some(RouteValue::Str(value.into_owned())),
                c => c.convert(&value),
            };
            if let Some(v) = converted {
                captured.insert(seg.key.clone(), v);
            }
        });
        Some(captured)
    }

    /// Returns `true` if `path` matches. Never allocates when the path has no
    /// percent-escapes.
    pub fn is_match(&self, path: &str) -> bool {
        self.walk(path, &mut |_, _| {})
    }

    /// Number of segments in the template (the root template has 0).
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Names (as written) of all parameters and the catch-all, in order.
    pub fn parameter_names(&self) -> impl Iterator<Item = &str> {
        self.segments
            .iter()
            .filter(|s| s.kind != SegmentKind::Literal)
            .map(|s| s.text.as_str())
    }

    /// Core matcher. Walks `path`, validating every segment, and reports each
    /// parameter / catch-all capture to `sink` as a (usually borrowed) value.
    fn walk<'p>(&self, path: &'p str, sink: &mut dyn FnMut(&Segment, Cow<'p, str>)) -> bool {
        let mut it = PathSegments::new(path);
        for seg in &self.segments {
            if seg.kind == SegmentKind::CatchAll {
                sink(seg, join_tail(it.remaining()));
                it.rest = "";
                continue;
            }
            let Some(raw) = it.next() else {
                if seg.optional {
                    continue;
                }
                return false;
            };
            let decoded = percent_decode(raw, false);
            if seg.kind == SegmentKind::Literal {
                if !decoded.eq_ignore_ascii_case(&seg.text) {
                    return false;
                }
            } else {
                if !seg.constraint.accepts(&decoded) {
                    return false;
                }
                sink(seg, decoded);
            }
        }
        it.next().is_none()
    }

    /// Sort comparator: most-specific first.
    pub(crate) fn cmp_specificity(a: &Self, b: &Self) -> std::cmp::Ordering {
        let shared = a.segments.len().min(b.segments.len());
        for i in 0..shared {
            let ra = a.segments[i].rank();
            let rb = b.segments[i].rank();
            match ra.cmp(&rb) {
                std::cmp::Ordering::Equal => {}
                o => return o,
            }
        }
        a.segments.len().cmp(&b.segments.len())
    }
}

impl fmt::Display for RouteTemplate {
    /// Writes the canonical [`template`](RouteTemplate::template) form.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.template)
    }
}

impl FromStr for RouteTemplate {
    type Err = String;

    /// Equivalent to [`RouteTemplate::parse`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// ─── Path iteration ──────────────────────────────────────────────────

/// Lazily yields the non-empty `/`-separated segments of a path as slices.
struct PathSegments<'a> {
    rest: &'a str,
}

impl<'a> PathSegments<'a> {
    fn new(path: &'a str) -> Self {
        Self { rest: path }
    }

    /// The unconsumed tail, with leading slashes removed.
    fn remaining(&self) -> &'a str {
        self.rest.trim_start_matches('/')
    }
}

impl<'a> Iterator for PathSegments<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let rest: &'a str = self.rest.trim_start_matches('/');
        if rest.is_empty() {
            self.rest = rest;
            return None;
        }
        match rest.find('/') {
            Some(i) => {
                self.rest = &rest[i..];
                Some(&rest[..i])
            }
            None => {
                self.rest = "";
                Some(rest)
            }
        }
    }
}

/// Produces the catch-all value: the remaining segments, percent-decoded and
/// joined with `/`. Borrows when no normalisation is required.
fn join_tail(rest: &str) -> Cow<'_, str> {
    let trimmed = rest.trim_matches('/');
    if !trimmed.contains("//") && !trimmed.as_bytes().contains(&b'%') {
        return Cow::Borrowed(trimmed);
    }
    let mut out = String::with_capacity(trimmed.len());
    for (i, seg) in PathSegments::new(trimmed).enumerate() {
        if i > 0 {
            out.push('/');
        }
        out.push_str(&percent_decode(seg, false));
    }
    Cow::Owned(out)
}

// ─── Parsing ─────────────────────────────────────────────────────────

fn parse_segment(raw: &str, template: &str) -> Result<Segment, String> {
    let is_param = raw.len() >= 2 && raw.starts_with('{') && raw.ends_with('}');
    if !is_param {
        if raw.contains('{') || raw.contains('}') {
            return Err(format!(
                "Malformed routing template '{}': segment '{}' mixes literal and parameter syntax",
                template, raw
            ));
        }
        return Ok(Segment {
            kind: SegmentKind::Literal,
            text: raw.to_string(),
            key: String::new(),
            constraint: Constraint::None,
            optional: false,
        });
    }

    let mut body = &raw[1..raw.len() - 1];
    let is_catch_all = body.starts_with('*');
    if is_catch_all {
        body = &body[1..];
    }
    let is_optional = body.ends_with('?');
    if is_optional {
        body = &body[..body.len() - 1];
    }
    let (name, constraint_text) = match body.find(':') {
        Some(i) => (&body[..i], &body[i + 1..]),
        None => (body, ""),
    };
    validate_param_name(name, raw, template)?;

    if is_catch_all {
        if is_optional || !constraint_text.is_empty() {
            return Err(format!(
                "Malformed routing template '{}': catch-all '{}' cannot be optional or constrained",
                template, raw
            ));
        }
        return Ok(Segment {
            kind: SegmentKind::CatchAll,
            text: name.to_string(),
            key: name.to_ascii_lowercase(),
            constraint: Constraint::None,
            optional: false,
        });
    }

    Ok(Segment {
        kind: SegmentKind::Parameter,
        text: name.to_string(),
        key: name.to_ascii_lowercase(),
        constraint: parse_constraint(constraint_text, template)?,
        optional: is_optional,
    })
}

fn validate_param_name(name: &str, raw: &str, template: &str) -> Result<(), String> {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => {
            return Err(format!(
                "Malformed routing template '{}': parameter '{}' must begin with a letter or underscore",
                template, raw
            ))
        }
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!(
            "Malformed routing template '{}': parameter '{}' contains illegal characters",
            template, raw
        ));
    }
    Ok(())
}

fn parse_constraint(text: &str, template: &str) -> Result<Constraint, String> {
    Constraint::from_name(text).ok_or_else(|| {
        format!(
            "Malformed routing template '{}': unknown constraint '{}'",
            template,
            text.to_ascii_lowercase()
        )
    })
}

fn validate_ordering(segments: &[Segment], template: &str) -> Result<(), String> {
    let mut seen: HashSet<&str> = HashSet::new();
    let mut saw_optional = false;
    for (i, s) in segments.iter().enumerate() {
        if s.kind != SegmentKind::Literal && !seen.insert(s.key.as_str()) {
            return Err(format!(
                "Malformed routing template '{}': parameter '{}' appears multiple times",
                template, s.text
            ));
        }
        if s.kind == SegmentKind::CatchAll && i != segments.len() - 1 {
            return Err(format!(
                "Malformed routing template '{}': catch-all must be the last segment",
                template
            ));
        }
        if saw_optional && !s.optional && s.kind != SegmentKind::CatchAll {
            return Err(format!(
                "Malformed routing template '{}': required segment follows optional segment",
                template
            ));
        }
        saw_optional |= s.optional;
    }
    Ok(())
}

fn is_uuid_format(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 36
        && b.iter().enumerate().all(|(i, &c)| match i {
        8 | 13 | 18 | 23 => c == b'-',
        _ => c.is_ascii_hexdigit(),
    })
}

fn is_datetime_like(s: &str) -> bool {
    s.len() >= 10 && s.as_bytes().first().is_some_and(u8::is_ascii_digit)
}

impl Segment {
    fn rank(&self) -> i32 {
        match self.kind {
            SegmentKind::Literal => 0,
            SegmentKind::CatchAll => 4,
            SegmentKind::Parameter if self.optional => 3,
            SegmentKind::Parameter if self.constraint != Constraint::None => 1,
            SegmentKind::Parameter => 2,
        }
    }

    fn to_display(&self) -> String {
        match self.kind {
            SegmentKind::Literal => self.text.clone(),
            SegmentKind::CatchAll => format!("{{*{}}}", self.text),
            SegmentKind::Parameter => {
                let mut s = String::with_capacity(self.text.len() + 12);
                s.push('{');
                s.push_str(&self.text);
                if self.constraint != Constraint::None {
                    s.push(':');
                    s.push_str(self.constraint.name());
                }
                if self.optional {
                    s.push('?');
                }
                s.push('}');
                s
            }
        }
    }

    fn to_signature(&self) -> String {
        match self.kind {
            SegmentKind::Literal => self.text.to_ascii_lowercase(),
            SegmentKind::CatchAll => "{*}".to_string(),
            SegmentKind::Parameter => {
                let mut s = String::with_capacity(12);
                s.push('{');
                if self.constraint != Constraint::None {
                    s.push(':');
                    s.push_str(self.constraint.name());
                }
                if self.optional {
                    s.push('?');
                }
                s.push('}');
                s
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(template: &str) -> RouteTemplate {
        RouteTemplate::parse(template).expect("template should parse")
    }

    fn m(template: &str, path: &str) -> Option<HashMap<String, RouteValue>> {
        t(template).try_match(path)
    }

    // ── literals / structure ──

    #[test]
    fn literal_match_is_ascii_case_insensitive() {
        assert!(m("/Orders/List", "/orders/LIST").is_some());
        assert!(m("/orders", "/order").is_none());
    }

    #[test]
    fn root_template_matches_only_empty_path() {
        for p in ["", "/", "///"] {
            assert_eq!(m("/", p), Some(HashMap::new()), "path {p:?}");
        }
        assert!(m("/", "/x").is_none());
        assert_eq!(RouteTemplate::default().template, "/");
        assert!(RouteTemplate::default().is_match("/"));
    }

    #[test]
    fn extra_or_missing_segments_do_not_match() {
        assert!(m("/a", "/a/b").is_none());
        assert!(m("/a/b", "/a").is_none());
    }

    #[test]
    fn redundant_slashes_are_ignored() {
        assert!(m("/a/b", "a//b/").is_some());
        assert!(m("a/b/", "/a/b").is_some());
    }

    // ── typed parameters ──

    #[test]
    fn int_constraint() {
        let v = m("/orders/{id:int}", "/orders/42").unwrap();
        assert_eq!(v["id"], RouteValue::Int(42));
        assert_eq!(m("/orders/{id:int}", "/orders/-7").unwrap()["id"], RouteValue::Int(-7));
        assert!(m("/orders/{id:int}", "/orders/abc").is_none());
        assert!(m("/orders/{id:int}", "/orders/4.2").is_none());
        assert!(m("/orders/{id:int}", "/orders/").is_none());
    }

    #[test]
    fn long_constraint_accepts_64_bit() {
        let v = m("/n/{x:long}", "/n/9999999999").unwrap();
        assert_eq!(v["x"], RouteValue::Int(9_999_999_999));
        assert!(m("/n/{x:long}", "/n/99999999999999999999").is_none());
    }

    #[test]
    fn floating_point_constraints() {
        for c in ["double", "float", "decimal"] {
            let tpl = format!("/p/{{x:{c}}}");
            assert_eq!(m(&tpl, "/p/3.5").unwrap()["x"], RouteValue::Float(3.5), "{c}");
            assert_eq!(m(&tpl, "/p/-0.25").unwrap()["x"], RouteValue::Float(-0.25), "{c}");
            assert_eq!(m(&tpl, "/p/2").unwrap()["x"], RouteValue::Float(2.0), "{c}");
            assert!(m(&tpl, "/p/abc").is_none(), "{c}");
        }
    }

    #[test]
    fn bool_constraint() {
        assert_eq!(m("/f/{on:bool}", "/f/true").unwrap()["on"], RouteValue::Bool(true));
        assert_eq!(m("/f/{on:bool}", "/f/false").unwrap()["on"], RouteValue::Bool(false));
        assert!(m("/f/{on:bool}", "/f/True").is_none());
        assert!(m("/f/{on:bool}", "/f/1").is_none());
    }

    #[test]
    fn guid_constraint_validates_shape_and_lowercases() {
        let tpl = "/g/{id:guid}";
        let ok = "123e4567-e89b-12d3-a456-426614174000";
        assert_eq!(m(tpl, &format!("/g/{ok}")).unwrap()["id"], RouteValue::Str(ok.into()));
        let upper = ok.to_ascii_uppercase();
        assert_eq!(m(tpl, &format!("/g/{upper}")).unwrap()["id"], RouteValue::Str(ok.into()));
        assert!(m(tpl, "/g/123e4567").is_none());
        assert!(m(tpl, "/g/123e4567-e89b-12d3-a456-42661417400z").is_none());
        assert!(m(tpl, "/g/123e4567e-89b-12d3-a456-426614174000").is_none());
        assert!(m(tpl, "/g/123e4567-e89b-12d3-a456-4266141740000").is_none());
    }

    #[test]
    fn datetime_constraint_is_shape_only() {
        let tpl = "/d/{when:datetime}";
        assert_eq!(
            m(tpl, "/d/2024-01-15").unwrap()["when"],
            RouteValue::Str("2024-01-15".into())
        );
        assert!(m(tpl, "/d/2024-01-15T10:30:00").is_some());
        assert!(m(tpl, "/d/20240115").is_none()); // too short
        assert!(m(tpl, "/d/abcdefghij").is_none()); // doesn't start with a digit
    }

    #[test]
    fn unconstrained_param_is_str() {
        assert_eq!(m("/u/{name}", "/u/alice").unwrap()["name"], RouteValue::Str("alice".into()));
    }

    #[test]
    fn captured_keys_are_lowercase() {
        let v = m("/orders/{OrderId:int}", "/orders/5").unwrap();
        assert!(v.contains_key("orderid"));
        assert!(!v.contains_key("OrderId"));
    }

    // ── optional ──

    #[test]
    fn optional_parameter() {
        assert_eq!(m("/users/{id?}", "/users"), Some(HashMap::new()));
        assert_eq!(m("/users/{id?}", "/users/5").unwrap()["id"], RouteValue::Str("5".into()));
        assert!(m("/users/{id?}", "/users/5/extra").is_none());
    }

    #[test]
    fn optional_constrained_parameter_still_validates_when_present() {
        assert_eq!(m("/u/{id:int?}", "/u/9").unwrap()["id"], RouteValue::Int(9));
        assert!(m("/u/{id:int?}", "/u").unwrap().is_empty());
        assert!(m("/u/{id:int?}", "/u/x").is_none());
    }

    #[test]
    fn optional_followed_by_catch_all_is_allowed() {
        assert!(RouteTemplate::parse("/{a?}/{*rest}").is_ok());
        let v = m("/{a?}/{*rest}", "/x/y/z").unwrap();
        assert_eq!(v["a"], RouteValue::Str("x".into()));
        assert_eq!(v["rest"], RouteValue::Str("y/z".into()));
    }

    // ── catch-all ──

    #[test]
    fn catch_all_captures_joined_tail() {
        let v = m("/files/{*path}", "/files/a/b/c.txt").unwrap();
        assert_eq!(v["path"], RouteValue::Str("a/b/c.txt".into()));
    }

    #[test]
    fn catch_all_may_be_empty() {
        for p in ["/files", "/files/", "/files//"] {
            assert_eq!(m("/files/{*path}", p).unwrap()["path"], RouteValue::Str(String::new()), "{p}");
        }
    }

    #[test]
    fn catch_all_normalises_empty_segments_and_decodes() {
        assert_eq!(
            m("/f/{*p}", "/f//a//b/").unwrap()["p"],
            RouteValue::Str("a/b".into())
        );
        assert_eq!(
            m("/f/{*p}", "/f/a%20b/c").unwrap()["p"],
            RouteValue::Str("a b/c".into())
        );
        assert_eq!(m("/f/{*p}", "/f/a%2Fb").unwrap()["p"], RouteValue::Str("a/b".into()));
    }

    #[test]
    fn catch_all_borrows_when_possible() {
        assert!(matches!(join_tail("a/b/c"), Cow::Borrowed("a/b/c")));
        assert!(matches!(join_tail("/a/b/"), Cow::Borrowed("a/b")));
        assert!(matches!(join_tail("a//b"), Cow::Owned(_)));
    }

    // ── decoding ──

    #[test]
    fn path_segments_are_percent_decoded() {
        assert_eq!(m("/s/{q}", "/s/a%20b").unwrap()["q"], RouteValue::Str("a b".into()));
        assert!(m("/caf%C3%A9", "/café").is_none()); // literals compare decoded path vs template text
        assert!(m("/café", "/caf%C3%A9").is_some());
    }

    #[test]
    fn plus_is_literal_in_paths() {
        assert_eq!(m("/s/{q}", "/s/a+b").unwrap()["q"], RouteValue::Str("a+b".into()));
    }

    #[test]
    fn ampersand_and_equals_are_not_query_delimiters_in_paths() {
        assert_eq!(m("/s/{q}", "/s/a&b=c").unwrap()["q"], RouteValue::Str("a&b=c".into()));
        assert_eq!(m("/s/{q}", "/s/a%26b").unwrap()["q"], RouteValue::Str("a&b".into()));
    }

    #[test]
    fn is_match_agrees_with_try_match() {
        let tpl = t("/orders/{id:int}");
        for p in ["/orders/1", "/orders/x", "/orders", "/orders/1/2"] {
            assert_eq!(tpl.is_match(p), tpl.try_match(p).is_some(), "{p}");
        }
    }

    // ── parsing / validation ──

    #[test]
    fn parse_normalises_display_and_signature() {
        let tpl = t("orders/{id:INT}/");
        assert_eq!(tpl.template, "/orders/{id:int}");
        assert_eq!(tpl.signature, "orders/{:int}");
        assert_eq!(tpl.to_string(), "/orders/{id:int}");
        assert_eq!(tpl.segment_count(), 2);
    }

    #[test]
    fn signatures_ignore_names_but_not_shape() {
        assert_eq!(t("/a/{x:int}").signature, t("/A/{y:int}").signature);
        assert_ne!(t("/a/{x:int}").signature, t("/a/{x}").signature);
        assert_ne!(t("/a/{x}").signature, t("/a/{x?}").signature);
        assert_eq!(t("/a/{*rest}").signature, "a/{*}");
        assert_eq!(t("/").signature, "");
    }

    #[test]
    fn parameter_names_lists_params_and_catch_all() {
        let tpl = t("/a/{Id:int}/b/{*Rest}");
        assert_eq!(tpl.parameter_names().collect::<Vec<_>>(), vec!["Id", "Rest"]);
    }

    #[test]
    fn from_str_delegates_to_parse() {
        let tpl: RouteTemplate = "/x/{id}".parse().unwrap();
        assert_eq!(tpl.template, "/x/{id}");
        assert!("".parse::<RouteTemplate>().is_err());
    }

    #[test]
    fn malformed_templates_are_rejected() {
        let bad = [
            "",
            "   ",
            "/a/{id",          // unterminated
            "/a/x{id}",        // mixes literal + param
            "/a/{}",           // empty name
            "/a/{1id}",        // bad first char
            "/a/{id-x}",       // illegal char
            "/a/{id:wat}",     // unknown constraint
            "/a/{*p}/b",       // catch-all not last
            "/a/{*p?}",        // optional catch-all
            "/a/{*p:int}",     // constrained catch-all
            "/{id}/{id}",      // duplicate
            "/{id}/{ID:int}",  // duplicate, case-insensitive
            "/a/{x?}/b",       // required after optional
            "/a/{x?}/{y}",     // required after optional
        ];
        for b in bad {
            assert!(RouteTemplate::parse(b).is_err(), "should reject {b:?}");
        }
    }

    #[test]
    fn error_messages_name_the_template() {
        let e = RouteTemplate::parse("/a/{id:wat}").unwrap_err();
        assert!(e.contains("unknown constraint 'wat'"), "{e}");
        assert!(e.contains("/a/{id:wat}"), "{e}");
        assert_eq!(RouteTemplate::parse("").unwrap_err(), "Empty route template");
    }

    #[test]
    fn constraint_names_are_case_insensitive() {
        assert!(RouteTemplate::parse("/a/{x:Int}").is_ok());
        assert!(RouteTemplate::parse("/a/{x:GUID}").is_ok());
        assert!(RouteTemplate::parse("/a/{x:DateTime?}").is_ok());
    }

    // ── specificity ──

    #[test]
    fn specificity_orders_literal_constrained_plain_optional_catch_all() {
        let mut v: Vec<RouteTemplate> = ["/a/{x}", "/a/{*r}", "/a/{z?}", "/a/b", "/a/{y:int}"]
            .iter()
            .map(|s| t(s))
            .collect();
        v.sort_by(RouteTemplate::cmp_specificity);
        let order: Vec<&str> = v.iter().map(|x| x.template.as_str()).collect();
        assert_eq!(order, vec!["/a/b", "/a/{y:int}", "/a/{x}", "/a/{z?}", "/a/{*r}"]);
    }

    #[test]
    fn shorter_template_wins_ties_on_shared_prefix() {
        let a = t("/a");
        let b = t("/a/b");
        assert_eq!(RouteTemplate::cmp_specificity(&a, &b), std::cmp::Ordering::Less);
    }

    #[test]
    fn clone_and_eq() {
        let a = t("/a/{x:int}");
        assert_eq!(a.clone(), a);
        assert_ne!(a, t("/a/{x}"));
    }

    #[test]
    fn uuid_and_datetime_helpers() {
        assert!(is_uuid_format("00000000-0000-0000-0000-000000000000"));
        assert!(!is_uuid_format(""));
        assert!(!is_uuid_format("é0000000-0000-0000-0000-00000000000")); // non-ASCII never panics
        assert!(is_datetime_like("2024-01-01"));
        assert!(!is_datetime_like(""));
    }
}