//! `MudList` — a scrollable, selectable vertical list with nested lists,
//! subheaders, icons, avatars, and per-item callbacks.
//!
//! Port of `MudList<T>.razor.cs`, `MudListItem<T>.razor.cs`, and
//! `MudListSubheader<T>.razor.cs` (MudBlazor 8.13.0).
//!
//! # Parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `SelectionMode`     | [`MudList::selection_mode`] | [`SelectionMode::Single`] |
//! | `Color`             | [`MudList::color`] | [`Color::Primary`] |
//! | `CheckBoxColor`     | [`MudList::checkbox_color`] | [`Color::Default`] |
//! | `CheckedIcon`       | [`MudList::checked_icon`] | `CHECK_BOX` |
//! | `UncheckedIcon`     | [`MudList::unchecked_icon`] | `CHECK_BOX_OUTLINE_BLANK` |
//! | `Dense`             | [`MudList::dense`] | `false` |
//! | `Disabled`          | [`MudList::disabled`] | `false` |
//! | `ReadOnly`          | [`MudList::readonly`] | `false` |
//! | `Gutters`           | [`MudList::gutters`] | `true` |
//! | `Padding`           | [`MudList::padding`] | `false` |
//! | `SelectedValue`     | [`ListState::selected_value`] | `None` |
//! | `SelectedValues`    | [`ListState::selected_values`] | `[]` |
//! | `Comparer`          | *(Rust `T: PartialEq`)* | — |
//! | `ChildContent`      | closure to [`MudList::show`] | required |
//!
//! `MudListItem` and `MudListSubheader` mirror their MudBlazor counterparts;
//! see their own doc comments for the full table.
//!
//! # Per-frame model
//!
//! Items are collected into a temporary `Vec<ListEntry>` on every frame by
//! the closure passed to [`MudList::show`], rendered, and dropped. Selection
//! and expansion live in [`ListState<T>`], which the caller owns across
//! frames — the egui equivalent of Blazor's `SelectedValue`,
//! `SelectedValues`, and `Expanded` bindings.
//!
//! # Text and icons
//!
//! Every visible string is rendered through [`MudText`], and every icon
//! through [`MudIcon`]. No raw `egui::Label` or `egui::Image` is used, so
//! theme and typography updates propagate automatically.
//!
//! # Selection
//!
//! [`apply_selection`] mirrors MudBlazor's `SelectionMode`:
//!
//! | Mode | Click unselected | Click selected |
//! |------|------------------|----------------|
//! | `Single` | selects it | no change |
//! | `Toggle` | selects it | deselects it |
//! | `Multi`  | adds it | removes it |
//!
//! # Ripple
//!
//! Item ripples are memory-backed and keyed by `ui.scope_id().with(path)`.
//! The start time is read **before** acquiring the memory lock, avoiding the
//! re-entrant `RwLock` deadlock documented in `mud_toggle_group`.

use std::any::Any;
use std::sync::Arc;

use egui::{
    vec2, Color32, CornerRadius, Layout, Margin, Pos2, Rect, Response, Sense, Ui, UiBuilder,
};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{Color, SelectionMode, Size, StyleFlavor, Typo};
use mud_egui_icons_material::filled;
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::typography::mud_text::MudText;

use super::list_entry::ListEntry;

/// Base rem size passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Default height of a non-dense list item.
pub const ITEM_HEIGHT: f32 = 48.0;
/// Height of a dense list item.
pub const DENSE_ITEM_HEIGHT: f32 = 32.0;
/// Default height of a subheader row.
pub const SUBHEADER_HEIGHT: f32 = 36.0;
/// Height of a dense subheader row.
pub const DENSE_SUBHEADER_HEIGHT: f32 = 28.0;
/// Horizontal gutter on each side of an item, when gutters are enabled.
pub const GUTTER_PX: f32 = 12.0;
/// Extra indent applied by [`MudListItem::inset`].
pub const INSET_PX: f32 = 16.0;
/// Indent per nesting level.
pub const INDENT_PER_DEPTH: f32 = 16.0;
/// Side length of the selection checkbox.
const CHECKBOX_SIZE: f32 = 20.0;
/// Side length of an avatar slot.
const AVATAR_SIZE: f32 = 40.0;
/// Side length of a dense avatar slot.
const DENSE_AVATAR_SIZE: f32 = 32.0;
/// Corner radius of the selected / hovered row background.
const ROW_RADIUS: u8 = 4;
/// Duration of the click ripple.
const RIPPLE_DURATION: f32 = 0.35;

// ---------------------------------------------------------------------------
// ListState
// ---------------------------------------------------------------------------

/// Caller-owned selection and expansion state.
///
/// This is the egui equivalent of MudBlazor's two-way bindings
/// `SelectedValue`, `SelectedValues`, and `Expanded`.
///
/// * [`selected_value`](Self::selected_value) is read/written when
///   [`SelectionMode::Single`] or [`SelectionMode::Toggle`] is active.
/// * [`selected_values`](Self::selected_values) is read/written when
///   [`SelectionMode::Multi`] is active.
/// * [`expanded`](Self::expanded) stores `(item key, is_expanded)` pairs for
///   nested lists.
#[derive(Debug, Clone)]
pub struct ListState<T> {
    /// Single selection (`Single` / `Toggle` mode).
    pub selected_value: Option<T>,
    /// Multi selection (`Multi` mode).
    pub selected_values: Vec<T>,
    /// Expansion state for nested items, keyed by item id (or path).
    pub expanded: Vec<(String, bool)>,
}

impl<T> Default for ListState<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ListState<T> {
    /// Creates empty state: nothing selected, nothing expanded.
    pub fn new() -> Self {
        Self {
            selected_value: None,
            selected_values: Vec::new(),
            expanded: Vec::new(),
        }
    }

