//! Collector passed to [`MudTimeline::show`](super::MudTimeline::show).

use super::TimelineItem;

/// Collector for timeline items.
pub struct TimelineUi<'a> {
    pub(crate) items: Vec<TimelineItem<'a>>,
}

impl<'a> TimelineUi<'a> {
    /// Adds an item.
    pub fn item(&mut self, item: TimelineItem<'a>) {
        self.items.push(item);
    }

    /// Adds a plain-text item.
    pub fn text(&mut self, text: impl Into<String>) {
        self.items.push(TimelineItem::text(text));
    }

    /// Number of items collected so far.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether no items have been collected.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_insertion_order() {
        let mut c = TimelineUi { items: Vec::new() };
        c.text("first");
        c.item(TimelineItem::new(|_| {}));
        c.text("third");
        assert_eq!(c.len(), 3);
    }
}