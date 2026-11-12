//! Pure helpers for deciding whether a link is **internal** (routed by the
//! `NavigationManager`) or **external** (opened in the system browser).
//!
//! The rule, per the user requirement, is:
//!
//! > If the link starts with `http(s)://` and does not match the app URL,
//! > open it in the browser.
//!
//! In practice a link is **external** when any of the following is true:
//!
//! 1. It has a scheme that is not the app base scheme, and that scheme is a
//!    known browser scheme (`http`, `https`, `mailto`, `tel`, `sms`, `ftp`,
//!    `file`, `data`) — or an unknown one (unknown schemes are treated as
//!    external by default).
//! 2. It has the same scheme as the app base URI, but a **different origin**
//!    (`scheme://host:port`). This is the "matches the app URL" clause.
//!
//! A URI with **no scheme** (a relative path) is always internal — it cannot
//! point to anything outside the app.
//!
//! Everything in this module is a pure function: no allocation beyond the
//! slices returned by `origin_of`, and no egui types.

/// Extracts the scheme from a URI (`"https"` from `"https://…"`,
/// `"mailto"` from `"mailto:…"`).
///
/// Returns `None` for relative paths and for strings that begin with
/// something that is not a valid scheme (per RFC 3986 §3.1: ASCII letter
/// followed by letters, digits, `+`, `-`, or `.`).
pub fn scheme_of(uri: &str) -> Option<&str> {
    let colon = uri.find(':')?;
    let scheme = &uri[..colon];
    let mut chars = scheme.chars();
    let first = chars.next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.') {
        return None;
    }
    Some(scheme)
}

/// Extracts the origin (`"https://example.com"`) from an absolute URI that
/// contains `://`.
///
/// Returns `None` for URIs without `://` (e.g. `"mailto:…"`) and for
/// relative paths.
pub fn origin_of(uri: &str) -> Option<&str> {
    let scheme_end = uri.find("://")?;
    let after = &uri[scheme_end + 3..];
    let origin_end = after
        .find(|c| c == '/' || c == '?' || c == '#')
        .unwrap_or(after.len());
    Some(&uri[..scheme_end + 3 + origin_end])
}

/// Whether `href` should be opened **externally** (system browser) rather
/// than routed by the `NavigationManager`.
///
/// `base_uri` is the app's base URI as configured on the `NavigationManager`
/// (e.g. `"app://my-app/"` or `"https://example.com/app/"`).
///
/// # Truth table
///
/// | `href`                          | `base_uri`                        | Result |
/// |---------------------------------|-----------------------------------|--------|
/// | `"/orders/42"`                  | `"https://example.com/"`          | **internal** (no scheme) |
/// | `"orders/42"`                   | `"https://example.com/"`          | **internal** (no scheme) |
/// | `"https://example.com/orders/42"` | `"https://example.com/"`        | **internal** (same origin) |
/// | `"https://other.com/foo"`       | `"https://example.com/"`          | **external** (different origin) |
/// | `"https://example.com/foo"`     | `"app://my-app/"`                 | **external** (different scheme) |
/// | `"app://my-app/orders/42"`      | `"app://my-app/"`                 | **internal** (same origin) |
/// | `"mailto:a@b.com"`              | `"https://example.com/"`          | **external** |
/// | `"tel:+15551234"`               | `"https://example.com/"`          | **external** |
pub fn is_external_url(href: &str, base_uri: &str) -> bool {
    let Some(href_scheme) = scheme_of(href) else {
        // No scheme → relative path → the router handles it.
        return false;
    };

    let base_scheme = scheme_of(base_uri);

    // Same scheme as the app → compare origins. If origins match, the link
    // targets the app itself and should be routed internally.
    if Some(href_scheme) == base_scheme {
        return origin_of(href) != origin_of(base_uri);
    }

    // Different scheme → external by definition. This covers `mailto:`,
    // `tel:`, and unknown schemes.
    true
}

