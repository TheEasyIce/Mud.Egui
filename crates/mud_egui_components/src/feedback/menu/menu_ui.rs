//! Collector passed to [`MudMenu::show`](super::MudMenu::show).

use super::menu_entry::MenuEntry;
use super::{MenuItem, MenuSeparator};

/// Collector for menu entries.
pub struct MenuUi<'a> {
    pub(crate) entries: Vec<MenuEntry<'a>>,
}

impl<'a> MenuUi<'a> {
    /// Adds a clickable menu item.
    pub fn item(&mut self, item: MenuItem<'a>) {
        self.entries.push(MenuEntry::Item(item));
    }

    /// Adds a text item (convenience).
    pub fn text_item(&mut self, text: impl Into<String>) {
        self.entries.push(MenuEntry::Item(MenuItem::new(text)));
    }

    /// Adds a link item (convenience).
    pub fn link_item(&mut self, text: impl Into<String>, href: impl Into<String>) {
        self.entries.push(MenuEntry::Item(MenuItem::link(text, href)));
    }

    /// Adds a separator.
    pub fn separator(&mut self) {
        self.entries.push(MenuEntry::Separator(MenuSeparator::new()));
    }

    /// Number of collected entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing has been collected.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_order_and_kind() {
        let mut c: MenuUi<'static> = MenuUi { entries: Vec::new() };
        c.text_item("A");
        c.separator();
        c.link_item("B", "/b");
        assert_eq!(c.len(), 3);
        assert!(matches!(c.entries[0], MenuEntry::Item(_)));
        assert!(matches!(c.entries[1], MenuEntry::Separator(_)));
        assert!(matches!(c.entries[2], MenuEntry::Item(_)));
    }
}