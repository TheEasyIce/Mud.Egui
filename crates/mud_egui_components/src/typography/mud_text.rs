//! A themed, Material Design text component for `egui`.
//!
//! This component is a port of MudBlazor's `MudText`. It provides a fluent
//! builder API for applying semantic typography scales (`h1`, `body1`, `caption`),
//! semantic colors, text alignment, and standard text decorations.
//!
//! In addition to explicit builder methods, it features a mini utility-class engine
//! (driven by `ComponentBase`) that parses standard spacing (`p-4`, `mx-2`) and
//! background color (`bg-primary`) strings.
//!
//! # MudBlazor parity
//!
//! MudBlazor's `MudText` exposes `Typo`, `Color`, `Align`, `GutterBottom`, `Inline`,
//! `HtmlTag`, `ChildContent` and a cascading `RightToLeft`, plus `Class`, `Style` and
//! `UserAttributes` inherited from `MudComponentBase`. `MudText` has no value binding,
//! no click/focus/key callbacks, no `Variant`, `Size`, `Elevation`, `Disabled` or
//! `ReadOnly`, so none of those are invented here. The Rust-only additions
//! (`opacity`, `tooltip`, `selectable`, `sense`, `id`, `aria_label`, font overrides)
//! are `egui`-native conveniences and are documented as such.
//!
//! | MudBlazor                | This crate                                         |
//! |--------------------------|----------------------------------------------------|
//! | `Typo`                   | [`MudText::typo`] and the `h1()`..`overline()` constructors |
//! | `Color`                  | [`MudText::color`], [`MudText::custom_color`]      |
//! | `Align`                  | [`MudText::align`]                                 |
//! | `GutterBottom`           | [`MudText::gutter_bottom`]                         |
//! | `Inline`                 | [`MudText::inline`]                                |
//! | `HtmlTag`                | [`MudText::html_tag`]                              |
//! | `Class`                  | [`MudText::classes`]                               |
//! | `UserAttributes` (`id`, `aria-label`) | [`MudText::id`], [`MudText::aria_label`] |
//! | `ChildContent`           | the text passed to [`MudText::new`]                |
//!
//! # Architecture
//!
//! Rendering is a two-stage pipeline:
//!
//! 1. **Resolve** ([`MudText::resolve_style`]): a pure function from
//!    `(component state, theme, fallback text color)` to a [`ResolvedStyle`]. It never
//!    touches a `Ui`, never allocates (apart from cloning a named `FontFamily`'s
//!    `Arc`), and is fully unit-testable.
//! 2. **Render** ([`MudText::ui`] / [`MudText::ui_with_theme`]): turns the resolved
//!    style into an `egui::Label`, optionally wrapped in alignment and utility
//!    `Frame` layers.
//!
//! # Performance notes
//!
//! * The text buffer is *moved* into `egui::RichText` unless an uppercase transform
//!   is required (see [`MudText::display_text`] and the private consuming fast path).
//! * [`MudText::ui_with_theme`] lets a caller that draws many texts look the theme up
//!   once and share it, instead of performing a theme lookup per widget.
//! * Invalid numeric overrides (`NaN`, infinite, zero or negative font sizes, `NaN`
//!   opacity) are rejected at resolution time, so rendering never receives them.

use std::borrow::Cow;
use std::hash::Hash;

use egui::{
    Color32, FontFamily, FontId, Frame, Id, Label, Layout, Response, RichText, Sense,
    TextWrapMode, Ui, Widget, WidgetInfo, WidgetType,
};

use mud_egui_core::base::component_base::ComponentBase;
use crate::foundation::utilities::Utilities;

use mud_egui_core::enums::{MudAlign, Color, Typo};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::models::ui::typography::FontVariant;
use mud_egui_theming::theme::{to_egui_family, MudTheme};

/// The relative margin size applied to the bottom of the text when `gutter_bottom` is true.
/// Expressed in `em` units (relative to the current font size).
const GUTTER_EM: f32 = 0.35;

/// HTML tags that are inline-level (phrasing content). When [`MudText::html_tag`]
/// names one of these, the component behaves as if [`MudText::inline`] were set,
/// mirroring MudBlazor's remark that the tag affects `Align` and `GutterBottom`.
const INLINE_HTML_TAGS: &[&str] = &[
    "a", "abbr", "b", "bdi", "bdo", "cite", "code", "data", "dfn", "em", "i", "kbd", "label",
    "mark", "q", "s", "samp", "small", "span", "strong", "sub", "sup", "time", "u", "var",
];

/// Returns `true` when `tag` names an inline-level HTML element (for example `"span"`).
///
/// The comparison is ASCII case-insensitive. Block-level tags (`"p"`, `"div"`, `"h1"`)
/// and unknown tags return `false`.
///
/// ```ignore
/// assert!(is_inline_html_tag("SPAN"));
/// assert!(!is_inline_html_tag("p"));
/// ```
#[must_use]
pub fn is_inline_html_tag(tag: &str) -> bool {
    INLINE_HTML_TAGS.iter().any(|t| t.eq_ignore_ascii_case(tag))
}

// =============================================================================
// Resolved style
// =============================================================================

/// The fully resolved, `Ui`-independent visual description of a [`MudText`].
///
/// Produced by [`MudText::resolve_style`]. Every field is the *final* value the
/// renderer will use, after applying the precedence rules
/// `builder override > theme variant`. Because it is plain data, it can be
/// compared in tests, logged, or used by callers to pre-compute layout.
///
/// ```ignore
/// let theme = MudTheme::default();
/// let style = MudText::h3("Title").bold(true).resolve_style(&theme, Color32::WHITE);
/// assert!(style.bold);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedStyle {
    /// Final font size in points. Always finite and greater than zero.
    pub font_size: f32,
    /// Final font family (explicit family, then monospace, then the theme variant).
    pub family: FontFamily,
    /// Whether the text is emphasised as bold.
    pub bold: bool,
    /// Whether the text is italic.
    pub italic: bool,
    /// Whether an underline is drawn.
    pub underline: bool,
    /// Whether a strikethrough is drawn.
    pub strikethrough: bool,
    /// Final text color, or `None` to let `egui`'s default text color apply.
    pub color: Option<Color32>,
    /// Whether the text is transformed to upper case before rendering.
    pub uppercase: bool,
    /// Whether the component is laid out inline (explicit flag or inline HTML tag).
    pub inline: bool,
    /// Extra vertical space added below the text, in points. `0.0` when
    /// `gutter_bottom` is off or the component is inline.
    pub gutter: f32,
    /// The layout used to align the text across the full available width, or
    /// `None` when no alignment layer is needed (inline, or `MudAlign::Inherit`).
    pub layout: Option<Layout>,
}

// =============================================================================
// Component
// =============================================================================

/// A themed text component with support for semantic typography and utility classes.
///
/// Use the builder pattern to configure the text before passing it to the UI.
///
/// ```ignore
/// ui.add(MudText::h3("Welcome!").color(Color::Primary).bold(true));
/// ```
///
/// Every setter consumes and returns `Self`, so a `MudText` is typically built and
/// rendered in one expression. If you need to inspect or reuse a partially built
/// instance, simply bind it to a variable before calling [`MudText::ui`]; the
/// struct has no interior mutability and every field is a plain, inspectable value.
#[must_use = "MudText does nothing until you call `.ui(ui)` or `ui.add(..)`"]
pub struct MudText {
    /// The actual string content to display.
    pub text: String,

