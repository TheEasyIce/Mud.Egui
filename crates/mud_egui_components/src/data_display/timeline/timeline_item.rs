//! Data model for one timeline entry (`MudTimelineItem` equivalent).

use egui::Ui;
use mud_egui_core::enums::{Color, Size, TimelineAlign, Variant};
use mud_egui_icons_material::filled;

/// One item of a [`MudTimeline`](super::MudTimeline).
///
/// Mirrors `MudTimelineItem` in MudBlazor 8.13.0:
///
/// | MudBlazor parameter | Rust method | Default |
/// |---------------------|-------------|---------|
/// | `Color`             | [`TimelineItem::color`] | [`Color::Default`] |
/// | `Variant`           | [`TimelineItem::variant`] | [`Variant::Outlined`] |
/// | `Size`              | [`TimelineItem::size`] | [`Size::Small`] |
/// | `HideDot`           | [`TimelineItem::hide_dot`] | `false` |
/// | `Icon`              | [`TimelineItem::icon`] | `None` |
/// | `ItemDot`           | [`TimelineItem::dot`] | `None` |
/// | `ItemContent`       | [`TimelineItem::new`] / [`TimelineItem::content`] | required |
/// | `ItemOpposite`      | [`TimelineItem::opposite`] | `None` |
/// | `TimelineAlign`     | [`TimelineItem::timeline_align`] | `None` (inherit) |
/// | `DotStyle` / `DotClass` | **not ported** (no CSS strings) | — |
///
/// `ItemOpposite` is only rendered when the timeline's
/// [`TimelinePosition`](mud_egui_core::enums::TimelinePosition) is
/// `Alternate`, matching MudBlazor issue #11799.
#[must_use = "TimelineItem does nothing until you add it to a TimelineUi"]
pub struct TimelineItem<'a> {
    /// Dot colour (`Color`, default `Default`).
    pub color: Color,
    /// Dot variant (`Variant`, default `Outlined`).
    pub variant: Variant,
    /// Dot size preset (`Size`, default `Small`).
    pub size: Size,
    /// Hides the dot while keeping the connector line (`HideDot`).
    pub hide_dot: bool,
    /// Icon drawn inside the dot (`Icon`).
    pub icon: Option<String>,
    /// Custom dot renderer (`ItemDot`). Overrides the default circle.
    pub dot: Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    /// Main content renderer (`ItemContent`).
    pub content: Box<dyn FnMut(&mut Ui) + 'a>,
    /// Opposite-side content (`ItemOpposite`); only rendered in
    /// `Alternate` position.
    pub opposite: Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    /// Per-item alignment override (`TimelineAlign`); `None` inherits the
    /// timeline's value.
    pub timeline_align: Option<TimelineAlign>,
}

impl<'a> TimelineItem<'a> {
    /// Creates an item with the given content and MudBlazor defaults.
    pub fn new(content: impl FnMut(&mut Ui) + 'a) -> Self {
        Self {
            color: Color::Default,
            variant: Variant::Outlined,
            size: Size::Small,
            hide_dot: false,
            icon: None,
            dot: None,
            content: Box::new(content),
            opposite: None,
            timeline_align: None,
        }
    }

    /// Creates an item from plain text (rendered through
    /// [`MudText`](crate::MudText)).
    pub fn text(text: impl Into<String>) -> Self {
        let text = text.into();
        Self::new(move |ui| {
            ui.add(crate::MudText::new(text.clone()));
        })
    }

    /// Sets the dot colour (`Color`).
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Sets the dot variant (`Variant`).
    #[inline]
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    /// Sets the dot size (`Size`).
    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    /// Hides the dot (`HideDot`).
    #[inline]
    pub fn hide_dot(mut self, b: bool) -> Self {
        self.hide_dot = b;
        self
    }

    /// Sets an icon inside the dot (`Icon`).
    #[inline]
    pub fn icon(mut self, svg: impl Into<String>) -> Self {
        self.icon = Some(svg.into());
        self
    }

    /// Sets a custom dot renderer (`ItemDot`).
    #[inline]
    pub fn dot(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self {
        self.dot = Some(Box::new(f));
        self
    }

    /// Replaces the main content (`ItemContent`).
    #[inline]
    pub fn content(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self {
        self.content = Box::new(f);
        self
    }

    /// Sets the opposite-side content (`ItemOpposite`).
    #[inline]
    pub fn opposite(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self {
        self.opposite = Some(Box::new(f));
        self
    }

    /// Overrides the timeline's `TimelineAlign` for this item.
    #[inline]
    pub fn timeline_align(mut self, a: TimelineAlign) -> Self {
        self.timeline_align = Some(a);
        self
    }

    /// Convenience: use a Material check icon inside the dot.
    #[inline]
    pub fn with_check_icon(mut self) -> Self {
        self.icon = Some(filled::CHECK.to_owned());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_mudblazor() {
        let it = TimelineItem::new(|_| {});
        assert!(matches!(it.color, Color::Default));
        assert!(matches!(it.variant, Variant::Outlined));
        assert!(matches!(it.size, Size::Small));
        assert!(!it.hide_dot);
        assert!(it.icon.is_none());
        assert!(it.dot.is_none());
        assert!(it.opposite.is_none());
        assert!(it.timeline_align.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let it = TimelineItem::new(|_| {})
            .color(Color::Success)
            .variant(Variant::Filled)
            .size(Size::Large)
            .hide_dot(true)
            .icon("M0 0")
            .dot(|_| {})
            .content(|_| {})
            .opposite(|_| {})
            .timeline_align(TimelineAlign::Start);
        assert!(matches!(it.color, Color::Success));
        assert!(matches!(it.variant, Variant::Filled));
        assert!(matches!(it.size, Size::Large));
        assert!(it.hide_dot);
        assert_eq!(it.icon.as_deref(), Some("M0 0"));
        assert!(it.dot.is_some());
        assert!(it.opposite.is_some());
        assert_eq!(it.timeline_align, Some(TimelineAlign::Start));
    }

    #[test]
    fn text_constructor_installs_content() {
        let it = TimelineItem::text("hello");
        assert!(it.opposite.is_none());
        assert!(it.dot.is_none());
    }
}