    /// Whether `value` is selected under `mode`.
    pub fn is_selected(&self, value: &T, mode: SelectionMode) -> bool
    where
        T: PartialEq,
    {
        match mode {
            SelectionMode::Multi => self.selected_values.iter().any(|v| v == value),
            _ => self.selected_value.as_ref() == Some(value),
        }
    }

    /// Reads the stored expansion state for `key`, or `None` if it has never
    /// been initialised.
    pub fn is_expanded(&self, key: &str) -> Option<bool> {
        self.expanded
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| *v)
    }

    /// Initialises the expansion state for `key` if it is not yet known.
    ///
    /// Called by the renderer with the item's `expanded` parameter as the
    /// initial value, matching MudBlazor's "set `Expanded` to initialise"
    /// behaviour.
    pub fn ensure_expanded(&mut self, key: &str, initial: bool) {
        if self.is_expanded(key).is_none() {
            self.expanded.push((key.to_owned(), initial));
        }
    }

    /// Flips the expansion state for `key`, creating it as `true` if it was
    /// unknown.
    pub fn toggle_expanded(&mut self, key: &str) {
        if let Some(entry) = self.expanded.iter_mut().find(|(k, _)| k == key) {
            entry.1 = !entry.1;
        } else {
            self.expanded.push((key.to_owned(), true));
        }
    }

    /// Removes every expansion entry. Useful when the item list changes
    /// shape and stale keys accumulate.
    pub fn clear_expanded(&mut self) {
        self.expanded.clear();
    }
}

// ---------------------------------------------------------------------------
// MudListSubheader
// ---------------------------------------------------------------------------

/// A non-interactive subheader row inside a [`MudList`].
///
/// Mirrors `MudListSubheader<T>`. The text is rendered through [`MudText`]
/// with the theme's secondary colour.
#[must_use = "MudListSubheader does nothing until you add it to a MudList"]
#[derive(Debug, Clone, Default)]
pub struct MudListSubheader {
    /// The header caption.
    pub text: String,
    /// Overrides the list's gutters when set.
    pub gutters: Option<bool>,
    /// Adds an extra indent.
    pub inset: bool,
}

impl MudListSubheader {
    /// Creates a subheader with the given caption.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            gutters: None,
            inset: false,
        }
    }

    /// Overrides the list's `gutters` setting for this row.
    #[inline]
    pub fn gutters(mut self, b: bool) -> Self {
        self.gutters = Some(b);
        self
    }

    /// Adds an extra indent.
    #[inline]
    pub fn inset(mut self, b: bool) -> Self {
        self.inset = b;
        self
    }
}

// ---------------------------------------------------------------------------
// MudListItem
// ---------------------------------------------------------------------------

/// A single item inside a [`MudList`].
///
/// Mirrors `MudListItem<T>`. Content precedence matches MudBlazor:
/// `child` overrides `text` + `secondary_text`; `avatar` overrides `icon`.
#[must_use = "MudListItem does nothing until you add it to a MudList"]
pub struct MudListItem<'a, T> {
    /// The value associated with this item (`Value`).
    pub value: T,
    /// Primary caption (`Text`).
    pub text: Option<String>,
    /// Secondary caption shown under `text` in a smaller size (`SecondaryText`).
    pub secondary_text: Option<String>,
    /// Leading icon (Material SVG path). Ignored when `avatar` is set (`Icon`).
    pub icon: Option<String>,
    /// Icon colour. `Default` / `Inherit` uses the selected accent or the
    /// theme text colour.
    pub icon_color: Color,
    /// Icon size preset (`IconSize`).
    pub icon_size: Size,
    /// Custom avatar renderer; overrides `icon` when set (`AvatarContent`).
    pub avatar: Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    /// Custom row content; overrides `text` + `secondary_text` when set
    /// (`ChildContent`).
    pub child: Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    /// Nested list content (`NestedList`).
    pub nested: Option<Box<dyn FnMut(&mut ListUi<'a, T>) + 'a>>,
    /// Initial expansion state (`Expanded`). `None` means "uncontrolled":
    /// [`ListState`] remembers the value across frames.
    pub expanded: Option<bool>,
    /// Overrides the list's `dense` setting.
    pub dense: Option<bool>,
    /// Disables this item only (`Disabled`).
    pub disabled: bool,
    /// Adds an extra left indent (`Inset`).
    pub inset: bool,
    /// Overrides the list's `gutters` setting.
    pub gutters: Option<bool>,
    /// URL stored for parity (`Href`). egui has no router; navigate inside
    /// `on_click`.
    pub href: Option<String>,
    /// Browser target stored for parity (`Target`). Unused in egui.
    pub target: Option<String>,
    /// Stored for parity (`ForceLoad`). Unused in egui.
    pub force_load: bool,
    /// Draws a ripple on click (`Ripple`, default `true`).
    pub ripple: bool,
    /// Click callback (`OnClick`). Fires even when `href` is set.
    pub on_click: Option<Box<dyn FnMut(&T) + 'a>>,
    /// When true, clicking only fires `on_click`; selection and expansion are
    /// skipped (`OnClickPreventDefault`).
    pub on_click_prevent_default: bool,
    /// Stable key used by [`ListState::expanded`] and ripple identity. Falls
    /// back to the item's path (index chain) when `None`.
    pub id: Option<String>,
    /// Glyph shown while expanded (`ExpandLessIcon`).
    pub expand_less_icon: String,
    /// Glyph shown while collapsed (`ExpandMoreIcon`).
    pub expand_more_icon: String,
    /// Colour of the expand glyph (`ExpandIconColor`).
    pub expand_icon_color: Color,
}

