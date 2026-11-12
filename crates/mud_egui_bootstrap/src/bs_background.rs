//! Bootstrap-style `bg-*` utilities resolved live against the active theme.
//!
//! Avalonia/C# relies on event watchers (`CollectionChanged`).
//! In egui, we use `ComponentBase`'s `classes_version` to cache the result
//! and avoid parsing string classes 60 times a second.

use mud_egui_core::base::component_base::ComponentBase;

// =============================================================================
//  1. The Parser
// =============================================================================
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BgClass {
    Literal(egui::Color32),
    Resource { key: String, is_hover: bool },
}

/// Parses "bg-…". Returns None if it's not a bg class.
pub fn try_parse(cls: &str) -> Option<BgClass> {
    let payload = cls.strip_prefix("bg-")?;
    if payload.is_empty() {
        return None;
    }

    // 1. Check literals
    match payload {
        "transparent" => return Some(BgClass::Literal(egui::Color32::TRANSPARENT)),
        "white" => return Some(BgClass::Literal(egui::Color32::WHITE)),
        "black" => return Some(BgClass::Literal(egui::Color32::BLACK)),
        "body" => {
            return Some(BgClass::Resource {
                key: "MudPaletteBackground".to_owned(),
                is_hover: false,
            })
        }
        _ => {}
    }

    // 2. Check hover suffix
    let (token, is_hover) = if let Some(stripped) = payload.strip_suffix("-hover") {
        (stripped, true)
    } else {
        (payload, false)
    };

    if token.is_empty() {
        return None;
    }

    // 3. Build the Key ("appbar-background" -> "MudPaletteAppbarBackground")
    let mut key = String::with_capacity("MudPalette".len() + token.len() + 10);
    key.push_str("MudPalette");

    let mut uppercase_next = true;
    for c in token.chars() {
        if c == '-' {
            uppercase_next = true;
            continue;
        }
        if !c.is_ascii_alphanumeric() {
            return None; // Reject garbage
        }
        if uppercase_next {
            key.push(c.to_ascii_uppercase());
            uppercase_next = false;
        } else {
            key.push(c);
        }
    }

    if is_hover {
        key.push_str("HoverBrush");
    }

    Some(BgClass::Resource { key, is_hover })
}

// =============================================================================
//  2. The Cache
// =============================================================================
/// Holds the parsed result and the version it was parsed at.
#[derive(Default)]
pub struct BgCache {
    version: u64,
    // We cache the parsed "intent", NOT the final color.
    // This way, if your theme changes, the cache doesn't need to invalidate!
    parsed_class: Option<BgClass>,
}

impl BgCache {
    /// Resolves the background class from the ComponentBase, updating the cache if needed.
    /// Returns the parsed class (Literal or Resource).
    /// It is up to the caller to resolve `Resource` keys via `mud_egui_theming`.
    pub fn resolve(&mut self, base: &ComponentBase) -> Option<BgClass> {
        // 1. Check if classes changed (cheap u64 comparison)
        if self.version == base.classes_version() {
            return self.parsed_class.clone();
        }

        // 2. Classes changed! Parse them.
        let mut result = None;

        // "The last bg-* class written wins."
        // We iterate forward, so the last one we find overwrites the previous.
        for cls in base.class_names() {
            if let Some(bg) = try_parse(cls) {
                result = Some(bg);
            }
        }

        // 3. Save to cache
        self.version = base.classes_version();
        self.parsed_class = result.clone();

        result
    }
}

// =============================================================================
//  3. Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use egui::Color32;

    #[test]
    fn test_parse_literals() {
        assert_eq!(try_parse("bg-white"), Some(BgClass::Literal(Color32::WHITE)));
        assert_eq!(try_parse("bg-transparent"), Some(BgClass::Literal(Color32::TRANSPARENT)));
        assert_eq!(try_parse("bg-unknown"), None); // Not a literal, not a resource
    }

    #[test]
    fn test_parse_resources() {
        let parsed = try_parse("bg-primary").unwrap();
        assert_eq!(parsed, BgClass::Resource { key: "MudPalettePrimary".to_owned(), is_hover: false });

        let parsed_hover = try_parse("bg-appbar-background-hover").unwrap();
        assert_eq!(parsed_hover, BgClass::Resource { key: "MudPaletteAppbarBackgroundHoverBrush".to_owned(), is_hover: true });
    }

    #[test]
    fn test_last_class_wins() {
        let mut base = ComponentBase::new();
        let mut cache = BgCache::default();

        base.set_classes(vec!["bg-primary", "bg-white"]);
        assert_eq!(cache.resolve(&base), Some(BgClass::Literal(Color32::WHITE)));

        base.add_class("bg-black");
        assert_eq!(cache.resolve(&base), Some(BgClass::Literal(Color32::BLACK)));
    }
}