/// Resolves the `new_tab` flag from a MudBlazor `Target` value.
///
/// MudBlazor accepts `_blank | _self | _parent | _top | framename`. Only
/// `_blank` opens in a new frame, which maps to `new_tab: true` in egui's
/// `OpenUrl`.
#[inline]
pub fn target_opens_new_tab(target: Option<&str>) -> bool {
    matches!(target, Some(t) if t.eq_ignore_ascii_case("_blank"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- scheme_of --------------------------------------------------------

    #[test]
    fn scheme_of_handles_http_and_https() {
        assert_eq!(scheme_of("https://example.com"), Some("https"));
        assert_eq!(scheme_of("http://example.com"), Some("http"));
    }

    #[test]
    fn scheme_of_handles_non_slash_schemes() {
        assert_eq!(scheme_of("mailto:a@b.com"), Some("mailto"));
        assert_eq!(scheme_of("tel:+15551234"), Some("tel"));
    }

    #[test]
    fn scheme_of_rejects_relative_paths() {
        assert_eq!(scheme_of("/orders/42"), None);
        assert_eq!(scheme_of("orders/42"), None);
        assert_eq!(scheme_of(""), None);
    }

    #[test]
    fn scheme_of_rejects_invalid_schemes() {
        // Starts with a digit → invalid.
        assert_eq!(scheme_of("1http://x"), None);
        // Contains a space → invalid.
        assert_eq!(scheme_of("ht tp://x"), None);
    }

    // --- origin_of --------------------------------------------------------

    #[test]
    fn origin_of_strips_path() {
        assert_eq!(origin_of("https://example.com/orders/42"), Some("https://example.com"));
        assert_eq!(origin_of("https://example.com/"), Some("https://example.com"));
    }

    #[test]
    fn origin_of_preserves_port() {
        assert_eq!(
            origin_of("http://localhost:8080/x"),
            Some("http://localhost:8080"),
        );
    }

    #[test]
    fn origin_of_strips_query_and_fragment() {
        assert_eq!(origin_of("https://a.com?x=1"), Some("https://a.com"));
        assert_eq!(origin_of("https://a.com#frag"), Some("https://a.com"));
    }

    #[test]
    fn origin_of_returns_none_without_slashes() {
        assert_eq!(origin_of("mailto:a@b.com"), None);
        assert_eq!(origin_of("/relative"), None);
    }

    // --- is_external_url --------------------------------------------------

    #[test]
    fn relative_paths_are_internal() {
        assert!(!is_external_url("/orders/42", "https://example.com/"));
        assert!(!is_external_url("orders/42", "https://example.com/"));
        assert!(!is_external_url("", "https://example.com/"));
    }

    #[test]
    fn same_origin_is_internal() {
        assert!(!is_external_url(
            "https://example.com/orders/42",
            "https://example.com/",
        ));
        assert!(!is_external_url(
            "https://example.com/x?y=1",
            "https://example.com/app/",
        ));
    }

    #[test]
    fn different_origin_is_external() {
        assert!(is_external_url(
            "https://other.com/foo",
            "https://example.com/",
        ));
        assert!(is_external_url(
            "http://example.com/foo", // different scheme
            "https://example.com/",
        ));
    }

    #[test]
    fn different_scheme_from_custom_base_is_external() {
        assert!(is_external_url(
            "https://example.com/foo",
            "app://my-app/",
        ));
    }

    #[test]
    fn same_scheme_as_custom_base_is_internal() {
        assert!(!is_external_url(
            "app://my-app/orders/42",
            "app://my-app/",
        ));
    }

    #[test]
    fn mailto_and_tel_are_external() {
        assert!(is_external_url("mailto:a@b.com", "https://example.com/"));
        assert!(is_external_url("tel:+15551234", "https://example.com/"));
        assert!(is_external_url("sms:+15551234", "https://example.com/"));
    }

    #[test]
    fn ftp_and_file_are_external() {
        assert!(is_external_url("ftp://files.example.com/x", "app://my-app/"));
        assert!(is_external_url("file:///etc/hosts", "app://my-app/"));
    }

    // --- target_opens_new_tab --------------------------------------------

    #[test]
    fn blank_target_opens_new_tab() {
        assert!(target_opens_new_tab(Some("_blank")));
        assert!(target_opens_new_tab(Some("_BLANK")));
    }

    #[test]
    fn non_blank_targets_do_not_open_new_tab() {
        assert!(!target_opens_new_tab(Some("_self")));
        assert!(!target_opens_new_tab(Some("_parent")));
        assert!(!target_opens_new_tab(Some("_top")));
        assert!(!target_opens_new_tab(Some("myframe")));
        assert!(!target_opens_new_tab(None));
    }
}