impl<'a, T> MudListItem<'a, T> {
    /// Creates an item for `value` with no caption, icon, or content.
    pub fn new(value: T) -> Self {
        Self {
            value,
            text: None,
            secondary_text: None,
            icon: None,
            icon_color: Color::Inherit,
            icon_size: Size::Medium,
            avatar: None,
            child: None,
            nested: None,
            expanded: None,
            dense: None,
            disabled: false,
            inset: false,
            gutters: None,
            href: None,
            target: None,
            force_load: false,
            ripple: true,
            on_click: None,
            on_click_prevent_default: false,
            id: None,
            expand_less_icon: filled::EXPAND_LESS.to_string(),
            expand_more_icon: filled::EXPAND_MORE.to_string(),
            expand_icon_color: Color::Default,
        }
    }

    /// Creates an item with a primary caption.
    pub fn text(value: T, text: impl Into<String>) -> Self {
        Self::new(value).with_text(text)
    }

    /// Sets the primary caption (`Text`).
    #[inline]
    pub fn with_text(mut self, s: impl Into<String>) -> Self {
        self.text = Some(s.into());
        self
    }

    /// Sets the secondary caption (`SecondaryText`).
    #[inline]
    pub fn secondary_text(mut self, s: impl Into<String>) -> Self {
        self.secondary_text = Some(s.into());
        self
    }

    /// Sets the leading icon (`Icon`).
    #[inline]
    pub fn icon(mut self, svg: impl Into<String>) -> Self {
        self.icon = Some(svg.into());
        self
    }

    /// Sets the icon colour (`IconColor`).
    #[inline]
    pub fn icon_color(mut self, c: Color) -> Self {
        self.icon_color = c;
        self
    }

    /// Sets the icon size preset (`IconSize`).
    #[inline]
    pub fn icon_size(mut self, s: Size) -> Self {
        self.icon_size = s;
        self
    }

