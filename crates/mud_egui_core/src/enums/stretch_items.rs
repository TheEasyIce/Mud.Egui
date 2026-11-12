//! Which of a stack's items should expand to absorb the leftover main-axis
//! space.
//!
//! Port of `MudBlazor.StretchItems` (MudBlazor 8.13.0, introduced by
//! `MudBlazor#8545`). Only meaningful through `MudStack::show_items`: the
//! stack divides whatever main-axis room is left over, after the gap and every
//! item's natural size, evenly among the flagged positions, and hands each
//! flagged item's `Ui` that much extra room before drawing it. What a flagged
//! item does with the extra room is up to the child itself — a component that
//! fills `ui.available_width()` / `available_height()` visibly grows; a
//! fixed-size widget simply ignores the slack, exactly as in MudBlazor.

/// Which items of a `MudStack` absorb leftover main-axis space.
///
/// Mirrors `MudBlazor.StretchItems`. `None` is the default and leaves the
/// stack's layout to `Justify` alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StretchItems {
    /// No item receives extra space (the default).
    #[default]
    None,
    /// Only the first item grows.
    Start,
    /// Only the last item grows.
    End,
    /// The first and last items each grow by half the leftover space.
    StartAndEnd,
    /// Every item except the first and last grows, sharing the leftover space
    /// evenly. With zero, one or two items this is equivalent to `None`
    /// (there is no middle).
    Middle,
    /// Every item grows, sharing the leftover space evenly.
    All,
}

impl StretchItems {
    /// Every variant, in MudBlazor's declaration order.
    pub const ALL: [StretchItems; 6] = [
        StretchItems::None,
        StretchItems::Start,
        StretchItems::End,
        StretchItems::StartAndEnd,
        StretchItems::Middle,
        StretchItems::All,
    ];

    /// Lower-case name, useful for logging and test ids.
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Start => "start",
            Self::End => "end",
            Self::StartAndEnd => "start-end",
            Self::Middle => "middle",
            Self::All => "all",
        }
    }

    /// Whether this policy flags any item at all for the given item count.
    ///
    /// `Middle` is a no-op for 0, 1 or 2 items; every other non-`None` policy
    /// flags at least one item for any count `>= 1`. Useful for callers that
    /// want to short-circuit work before consulting the actual target indices.
    #[inline]
    pub const fn flags_any(self, item_count: usize) -> bool {
        match self {
            Self::None => false,
            Self::Middle => item_count >= 3,
            _ => item_count >= 1,
        }
    }
}

impl core::fmt::Display for StretchItems {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        assert_eq!(StretchItems::default(), StretchItems::None);
    }

    #[test]
    fn all_has_six_unique_variants() {
        assert_eq!(StretchItems::ALL.len(), 6);
        for (i, a) in StretchItems::ALL.iter().enumerate() {
            for b in &StretchItems::ALL[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    #[test]
    fn names_are_lowercase_and_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for s in StretchItems::ALL {
            assert!(seen.insert(s.name()), "duplicate name: {}", s.name());
            assert_eq!(s.name(), s.name().to_lowercase());
        }
    }

    #[test]
    fn display_matches_name() {
        for s in StretchItems::ALL {
            assert_eq!(s.to_string(), s.name());
        }
    }

    #[test]
    fn flags_any_matches_policy_semantics() {
        // None never flags anything.
        for n in 0..5 {
            assert!(!StretchItems::None.flags_any(n));
        }
        // Middle needs at least three items to have a middle.
        assert!(!StretchItems::Middle.flags_any(0));
        assert!(!StretchItems::Middle.flags_any(1));
        assert!(!StretchItems::Middle.flags_any(2));
        assert!(StretchItems::Middle.flags_any(3));
        assert!(StretchItems::Middle.flags_any(4));
        // Everything else flags at least one item for any non-empty stack.
        for policy in [
            StretchItems::Start,
            StretchItems::End,
            StretchItems::StartAndEnd,
            StretchItems::All,
        ] {
            assert!(!policy.flags_any(0));
            assert!(policy.flags_any(1));
            assert!(policy.flags_any(2));
        }
    }
}