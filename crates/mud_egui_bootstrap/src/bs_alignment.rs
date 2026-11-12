//! Bootstrap-style alignment utilities (`align-self-*`, `align-items-*`, `text-*`, `ms-auto`, ...).
//!
//! Same shape as `bs_background.rs`: a pure parser, a version-keyed cache, and unit tests.
//! Nothing in here touches egui, so every decision is testable without a `Ui`.
//!
//! # Supported classes
//!
//! | Class                                              | Meaning                                         |
//! |----------------------------------------------------|-------------------------------------------------|
//! | `align-self-start / center / end / stretch`        | Cross-axis placement of ONE item in its row     |
//! | `align-items-start / center / end / stretch`       | Default cross-axis placement for a container's items |
//! | `justify-content-start / center / end / between / around / evenly` | Main-axis distribution (container) |
//! | `text-start / center / end`                        | Horizontal placement of content inside an item  |
//! | `ms-auto` / `me-auto` / `mx-auto`                  | Auto margins: push right / push left / center   |
//! | `mt-auto` / `mb-auto` / `my-auto`                  | Auto margins: push bottom / push top / center   |
//! | `m-auto`                                           | Center both ways                                |
//! | `w-100` / `h-100`                                  | Fill the cell width / height                    |
//!
//! "Start" is top (vertical) or left (horizontal); "end" is bottom or right.
//!
//! # Precedence
//! * Within one category the **last class wins** (`align-self-start align-self-end` => end).
//! * Vertical: `align-self-*`, then auto margins (`mt-auto`/`mb-auto`/`my-auto`), then the
//!   container's `align-items-*` default.
//! * Horizontal: auto margins (`ms-auto`/`me-auto`/`mx-auto`), then `text-*`.
//!
//! # Why a class alone is not enough
//! A child only knows its own size. To sit at the bottom of a row it needs the row's height,
//! and in a single-pass immediate-mode UI that height is only known after drawing. The
//! container (`MudGrid`) therefore remembers the previous frame's row heights and hands the
//! slot height to [`vertical_offset`] / [`stretch_target`] below.

use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{AlignItems, Justify};

// =============================================================================
//  1. Types
// =============================================================================

/// Cross-axis placement (vertical inside a grid row or a horizontal stack).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossAlign {
    Start,
    Center,
    End,
    Stretch,
}

/// Horizontal placement of content inside its cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HAlign {
    Start,
    Center,
    End,
}

/// Main-axis distribution, `justify-content-*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainJustify {
    Start,
    Center,
    End,
    Between,
    Around,
    Evenly,
}

/// Which auto margins were requested. Flags are additive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AutoMargins {
    pub start: bool,
    pub end: bool,
    pub top: bool,
    pub bottom: bool,
}

/// One recognised alignment class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignClass {
    SelfCross(CrossAlign),
    ItemsCross(CrossAlign),
    Justify(MainJustify),
    Text(HAlign),
    Auto(AutoSide),
    FillWidth,
    FillHeight,
}

/// Which side an `*-auto` margin class names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoSide {
    Start,
    End,
    Top,
    Bottom,
    X,
    Y,
    All,
}

/// Everything the alignment classes of one element resolve to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AlignSpec {
    pub align_self: Option<CrossAlign>,
    pub align_items: Option<CrossAlign>,
    pub justify_content: Option<MainJustify>,
    pub text: Option<HAlign>,
    pub auto: AutoMargins,
    pub fill_width: bool,
    pub fill_height: bool,
}

// =============================================================================
//  2. The Parser
// =============================================================================

fn cross(name: &str) -> Option<CrossAlign> {
    match name {
        "start" => Some(CrossAlign::Start),
        "center" => Some(CrossAlign::Center),
        "end" => Some(CrossAlign::End),
        "stretch" => Some(CrossAlign::Stretch),
        _ => None, // `baseline` is not supported in egui's layout model
    }
}

fn main_justify(name: &str) -> Option<MainJustify> {
    match name {
        "start" => Some(MainJustify::Start),
        "center" => Some(MainJustify::Center),
        "end" => Some(MainJustify::End),
        "between" => Some(MainJustify::Between),
        "around" => Some(MainJustify::Around),
        "evenly" => Some(MainJustify::Evenly),
        _ => None,
    }
}

fn h_align(name: &str) -> Option<HAlign> {
    match name {
        "start" => Some(HAlign::Start),
        "center" => Some(HAlign::Center),
        "end" => Some(HAlign::End),
        _ => None,
    }
}