    /// Provides a custom avatar (`AvatarContent`); overrides `icon`.
    #[inline]
    pub fn avatar(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self {
        self.avatar = Some(Box::new(f));
        self
    }

    /// Provides custom row content (`ChildContent`); overrides `text` and
    /// `secondary_text`.
    #[inline]
    pub fn child(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self {
        self.child = Some(Box::new(f));
        self
    }

    /// Provides nested items (`NestedList`).
    #[inline]
    pub fn nested(mut self, f: impl FnMut(&mut ListUi<'a, T>) + 'a) -> Self {
        self.nested = Some(Box::new(f));
        self
    }

    /// Sets the initial expansion state (`Expanded`).
    #[inline]
    pub fn expanded(mut self, b: bool) -> Self {
        self.expanded = Some(b);
        self
    }

    /// Overrides the list's `dense` setting.
    #[inline]
    pub fn dense(mut self, b: bool) -> Self {
        self.dense = Some(b);
        self
    }

    /// Disables this item.
    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// Adds an extra left indent.
    #[inline]
    pub fn inset(mut self, b: bool) -> Self {
        self.inset = b;
        self
    }

    /// Overrides the list's `gutters` setting.
    #[inline]
    pub fn gutters(mut self, b: bool) -> Self {
        self.gutters = Some(b);
        self
    }

    /// Stores a URL (`Href`). Navigate inside `on_click`.
    #[inline]
    pub fn href(mut self, s: impl Into<String>) -> Self {
        self.href = Some(s.into());
        self
    }

    /// Stores a target (`Target`). Unused in egui.
    #[inline]
    pub fn target(mut self, s: impl Into<String>) -> Self {
        self.target = Some(s.into());
        self
    }

    /// Stores `ForceLoad`. Unused in egui.
    #[inline]
    pub fn force_load(mut self, b: bool) -> Self {
        self.force_load = b;
        self
    }

    /// Enables or disables the click ripple.
    #[inline]
    pub fn ripple(mut self, b: bool) -> Self {
        self.ripple = b;
        self
    }

    /// Registers a click callback (`OnClick`).
    #[inline]
    pub fn on_click(mut self, f: impl FnMut(&T) + 'a) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    /// When true, clicks only fire `on_click` (`OnClickPreventDefault`).
    #[inline]
    pub fn on_click_prevent_default(mut self, b: bool) -> Self {
        self.on_click_prevent_default = b;
        self
    }

    /// Sets a stable key for expansion and ripple identity.
    #[inline]
    pub fn id(mut self, s: impl Into<String>) -> Self {
        self.id = Some(s.into());
        self
    }

    /// Sets the expanded glyph.
    #[inline]
    pub fn expand_less_icon(mut self, s: impl Into<String>) -> Self {
        self.expand_less_icon = s.into();
        self
    }

    /// Sets the collapsed glyph.
    #[inline]
    pub fn expand_more_icon(mut self, s: impl Into<String>) -> Self {
        self.expand_more_icon = s.into();
        self
    }

    /// Sets the expand glyph colour.
    #[inline]
    pub fn expand_icon_color(mut self, c: Color) -> Self {
        self.expand_icon_color = c;
        self
    }
}

// ---------------------------------------------------------------------------
// ListUi
// ---------------------------------------------------------------------------

/// Collector passed to [`MudList::show`]'s closure.
pub struct ListUi<'a, T> {
    entries: Vec<ListEntry<'a, T>>,
}

impl<'a, T> ListUi<'a, T> {
    /// Adds a fully configured item.
    pub fn item(&mut self, item: MudListItem<'a, T>) {
        self.entries.push(ListEntry::Item(item));
    }

    /// Adds a text item.
    pub fn text_item(&mut self, value: T, text: impl Into<String>) {
        self.item(MudListItem::text(value, text));
    }

    /// Adds a subheader row.
    pub fn subheader(&mut self, text: impl Into<String>) {
        self.entries
            .push(ListEntry::Subheader(MudListSubheader::new(text)));
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

// ---------------------------------------------------------------------------
// MudList
// ---------------------------------------------------------------------------

/// A scrollable, selectable vertical list.
#[must_use = "MudList does nothing until you call `.show(ui, state, |list| ..)`"]
pub struct MudList<'a, T> {
    /// Selection behaviour (`SelectionMode`, default `Single`).
    pub selection_mode: SelectionMode,
    /// Accent colour for selected items (`Color`, default `Primary`).
    pub color: Color,
    /// Checkbox colour in `Multi` mode (`CheckBoxColor`, default `Default`).
    pub checkbox_color: Color,
    /// Checked checkbox glyph (`CheckedIcon`).
    pub checked_icon: String,
    /// Unchecked checkbox glyph (`UncheckedIcon`).
    pub unchecked_icon: String,
    /// Compact row heights (`Dense`, default `false`).
    pub dense: bool,
    /// Disables every item (`Disabled`, default `false`).
    pub disabled: bool,
    /// Prevents selection but keeps expansion (`ReadOnly`, default `false`).
    pub readonly: bool,
    /// Left/right padding on items (`Gutters`, default `true`).
    pub gutters: bool,
    /// Vertical padding around the list (`Padding`, default `false`).
    pub padding: bool,
    /// Utility classes for the wrapper frame (`Class`).
    pub base: ComponentBase,
    /// Bootstrap or Tailwind resolver.
    pub flavor: StyleFlavor,
    /// Bootstrap resolver state.
    pub bs_style: BsStyle,
    /// Tailwind resolver state.
    pub tw_style: TwStyle,
    /// Tailwind interaction state.
    pub tw_state: TwState,
    tag_value: Option<Arc<dyn Any + Send + Sync>>,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Default for MudList<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> MudList<'a, T> {
    /// Creates a list with MudBlazor defaults.
    pub fn new() -> Self {
        Self {
            selection_mode: SelectionMode::Single,
            color: Color::Primary,
            checkbox_color: Color::Default,
            checked_icon: filled::CHECK_BOX.to_string(),
            unchecked_icon: filled::CHECK_BOX_OUTLINE_BLANK.to_string(),
            dense: false,
            disabled: false,
            readonly: false,
            gutters: true,
            padding: false,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            tw_state: TwState::default(),
            tag_value: None,
            _marker: std::marker::PhantomData,
        }
    }

    /// Sets the selection mode (`SelectionMode`, default `Single`).
    #[inline]
    pub fn selection_mode(mut self, m: SelectionMode) -> Self {
        self.selection_mode = m;
        self
    }

    /// Sets the accent colour for selected items (`Color`, default `Primary`).
    #[inline]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Sets the checkbox colour (`CheckBoxColor`).
    #[inline]
    pub fn checkbox_color(mut self, c: Color) -> Self {
        self.checkbox_color = c;
        self
    }

    /// Sets the checked glyph (`CheckedIcon`).
    #[inline]
    pub fn checked_icon(mut self, s: impl Into<String>) -> Self {
        self.checked_icon = s.into();
        self
    }

    /// Sets the unchecked glyph (`UncheckedIcon`).
    #[inline]
    pub fn unchecked_icon(mut self, s: impl Into<String>) -> Self {
        self.unchecked_icon = s.into();
        self
    }

    /// Uses compact row heights (`Dense`).
    #[inline]
    pub fn dense(mut self, b: bool) -> Self {
        self.dense = b;
        self
    }

    /// Disables every item (`Disabled`).
    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// Prevents selection but keeps expansion (`ReadOnly`).
    #[inline]
    pub fn readonly(mut self, b: bool) -> Self {
        self.readonly = b;
        self
    }

    /// Enables or disables item gutters (`Gutters`, default `true`).
    #[inline]
    pub fn gutters(mut self, b: bool) -> Self {
        self.gutters = b;
        self
    }

    /// Adds vertical padding around the list (`Padding`).
    #[inline]
    pub fn padding(mut self, b: bool) -> Self {
        self.padding = b;
        self
    }

    /// Adds utility classes to the wrapper frame.
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Selects the utility-class resolver.
    #[inline]
    pub fn flavor(mut self, f: StyleFlavor) -> Self {
        self.flavor = f;
        self
    }

    /// Attaches arbitrary user data (`Tag`).
    pub fn tag<U: Any + Send + Sync>(mut self, value: U) -> Self {
        self.tag_value = Some(Arc::new(value));
        self
    }

    /// Reads the tag back if it is of type `U`.
    pub fn tag_ref<U: Any + Send + Sync>(&self) -> Option<&U> {
        self.tag_value.as_ref()?.downcast_ref::<U>()
    }

    /// Draws the list and returns `(response, add_items_result)`.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        state: &mut ListState<T>,
        add_items: impl FnOnce(&mut ListUi<'a, T>) -> R,
    ) -> (Response, R)
    where
        T: Clone + PartialEq,
    {
        // 1. Collect entries.
        let mut list_ui = ListUi {
            entries: Vec::new(),
        };
        let user_result = add_items(&mut list_ui);
        let mut entries = list_ui.entries;

        // 2. Theme + wrapper frame.
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();
        let lookup = |key: &str| palette.resolve(key).map(Color32::from);

        self.tw_state.dark = ui.visuals().dark_mode;
        let frame = match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => self
                .tw_style
                .build_frame(&self.base, self.tw_state, REM_SIZE_PT, &lookup),
        };

        // 3. Resolve config (moves strings out of `self`).
        let accent = self
            .color
            .resolve(palette)
            .map(Color32::from)
            .unwrap_or_else(|| palette.primary.into());
        let checkbox_color = self
            .checkbox_color
            .resolve(palette)
            .map(Color32::from)
            .unwrap_or(accent);
        let cfg = ListConfig {
            selection_mode: self.selection_mode,
            accent,
            checkbox_color,
            dense: self.dense,
            gutters: self.gutters,
            disabled: self.disabled,
            readonly: self.readonly,
            checked_icon: self.checked_icon,
            unchecked_icon: self.unchecked_icon,
        };
        let padding = self.padding;

        // 4. Render.
        let outer = frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                if padding {
                    ui.add_space(8.0);
                }
                for (i, entry) in entries.iter_mut().enumerate() {
                    match entry {
                        ListEntry::Item(item) => {
                            draw_item(
                                ui,
                                &cfg,
                                palette,
                                item,
                                state,
                                0,
                                &i.to_string(),
                            );
                        }
                        ListEntry::Subheader(sh) => {
                            draw_subheader(ui, &cfg, sh, 0);
                        }
                    }
                }
                if padding {
                    ui.add_space(8.0);
                }
            })
                .response
        });