    /// The base component tracking utility classes.
    pub base: ComponentBase,

    /// The semantic typographic scale (e.g., `H1`, `Body1`).
    pub typo: Typo,

    /// The semantic color of the text (e.g., `Primary`, `Error`).
    pub color: Color,

    /// The horizontal alignment of the text.
    pub align: MudAlign,

    /// If `true`, the text will not attempt to fill the available width (ignores alignment).
    pub is_inline: bool,

    /// If `true`, adds a standard bottom margin (`0.35em`) to the text.
    pub gutter_bottom: bool,

    /// If `true`, text that overflows its container will be truncated with an ellipsis (`...`).
    pub truncate: bool,

    /// If `true`, the text will never wrap to a new line, forcing the container to expand or clip.
    pub no_wrap: bool,

    /// If `true`, draws a line under the text.
    pub underline: bool,

    /// If `true`, draws a line through the center of the text.
    pub strikethrough: bool,

    /// If `true`, the user can highlight and copy the text with their mouse.
    pub is_selectable: bool,

    // --- Optional overrides that bypass the semantic `Typo` configuration ---
    /// Forces the text to be bold (or not bold), ignoring the `Typo` default.
    pub bold: Option<bool>,

    /// Forces the text to be italic.
    pub italic: bool,

    /// Forces the text to render in a monospace font.
    pub monospace: bool,

    /// Forces the text to be transformed to ALL CAPS (or not), ignoring the `Typo` default.
    pub uppercase: Option<bool>,

    /// Forces a specific font size (in points), ignoring the `Typo` scale.
    /// Values that are not finite or not greater than zero are ignored at resolution time.
    pub font_size: Option<f32>,

    /// Forces a specific font family, ignoring the theme defaults.
    pub font_family: Option<FontFamily>,

    /// Overrides the opacity of the text (0.0 to 1.0).
    pub opacity: Option<f32>,

    /// An optional tooltip that displays when the user hovers over the text.
    pub tooltip: Option<String>,

    /// Mirrors MudBlazor's `HtmlTag`: the semantic element the text stands for.
    /// `None` means "decided by `Typo`". Inline-level tags such as `span` make the
    /// component behave inline (see [`is_inline_html_tag`]).
    pub html_tag: Option<Cow<'static, str>>,

    /// Mirrors the `id` entry of MudBlazor's `UserAttributes`: a stable salt that
    /// scopes the widget's `egui` id (useful for selectable text and `sense`).
    pub id: Option<Id>,

    /// Mirrors the `aria-label` entry of MudBlazor's `UserAttributes`: the text
    /// exposed to assistive technology instead of the visible string.
    pub aria_label: Option<String>,

    /// An optional interaction sense (for example `Sense::click()`), so the returned
    /// [`Response`] can report clicks. `egui`-native; MudBlazor's `MudText` has no
    /// click callback.
    pub sense: Option<Sense>,
}

impl Default for MudText {
    /// Creates an empty `MudText` with `Body1`-equivalent (`Typo::Default`) styling.
    ///
    /// Equivalent to `MudText::new("")`.
    fn default() -> Self {
        Self::new("")
    }
}

impl From<&str> for MudText {
    /// Creates a `MudText` from a string slice, equivalent to [`MudText::new`].
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<&String> for MudText {
    /// Creates a `MudText` from a borrowed `String`, equivalent to [`MudText::new`].
    fn from(s: &String) -> Self {
        Self::new(s.as_str())
    }
}

impl From<String> for MudText {
    /// Creates a `MudText` from an owned `String` without copying its contents,
    /// equivalent to [`MudText::new`].
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<Cow<'_, str>> for MudText {
    /// Creates a `MudText` from a `Cow<str>`, allocating only if the value is borrowed.
    fn from(s: Cow<'_, str>) -> Self {
        Self::new(s.into_owned())
    }
}

/// Generates `MudText::h1("..")`-style shorthand constructors for the `Typo` enum.
macro_rules! typo_constructors {
    ($($name:ident => $variant:ident),* $(,)?) => {
        $(
            #[doc = concat!(
                "Creates a `MudText` using the `Typo::", stringify!($variant), "` scale.\n\n",
                "Equivalent to `MudText::new(text).typo(Typo::", stringify!($variant), ")`."
            )]
            #[inline]
            pub fn $name(text: impl Into<String>) -> Self {
                Self::new(text).typo(Typo::$variant)
            }
        )*
    };
}