/// Parses one class. Returns `None` when it is not an alignment class.
pub fn try_parse(cls: &str) -> Option<AlignClass> {
    if let Some(v) = cls.strip_prefix("align-self-") {
        return cross(v).map(AlignClass::SelfCross);
    }
    if let Some(v) = cls.strip_prefix("align-items-") {
        return cross(v).map(AlignClass::ItemsCross);
    }
    if let Some(v) = cls.strip_prefix("justify-content-") {
        return main_justify(v).map(AlignClass::Justify);
    }
    if let Some(v) = cls.strip_prefix("text-") {
        return h_align(v).map(AlignClass::Text);
    }
    match cls {
        "ms-auto" => Some(AlignClass::Auto(AutoSide::Start)),
        "me-auto" => Some(AlignClass::Auto(AutoSide::End)),
        "mt-auto" => Some(AlignClass::Auto(AutoSide::Top)),
        "mb-auto" => Some(AlignClass::Auto(AutoSide::Bottom)),
        "mx-auto" => Some(AlignClass::Auto(AutoSide::X)),
        "my-auto" => Some(AlignClass::Auto(AutoSide::Y)),
        "m-auto" => Some(AlignClass::Auto(AutoSide::All)),
        "w-100" => Some(AlignClass::FillWidth),
        "h-100" => Some(AlignClass::FillHeight),
        _ => None,
    }
}

impl AlignSpec {
    /// Folds a list of classes into a spec. Non-alignment classes are ignored, so this is safe
    /// to run over a mixed list such as `["p-4", "bg-primary", "align-self-end"]`.
    pub fn from_classes<I, S>(classes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut spec = Self::default();
        for cls in classes {
            if let Some(parsed) = try_parse(cls.as_ref()) {
                spec.apply(parsed);
            }
        }
        spec
    }

    fn apply(&mut self, class: AlignClass) {
        match class {
            AlignClass::SelfCross(a) => self.align_self = Some(a),
            AlignClass::ItemsCross(a) => self.align_items = Some(a),
            AlignClass::Justify(j) => self.justify_content = Some(j),
            AlignClass::Text(t) => self.text = Some(t),
            AlignClass::FillWidth => self.fill_width = true,
            AlignClass::FillHeight => self.fill_height = true,
            AlignClass::Auto(side) => match side {
                AutoSide::Start => self.auto.start = true,
                AutoSide::End => self.auto.end = true,
                AutoSide::Top => self.auto.top = true,
                AutoSide::Bottom => self.auto.bottom = true,
                AutoSide::X => {
                    self.auto.start = true;
                    self.auto.end = true;
                }
                AutoSide::Y => {
                    self.auto.top = true;
                    self.auto.bottom = true;
                }
                AutoSide::All => {
                    self.auto.start = true;
                    self.auto.end = true;
                    self.auto.top = true;
                    self.auto.bottom = true;
                }
            },
        }
    }

    /// `true` when no alignment class was found at all.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// Effective vertical placement of this element, given the container's default.
    ///
    /// `align-self-*` > auto margins > `container_default` (the container's `align-items-*`).
    pub fn vertical(&self, container_default: Option<CrossAlign>) -> Option<CrossAlign> {
        if let Some(a) = self.align_self {
            return Some(a);
        }
        match (self.auto.top, self.auto.bottom) {
            (true, true) => return Some(CrossAlign::Center),
            (true, false) => return Some(CrossAlign::End), // margin above pushes it down
            (false, true) => return Some(CrossAlign::Start), // margin below pushes it up
            (false, false) => {}
        }
        container_default
    }

    /// Effective horizontal placement of this element's content.
    ///
    /// Auto margins > `text-*`.
    pub fn horizontal(&self) -> Option<HAlign> {
        match (self.auto.start, self.auto.end) {
            (true, true) => Some(HAlign::Center),
            (true, false) => Some(HAlign::End), // margin on the left pushes it right
            (false, true) => Some(HAlign::Start), // margin on the right pushes it left
            (false, false) => self.text,
        }
    }
}

// =============================================================================
//  3. The Cache
// =============================================================================

/// Holds the parsed spec and the `ComponentBase` class version it was parsed at.
#[derive(Default)]
pub struct AlignCache {
    version: u64,
    parsed: AlignSpec,
}

