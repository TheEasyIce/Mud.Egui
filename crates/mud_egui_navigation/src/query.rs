//! Order-preserving query-string parsing/building. Port of `QueryString.cs`.
//!
//! # Allocation strategy
//!
//! * [`pairs`] / [`QueryPairs`] iterate a query string lazily and yield
//!   [`Cow<str>`] slices: nothing is allocated unless a component actually
//!   contains a percent-escape or a `+`.
//! * [`with_parameter`] rewrites a query string in a single pass into one
//!   pre-sized `String`, with no intermediate `Vec` of pairs.
//! * The owned APIs ([`parse`], [`to_first_value_map`], [`set_parameter`],
//!   [`build`]) are retained for backwards compatibility.
//!
//! Names are compared **case-insensitively** everywhere in this module.

use std::borrow::Cow;
use std::collections::HashMap;

// ─── Borrowed iteration ──────────────────────────────────────────────

/// Lazy iterator over the `name=value` pairs of a query string.
///
/// Created by [`pairs`]. A leading `?` is ignored, empty segments (`a=1&&b=2`)
/// are skipped, and a segment without `=` yields an empty value. Components are
/// percent-decoded with `+` treated as a space; decoding only allocates when an
/// escape sequence or `+` is present.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QueryPairs<'a> {
    rest: &'a str,
}

impl<'a> QueryPairs<'a> {
    /// Creates an iterator over `query` (with or without a leading `?`).
    pub fn new(query: &'a str) -> Self {
        Self {
            rest: query.strip_prefix('?').unwrap_or(query),
        }
    }
}

impl<'a> Iterator for QueryPairs<'a> {
    type Item = (Cow<'a, str>, Cow<'a, str>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let rest: &'a str = self.rest;
            if rest.is_empty() {
                return None;
            }
            let (segment, tail): (&'a str, &'a str) = match rest.find('&') {
                Some(i) => (&rest[..i], &rest[i + 1..]),
                None => (rest, ""),
            };
            self.rest = tail;
            if segment.is_empty() {
                continue;
            }
            let (name, value): (&'a str, &'a str) = match segment.find('=') {
                Some(i) => (&segment[..i], &segment[i + 1..]),
                None => (segment, ""),
            };
            return Some((percent_decode(name, true), percent_decode(value, true)));
        }
    }
}

/// Returns a lazy, allocation-free (unless escapes are present) iterator over
/// the pairs of `query`. See [`QueryPairs`].
pub fn pairs(query: &str) -> QueryPairs<'_> {
    QueryPairs::new(query)
}

/// Returns the first value whose name equals `name` (case-insensitive).
///
/// ```ignore
/// assert_eq!(query::get_first("?a=1&A=2", "a").as_deref(), Some("1"));
/// ```
pub fn get_first<'a>(query: &'a str, name: &str) -> Option<Cow<'a, str>> {
    pairs(query)
        .find(|(k, _)| eq_ignore_case(k, name))
        .map(|(_, v)| v)
}

// ─── Owned convenience API (backwards compatible) ────────────────────

/// Parses a query string into ordered key-value pairs. Leading `?` is stripped.
/// Percent-decoding is applied; `+` becomes a space.
///
/// Prefer [`pairs`] when you only need to inspect the pairs.
pub fn parse(query: &str) -> Vec<(String, String)> {
    pairs(query)
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect()
}

/// Case-insensitive first-value map. Later duplicates are ignored.
/// Keys are lower-cased.
pub fn to_first_value_map(pairs: Vec<(String, String)>) -> HashMap<String, String> {
    let mut map = HashMap::with_capacity(pairs.len());
    for (k, v) in pairs {
        map.entry(to_lower_cow(&k).into_owned()).or_insert(v);
    }
    map
}

/// Builds the case-insensitive first-value map straight from a raw query
/// string, skipping the intermediate `Vec<(String, String)>` of [`parse`].
///
/// Keys are lower-cased; later duplicates are ignored.
pub fn first_value_map(query: &str) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    for (k, v) in pairs(query) {
        let key = to_lower_cow(&k);
        if !map.contains_key(&*key) {
            map.insert(key.into_owned(), v.into_owned());
        }
    }
    map
}