impl MudText {
    /// Creates a new `MudText` initialized with standard `Body1` typography.
    ///
    /// All optional overrides (`bold`, `italic`, `uppercase`, `font_size`, ...) start
    /// unset, so rendering falls back entirely to the active [`MudTheme`].
    ///
    /// ```ignore
    /// ui.add(MudText::new("Plain body text"));
    /// ```
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            base: ComponentBase::new(),
            typo: Typo::Default,
            color: Color::Default,
            align: MudAlign::Inherit,
            is_inline: false,
            gutter_bottom: false,
            truncate: false,
            no_wrap: false,
            underline: false,
            strikethrough: false,
            is_selectable: false,
            bold: None,
            italic: false,
            monospace: false,
            uppercase: None,
            font_size: None,
            font_family: None,
            opacity: None,
            tooltip: None,
            html_tag: None,
            id: None,
            aria_label: None,
            sense: None,
        }
    }

    typo_constructors! {
        h1 => H1, h2 => H2, h3 => H3, h4 => H4, h5 => H5, h6 => H6,
        subtitle1 => Subtitle1, subtitle2 => Subtitle2,
        body1 => Body1, body2 => Body2,
        button => Button, caption => Caption, overline => Overline,
    }

    // =========================================================================
    // Builder Methods
    // =========================================================================

    /// Sets the semantic typographic scale.
    #[inline]
    pub fn typo(mut self, typo: Typo) -> Self {
        self.typo = typo;
        self
    }

    /// Sets the semantic color (e.g., `Color::Primary`, `Color::Error`).
    #[inline]
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Explicitly forces a specific `egui` color, bypassing the MudBlazor theme entirely.
    #[inline]
    pub fn custom_color(self, color: Color32) -> Self {
        self.color(Color::Custom(color))
    }

    /// Sets the horizontal alignment of the text.
    ///
    /// Like MudBlazor, this has no effect on inline displays.
    #[inline]
    pub fn align(mut self, align: MudAlign) -> Self {
        self.align = align;
        self
    }

    /// If `true`, the text will only occupy the space it needs, rather than expanding
    /// horizontally to fill its container (which breaks text alignment).
    #[inline]
    pub fn inline(mut self, is_inline: bool) -> Self {
        self.is_inline = is_inline;
        self
    }

    /// If `true`, applies a standard bottom margin of `0.35em` below the text.
    ///
    /// Like MudBlazor, this has no effect on inline displays.
    #[inline]
    pub fn gutter_bottom(mut self, gutter: bool) -> Self {
        self.gutter_bottom = gutter;
        self
    }

    /// If `true`, text that overflows its container will be truncated with an ellipsis (`...`).
    #[inline]
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }

    /// If `true`, prevents the text from wrapping to a new line.
    #[inline]
    pub fn no_wrap(mut self, no_wrap: bool) -> Self {
        self.no_wrap = no_wrap;
        self
    }

    /// If `true`, renders a horizontal line under the text.
    #[inline]
    pub fn underline(mut self, underline: bool) -> Self {
        self.underline = underline;
        self
    }

    /// If `true`, renders a horizontal line through the center of the text.
    #[inline]
    pub fn strikethrough(mut self, strikethrough: bool) -> Self {
        self.strikethrough = strikethrough;
        self
    }

    /// If `true`, allows the user to highlight and copy the text.
    #[inline]
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.is_selectable = selectable;
        self
    }

    /// Overrides the bold weight of the text.
    #[inline]
    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = Some(bold);
        self
    }

    /// If `true`, italicizes the text.
    #[inline]
    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    /// If `true`, renders the text using a monospace (fixed-width) font.
    ///
    /// An explicit [`MudText::font_family`] takes precedence over this flag.
    #[inline]
    pub fn monospace(mut self, monospace: bool) -> Self {
        self.monospace = monospace;
        self
    }

    /// If `true`, forces the text string to ALL CAPS before rendering; if `false`,
    /// suppresses the automatic upper-casing of `Button` and `Overline`.
    #[inline]
    pub fn uppercase(mut self, uppercase: bool) -> Self {
        self.uppercase = Some(uppercase);
        self
    }

    /// Overrides the font size (in points), bypassing the `Typo` configuration.
    ///
    /// The raw value is stored as given; values that are not finite or not greater
    /// than zero are ignored by [`MudText::resolve_style`] and the theme size is used.
    #[inline]
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    /// Overrides the font family, bypassing the global theme.
    #[inline]
    pub fn font_family(mut self, family: FontFamily) -> Self {
        self.font_family = Some(family);
        self
    }

    /// Applies an alpha opacity modifier to the text color (0.0 to 1.0).
    ///
    /// The value is clamped to `[0.0, 1.0]` at build time so that downstream
    /// rendering code never has to re-validate it. `NaN` is treated as "no opacity
    /// override" because it has no meaningful clamp.
    #[inline]
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = if opacity.is_nan() {
            None
        } else {
            Some(opacity.clamp(0.0, 1.0))
        };
        self
    }

    /// Attaches a hover tooltip to the text element.
    #[inline]
    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    /// Sets the semantic HTML tag the text stands for (MudBlazor's `HtmlTag`).
    ///
    /// Inline-level tags (`"span"`, `"b"`, `"code"`, ...) make the component behave
    /// as if [`MudText::inline`] were set; block-level tags (`"p"`, `"h1"`, `"div"`)
    /// leave the layout untouched. Passing a `&'static str` does not allocate.
    ///
    /// ```ignore
    /// ui.add(MudText::body2("inline note").html_tag("span"));
    /// ```
    #[inline]
    pub fn html_tag(mut self, tag: impl Into<Cow<'static, str>>) -> Self {
        self.html_tag = Some(tag.into());
        self
    }

    /// Scopes the widget under a stable id salt (MudBlazor's `id` user attribute).
    ///
    /// Useful for selectable text and click-sensing text whose position in the
    /// layout can change between frames.
    #[inline]
    pub fn id(mut self, id_salt: impl Hash + std::fmt::Debug) -> Self {
        self.id = Some(Id::new(id_salt));
        self
    }

    /// Sets the accessibility label exposed to assistive technology (MudBlazor's
    /// `aria-label` user attribute). The visible text is unaffected.
    #[inline]
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// Makes the text respond to the given interaction kind, so the [`Response`]
    /// returned from [`MudText::ui`] reports it (for example `Sense::click()`).
    #[inline]
    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = Some(sense);
        self
    }

    /// Attaches utility classes to the component (e.g., `["mt-4", "bg-primary"]`).
    ///
    /// Classes accumulate across multiple calls; calling this more than once adds
    /// to the existing set rather than replacing it (delegated to [`ComponentBase::classes`]).
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    // =========================================================================
    // Resolution helpers
    // =========================================================================

    /// Returns the HTML tag name set via [`MudText::html_tag`], if any.
    #[must_use]
    pub fn tag_name(&self) -> Option<&str> {
        self.html_tag.as_deref()
    }

    /// Returns `true` when the component lays out inline: either
    /// [`MudText::inline`] was set, or [`MudText::html_tag`] names an inline-level tag.
    #[must_use]
    pub fn is_effectively_inline(&self) -> bool {
        self.is_inline || self.html_tag.as_deref().is_some_and(is_inline_html_tag)
    }

    /// Returns `true` when the text will be upper-cased: the explicit
    /// [`MudText::uppercase`] override if set, otherwise the `Typo` default
    /// (`Button` and `Overline`).
    #[must_use]
    pub fn is_uppercase(&self) -> bool {
        self.uppercase.unwrap_or(self.typo.is_uppercase_by_default())
    }

    /// Parses the component's string classes into [`Utilities`] (padding, margin,
    /// and background color).
    ///
    /// This is re-parsed on every call rather than cached, since `base` can be
    /// mutated via [`MudText::classes`] at any point before rendering; the cost is
    /// bounded by the (typically small) number of classes attached to the component.
    pub fn utilities(&self) -> Utilities {
        Utilities::parse(self.base.class_names().iter().map(AsRef::as_ref))
    }

    /// Returns the final display text. Uses [`Cow`] to avoid allocating a new string
    /// unless an `uppercase` transformation is actually required.
    ///
    /// This method is intended for callers who want to *preview* the rendered text
    /// (e.g. for logging, testing, or measuring) without consuming the `MudText`.
    /// The internal rendering path in [`MudText::ui`] uses a consuming fast path
    /// instead, which additionally avoids the clone this method must perform when
    /// returning a borrowed value to an unconsumed `&self`.
    #[must_use]
    pub fn display_text(&self) -> Cow<'_, str> {
        if self.is_uppercase() {
            Cow::Owned(self.text.to_uppercase())
        } else {
            Cow::Borrowed(&self.text)
        }
    }

    /// Consumes `self.text`, producing the final display `String` with as few
    /// allocations as possible.
    ///
    /// * When no uppercase transform is needed, the original `String` buffer is
    ///   moved out via [`std::mem::take`]: zero allocations, zero copies.
    /// * When an uppercase transform *is* needed, one allocation is unavoidable
    ///   (the transformed text generally differs in byte length from the source,
    ///   e.g. for certain Unicode case mappings), so `str::to_uppercase` is used.
    ///
    /// This is a private helper used only by [`MudText::ui_with_theme`]; external
    /// callers that do not want to consume the component should use
    /// [`MudText::display_text`].
    fn into_display_text(&mut self) -> String {
        if self.is_uppercase() {
            self.text.to_uppercase()
        } else {
            std::mem::take(&mut self.text)
        }
    }

    /// Retrieves the typographic configuration for the current `Typo` scale from the theme.
    fn get_variant<'a>(&self, theme: &'a MudTheme) -> &'a FontVariant {
        match self.typo {
            Typo::Default => &theme.typography.default,
            Typo::H1 => &theme.typography.h1,
            Typo::H2 => &theme.typography.h2,
            Typo::H3 => &theme.typography.h3,
            Typo::H4 => &theme.typography.h4,
            Typo::H5 => &theme.typography.h5,
            Typo::H6 => &theme.typography.h6,
            Typo::Subtitle1 => &theme.typography.subtitle1,
            Typo::Subtitle2 => &theme.typography.subtitle2,
            Typo::Body1 => &theme.typography.body1,
            Typo::Body2 => &theme.typography.body2,
            Typo::Button => &theme.typography.button,
            Typo::Caption => &theme.typography.caption,
            Typo::Overline => &theme.typography.overline,
        }
    }

    /// Resolves the semantic color against the active palette.
    fn resolve_color(&self, palette: &Palette) -> Option<Color32> {
        self.color.resolve(palette)
    }

    /// Resolves every visual property of this component against `theme`, without
    /// touching a `Ui`.
    ///
    /// Precedence is always `builder override > theme variant`. `fallback_text_color`
    /// is only consulted when an [`MudText::opacity`] is set but no semantic color
    /// resolves (`Color::Inherit`); pass `ui.visuals().text_color()` when calling
    /// from rendering code.
    ///
    /// ```ignore
    /// let theme = MudTheme::default();
    /// let style = MudText::button("Save").resolve_style(&theme, Color32::WHITE);
    /// assert!(style.uppercase);
    /// ```
    #[must_use]
    pub fn resolve_style(&self, theme: &MudTheme, fallback_text_color: Color32) -> ResolvedStyle {
        let palette = theme.current_palette();
        let variant = self.get_variant(theme);

        // Reject nonsensical sizes so the renderer never sees NaN, zero or negatives.
        let font_size = match self.font_size {
            Some(size) if size.is_finite() && size > 0.0 => size,
            _ => variant.font_size,
        };
        let bold = self.bold.unwrap_or(variant.is_bold);

        let mut color = self.resolve_color(palette);
        if let Some(opacity) = self.opacity {
            // Multiply against the resolved color, or against the default text color
            // if no semantic color was set.
            let base = color.unwrap_or(fallback_text_color);
            color = Some(base.gamma_multiply(opacity));
        }

        let family = match (&self.font_family, self.monospace) {
            (Some(family), _) => family.clone(),
            (None, true) => FontFamily::Monospace,
            (None, false) => to_egui_family(variant.font_family),
        };

        let inline = self.is_effectively_inline();
        let gutter = if self.gutter_bottom && !inline {
            font_size * GUTTER_EM
        } else {
            0.0
        };
        let layout = if inline { None } else { self.align.layout() };

        ResolvedStyle {
            font_size,
            family,
            bold,
            italic: self.italic,
            underline: self.underline,
            strikethrough: self.strikethrough,
            color,
            uppercase: self.is_uppercase(),
            inline,
            gutter,
            layout,
        }
    }

    // =========================================================================
    // Rendering
    // =========================================================================

    /// Renders the `MudText` component into the `egui::Ui`, looking the active
    /// theme up from the `egui::Context` (falling back to the default theme).
    ///
    /// This consumes `self`: the component is a one-shot description of what to
    /// draw, mirroring `egui`'s typical builder-widget convention (see [`Widget`]).
    /// When drawing many texts in a row, prefer [`MudText::ui_with_theme`].
    pub fn ui(self, ui: &mut Ui) -> Response {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        self.ui_with_theme(ui, &theme)
    }

    /// Renders the component using an already-fetched theme.
    ///
    /// Fetch the theme once and share it across a list, table or log view to avoid
    /// a per-widget theme lookup:
    ///
    /// ```ignore
    /// let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
    /// for line in &lines {
    ///     MudText::body2(line.as_str()).ui_with_theme(ui, &theme);
    /// }
    /// ```
    pub fn ui_with_theme(mut self, ui: &mut Ui, theme: &MudTheme) -> Response {
        let palette = theme.current_palette();
        let style = self.resolve_style(theme, ui.visuals().text_color());

        // Fast path: moves the text buffer out of `self` instead of cloning it
        // whenever no uppercase transform is required (see `into_display_text`).
        let display_text = self.into_display_text();

        // Construct the rich text
        let mut rich =
            RichText::new(display_text).font(FontId::new(style.font_size, style.family));

        if let Some(c) = style.color {
            rich = rich.color(c);
        }
        if style.bold {
            rich = rich.strong();
        }
        if style.italic {
            rich = rich.italics();
        }
        if style.underline {
            rich = rich.underline();
        }
        if style.strikethrough {
            rich = rich.strikethrough();
        }

        let mut label = Label::new(rich).selectable(self.is_selectable);

        if let Some(sense) = self.sense {
            label = label.sense(sense);
        }
        if self.no_wrap {
            label = label.wrap_mode(TextWrapMode::Extend);
        }
        if self.truncate {
            label = label.truncate();
        }

        // Horizontal Alignment
        // To align text in `egui`, the container must allocate the full available width.
        // Inline texts (explicit flag or inline HTML tag) skip this so they only take
        // the space they need; that decision is already baked into `style.layout`.
        let layout = style.layout;

        // A closure to actually add the label, handling the layout allocation if necessary.
        let add_label = move |ui: &mut Ui| -> Response {
            match layout {
                Some(layout) => {
                    let size = egui::vec2(ui.available_width(), 0.0);
                    ui.allocate_ui_with_layout(size, layout, |ui| ui.add(label)).inner
                }
                None => ui.add(label),
            }
        };

        // Handle Utility Classes (p-*, m-*, bg-*)
        let utilities = self.utilities();

        let render = move |ui: &mut Ui| -> Response {
            if utilities.is_empty() {
                add_label(ui)
            } else {
                // If the user applied utility classes, we wrap the text in an `egui::Frame`
                // to apply the padding, margin, and background color.
                let mut frame = Frame::new()
                    .inner_margin(utilities.padding.to_margin())
                    .outer_margin(utilities.margin.to_margin());

                if let Some(bg) = utilities.background.and_then(|r| r.resolve(palette)) {
                    frame = frame.fill(bg);
                }

                frame.show(ui, add_label).inner
            }
        };

        let response = match self.id {
            Some(id) => ui.push_id(id, render).inner,
            None => render(ui),
        };

        // Accessibility: expose the aria label instead of the visible string.
        if let Some(label_text) = self.aria_label.as_deref() {
            let enabled = ui.is_enabled();
            response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, enabled, label_text));
        }

        // Gutter Bottom (already zero for inline texts)
        if style.gutter > 0.0 {
            ui.add_space(style.gutter);
        }

        // Tooltip
        match self.tooltip {
            Some(tip) => response.on_hover_text(tip),
            None => response,
        }
    }
}

