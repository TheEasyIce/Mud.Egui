//! `NavLinkMatch` — how a navigation link decides whether it is active.
//!
//! Port of `Microsoft.AspNetCore.Components.Routing.NavLinkMatch`, which
//! MudBlazor's `MudNavLink` exposes as its `Match` parameter (MudBlazor 8.13.0).

/// Specifies when a navigation link is highlighted.
///
/// Mirrors ASP.NET Core's `NavLinkMatch` (the type MudBlazor re-exports).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NavLinkMatch {
    /// The link is active when the current path **starts with** the `href`.
    ///
    /// Matches `/orders` for the current path `/orders/42`, but does **not**
    /// match `/orders-history` — the boundary check requires the next
    /// character to be `/`, `?`, `#`, or end-of-string.
    ///
    /// This is ASP.NET Core's default.
    #[default]
    Prefix,

    /// The link is active only when the current path **exactly equals** the
    /// `href` (ignoring any query or fragment on the current path).
    All,
}

impl NavLinkMatch {
    /// Human-readable name matching the C# enum member.
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::Prefix => "Prefix",
            Self::All => "All",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_prefix() {
        assert_eq!(NavLinkMatch::default(), NavLinkMatch::Prefix);
    }

    #[test]
    fn name_matches_variant() {
        assert_eq!(NavLinkMatch::Prefix.name(), "Prefix");
        assert_eq!(NavLinkMatch::All.name(), "All");
    }
}