/// Sets, replaces, or removes a parameter (in place). `None` removes all
/// occurrences; `Some` replaces the first and drops later duplicates.
///
/// The replaced pair adopts the spelling of `name` passed here.
pub fn set_parameter(pairs: &mut Vec<(String, String)>, name: &str, value: Option<&str>) {
    let mut replaced = false;
    pairs.retain_mut(|pair| {
        if !eq_ignore_case(&pair.0, name) {
            return true;
        }
        if !replaced {
            if let Some(v) = value {
                pair.0.clear();
                pair.0.push_str(name);
                pair.1.clear();
                pair.1.push_str(v);
                replaced = true;
                return true;
            }
        }
        false
    });
    if !replaced {
        if let Some(v) = value {
            pairs.push((name.to_string(), v.to_string()));
        }
    }
}

/// Returns `query` with `name` set to `value` (or removed when `None`), in one
/// pass and one allocation. The result has a leading `?` unless it is empty.
///
/// Semantics are identical to `parse` → [`set_parameter`] → [`build`]:
/// the first occurrence is replaced in place, later duplicates are dropped,
/// and a missing parameter is appended.
///
/// ```ignore
/// assert_eq!(query::with_parameter("?a=1", "b", Some("2")), "?a=1&b=2");
/// assert_eq!(query::with_parameter("?a=1&b=2", "a", None), "?b=2");
/// ```
pub fn with_parameter(query: &str, name: &str, value: Option<&str>) -> String {
    let mut out = String::with_capacity(query.len() + name.len() + value.map_or(0, str::len) + 2);
    let mut replaced = false;
    for (k, v) in pairs(query) {
        if eq_ignore_case(&k, name) {
            if !replaced {
                if let Some(new_value) = value {
                    push_pair(&mut out, name, new_value);
                    replaced = true;
                }
            }
            continue;
        }
        push_pair(&mut out, &k, &v);
    }
    if !replaced {
        if let Some(new_value) = value {
            push_pair(&mut out, name, new_value);
        }
    }
    out
}

/// Builds a well-formed query string (leading `?`). Empty input → `""`.
///
/// Components are percent-encoded; spaces become `+`.
pub fn build<K: AsRef<str>, V: AsRef<str>>(pairs: &[(K, V)]) -> String {
    let mut out = String::with_capacity(pairs.len() * 20 + 1);
    for (k, v) in pairs {
        push_pair(&mut out, k.as_ref(), v.as_ref());
    }
    out
}

fn push_pair(out: &mut String, k: &str, v: &str) {
    out.push(if out.is_empty() { '?' } else { '&' });
    encode_component_into(k, out);
    out.push('=');
    encode_component_into(v, out);
}

// ─── URI splitting ───────────────────────────────────────────────────

/// Splits a URI-reference into `(path, query, fragment)` slices.
///
/// `query` keeps its leading `?` and `fragment` its leading `#`; either is
/// empty when absent. A `?` that appears after a `#` belongs to the fragment.
///
/// ```ignore
/// assert_eq!(query::split_uri("/a?x=1#top"), ("/a", "?x=1", "#top"));
/// ```
pub fn split_uri(uri: &str) -> (&str, &str, &str) {
    let (without_frag, frag) = match uri.find('#') {
        Some(i) => (&uri[..i], &uri[i..]),
        None => (uri, ""),
    };
    match without_frag.find('?') {
        Some(i) => (&without_frag[..i], &without_frag[i..], frag),
        None => (without_frag, "", frag),
    }
}

/// Like [`split_uri`] but discards the fragment: returns `(path, query)`.
pub fn split_path_query(uri: &str) -> (&str, &str) {
    let (p, q, _) = split_uri(uri);
    (p, q)
}

// ─── Case-insensitive helpers ────────────────────────────────────────

/// Allocation-free, Unicode-aware case-insensitive equality.
pub fn eq_ignore_case(a: &str, b: &str) -> bool {
    if a.is_ascii() && b.is_ascii() {
        return a.eq_ignore_ascii_case(b);
    }
    a.chars()
        .flat_map(char::to_lowercase)
        .eq(b.chars().flat_map(char::to_lowercase))
}

/// Lower-cases `s`, borrowing when it is already lower-case.
pub fn to_lower_cow(s: &str) -> Cow<'_, str> {
    if s.chars().all(|c| c.to_lowercase().eq(std::iter::once(c))) {
        Cow::Borrowed(s)
    } else {
        Cow::Owned(s.to_lowercase())
    }
}

// ─── Percent encoding ────────────────────────────────────────────────