// Allows `MudText` to be used with standard `egui` `ui.add()` syntax.
impl Widget for MudText {
    /// Renders this `MudText` the same way [`MudText::ui`] does. Provided so
    /// `MudText` instances can be passed directly to [`Ui::add`].
    fn ui(self, ui: &mut Ui) -> Response {
        MudText::ui(self, ui)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::utilities::Edges;
    use mud_egui_core::enums::align::MudAlign;
    use mud_egui_core::enums::color::Color;
    use mud_egui_core::enums::typo::Typo;

    // -------------------------------------------------------------------
    // Test harness
    // -------------------------------------------------------------------

    /// Runs `f` once inside a single `egui` frame and returns its result.
    fn run_ui<R>(f: impl FnOnce(&mut Ui) -> R) -> R {
        let ctx = egui::Context::default();
        let mut f = Some(f);
        let mut out: Option<R> = None;
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run_ui(raw, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(f) = f.take() {
                    out = Some(f(ui));
                }
            });
        });
        out.expect("the UI closure should run exactly once")
    }

    /// Renders `text` with the proportional family (always registered in a bare
    /// context) and returns the response rectangle size.
    fn rendered_size(text: MudText) -> egui::Vec2 {
        let theme = MudTheme::default();
        run_ui(|ui| {
            text.font_family(FontFamily::Proportional)
                .ui_with_theme(ui, &theme)
                .rect
                .size()
        })
    }

    /// Renders `text` and returns how far the layout cursor advanced vertically.
    fn cursor_delta(text: MudText) -> f32 {
        let theme = MudTheme::default();
        run_ui(|ui| {
            let y0 = ui.next_widget_position().y;
            text.font_family(FontFamily::Proportional)
                .ui_with_theme(ui, &theme);
            ui.next_widget_position().y - y0
        })
    }

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.01
    }

    // -------------------------------------------------------------------
    // Construction & defaults
    // -------------------------------------------------------------------

    #[test]
    fn test_new_sets_defaults() {
        let text = MudText::new("Hello");
        assert_eq!(text.text, "Hello");
        assert_eq!(text.typo, Typo::Default);
        assert_eq!(text.color, Color::Default);
        assert_eq!(text.align, MudAlign::Inherit);
        assert!(!text.is_inline);
        assert!(!text.gutter_bottom);
        assert!(!text.truncate);
        assert!(!text.no_wrap);
        assert!(!text.underline);
        assert!(!text.strikethrough);
        assert!(!text.is_selectable);
        assert_eq!(text.bold, None);
        assert!(!text.italic);
        assert!(!text.monospace);
        assert_eq!(text.uppercase, None);
        assert_eq!(text.font_size, None);
        assert!(text.font_family.is_none());
        assert_eq!(text.opacity, None);
        assert_eq!(text.tooltip, None);
        assert_eq!(text.html_tag, None);
        assert_eq!(text.id, None);
        assert_eq!(text.aria_label, None);
        assert_eq!(text.sense, None);
    }

    #[test]
    fn test_default_matches_new_empty() {
        let default_text = MudText::default();
        assert_eq!(default_text.text, "");
        assert_eq!(default_text.typo, Typo::Default);
    }

    #[test]
    fn test_from_str_and_string() {
        let from_slice: MudText = "slice".into();
        assert_eq!(from_slice.text, "slice");

        let owned = String::from("owned");
        let from_owned: MudText = owned.into();
        assert_eq!(from_owned.text, "owned");
    }

    #[test]
    fn test_from_borrowed_string_and_cow() {
        let owned = String::from("borrowed");
        let from_ref: MudText = (&owned).into();
        assert_eq!(from_ref.text, "borrowed");
        // The source string is still usable.
        assert_eq!(owned, "borrowed");

        let from_cow: MudText = Cow::Borrowed("cow").into();
        assert_eq!(from_cow.text, "cow");
        let from_owned_cow: MudText = Cow::<str>::Owned("cow2".to_string()).into();
        assert_eq!(from_owned_cow.text, "cow2");
    }

    // -------------------------------------------------------------------
    // Builder pattern
    // -------------------------------------------------------------------

    #[test]
    fn test_builder_pattern() {
        // Ensures the fluent builder pattern mutates state correctly.
        let text = MudText::new("Hello")
            .typo(Typo::H1)
            .color(Color::Primary)
            .align(MudAlign::Center)
            .underline(true)
            .classes(["p-4", "bg-surface"]);

        assert_eq!(text.text, "Hello");
        assert_eq!(text.typo, Typo::H1);
        assert_eq!(text.color, Color::Primary);
        assert_eq!(text.align, MudAlign::Center);
        assert!(text.underline);
        assert_eq!(text.base.class_names().len(), 2);
    }

    #[test]
    fn test_all_boolean_toggles() {
        let text = MudText::new("x")
            .inline(true)
            .gutter_bottom(true)
            .truncate(true)
            .no_wrap(true)
            .underline(true)
            .strikethrough(true)
            .selectable(true)
            .italic(true)
            .monospace(true);

        assert!(text.is_inline);
        assert!(text.gutter_bottom);
        assert!(text.truncate);
        assert!(text.no_wrap);
        assert!(text.underline);
        assert!(text.strikethrough);
        assert!(text.is_selectable);
        assert!(text.italic);
        assert!(text.monospace);
    }

    #[test]
    fn test_bold_and_uppercase_are_optional_overrides() {
        let text = MudText::new("x").bold(true).uppercase(false);
        assert_eq!(text.bold, Some(true));
        assert_eq!(text.uppercase, Some(false));

        let untouched = MudText::new("x");
        assert_eq!(untouched.bold, None);
        assert_eq!(untouched.uppercase, None);
    }

    #[test]
    fn test_font_size_and_family_overrides() {
        let text = MudText::new("x")
            .font_size(24.0)
            .font_family(FontFamily::Monospace);

        assert_eq!(text.font_size, Some(24.0));
        assert_eq!(text.font_family, Some(FontFamily::Monospace));
    }

    #[test]
    fn test_opacity_is_clamped() {
        assert_eq!(MudText::new("x").opacity(0.5).opacity, Some(0.5));
        assert_eq!(MudText::new("x").opacity(1.5).opacity, Some(1.0));
        assert_eq!(MudText::new("x").opacity(-0.5).opacity, Some(0.0));
        assert_eq!(MudText::new("x").opacity(0.0).opacity, Some(0.0));
        assert_eq!(MudText::new("x").opacity(1.0).opacity, Some(1.0));
    }

    #[test]
    fn test_opacity_nan_is_ignored() {
        assert_eq!(MudText::new("x").opacity(f32::NAN).opacity, None);
        // A later valid value still applies after a NaN.
        assert_eq!(MudText::new("x").opacity(f32::NAN).opacity(0.25).opacity, Some(0.25));
    }

    #[test]
    fn test_tooltip_storage() {
        let text = MudText::new("x").tooltip("Click me");
        assert_eq!(text.tooltip.as_deref(), Some("Click me"));

        let untouched = MudText::new("x");
        assert_eq!(untouched.tooltip, None);
    }

    #[test]
    fn test_custom_color_overrides_semantic_color() {
        let text = MudText::new("x").color(Color::Primary).custom_color(Color32::RED);
        assert_eq!(text.color, Color::Custom(Color32::RED));
    }

    #[test]
    fn test_classes_accumulate_across_calls() {
        let text = MudText::new("x").classes(["p-2"]).classes(["m-2", "bg-primary"]);
        assert_eq!(text.base.class_names().len(), 3);
    }

    #[test]
    fn test_html_tag_id_aria_and_sense_storage() {
        let text = MudText::new("x")
            .html_tag("span")
            .id("status-line")
            .aria_label("Connection status")
            .sense(Sense::click());

        assert_eq!(text.tag_name(), Some("span"));
        assert_eq!(text.id, Some(Id::new("status-line")));
        assert_eq!(text.aria_label.as_deref(), Some("Connection status"));
        assert_eq!(text.sense, Some(Sense::click()));

        let owned_tag = MudText::new("x").html_tag(String::from("h2"));
        assert_eq!(owned_tag.tag_name(), Some("h2"));
        assert_eq!(MudText::new("x").tag_name(), None);
    }

    // -------------------------------------------------------------------
    // Shorthand constructors
    // -------------------------------------------------------------------

    #[test]
    fn test_shorthand_constructors() {
        // Ensures the macro-generated shorthands map to the correct Typo enum.
        assert_eq!(MudText::h3("x").typo, Typo::H3);
        assert_eq!(MudText::caption("x").typo, Typo::Caption);
        assert_eq!(MudText::from("x").text, "x");
    }

    #[test]
    fn test_all_typo_shorthand_variants() {
        // Exhaustively checks every macro-generated shorthand maps to its Typo variant.
        assert_eq!(MudText::h1("x").typo, Typo::H1);
        assert_eq!(MudText::h2("x").typo, Typo::H2);
        assert_eq!(MudText::h3("x").typo, Typo::H3);
        assert_eq!(MudText::h4("x").typo, Typo::H4);
        assert_eq!(MudText::h5("x").typo, Typo::H5);
        assert_eq!(MudText::h6("x").typo, Typo::H6);
        assert_eq!(MudText::subtitle1("x").typo, Typo::Subtitle1);
        assert_eq!(MudText::subtitle2("x").typo, Typo::Subtitle2);
        assert_eq!(MudText::body1("x").typo, Typo::Body1);
        assert_eq!(MudText::body2("x").typo, Typo::Body2);
        assert_eq!(MudText::button("x").typo, Typo::Button);
        assert_eq!(MudText::caption("x").typo, Typo::Caption);
        assert_eq!(MudText::overline("x").typo, Typo::Overline);
    }

    // -------------------------------------------------------------------
    // Theme / palette resolution
    // -------------------------------------------------------------------

    #[test]
    fn test_get_variant() {
        // Ensures `MudText` pulls the correct configuration from the Theme.
        let theme = MudTheme::default();
        let text = MudText::new("Test").typo(Typo::H1);

        let variant = text.get_variant(&theme);
        assert_eq!(variant.font_size, theme.typography.h1.font_size);
        assert_eq!(variant.is_bold, theme.typography.h1.is_bold);
    }

    #[test]
    fn test_get_variant_covers_every_typo_scale() {
        let theme = MudTheme::default();
        let scales = [
            Typo::Default,
            Typo::H1,
            Typo::H2,
            Typo::H3,
            Typo::H4,
            Typo::H5,
            Typo::H6,
            Typo::Subtitle1,
            Typo::Subtitle2,
            Typo::Body1,
            Typo::Body2,
            Typo::Button,
            Typo::Caption,
            Typo::Overline,
        ];

        for scale in scales {
            // Should not panic for any variant, and should return a variant whose
            // font size is finite and positive.
            let text = MudText::new("x").typo(scale);
            let variant = text.get_variant(&theme);
            assert!(variant.font_size > 0.0);
        }
    }

    #[test]
    fn test_resolve_color() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();

        // Standard Semantic Color
        let primary_text = MudText::new("Test").color(Color::Primary);
        assert_eq!(primary_text.resolve_color(palette), Some(palette.primary.into()));

        // Inherited Color (Falls back to None so egui's default text color takes over)
        let inherit_text = MudText::new("Test").color(Color::Inherit);
        assert_eq!(inherit_text.resolve_color(palette), None);

        // Custom Egui Color
        let custom = MudText::new("Test").custom_color(Color32::RED);
        assert_eq!(custom.resolve_color(palette), Some(Color32::RED));
    }

    // -------------------------------------------------------------------
    // Style resolution (pure, no Ui)
    // -------------------------------------------------------------------

    #[test]
    fn test_resolve_style_uses_theme_variant_by_default() {
        let theme = MudTheme::default();
        let style = MudText::h1("x").resolve_style(&theme, Color32::WHITE);

        assert_eq!(style.font_size, theme.typography.h1.font_size);
        assert_eq!(style.bold, theme.typography.h1.is_bold);
        assert_eq!(style.family, to_egui_family(theme.typography.h1.font_family));
        assert!(!style.italic);
        assert!(!style.underline);
        assert!(!style.strikethrough);
        assert!(!style.inline);
        assert!(approx(style.gutter, 0.0));
    }

    #[test]
    fn test_resolve_style_builder_overrides_win() {
        let theme = MudTheme::default();
        let style = MudText::h1("x")
            .font_size(31.0)
            .bold(!theme.typography.h1.is_bold)
            .italic(true)
            .underline(true)
            .strikethrough(true)
            .resolve_style(&theme, Color32::WHITE);

        assert!(approx(style.font_size, 31.0));
        assert_eq!(style.bold, !theme.typography.h1.is_bold);
        assert!(style.italic);
        assert!(style.underline);
        assert!(style.strikethrough);
    }

    #[test]
    fn test_resolve_style_rejects_invalid_font_sizes() {
        let theme = MudTheme::default();
        let expected = theme.typography.h2.font_size;

        for bad in [0.0, -4.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let style = MudText::h2("x").font_size(bad).resolve_style(&theme, Color32::WHITE);
            assert_eq!(style.font_size, expected, "size {bad} should be ignored");
        }
    }

    #[test]
    fn test_resolve_style_family_precedence() {
        let theme = MudTheme::default();

        // Explicit family beats the monospace flag.
        let explicit = MudText::new("x")
            .monospace(true)
            .font_family(FontFamily::Proportional)
            .resolve_style(&theme, Color32::WHITE);
        assert_eq!(explicit.family, FontFamily::Proportional);

        // Monospace flag beats the theme.
        let mono = MudText::new("x").monospace(true).resolve_style(&theme, Color32::WHITE);
        assert_eq!(mono.family, FontFamily::Monospace);

        // Without either, the theme variant decides.
        let themed = MudText::new("x").resolve_style(&theme, Color32::WHITE);
        assert_eq!(themed.family, to_egui_family(theme.typography.default.font_family));
    }

    #[test]
    fn test_resolve_style_color_without_opacity() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();

        let inherit = MudText::new("x").color(Color::Inherit).resolve_style(&theme, Color32::WHITE);
        assert_eq!(inherit.color, None);

        let primary: Color32 = palette.primary.into();
        let semantic = MudText::new("x").color(Color::Primary).resolve_style(&theme, Color32::WHITE);
        assert_eq!(semantic.color, Some(primary));

        let custom = MudText::new("x").custom_color(Color32::RED).resolve_style(&theme, Color32::WHITE);
        assert_eq!(custom.color, Some(Color32::RED));
    }

    #[test]
    fn test_resolve_style_opacity_multiplies_resolved_color() {
        let theme = MudTheme::default();
        let palette = theme.current_palette();
        let primary: Color32 = palette.primary.into();

        let style = MudText::new("x")
            .color(Color::Primary)
            .opacity(0.5)
            .resolve_style(&theme, Color32::WHITE);
        assert_eq!(style.color, Some(primary.gamma_multiply(0.5)));
    }

    #[test]
    fn test_resolve_style_opacity_uses_fallback_when_color_inherits() {
        let theme = MudTheme::default();
        let fallback = Color32::from_rgb(200, 100, 50);

        let style = MudText::new("x")
            .color(Color::Inherit)
            .opacity(0.5)
            .resolve_style(&theme, fallback);
        assert_eq!(style.color, Some(fallback.gamma_multiply(0.5)));
    }

    #[test]
    fn test_resolve_style_zero_opacity_is_fully_transparent() {
        let theme = MudTheme::default();
        let style = MudText::new("x")
            .custom_color(Color32::from_rgb(255, 255, 255))
            .opacity(0.0)
            .resolve_style(&theme, Color32::WHITE);
        assert_eq!(style.color.map(|c| c.a()), Some(0));
    }

    #[test]
    fn test_resolve_style_uppercase_flag() {
        let theme = MudTheme::default();
        assert!(MudText::button("x").resolve_style(&theme, Color32::WHITE).uppercase);
        assert!(MudText::overline("x").resolve_style(&theme, Color32::WHITE).uppercase);
        assert!(!MudText::body1("x").resolve_style(&theme, Color32::WHITE).uppercase);
        assert!(!MudText::button("x").uppercase(false).resolve_style(&theme, Color32::WHITE).uppercase);
        assert!(MudText::body1("x").uppercase(true).resolve_style(&theme, Color32::WHITE).uppercase);
    }

    #[test]
    fn test_resolve_style_gutter_is_tied_to_font_size() {
        let theme = MudTheme::default();
        let style = MudText::new("x")
            .font_size(20.0)
            .gutter_bottom(true)
            .resolve_style(&theme, Color32::WHITE);
        assert!(approx(style.gutter, 20.0 * GUTTER_EM));

        let none = MudText::new("x").font_size(20.0).resolve_style(&theme, Color32::WHITE);
        assert!(approx(none.gutter, 0.0));
    }

    #[test]
    fn test_resolve_style_inline_disables_layout_and_gutter() {
        let theme = MudTheme::default();
        let style = MudText::new("x")
            .align(MudAlign::Center)
            .gutter_bottom(true)
            .inline(true)
            .resolve_style(&theme, Color32::WHITE);

        assert!(style.inline);
        assert_eq!(style.layout, None);
        assert!(approx(style.gutter, 0.0));
    }

    #[test]
    fn test_resolve_style_layout_follows_alignment() {
        let theme = MudTheme::default();

        let centered = MudText::new("x").align(MudAlign::Center).resolve_style(&theme, Color32::WHITE);
        assert_eq!(centered.layout, MudAlign::Center.layout());

        let inherited = MudText::new("x").resolve_style(&theme, Color32::WHITE);
        assert_eq!(inherited.layout, MudAlign::Inherit.layout());
    }

    // -------------------------------------------------------------------
    // HTML tag semantics
    // -------------------------------------------------------------------

    #[test]
    fn test_is_inline_html_tag() {
        for tag in ["span", "SPAN", "Span", "b", "code", "a", "label"] {
            assert!(is_inline_html_tag(tag), "{tag} should be inline");
        }
        for tag in ["p", "div", "h1", "h6", "section", "", "not-a-tag"] {
            assert!(!is_inline_html_tag(tag), "{tag} should not be inline");
        }
    }

    #[test]
    fn test_inline_tag_makes_component_inline() {
        assert!(MudText::new("x").html_tag("span").is_effectively_inline());
        assert!(!MudText::new("x").html_tag("p").is_effectively_inline());
        assert!(!MudText::new("x").is_effectively_inline());
        assert!(MudText::new("x").inline(true).is_effectively_inline());
        // An explicit inline flag is not undone by a block tag.
        assert!(MudText::new("x").inline(true).html_tag("p").is_effectively_inline());
    }

    #[test]
    fn test_inline_tag_disables_alignment_and_gutter_in_style() {
        let theme = MudTheme::default();
        let style = MudText::new("x")
            .html_tag("span")
            .align(MudAlign::Center)
            .gutter_bottom(true)
            .resolve_style(&theme, Color32::WHITE);

        assert!(style.inline);
        assert_eq!(style.layout, None);
        assert!(approx(style.gutter, 0.0));
    }

    // -------------------------------------------------------------------
    // Display text (uppercase handling)
    // -------------------------------------------------------------------

    #[test]
    fn test_uppercase_defaults() {
        // Ensures specific typography scales (Button, Overline) automatically transform to ALL CAPS,
        // but can be overridden.
        assert_eq!(MudText::button("save").display_text(), "SAVE");
        assert_eq!(MudText::overline("tag").display_text(), "TAG");
        assert_eq!(MudText::body1("Keep Me").display_text(), "Keep Me");
        assert_eq!(MudText::button("save").uppercase(false).display_text(), "save");
    }

    #[test]
    fn test_is_uppercase_helper() {
        assert!(MudText::button("x").is_uppercase());
        assert!(!MudText::body1("x").is_uppercase());
        assert!(MudText::body1("x").uppercase(true).is_uppercase());
        assert!(!MudText::overline("x").uppercase(false).is_uppercase());
    }

    #[test]
    fn test_display_text_borrowed_when_not_uppercased() {
        let text = MudText::body1("Keep Me");
        match text.display_text() {
            Cow::Borrowed(s) => assert_eq!(s, "Keep Me"),
            Cow::Owned(_) => panic!("expected a borrowed Cow when no uppercase transform applies"),
        }
    }

    #[test]
    fn test_display_text_owned_when_uppercased() {
        let text = MudText::button("save");
        match text.display_text() {
            Cow::Owned(s) => assert_eq!(s, "SAVE"),
            Cow::Borrowed(_) => panic!("expected an owned Cow when an uppercase transform applies"),
        }
    }

    #[test]
    fn test_display_text_handles_unicode() {
        // German "straße" -> "STRASSE" under full Unicode uppercasing rules.
        let text = MudText::button("straße");
        assert_eq!(text.display_text(), "STRASSE");
    }

    #[test]
    fn test_display_text_empty_string() {
        let text = MudText::new("");
        assert_eq!(text.display_text(), "");

        let upper_empty = MudText::button("");
        assert_eq!(upper_empty.display_text(), "");
    }

    #[test]
    fn test_into_display_text_moves_without_uppercase() {
        let mut text = MudText::body1("Keep Me");
        let extracted = text.into_display_text();
        assert_eq!(extracted, "Keep Me");
        // The source buffer was moved out, leaving the original field empty.
        assert_eq!(text.text, "");
    }

    #[test]
    fn test_into_display_text_uppercases_without_mutating_source() {
        let mut text = MudText::button("save");
        let extracted = text.into_display_text();
        assert_eq!(extracted, "SAVE");
        // Uppercasing allocates a new string, so the original field is untouched.
        assert_eq!(text.text, "save");
    }

    // -------------------------------------------------------------------
    // Utility-class parsing
    // -------------------------------------------------------------------

    #[test]
    fn test_utilities_spacing() {
        // Ensures the utility parser correctly interprets and aggregates padding and margins.
        let u = Utilities::parse(["p-4", "mx-2", "pt-0.5", "unknown", "bg-primary"]);

        // p-4 = 16px all sides. pt-0.5 overrides top to 2.0px.
        assert_eq!(u.padding, Edges { top: 2.0, right: 16.0, bottom: 16.0, left: 16.0 });

        // mx-2 = 8px left/right.
        assert_eq!(u.margin, Edges { top: 0.0, right: 8.0, bottom: 0.0, left: 8.0 });

        assert_eq!(u.background, Some(Color::Primary));
        assert!(!u.is_empty());
    }

    #[test]
    fn test_utilities_ignore_garbage() {
        // Ensures the parser doesn't panic or pollute state when given invalid strings.
        let u = Utilities::parse(["p-", "p-abc", "p--2", "flex", "bg-nope", ""]);
        assert!(u.is_empty());
    }

    #[test]
    fn test_utilities_from_component() {
        let u = MudText::new("x").classes(["py-1", "ml-3"]).utilities();
        assert_eq!(u.padding.top, 4.0);
        assert_eq!(u.padding.bottom, 4.0);
        assert_eq!(u.margin.left, 12.0);
    }

    #[test]
    fn test_utilities_last_class_wins_on_conflict() {
        // Later classes in the list override earlier, conflicting ones.
        let u = Utilities::parse(["p-4", "p-2"]);
        assert_eq!(u.padding, Edges { top: 8.0, right: 8.0, bottom: 8.0, left: 8.0 });
    }

    #[test]
    fn test_utilities_empty_when_no_classes() {
        let u = MudText::new("x").utilities();
        assert!(u.is_empty());
    }

    // -------------------------------------------------------------------
    // Rendering (real egui frame)
    // -------------------------------------------------------------------

    #[test]
    fn test_render_allocates_a_non_empty_rect() {
        let size = rendered_size(MudText::new("Hello, world"));
        assert!(size.x > 0.0, "width should be positive, got {}", size.x);
        assert!(size.y > 0.0, "height should be positive, got {}", size.y);
    }

    #[test]
    fn test_render_larger_font_is_taller_and_wider() {
        let small = rendered_size(MudText::new("Hello").font_size(12.0).inline(true));
        let large = rendered_size(MudText::new("Hello").font_size(36.0).inline(true));
        assert!(large.y > small.y);
        assert!(large.x > small.x);
    }

    #[test]
    fn test_render_longer_text_is_wider() {
        let short = rendered_size(MudText::new("Hi").font_size(16.0).inline(true));
        let long = rendered_size(MudText::new("Hi there, long line").font_size(16.0).inline(true));
        assert!(long.x > short.x);
    }

    #[test]
    fn test_render_gutter_bottom_advances_cursor_by_035_em() {
        let plain = cursor_delta(MudText::new("Hello").font_size(20.0));
        let guttered = cursor_delta(MudText::new("Hello").font_size(20.0).gutter_bottom(true));
        assert!(
            approx(guttered - plain, 20.0 * GUTTER_EM),
            "expected a {} pt difference, got {}",
            20.0 * GUTTER_EM,
            guttered - plain
        );
    }

    #[test]
    fn test_render_gutter_bottom_ignored_when_inline() {
        let plain = cursor_delta(MudText::new("Hello").font_size(20.0).inline(true));
        let guttered = cursor_delta(
            MudText::new("Hello").font_size(20.0).inline(true).gutter_bottom(true),
        );
        assert!(approx(guttered, plain));
    }

    #[test]
    fn test_render_gutter_bottom_ignored_for_inline_html_tag() {
        let plain = cursor_delta(MudText::new("Hello").font_size(20.0).html_tag("span"));
        let guttered = cursor_delta(
            MudText::new("Hello").font_size(20.0).html_tag("span").gutter_bottom(true),
        );
        assert!(approx(guttered, plain));
    }

    #[test]
    fn test_render_padding_utility_grows_allocated_height() {
        // `p-4` is 16 px on every side, so the frame adds 32 px of height.
        let plain = cursor_delta(MudText::new("Hello").font_size(16.0));
        let padded = cursor_delta(MudText::new("Hello").font_size(16.0).classes(["p-4"]));
        assert!(
            padded - plain >= 31.9,
            "expected at least 32 px of extra height, got {}",
            padded - plain
        );
    }

    #[test]
    fn test_render_ui_matches_ui_with_theme_for_default_theme() {
        let theme = MudTheme::default();
        let via_theme = run_ui(|ui| {
            MudText::new("Same")
                .font_family(FontFamily::Proportional)
                .ui_with_theme(ui, &theme)
                .rect
                .size()
        });
        let via_ui = run_ui(|ui| {
            MudText::new("Same")
                .font_family(FontFamily::Proportional)
                .ui(ui)
                .rect
                .size()
        });
        assert_eq!(via_theme, via_ui);
    }

    #[test]
    fn test_render_through_widget_trait() {
        let size = run_ui(|ui| {
            ui.add(MudText::new("Via add").font_family(FontFamily::Proportional))
                .rect
                .size()
        });
        assert!(size.x > 0.0 && size.y > 0.0);
    }

    #[test]
    fn test_render_with_every_modifier_does_not_panic() {
        let size = rendered_size(
            MudText::h4("Everything")
                .color(Color::Primary)
                .opacity(0.8)
                .bold(true)
                .italic(true)
                .underline(true)
                .strikethrough(true)
                .selectable(true)
                .truncate(true)
                .no_wrap(true)
                .uppercase(true)
                .tooltip("tip")
                .aria_label("Everything label")
                .id("everything")
                .sense(Sense::click())
                .align(MudAlign::Center)
                .gutter_bottom(true)
                .classes(["p-2", "mx-1", "bg-primary"]),
        );
        assert!(size.x > 0.0 && size.y > 0.0);
    }

    #[test]
    fn test_render_empty_text_does_not_panic() {
        let size = rendered_size(MudText::new(""));
        assert!(size.x >= 0.0 && size.y >= 0.0);
    }

    #[test]
    fn test_render_same_id_twice_is_stable() {
        // Two texts sharing an id salt must still both render (egui may warn about
        // clashing ids in debug builds, but must not panic).
        let theme = MudTheme::default();
        let both = run_ui(|ui| {
            let a = MudText::new("A").id("dup").font_family(FontFamily::Proportional).ui_with_theme(ui, &theme);
            let b = MudText::new("B").id("dup").font_family(FontFamily::Proportional).ui_with_theme(ui, &theme);
            (a.rect.size(), b.rect.size())
        });
        assert!(both.0.x > 0.0 && both.1.x > 0.0);
    }
}