impl AlignCache {
    /// Resolves the alignment classes of `base`, re-parsing only when its classes changed.
    pub fn resolve(&mut self, base: &ComponentBase) -> AlignSpec {
        if self.version == base.classes_version() {
            return self.parsed;
        }
        self.parsed = AlignSpec::from_classes(base.class_names());
        self.version = base.classes_version();
        self.parsed
    }
}

// =============================================================================
//  4. Pure layout math (what the container does with the spec)
// =============================================================================

/// Space to insert before an item so it sits at `align` inside a slot of `slot` points,
/// given its own `content` size. Never negative; non-finite input yields `0.0`.
pub fn vertical_offset(align: CrossAlign, slot: f32, content: f32) -> f32 {
    let free = slot - content;
    if !free.is_finite() || free <= 0.0 {
        return 0.0;
    }
    match align {
        CrossAlign::Start | CrossAlign::Stretch => 0.0,
        CrossAlign::Center => free * 0.5,
        CrossAlign::End => free,
    }
}

/// Minimum height an item should be given so it fills `slot`, or `None` when it should not
/// be stretched (not `Stretch`, or the slot is unknown / not positive / not finite).
pub fn stretch_target(align: CrossAlign, slot: f32) -> Option<f32> {
    if align == CrossAlign::Stretch && slot.is_finite() && slot > 0.0 {
        Some(slot)
    } else {
        None
    }
}

/// `true` when two height lists differ enough to be worth storing and repainting for.
pub fn heights_changed(previous: Option<&[f32]>, current: &[f32], epsilon: f32) -> bool {
    match previous {
        None => true,
        Some(prev) => {
            prev.len() != current.len()
                || prev.iter().zip(current).any(|(a, b)| (a - b).abs() > epsilon)
        }
    }
}

// =============================================================================
//  5. Bridges to the existing enums (so MudStack can accept the same classes)
// =============================================================================

impl From<CrossAlign> for AlignItems {
    fn from(a: CrossAlign) -> Self {
        match a {
            CrossAlign::Start => AlignItems::Start,
            CrossAlign::Center => AlignItems::Center,
            CrossAlign::End => AlignItems::End,
            CrossAlign::Stretch => AlignItems::Stretch,
        }
    }
}

impl From<MainJustify> for Justify {
    fn from(j: MainJustify) -> Self {
        match j {
            MainJustify::Start => Justify::Start,
            MainJustify::Center => Justify::Center,
            MainJustify::End => Justify::End,
            MainJustify::Between => Justify::SpaceBetween,
            MainJustify::Around => Justify::SpaceAround,
            MainJustify::Evenly => Justify::SpaceEvenly,
        }
    }
}

