use egui::Color32;

/// Highest elevation (shadow depth). MudBlazor uses 0..=24 as well.
pub const MAX_ELEVATION: u8 = 24;

// -----------------------------------------------------------------------------
//  Ripple settings
// -----------------------------------------------------------------------------

/// Settings for the click ripple effect.
///
/// This holds only the CONFIGURATION ("is it on? which color?"). The actual
/// animation needs per-frame state and will live in a ripple widget/helper.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Ripple {
    /// Is the ripple effect turned on? Defaults to `false` (off).
    pub enabled: bool,

    /// Fixed ripple color. `None` means "pick automatically" (usually the
    /// widget's text color at low opacity).
    pub color: Option<Color32>,
}

// -----------------------------------------------------------------------------
//  ComponentBase
// -----------------------------------------------------------------------------

/// Shared data for every classed widget. Embed it as a field, configure it with
/// the builder methods, and let the `bs_*` / `tw_*` modules read it.
///
/// ```ignore
/// let base = ComponentBase::new()
///     .classes(["bg-primary", "rounded"])
///     .elevation(4)
///     .ripple(true);
/// ```
#[derive(Clone, Debug, Default)]
pub struct ComponentBase {
    /// Style classes in the order they were written. Order matters because for
    /// equal specificity "the last class written wins" (like CSS).
    classes: Vec<String>,

    /// Goes up by one every time `classes` actually changes. Used by other
    /// files as a cheap "did anything change?" check.
    classes_version: u64,

    /// Shadow depth, always within 0..=MAX_ELEVATION.
    elevation: u8,

    /// Ripple configuration.
    ripple: Ripple,
}

impl ComponentBase {
    // =========================================================================
    //  Creating and configuring (builder style)
    // =========================================================================

    /// An empty base: no classes, elevation 0, ripple off.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// REPLACE all classes with the given ones.
    ///
    /// Each entry may itself contain several space-separated classes, so
    /// `.classes(["bg-primary p-2"])` works too.
    #[must_use]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.set_classes(classes);
        self
    }

    /// ADD one class (or several space-separated ones) to the existing list.
    #[must_use]
    pub fn class(mut self, class: impl AsRef<str>) -> Self {
        self.add_class(class);
        self
    }

    /// Set the elevation (shadow depth). Values above 24 are clamped to 24.
    #[must_use]
    pub fn elevation(mut self, elevation: u8) -> Self {
        self.set_elevation(elevation);
        self
    }

    /// Turn the ripple effect on or off.
    #[must_use]
    pub fn ripple(mut self, enabled: bool) -> Self {
        self.ripple.enabled = enabled;
        self
    }

    /// Use a fixed ripple color (does not turn the ripple on by itself).
    #[must_use]
    pub fn ripple_color(mut self, color: Color32) -> Self {
        self.ripple.color = Some(color);
        self
    }

    // =========================================================================
    //  Changing after creation (mutating style)
    // =========================================================================

    /// Replace all classes. Only bumps the version if the result differs.
    pub fn set_classes<S: AsRef<str>>(&mut self, classes: impl IntoIterator<Item = S>) {
        let mut new_list: Vec<String> = Vec::new();
        for entry in classes {
            push_unique(&mut new_list, entry.as_ref());
        }
        if new_list != self.classes {
            self.classes = new_list;
            self.classes_version += 1;
        }
    }

    /// Add class(es). Returns `true` if something was actually added, `false`
    /// if every class was already present.
    pub fn add_class(&mut self, class: impl AsRef<str>) -> bool {
        let before = self.classes.len();
        push_unique(&mut self.classes, class.as_ref());
        let changed = self.classes.len() != before;
        if changed {
            self.classes_version += 1;
        }
        changed
    }

    /// Remove a class. Returns `true` if it was present.
    pub fn remove_class(&mut self, class: &str) -> bool {
        let before = self.classes.len();
        self.classes.retain(|c| c != class);
        let changed = self.classes.len() != before;
        if changed {
            self.classes_version += 1;
        }
        changed
    }

    /// Add the class if absent, remove it if present. Handy for state classes
    /// like "selected". Returns `true` if the class is present afterwards.
    pub fn toggle_class(&mut self, class: &str) -> bool {
        if self.remove_class(class) {
            false
        } else {
            self.add_class(class);
            true
        }
    }

    /// Set the elevation, clamped to 0..=24.
    pub fn set_elevation(&mut self, elevation: u8) {
        self.elevation = elevation.min(MAX_ELEVATION);
    }

    /// Replace the whole ripple configuration.
    pub fn set_ripple(&mut self, ripple: Ripple) {
        self.ripple = ripple;
    }

    // =========================================================================
    //  Reading (allocation-free, called every frame)
    // =========================================================================

    /// The classes, in the order written. Read-only view.
    #[inline]
    pub fn class_names(&self) -> &[String] {
        &self.classes
    }

    /// Is this exact class present? Cost: a short scan, no allocation.
    #[inline]
    pub fn has_class(&self, class: &str) -> bool {
        self.classes.iter().any(|c| c == class)
    }

    /// Changes whenever the classes change. See the guide at the top for how
    /// other files use this to skip re-parsing.
    #[inline]
    pub fn classes_version(&self) -> u64 {
        self.classes_version
    }

    /// Shadow depth, 0..=24.
    #[inline]
    pub fn elevation_value(&self) -> u8 {
        self.elevation
    }

    /// The ripple settings.
    #[inline]
    pub fn ripple_settings(&self) -> Ripple {
        self.ripple
    }

    /// Shortcut: is the ripple on?
    #[inline]
    pub fn ripple_enabled(&self) -> bool {
        self.ripple.enabled
    }
}

// -----------------------------------------------------------------------------
//  Private helper
// -----------------------------------------------------------------------------

/// Split `input` on whitespace and append each class not already in `list`.
fn push_unique(list: &mut Vec<String>, input: &str) {
    for word in input.split_whitespace() {
        if !list.iter().any(|c| c == word) {
            list.push(word.to_owned());
        }
    }
}

// -----------------------------------------------------------------------------
//  Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_space_separated_classes_and_ignores_duplicates() {
        let base = ComponentBase::new().class("bg-primary p-2").class("p-2");
        assert_eq!(base.class_names(), ["bg-primary", "p-2"]);
    }

    #[test]
    fn version_only_changes_when_classes_change() {
        let mut base = ComponentBase::new().class("a");
        let v = base.classes_version();

        assert!(!base.add_class("a"));
        assert_eq!(base.classes_version(), v);

        assert!(base.add_class("b"));
        assert_eq!(base.classes_version(), v + 1);
    }

    #[test]
    fn toggle_adds_then_removes() {
        let mut base = ComponentBase::new();
        assert!(base.toggle_class("selected"));
        assert!(base.has_class("selected"));
        assert!(!base.toggle_class("selected"));
        assert!(!base.has_class("selected"));
    }

    #[test]
    fn elevation_is_clamped() {
        assert_eq!(ComponentBase::new().elevation(200).elevation_value(), MAX_ELEVATION);
    }
}