        (outer.response, user_result)
    }
}

// ---------------------------------------------------------------------------
// Resolved config
// ---------------------------------------------------------------------------

struct ListConfig {
    selection_mode: SelectionMode,
    accent: Color32,
    checkbox_color: Color32,
    dense: bool,
    gutters: bool,
    disabled: bool,
    readonly: bool,
    checked_icon: String,
    unchecked_icon: String,
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Height of a list item for the given density.
#[inline]
pub fn item_height(dense: bool) -> f32 {
    if dense {
        DENSE_ITEM_HEIGHT
    } else {
        ITEM_HEIGHT
    }
}

/// Height of a subheader row for the given density.
#[inline]
pub fn subheader_height(dense: bool) -> f32 {
    if dense {
        DENSE_SUBHEADER_HEIGHT
    } else {
        SUBHEADER_HEIGHT
    }
}

/// Indent contributed by `depth` nesting levels.
#[inline]
pub fn indent_px(depth: usize) -> f32 {
    depth as f32 * INDENT_PER_DEPTH
}

/// Pixel size of an icon for a [`Size`] preset.
#[inline]
pub fn icon_px(size: Size) -> f32 {
    match size {
        Size::Small => 20.0,
        Size::Medium => 24.0,
        Size::Large => 28.0,
        _ => 24.0,
    }
}

/// Side length of an avatar slot for the given density.
#[inline]
pub fn avatar_px(dense: bool) -> f32 {
    if dense {
        DENSE_AVATAR_SIZE
    } else {
        AVATAR_SIZE
    }
}

/// Applies one click on `value` to `state` according to `mode`.
///
/// Returns `true` when the state changed.
pub fn apply_selection<T: Clone + PartialEq>(
    mode: SelectionMode,
    state: &mut ListState<T>,
    value: &T,
) -> bool {
    match mode {
        SelectionMode::Single => {
            if state.selected_value.as_ref() == Some(value) {
                false
            } else {
                state.selected_value = Some(value.clone());
                state.selected_values.clear();
                true
            }
        }
        SelectionMode::Toggle => {
            if state.selected_value.as_ref() == Some(value) {
                state.selected_value = None;
            } else {
                state.selected_value = Some(value.clone());
            }
            state.selected_values.clear();
            true
        }
        SelectionMode::Multi => {
            if let Some(pos) = state.selected_values.iter().position(|v| v == value) {
                state.selected_values.remove(pos);
            } else {
                state.selected_values.push(value.clone());
            }
            state.selected_value = None;
            true
        }
    }
}

#[inline]
fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

/// Resolves the text colour for the current state (`None` = theme default).
#[inline]
fn text_color(selected: bool, disabled: bool, accent: Color32, ui: &Ui) -> Option<Color32> {
    if disabled {
        Some(ui.visuals().weak_text_color())
    } else if selected {
        Some(accent)
    } else {
        None
    }
}

/// Resolves the leading-icon colour.
fn icon_color(
    item_color: Color,
    selected: bool,
    disabled: bool,
    accent: Color32,
    palette: &Palette,
    ui: &Ui,
) -> Color32 {
    if disabled {
        return ui.visuals().weak_text_color();
    }
    match item_color {
        Color::Inherit | Color::Default => {
            if selected {
                accent
            } else {
                ui.visuals().text_color()
            }
        }
        other => other
            .resolve(palette)
            .map(Color32::from)
            .unwrap_or_else(|| ui.visuals().text_color()),
    }
}

/// Resolves the expand-glyph colour.
fn expand_icon_color(c: Color, palette: &Palette, ui: &Ui) -> Color32 {
    match c {
        Color::Default | Color::Inherit => ui.visuals().weak_text_color(),
        other => other
            .resolve(palette)
            .map(Color32::from)
            .unwrap_or_else(|| ui.visuals().weak_text_color()),
    }
}

// ---------------------------------------------------------------------------
// Rendering helpers
// ---------------------------------------------------------------------------

/// Draws one item (and, if expanded, its nested items).
fn draw_item<'a, T: Clone + PartialEq>(
    ui: &mut Ui,
    cfg: &ListConfig,
    palette: &Palette,
    item: &mut MudListItem<'a, T>,
    state: &mut ListState<T>,
    depth: usize,
    path: &str,
) {
    let dense = item.dense.unwrap_or(cfg.dense);
    let h = item_height(dense);
    let gutters = item.gutters.unwrap_or(cfg.gutters);
    let disabled = cfg.disabled || item.disabled;
    let has_nested = item.nested.is_some();
    let key = item.id.clone().unwrap_or_else(|| path.to_owned());

    // Seed expansion state from the item's `expanded` parameter once.
    state.ensure_expanded(&key, item.expanded.unwrap_or(false));

    // Allocate the row.
    let full_w = ui.available_width();
    let sense = if disabled {
        Sense::hover()
    } else {
        Sense::click()
    };
    let (rect, resp) = ui.allocate_exact_size(vec2(full_w, h), sense);

    let ripple_key = ui.scope_id().with(("mud_list_ripple", path));
    let mut clicked = false;

    // Handle the click *before* painting so the UI reflects the new state
    // immediately.
    if resp.clicked() && !disabled {
        clicked = true;
        let prevent = item.on_click_prevent_default;
        if let Some(cb) = item.on_click.as_mut() {
            cb(&item.value);
        }
        if !prevent {
            if has_nested {
                state.toggle_expanded(&key);
            }
            if !cfg.readonly {
                apply_selection(cfg.selection_mode, state, &item.value);
            }
        }
    }

    // Recompute state for painting.
    let selected = state.is_selected(&item.value, cfg.selection_mode);
    let expanded = state.is_expanded(&key).unwrap_or(false);
    let hovered = resp.hovered() && !disabled;

    // Background.
    let bg = if selected {
        with_alpha(cfg.accent, 24)
    } else if hovered {
        ui.visuals().widgets.hovered.weak_bg_fill
    } else {
        Color32::TRANSPARENT
    };
    if bg != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(ROW_RADIUS), bg);
    }

    // Ripple.
    if clicked && item.ripple {
        start_ripple(ui, ripple_key);
    }
    if item.ripple {
        paint_ripple(ui, rect, ripple_key, cfg.accent);
    }

    // Layout cursor.
    let mut pad = if gutters { GUTTER_PX } else { 0.0 }
        + indent_px(depth)
        + if item.inset { INSET_PX } else { 0.0 };
    let mut cursor_x = rect.left() + pad;
    let mut content_right = rect.right() - pad;

    // Cursor affordance for links and clickable rows.
    if !disabled && (item.on_click.is_some() || item.href.is_some()) {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    // Selection checkbox (Multi mode).
    if cfg.selection_mode == SelectionMode::Multi {
        let cb_rect = Rect::from_center_size(
            egui::pos2(cursor_x + CHECKBOX_SIZE * 0.5, rect.center().y),
            vec2(CHECKBOX_SIZE, CHECKBOX_SIZE),
        );
        paint_checkbox(
            ui,
            cb_rect,
            selected,
            cfg.checkbox_color,
            &cfg.checked_icon,
            &cfg.unchecked_icon,
            disabled,
        );
        cursor_x += CHECKBOX_SIZE + 8.0;
    }

    // Avatar or icon.
    if let Some(avatar) = item.avatar.as_mut() {
        let size = avatar_px(dense);
        let av_rect = Rect::from_center_size(
            egui::pos2(cursor_x + size * 0.5, rect.center().y),
            vec2(size, size),
        );
        let mut child = ui.new_child(UiBuilder::new().max_rect(av_rect));
        avatar(&mut child);
        cursor_x += size + 8.0;
    } else if let Some(icon) = item.icon.as_ref() {
        let px = icon_px(item.icon_size);
        let ir = Rect::from_center_size(
            egui::pos2(cursor_x + px * 0.5, rect.center().y),
            vec2(px, px),
        );
        let col = icon_color(
            item.icon_color,
            selected,
            disabled,
            cfg.accent,
            palette,
            ui,
        );
        ui.put(
            ir,
            MudIcon::new(icon.clone()).size_px(px).custom_color(col),
        );
        cursor_x += px + 8.0;
    }

    // Expand glyph on the trailing edge.
    if has_nested {
        let px = 24.0;
        let ir = Rect::from_center_size(
            egui::pos2(content_right - px * 0.5, rect.center().y),
            vec2(px, px),
        );
        let glyph = if expanded {
            &item.expand_less_icon
        } else {
            &item.expand_more_icon
        };
        let col = if disabled {
            ui.visuals().weak_text_color()
        } else {
            expand_icon_color(item.expand_icon_color, palette, ui)
        };
        ui.put(
            ir,
            MudIcon::new(glyph.clone()).size_px(px).custom_color(col),
        );
        content_right -= px + 8.0;
    }

    // Text or custom content.
    let text_rect = Rect::from_min_max(
        egui::pos2(cursor_x, rect.top()),
        egui::pos2(content_right.max(cursor_x), rect.bottom()),
    );
    if let Some(child) = item.child.as_mut() {
        let mut child_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(text_rect)
                .layout(Layout::left_to_right(egui::Align::Center)),
        );
        child(&mut child_ui);
    } else {
        let mut text_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(text_rect)
                .layout(Layout::top_down(egui::Align::Min)),
        );
        let block_h = if item.secondary_text.is_some() {
            38.0
        } else {
            22.0
        };
        text_ui.add_space(((text_rect.height() - block_h) * 0.5).max(0.0));

        if let Some(text) = item.text.as_ref() {
            let mut t = MudText::new(text.clone()).typo(Typo::Body1);
            if let Some(c) = text_color(selected, disabled, cfg.accent, ui) {
                t = t.custom_color(c);
            }
            text_ui.add(t);
        }
        if let Some(sec) = item.secondary_text.as_ref() {
            let mut t = MudText::new(sec.clone()).typo(Typo::Caption);
            t = t.custom_color(if disabled {
                ui.visuals().weak_text_color()
            } else {
                ui.visuals().weak_text_color()
            });
            text_ui.add(t);
        }
    }

    // Nested items.
    if has_nested && expanded {
        if let Some(nested) = item.nested.as_mut() {
            let mut nested_ui = ListUi {
                entries: Vec::new(),
            };
            nested(&mut nested_ui);
            for (j, child) in nested_ui.entries.iter_mut().enumerate() {
                let child_path = format!("{path}/{j}");
                match child {
                    ListEntry::Item(ci) => {
                        draw_item(ui, cfg, palette, ci, state, depth + 1, &child_path);
                    }
                    ListEntry::Subheader(sh) => {
                        draw_subheader(ui, cfg, sh, depth + 1);
                    }
                }
            }
        }
    }
}

