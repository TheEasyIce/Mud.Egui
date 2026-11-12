//! Data model for one tree node ([`MudTreeViewItem`](super::MudTreeViewItem)
//! equivalent).

use egui::Ui;
use mud_egui_core::enums::{Color, Typo};
use mud_egui_icons_material::filled;

/// One node of a [`MudTreeView`](super::MudTreeView).
///
/// Build a tree with [`TreeItem::new`] and chain `.child(..)` for descendants.
/// The node is consumed by rendering; the `on_click` / `on_double_click`
/// closures are `FnMut` so the caller can capture state.
#[must_use = "TreeItem does nothing until you add it to a TreeUi"]
pub struct TreeItem<'a, T> {
    // ── Data ───────────────────────────────────────────────────────────────
    pub(crate) value: T,
    pub(crate) text: Option<String>,
    pub(crate) children: Vec<TreeItem<'a, T>>,

    // ── Leading icon ───────────────────────────────────────────────────────
    pub(crate) icon: Option<String>,
    pub(crate) icon_color: Color,
    pub(crate) icon_expanded: Option<String>,

    // ── End ────────────────────────────────────────────────────────────────
    pub(crate) end_icon: Option<String>,
    pub(crate) end_icon_color: Color,
    pub(crate) end_text: Option<String>,
    pub(crate) end_text_typo: Typo,

    // ── Expand button ──────────────────────────────────────────────────────
    pub(crate) expand_button_icon: String,
    pub(crate) expand_button_icon_color: Color,

    // ── Behavior ───────────────────────────────────────────────────────────
    pub(crate) expanded_init: Option<bool>,
    pub(crate) can_expand: bool,
    pub(crate) disabled: bool,
    pub(crate) read_only: bool,
    pub(crate) visible: bool,

    // ── Custom rendering / callbacks ───────────────────────────────────────
    pub(crate) body: Option<Box<dyn FnMut(&mut Ui, bool) + 'a>>,
    pub(crate) on_click: Option<Box<dyn FnMut(&T) + 'a>>,
    pub(crate) on_double_click: Option<Box<dyn FnMut(&T) + 'a>>,
}

impl<'a, T> TreeItem<'a, T> {
    /// Creates a node for `value` with no caption.
    pub fn new(value: T) -> Self {
        Self {
            value,
            text: None,
            children: Vec::new(),
            icon: None,
            icon_color: Color::Inherit,
            icon_expanded: None,
            end_icon: None,
            end_icon_color: Color::Inherit,
            end_text: None,
            end_text_typo: Typo::Body1,
            expand_button_icon: filled::CHEVRON_RIGHT.to_owned(),
            expand_button_icon_color: Color::Default,
            expanded_init: None,
            can_expand: true,
            disabled: false,
            read_only: false,
            visible: true,
            body: None,
            on_click: None,
            on_double_click: None,
        }
    }

    /// Creates a node with a caption.
    pub fn text(value: T, text: impl Into<String>) -> Self {
        Self::new(value).with_text(text)
    }

    /// Sets the caption (`Text`). When omitted, the value's `Debug` text is
    /// used as a fallback.
    #[inline]
    pub fn with_text(mut self, s: impl Into<String>) -> Self {
        self.text = Some(s.into());
        self
    }

    /// Adds a child node (`ChildContent`).
    #[inline]
    pub fn child(mut self, child: TreeItem<'a, T>) -> Self {
        self.children.push(child);
        self
    }

    /// Adds several child nodes.
    pub fn children<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = TreeItem<'a, T>>,
    {
        self.children.extend(iter);
        self
    }

    #[inline]
    pub fn icon(mut self, svg: impl Into<String>) -> Self {
        self.icon = Some(svg.into());
        self
    }

    #[inline]
    pub fn icon_color(mut self, c: Color) -> Self {
        self.icon_color = c;
        self
    }

    #[inline]
    pub fn icon_expanded(mut self, svg: impl Into<String>) -> Self {
        self.icon_expanded = Some(svg.into());
        self
    }

    #[inline]
    pub fn end_icon(mut self, svg: impl Into<String>) -> Self {
        self.end_icon = Some(svg.into());
        self
    }

    #[inline]
    pub fn end_icon_color(mut self, c: Color) -> Self {
        self.end_icon_color = c;
        self
    }

    #[inline]
    pub fn end_text(mut self, s: impl Into<String>) -> Self {
        self.end_text = Some(s.into());
        self
    }

    #[inline]
    pub fn end_text_typo(mut self, t: Typo) -> Self {
        self.end_text_typo = t;
        self
    }

    #[inline]
    pub fn expand_button_icon(mut self, svg: impl Into<String>) -> Self {
        self.expand_button_icon = svg.into();
        self
    }

    #[inline]
    pub fn expand_button_icon_color(mut self, c: Color) -> Self {
        self.expand_button_icon_color = c;
        self
    }

    /// Sets the initial expansion state (`Expanded`). Once seeded, clicks
    /// toggle the value from [`TreeState`](super::TreeState).
    #[inline]
    pub fn expanded(mut self, b: bool) -> Self {
        self.expanded_init = Some(b);
        self
    }

    #[inline]
    pub fn can_expand(mut self, b: bool) -> Self {
        self.can_expand = b;
        self
    }

    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    #[inline]
    pub fn read_only(mut self, b: bool) -> Self {
        self.read_only = b;
        self
    }

    #[inline]
    pub fn visible(mut self, b: bool) -> Self {
        self.visible = b;
        self
    }

    /// Custom row body (`BodyContent`). Replaces the icon + text + end area;
    /// the expand button and checkbox still render. The closure receives
    /// `is_selected`.
    #[inline]
    pub fn body(mut self, f: impl FnMut(&mut Ui, bool) + 'a) -> Self {
        self.body = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn on_click(mut self, f: impl FnMut(&T) + 'a) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    #[inline]
    pub fn on_double_click(mut self, f: impl FnMut(&T) + 'a) -> Self {
        self.on_double_click = Some(Box::new(f));
        self
    }
}