// =============================================================================
//  6. Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_each_family() {
        assert_eq!(try_parse("align-self-end"), Some(AlignClass::SelfCross(CrossAlign::End)));
        assert_eq!(try_parse("align-items-center"), Some(AlignClass::ItemsCross(CrossAlign::Center)));
        assert_eq!(try_parse("justify-content-between"), Some(AlignClass::Justify(MainJustify::Between)));
        assert_eq!(try_parse("text-end"), Some(AlignClass::Text(HAlign::End)));
        assert_eq!(try_parse("ms-auto"), Some(AlignClass::Auto(AutoSide::Start)));
        assert_eq!(try_parse("w-100"), Some(AlignClass::FillWidth));
    }

    #[test]
    fn rejects_non_alignment_and_unsupported() {
        assert_eq!(try_parse("bg-primary"), None);
        assert_eq!(try_parse("p-4"), None);
        assert_eq!(try_parse("align-self-baseline"), None);
        assert_eq!(try_parse("align-self-"), None);
        assert_eq!(try_parse("text-muted"), None);
        assert_eq!(try_parse("text-"), None);
    }

    #[test]
    fn last_class_wins_per_category() {
        let s = AlignSpec::from_classes(["align-self-start", "p-2", "align-self-end"]);
        assert_eq!(s.align_self, Some(CrossAlign::End));
        let s = AlignSpec::from_classes(["text-start", "text-center"]);
        assert_eq!(s.text, Some(HAlign::Center));
    }

    #[test]
    fn mixed_lists_ignore_foreign_classes() {
        let s = AlignSpec::from_classes(["bg-primary", "rounded-0", "align-self-center"]);
        assert_eq!(s.align_self, Some(CrossAlign::Center));
        assert!(AlignSpec::from_classes(["bg-primary", "p-4"]).is_empty());
    }

    #[test]
    fn vertical_precedence() {
        let default = Some(CrossAlign::Center);
        // container default only
        assert_eq!(AlignSpec::default().vertical(default), default);
        // auto margin beats container default
        let s = AlignSpec::from_classes(["mt-auto"]);
        assert_eq!(s.vertical(default), Some(CrossAlign::End));
        let s = AlignSpec::from_classes(["mb-auto"]);
        assert_eq!(s.vertical(default), Some(CrossAlign::Start));
        let s = AlignSpec::from_classes(["my-auto"]);
        assert_eq!(s.vertical(None), Some(CrossAlign::Center));
        // align-self beats everything
        let s = AlignSpec::from_classes(["mt-auto", "align-self-start"]);
        assert_eq!(s.vertical(default), Some(CrossAlign::Start));
        // nothing set anywhere
        assert_eq!(AlignSpec::default().vertical(None), None);
    }

    #[test]
    fn horizontal_precedence() {
        assert_eq!(AlignSpec::from_classes(["ms-auto"]).horizontal(), Some(HAlign::End));
        assert_eq!(AlignSpec::from_classes(["me-auto"]).horizontal(), Some(HAlign::Start));
        assert_eq!(AlignSpec::from_classes(["mx-auto"]).horizontal(), Some(HAlign::Center));
        assert_eq!(AlignSpec::from_classes(["text-end"]).horizontal(), Some(HAlign::End));
        // auto margin beats text-*
        assert_eq!(AlignSpec::from_classes(["text-start", "ms-auto"]).horizontal(), Some(HAlign::End));
        assert_eq!(AlignSpec::default().horizontal(), None);
    }

    #[test]
    fn m_auto_centers_both_ways() {
        let s = AlignSpec::from_classes(["m-auto"]);
        assert_eq!(s.vertical(None), Some(CrossAlign::Center));
        assert_eq!(s.horizontal(), Some(HAlign::Center));
    }

    #[test]
    fn vertical_offset_math() {
        assert_eq!(vertical_offset(CrossAlign::Start, 100.0, 40.0), 0.0);
        assert_eq!(vertical_offset(CrossAlign::Center, 100.0, 40.0), 30.0);
        assert_eq!(vertical_offset(CrossAlign::End, 100.0, 40.0), 60.0);
        assert_eq!(vertical_offset(CrossAlign::Stretch, 100.0, 40.0), 0.0);
    }

    #[test]
    fn vertical_offset_never_negative_or_non_finite() {
        assert_eq!(vertical_offset(CrossAlign::End, 40.0, 100.0), 0.0); // overflow
        assert_eq!(vertical_offset(CrossAlign::End, 40.0, 40.0), 0.0);
        assert_eq!(vertical_offset(CrossAlign::End, f32::INFINITY, 10.0), 0.0);
        assert_eq!(vertical_offset(CrossAlign::Center, f32::NAN, 10.0), 0.0);
    }

    #[test]
    fn stretch_target_only_for_stretch_with_a_real_slot() {
        assert_eq!(stretch_target(CrossAlign::Stretch, 80.0), Some(80.0));
        assert_eq!(stretch_target(CrossAlign::End, 80.0), None);
        assert_eq!(stretch_target(CrossAlign::Stretch, 0.0), None);
        assert_eq!(stretch_target(CrossAlign::Stretch, f32::INFINITY), None);
        assert_eq!(stretch_target(CrossAlign::Stretch, f32::NAN), None);
    }

    #[test]
    fn heights_changed_uses_an_epsilon() {
        assert!(heights_changed(None, &[1.0], 0.5));
        assert!(!heights_changed(Some(&[1.0, 2.0]), &[1.0, 2.4], 0.5));
        assert!(heights_changed(Some(&[1.0, 2.0]), &[1.0, 2.6], 0.5));
        assert!(heights_changed(Some(&[1.0]), &[1.0, 2.0], 0.5));
    }

    #[test]
    fn cache_reparses_only_when_version_changes() {
        let mut base = ComponentBase::new();
        let mut cache = AlignCache::default();

        base.set_classes(vec!["align-self-end"]);
        assert_eq!(cache.resolve(&base).align_self, Some(CrossAlign::End));

        base.add_class("align-self-center");
        assert_eq!(cache.resolve(&base).align_self, Some(CrossAlign::Center));
    }
}