/// Draws a subheader row.
fn draw_subheader(ui: &mut Ui, cfg: &ListConfig, sh: &MudListSubheader, depth: usize) {
    let h = subheader_height(cfg.dense);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::hover());
    let gutters = sh.gutters.unwrap_or(cfg.gutters);
    let pad = if gutters { GUTTER_PX } else { 0.0 }
        + indent_px(depth)
        + if sh.inset { INSET_PX } else { 0.0 };
    let tr = Rect::from_min_max(
        egui::pos2(rect.left() + pad, rect.top()),
        egui::pos2(rect.right() - pad, rect.bottom()),
    );
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(tr)
            .layout(Layout::left_to_right(egui::Align::Center)),
    );
    child.add(
        MudText::new(sh.text.clone())
            .typo(Typo::Subtitle2)
            .custom_color(ui.visuals().weak_text_color()),
    );
}

/// Paints a Mud-style checkbox glyph.
fn paint_checkbox(
    ui: &mut Ui,
    rect: Rect,
    checked: bool,
    color: Color32,
    checked_icon: &str,
    unchecked_icon: &str,
    disabled: bool,
) {
    let glyph = if checked {
        checked_icon
    } else {
        unchecked_icon
    };
    let col = if disabled {
        ui.visuals().weak_text_color()
    } else if checked {
        color
    } else {
        ui.visuals().weak_text_color()
    };
    ui.put(
        rect,
        MudIcon::new(glyph.to_owned())
            .size_px(rect.width())
            .custom_color(col),
    );
}