/// Percent-decodes `s`. Invalid escapes (`%zz`, a trailing `%`) are kept
/// literally. When `plus_as_space` is set, `+` decodes to a space
/// (form/query semantics); path segments should pass `false`.
///
/// Returns [`Cow::Borrowed`] (no allocation) when there is nothing to decode.
/// Bytes that do not form valid UTF-8 are replaced with U+FFFD.
pub fn percent_decode(s: &str, plus_as_space: bool) -> Cow<'_, str> {
    let bytes = s.as_bytes();
    if !bytes
        .iter()
        .any(|&b| b == b'%' || (plus_as_space && b == b'+'))
    {
        return Cow::Borrowed(s);
    }
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => match (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                (Some(h), Some(l)) => {
                    out.push(h * 16 + l);
                    i += 3;
                }
                _ => {
                    out.push(b'%');
                    i += 1;
                }
            },
            b'+' if plus_as_space => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    Cow::Owned(
        String::from_utf8(out)
            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned()),
    )
}

/// Appends the form-encoded (`application/x-www-form-urlencoded`) form of `s`
/// to `out`: unreserved characters pass through, space becomes `+`, every other
/// byte becomes `%XX`.
pub fn encode_component_into(s: &str, out: &mut String) {
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b' ' => out.push('+'),
            _ => {
                out.push('%');
                out.push(HEX[(byte >> 4) as usize] as char);
                out.push(HEX[(byte & 0xf) as usize] as char);
            }
        }
    }
}

const HEX: &[u8; 16] = b"0123456789ABCDEF";

fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(a: &str, b: &str) -> (String, String) {
        (a.to_string(), b.to_string())
    }

    #[test]
    fn parse_basic_and_leading_question_mark() {
        assert_eq!(parse("?a=1&b=2"), vec![s("a", "1"), s("b", "2")]);
        assert_eq!(parse("a=1&b=2"), vec![s("a", "1"), s("b", "2")]);
    }

    #[test]
    fn parse_empty_inputs() {
        assert!(parse("").is_empty());
        assert!(parse("?").is_empty());
        assert!(parse("&&&").is_empty());
    }

    #[test]
    fn parse_missing_value_and_empty_value() {
        assert_eq!(parse("?flag&x="), vec![s("flag", ""), s("x", "")]);
    }

    #[test]
    fn parse_only_splits_on_first_equals() {
        assert_eq!(parse("?q=a=b"), vec![s("q", "a=b")]);
    }

    #[test]
    fn parse_decodes_percent_and_plus() {
        assert_eq!(parse("?q=hello+world%21"), vec![s("q", "hello world!")]);
        assert_eq!(parse("?n%61me=v"), vec![s("name", "v")]);
        assert_eq!(parse("?e=%C3%A9"), vec![s("e", "é")]);
    }

    #[test]
    fn invalid_escapes_are_kept_literally() {
        assert_eq!(parse("?a=%zz"), vec![s("a", "%zz")]);
        assert_eq!(parse("?a=%4"), vec![s("a", "%4")]);
        assert_eq!(parse("?a=%"), vec![s("a", "%")]);
    }

    #[test]
    fn invalid_utf8_is_replaced_not_panicking() {
        let p = parse("?a=%FF");
        assert_eq!(p, vec![s("a", "\u{FFFD}")]);
    }

    #[test]
    fn pairs_borrows_when_nothing_to_decode() {
        let mut it = pairs("?a=1&b=two");
        let (k, v) = it.next().unwrap();
        assert!(matches!(k, Cow::Borrowed("a")));
        assert!(matches!(v, Cow::Borrowed("1")));
        let (k, v) = it.next().unwrap();
        assert!(matches!(k, Cow::Borrowed("b")));
        assert!(matches!(v, Cow::Borrowed("two")));
        assert!(it.next().is_none());
    }

    #[test]
    fn pairs_allocates_only_when_decoding() {
        let (_, v) = pairs("a=x%20y").next().unwrap();
        assert!(matches!(v, Cow::Owned(_)));
        assert_eq!(v, "x y");
    }

    #[test]
    fn percent_decode_plus_flag() {
        assert_eq!(percent_decode("a+b", true), "a b");
        assert_eq!(percent_decode("a+b", false), "a+b");
        assert!(matches!(percent_decode("a+b", false), Cow::Borrowed(_)));
        assert_eq!(percent_decode("a%2Bb", true), "a+b");
    }

    #[test]
    fn get_first_is_case_insensitive_and_first_wins() {
        assert_eq!(get_first("?Name=1&name=2", "NAME").as_deref(), Some("1"));
        assert!(get_first("?a=1", "b").is_none());
    }

    #[test]
    fn first_value_map_lowercases_and_ignores_duplicates() {
        let m = first_value_map("?Filter=active&FILTER=done&Page=2");
        assert_eq!(m.get("filter").map(String::as_str), Some("active"));
        assert_eq!(m.get("page").map(String::as_str), Some("2"));
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn to_first_value_map_matches_first_value_map() {
        let q = "?A=1&a=2&b=%20";
        assert_eq!(to_first_value_map(parse(q)), first_value_map(q));
    }

    #[test]
    fn set_parameter_replaces_first_and_drops_duplicates() {
        let mut p = parse("?a=1&b=2&A=3");
        set_parameter(&mut p, "a", Some("9"));
        assert_eq!(p, vec![s("a", "9"), s("b", "2")]);
    }

    #[test]
    fn set_parameter_appends_when_missing() {
        let mut p = parse("?a=1");
        set_parameter(&mut p, "b", Some("2"));
        assert_eq!(p, vec![s("a", "1"), s("b", "2")]);
    }

    #[test]
    fn set_parameter_none_removes_all_occurrences() {
        let mut p = parse("?a=1&b=2&a=3");
        set_parameter(&mut p, "A", None);
        assert_eq!(p, vec![s("b", "2")]);
        set_parameter(&mut p, "missing", None);
        assert_eq!(p, vec![s("b", "2")]);
    }

    #[test]
    fn set_parameter_adopts_new_spelling() {
        let mut p = parse("?Page=1");
        set_parameter(&mut p, "page", Some("2"));
        assert_eq!(p, vec![s("page", "2")]);
    }

    #[test]
    fn with_parameter_matches_vec_pipeline() {
        let cases = [
            ("?a=1&b=2&A=3", "a", Some("9")),
            ("?a=1", "b", Some("2")),
            ("?a=1&b=2&a=3", "A", None),
            ("", "x", Some("y z")),
            ("", "x", None),
            ("?only=1", "only", None),
            ("?q=a%26b&r=1", "r", Some("2")),
        ];
        for (q, name, value) in cases {
            let mut v = parse(q);
            set_parameter(&mut v, name, value);
            assert_eq!(with_parameter(q, name, value), build(&v), "case {q:?} {name} {value:?}");
        }
    }

    #[test]
    fn with_parameter_examples() {
        assert_eq!(with_parameter("?a=1", "b", Some("2")), "?a=1&b=2");
        assert_eq!(with_parameter("?a=1&b=2", "a", None), "?b=2");
        assert_eq!(with_parameter("", "q", None), "");
    }

    #[test]
    fn build_empty_and_basic() {
        let empty: Vec<(String, String)> = Vec::new();
        assert_eq!(build(&empty), "");
        assert_eq!(build(&[s("a", "1"), s("b", "2")]), "?a=1&b=2");
    }

    #[test]
    fn build_accepts_borrowed_pairs() {
        assert_eq!(build(&[("a", "1"), ("b", "2")]), "?a=1&b=2");
    }

    #[test]
    fn build_encodes_reserved_characters() {
        assert_eq!(build(&[s("a b", "c&d=e")]), "?a+b=c%26d%3De");
        assert_eq!(build(&[s("k", "é")]), "?k=%C3%A9");
        assert_eq!(build(&[s("k", "~-_.")]), "?k=~-_.");
    }

    #[test]
    fn build_parse_round_trip() {
        let original = vec![s("na me", "v&a=l+ue"), s("é", "100%")];
        assert_eq!(parse(&build(&original)), original);
    }

    #[test]
    fn split_uri_parts() {
        assert_eq!(split_uri("/a?x=1#top"), ("/a", "?x=1", "#top"));
        assert_eq!(split_uri("/a#top"), ("/a", "", "#top"));
        assert_eq!(split_uri("/a?x=1"), ("/a", "?x=1", ""));
        assert_eq!(split_uri("/a"), ("/a", "", ""));
        assert_eq!(split_uri("/a#f?not=query"), ("/a", "", "#f?not=query"));
        assert_eq!(split_uri(""), ("", "", ""));
        assert_eq!(split_path_query("/a?x=1#t"), ("/a", "?x=1"));
    }

    #[test]
    fn eq_ignore_case_ascii_and_unicode() {
        assert!(eq_ignore_case("Hello", "hELLO"));
        assert!(!eq_ignore_case("a", "b"));
        assert!(eq_ignore_case("ÉCOLE", "école"));
        assert!(!eq_ignore_case("a", "ab"));
    }

    #[test]
    fn to_lower_cow_borrows_when_possible() {
        assert!(matches!(to_lower_cow("abc-1"), Cow::Borrowed(_)));
        assert_eq!(to_lower_cow("AbC"), "abc");
        assert!(matches!(to_lower_cow("AbC"), Cow::Owned(_)));
    }
}