/// Starts a ripple. Reads the time **before** locking memory.
fn start_ripple(ui: &Ui, key: egui::Id) {
    let now = ui.input(|i| i.time);
    ui.ctx().memory_mut(|m| m.data.insert_temp(key, now));
}

/// Paints and advances a ripple.
fn paint_ripple(ui: &Ui, rect: Rect, key: egui::Id, accent: Color32) {
    let start: Option<f64> = ui.ctx().memory(|m| m.data.get_temp(key));
    let Some(start) = start else {
        return;
    };
    let now = ui.input(|i| i.time);
    let elapsed = (now - start) as f32;
    if elapsed >= RIPPLE_DURATION {
        ui.ctx().memory_mut(|m| m.data.remove_temp::<f64>(key));
        return;
    }
    let t = (elapsed / RIPPLE_DURATION).clamp(0.0, 1.0);
    let max_r = rect.width().max(rect.height()) * 0.75;
    let r = max_r * t;
    let a = ((1.0 - t) * 48.0) as u8;
    let col = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), a);
    let painter = ui.painter().with_clip_rect(rect);
    painter.circle_filled(rect.center(), r, col);
    ui.ctx().request_repaint();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- apply_selection ---------------------------------------------------

    #[test]
    fn single_selects_from_empty_and_ignores_reclick() {
        let mut s: ListState<&str> = ListState::new();
        assert!(apply_selection(SelectionMode::Single, &mut s, &"a"));
        assert_eq!(s.selected_value, Some("a"));
        assert!(!apply_selection(SelectionMode::Single, &mut s, &"a"));
        assert_eq!(s.selected_value, Some("a"));
    }

    #[test]
    fn single_replaces_previous_selection() {
        let mut s: ListState<&str> = ListState::new();
        apply_selection(SelectionMode::Single, &mut s, &"a");
        apply_selection(SelectionMode::Single, &mut s, &"b");
        assert_eq!(s.selected_value, Some("b"));
        assert!(s.selected_values.is_empty());
    }

    #[test]
    fn toggle_deselects_on_reclick() {
        let mut s: ListState<&str> = ListState::new();
        apply_selection(SelectionMode::Toggle, &mut s, &"a");
        assert_eq!(s.selected_value, Some("a"));
        apply_selection(SelectionMode::Toggle, &mut s, &"a");
        assert_eq!(s.selected_value, None);
    }

    #[test]
    fn toggle_replaces_other_selection() {
        let mut s: ListState<&str> = ListState::new();
        apply_selection(SelectionMode::Toggle, &mut s, &"a");
        apply_selection(SelectionMode::Toggle, &mut s, &"b");
        assert_eq!(s.selected_value, Some("b"));
    }

    #[test]
    fn multi_toggles_membership_and_preserves_order() {
        let mut s: ListState<&str> = ListState::new();
        apply_selection(SelectionMode::Multi, &mut s, &"a");
        apply_selection(SelectionMode::Multi, &mut s, &"b");
        apply_selection(SelectionMode::Multi, &mut s, &"c");
        assert_eq!(s.selected_values, vec!["a", "b", "c"]);
        apply_selection(SelectionMode::Multi, &mut s, &"b");
        assert_eq!(s.selected_values, vec!["a", "c"]);
        assert_eq!(s.selected_value, None);
    }

    // --- is_selected -------------------------------------------------------

    #[test]
    fn is_selected_is_mode_aware() {
        let mut s: ListState<&str> = ListState::new();
        apply_selection(SelectionMode::Single, &mut s, &"a");
        assert!(s.is_selected(&"a", SelectionMode::Single));
        assert!(!s.is_selected(&"b", SelectionMode::Single));

        let mut s: ListState<&str> = ListState::new();
        apply_selection(SelectionMode::Multi, &mut s, &"a");
        assert!(s.is_selected(&"a", SelectionMode::Multi));
        assert!(!s.is_selected(&"a", SelectionMode::Single));
    }

    // --- expansion ---------------------------------------------------------

    #[test]
    fn ensure_expanded_seeds_once() {
        let mut s: ListState<&str> = ListState::new();
        assert_eq!(s.is_expanded("k"), None);
        s.ensure_expanded("k", true);
        assert_eq!(s.is_expanded("k"), Some(true));
        s.ensure_expanded("k", false); // ignored
        assert_eq!(s.is_expanded("k"), Some(true));
    }

    #[test]
    fn toggle_expanded_flips_and_creates() {
        let mut s: ListState<&str> = ListState::new();
        s.toggle_expanded("k");
        assert_eq!(s.is_expanded("k"), Some(true));
        s.toggle_expanded("k");
        assert_eq!(s.is_expanded("k"), Some(false));
    }

    #[test]
    fn clear_expanded_empties_the_map() {
        let mut s: ListState<&str> = ListState::new();
        s.toggle_expanded("a");
        s.toggle_expanded("b");
        s.clear_expanded();
        assert_eq!(s.is_expanded("a"), None);
    }

    // --- geometry ----------------------------------------------------------

    #[test]
    fn heights_match_density() {
        assert_eq!(item_height(false), ITEM_HEIGHT);
        assert_eq!(item_height(true), DENSE_ITEM_HEIGHT);
        assert_eq!(subheader_height(false), SUBHEADER_HEIGHT);
        assert_eq!(subheader_height(true), DENSE_SUBHEADER_HEIGHT);
    }

    #[test]
    fn indent_scales_with_depth() {
        assert_eq!(indent_px(0), 0.0);
        assert_eq!(indent_px(1), INDENT_PER_DEPTH);
        assert_eq!(indent_px(3), 3.0 * INDENT_PER_DEPTH);
    }

    #[test]
    fn icon_px_maps_sizes() {
        assert_eq!(icon_px(Size::Small), 20.0);
        assert_eq!(icon_px(Size::Medium), 24.0);
        assert_eq!(icon_px(Size::Large), 28.0);
        assert_eq!(icon_px(Size::ExtraLarge), 24.0);
    }

    #[test]
    fn avatar_px_matches_density() {
        assert_eq!(avatar_px(false), AVATAR_SIZE);
        assert_eq!(avatar_px(true), DENSE_AVATAR_SIZE);
    }

    // --- defaults ----------------------------------------------------------

    #[test]
    fn list_defaults_match_mudblazor() {
        let l: MudList<'_, i32> = MudList::new();
        assert!(matches!(l.selection_mode, SelectionMode::Single));
        assert!(matches!(l.color, Color::Primary));
        assert!(matches!(l.checkbox_color, Color::Default));
        assert!(!l.dense);
        assert!(!l.disabled);
        assert!(!l.readonly);
        assert!(l.gutters);
        assert!(!l.padding);
    }

    #[test]
    fn item_defaults() {
        let it = MudListItem::new(1);
        assert!(it.text.is_none());
        assert!(it.secondary_text.is_none());
        assert!(it.icon.is_none());
        assert!(matches!(it.icon_color, Color::Inherit));
        assert!(matches!(it.icon_size, Size::Medium));
        assert!(it.avatar.is_none());
        assert!(it.child.is_none());
        assert!(it.nested.is_none());
        assert_eq!(it.expanded, None);
        assert!(!it.disabled);
        assert!(!it.inset);
        assert!(it.ripple);
        assert!(!it.on_click_prevent_default);
    }

    #[test]
    fn item_builders_set_fields() {
        let it: MudListItem<'_, i32> = MudListItem::new(1)
            .with_text("One")
            .secondary_text("first")
            .icon("M0 0")
            .icon_color(Color::Error)
            .icon_size(Size::Large)
            .disabled(true)
            .inset(true)
            .dense(true)
            .gutters(false)
            .href("/one")
            .target("_blank")
            .force_load(true)
            .ripple(false)
            .on_click_prevent_default(true)
            .id("one")
            .expand_more_icon("M")
            .expand_less_icon("L")
            .expand_icon_color(Color::Primary);
        assert_eq!(it.text.as_deref(), Some("One"));
        assert_eq!(it.secondary_text.as_deref(), Some("first"));
        assert_eq!(it.icon.as_deref(), Some("M0 0"));
        assert!(matches!(it.icon_color, Color::Error));
        assert!(matches!(it.icon_size, Size::Large));
        assert!(it.disabled && it.inset);
        assert_eq!(it.dense, Some(true));
        assert_eq!(it.gutters, Some(false));
        assert_eq!(it.href.as_deref(), Some("/one"));
        assert_eq!(it.target.as_deref(), Some("_blank"));
        assert!(it.force_load);
        assert!(!it.ripple);
        assert!(it.on_click_prevent_default);
        assert_eq!(it.id.as_deref(), Some("one"));
        assert_eq!(it.expand_more_icon, "M");
        assert_eq!(it.expand_less_icon, "L");
        assert!(matches!(it.expand_icon_color, Color::Primary));
    }

    #[test]
    fn subheader_defaults() {
        let sh = MudListSubheader::new("Section");
        assert_eq!(sh.text, "Section");
        assert_eq!(sh.gutters, None);
        assert!(!sh.inset);
    }

    // --- collector ---------------------------------------------------------

    #[test]
    fn collector_preserves_order_and_subheaders() {
        let mut ui: ListUi<'_, i32> = ListUi {
            entries: Vec::new(),
        };
        ui.subheader("A");
        ui.text_item(1, "one");
        ui.item(MudListItem::new(2).with_text("two"));
        ui.subheader("B");

        assert_eq!(ui.len(), 4);
        assert!(matches!(ui.entries[0], ListEntry::Subheader(_)));
        assert!(matches!(ui.entries[1], ListEntry::Item(_)));
        assert!(matches!(ui.entries[2], ListEntry::Item(_)));
        assert!(matches!(ui.entries[3], ListEntry::Subheader(_)));
    }

    // --- tag ---------------------------------------------------------------

    #[test]
    fn tag_round_trips_by_type() {
        let l: MudList<'_, i32> = MudList::new().tag("section");
        assert_eq!(l.tag_ref::<&str>(), Some(&"section"));
        assert!(l.tag_ref::<u32>().is_none());
    }
}