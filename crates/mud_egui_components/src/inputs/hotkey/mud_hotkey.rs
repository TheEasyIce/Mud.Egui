//! A hotkey listener that reveals or hides its child content.
//!
//! **Version note.** This component is a port of **MudBlazor 9.0.0's
//! `MudHotkey`**, not of any 8.13.0 component — MudHotkey was introduced in
//! the v9.0.0 release notes ("New Features → MudHotkey: Add new component for
//! handling hotkeys"). The rest of the workspace targets 8.13.0; this file is
//! the one deliberate version exception, documented in the README.
//!
//! The public parameter surface mirrors the 9.x Blazor component:
//!
//! | MudBlazor parameter | Rust field / method |
//! |---------------------|---------------------|
//! | `Key` | [`MudHotkey::key`] |
//! | `KeyModifiers` | [`MudHotkey::key_modifiers`] |
//! | `OnHotkeyPressed` | [`MudHotkey::on_hotkey_pressed`] + [`HotkeyResponse::pressed`] |
//! | `Disabled` | [`MudHotkey::disabled`] |
//! | `PreventEventPropagation` | [`MudHotkey::prevent_event_propagation`] |
//! | `HideChildContentOnRepress` | [`MudHotkey::hide_child_content_on_repress`] |
//! | `ChildContent` | the closure passed to [`MudHotkey::show`] |
//! | `Class` | [`MudHotkey::classes`] (stored, not resolved) |
//! | `Tag` | [`MudHotkey::tag`] / [`MudHotkey::tag_ref`] |
//! | `Style` / `UserAttributes` | not ported — no CSS strings, no HTML tag |
//!
//! # Key and modifier substitution
//!
//! | MudBlazor | Rust | Note |
//! |-----------|------|------|
//! | `JsKey` (browser `KeyboardEvent.code`) | [`egui::Key`] | egui's key enum; the common keys map 1:1, browser-only codes (e.g. `NumpadEqual`) have no egui equivalent. |
//! | `JsKeyModifier` (`Control`, `ControlLeft`, `ControlRight`, …) | [`egui::Modifiers`] | egui's modifier state is a set of flags (`ctrl`, `shift`, `alt`, `mac_cmd`, `command`); it does **not** distinguish left from right. |
//!
//! The left/right distinction is the reason MudBlazor's own bug tracker has an
//! open issue ("XxxRight modifiers do not work", #13390); egui sidesteps the
//! problem by not distinguishing them at all.
//!
//! # Behaviour
//!
//! Every frame the component checks whether `key` was pressed while
//! `key_modifiers` were held. When it was:
//!
//! 1. [`on_hotkey_pressed`](Self::on_hotkey_pressed) fires with a
//!    [`HotkeyEventArgs`] carrying the key and whether the event was a repeat.
//! 2. If [`prevent_event_propagation`](Self::prevent_event_propagation) is
//!    set, the key is consumed via `InputState::consume_key`, so other widgets
//!    in the same frame do not see it.
//! 3. If [`hide_child_content_on_repress`](Self::hide_child_content_on_repress)
//!    is set (the default), the child content's visibility **toggles**. The
//!    content starts visible; the first press hides it, the next press shows it
//!    again.
//!
//! When [`disabled`](Self::disabled) is set, the hotkey is not tested and the
//! content stays visible.
//!
//! # Rule 3 compliance
//!
//! `MudHotkey` is a visibility gate plus a callback; it paints no text and no
//! glyphs of its own. The closure passed to [`show`](MudHotkey::show) supplies
//! the child content, which should route its strings through
//! [`MudText`](crate::MudText) and its glyphs through
//! [`MudIcon`](crate::MudIcon). `MudHotkey` never calls `painter.text` or
//! `painter.galley`.
//!
//! # Persisted state
//!
//! One `bool` per instance lives in `egui::Memory`, keyed by
//! `<ui.id().with(id_salt)>.hotkey_visible`. It exists only for the
//! `HideChildContentOnRepress` toggle; a hotkey used purely as a callback
//! writes it but never reads it.

use std::any::Any;

use egui::{Id, Key, Modifiers, Response, Sense, Ui, Vec2};

use mud_egui_core::base::component_base::ComponentBase;

// ---------------------------------------------------------------------------
// HotkeyEventArgs
// ---------------------------------------------------------------------------

/// The payload passed to [`MudHotkey::on_hotkey_pressed`].
///
/// Mirrors `MudBlazor.HotkeyEventArgs` (MudBlazor 9.0.0): the pressed key and
/// whether the event is a repeat from a held key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotkeyEventArgs {
    /// The key that completed the hotkey combination. Mirrors `Key`.
    pub key: Key,
    /// `true` when the key is being held down and this is a repeat event.
    /// Mirrors `OnRepeat`.
    pub repeat: bool,
}

// ---------------------------------------------------------------------------
// MudHotkey
// ---------------------------------------------------------------------------

/// A hotkey listener that reveals or hides its child content.
///
/// Port of **MudBlazor 9.0.0's** `MudHotkey` (see the module docs for the
/// version note). Build it with [`new`](Self::new) and the builder methods,
/// then call [`show`](Self::show) with the content closure.
#[must_use = "MudHotkey does nothing until you call `.show(ui, |ui| ..)`"]
pub struct MudHotkey {
    /// Shared component state (utility classes). Stored for parity; not
    /// resolved, because `MudHotkey` adds no wrapper frame. Mirrors MudBlazor
    /// `Class`.
    pub base: ComponentBase,
    /// The key that completes the hotkey. Mirrors MudBlazor `Key`; `egui::Key`
    /// stands in for `JsKey`.
    pub key: Key,
    /// The modifier set that must be held with `key`. Mirrors MudBlazor
    /// `KeyModifiers`; `egui::Modifiers` stands in for `JsKeyModifier` and
    /// does not distinguish left from right.
    pub key_modifiers: Modifiers,
    /// When set, fires with a [`HotkeyEventArgs`] every time the hotkey is
    /// pressed. Mirrors MudBlazor `OnHotkeyPressed`.
    pub on_hotkey_pressed: Option<Box<dyn FnMut(&HotkeyEventArgs) + 'static>>,
    /// When set, the hotkey is not tested and the content stays visible.
    /// Mirrors MudBlazor `Disabled`.
    pub disabled: bool,
    /// When set, the key is consumed via `InputState::consume_key` so other
    /// widgets do not see it. Mirrors MudBlazor `PreventEventPropagation`.
    pub prevent_event_propagation: bool,
    /// When set (the default), the child content's visibility toggles on each
    /// hotkey press. Mirrors MudBlazor `HideChildContentOnRepress`.
    pub hide_child_content_on_repress: bool,
    /// Distinguishes two `MudHotkey`s in one parent. **Extension**, same shape
    /// as `MudCollapse::id_salt`.
    pub id_salt: Option<Id>,
    /// Arbitrary user data. Mirrors MudBlazor `Tag`.
    pub tag: Option<Box<dyn Any + Send + Sync>>,
}

impl Default for MudHotkey {
    fn default() -> Self {
        Self {
            base: ComponentBase::new(),
            key: Key::Space,
            key_modifiers: Modifiers::NONE,
            on_hotkey_pressed: None,
            disabled: false,
            prevent_event_propagation: false,
            hide_child_content_on_repress: true,
            id_salt: None,
            tag: None,
        }
    }
}

impl MudHotkey {
    /// Creates a hotkey listener for `key` with no modifiers and MudBlazor's
    /// other defaults (`HideChildContentOnRepress = true`).
    pub fn new(key: Key) -> Self {
        Self {
            key,
            ..Self::default()
        }
    }

    /// Creates a hotkey listener that requires the given modifiers. Convenience
    /// over `MudHotkey::new(key).key_modifiers(m)`.
    pub fn with_modifiers(key: Key, modifiers: Modifiers) -> Self {
        Self::new(key).key_modifiers(modifiers)
    }

    /// Sets the required modifier set. Mirrors MudBlazor `KeyModifiers`.
    #[inline]
    pub fn key_modifiers(mut self, modifiers: Modifiers) -> Self {
        self.key_modifiers = modifiers;
        self
    }

    /// Sets the callback fired on every hotkey press. Mirrors MudBlazor
    /// `OnHotkeyPressed`.
    #[inline]
    pub fn on_hotkey_pressed(
        mut self,
        f: impl FnMut(&HotkeyEventArgs) + 'static,
    ) -> Self {
        self.on_hotkey_pressed = Some(Box::new(f));
        self
    }

    /// Disables the hotkey. Mirrors MudBlazor `Disabled`.
    #[inline]
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }

    /// Consumes the key so other widgets do not see it. Mirrors MudBlazor
    /// `PreventEventPropagation`.
    #[inline]
    pub fn prevent_event_propagation(mut self, b: bool) -> Self {
        self.prevent_event_propagation = b;
        self
    }

    /// Toggles child-content visibility on each press. Mirrors MudBlazor
    /// `HideChildContentOnRepress` (default `true`).
    #[inline]
    pub fn hide_child_content_on_repress(mut self, b: bool) -> Self {
        self.hide_child_content_on_repress = b;
        self
    }

    /// Distinguishes two `MudHotkey`s in one parent. **Extension**, same shape
    /// as `MudCollapse::id_salt`.
    #[inline]
    pub fn id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_salt = Some(Id::new(salt));
        self
    }

    /// Adds utility classes to the component's [`ComponentBase`]. Stored for
    /// parity; `MudHotkey` renders no wrapper. Mirrors MudBlazor `Class`.
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    /// Attaches arbitrary user data. Mirrors MudBlazor `Tag`.
    #[inline]
    pub fn tag<T: Any + Send + Sync>(mut self, tag: T) -> Self {
        self.tag = Some(Box::new(tag));
        self
    }

    /// Reads user data attached with [`tag`](Self::tag).
    #[inline]
    pub fn tag_ref<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.tag.as_ref().and_then(|b| b.downcast_ref::<T>())
    }

    /// Tests the hotkey, fires the callback, and reveals or hides
    /// `add_content`.
    ///
    /// The content closure runs **only when the content is visible**; when
    /// hidden, the component allocates nothing and returns `inner = None`.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        add_content: impl FnOnce(&mut Ui) -> R,
    ) -> HotkeyResponse<R> {
        let (pressed, repeat) = if self.disabled {
            (false, false)
        } else {
            detect_press(ui, self.key, self.key_modifiers)
        };

        // `consume_key` lives on `InputState`, not on `Context`; reach it
        // through `Ui::input_mut`, which is the documented convenience for
        // "modify how subsequent widgets see the input state".
        if pressed && self.prevent_event_propagation {
            ui.input_mut(|i| i.consume_key(self.key_modifiers, self.key));
        }

        if pressed {
            if let Some(cb) = self.on_hotkey_pressed.as_mut() {
                cb(&HotkeyEventArgs {
                    key: self.key,
                    repeat,
                });
            }
        }

        let visible = if self.hide_child_content_on_repress {
            let id = ui
                .id()
                .with(self.id_salt)
                .with("hotkey_visible");
            let current = ui
                .ctx()
                .memory(|m| m.data.get_temp::<bool>(id))
                .unwrap_or(true);
            let next = if pressed { !current } else { current };
            if next != current {
                ui.ctx()
                    .memory_mut(|m| m.data.insert_temp(id, next));
            }
            next
        } else {
            true
        };

        if !visible {
            return HotkeyResponse {
                response: ui.allocate_response(Vec2::ZERO, Sense::hover()),
                pressed,
                repeat,
                visible: false,
                inner: None,
            };
        }

        let out = ui.scope(add_content);
        HotkeyResponse {
            response: out.response,
            pressed,
            repeat,
            visible: true,
            inner: Some(out.inner),
        }
    }
}

impl egui::Widget for MudHotkey {
    /// Tests the hotkey but draws no content. Prefer [`MudHotkey::show`] so
    /// the content closure runs when visible.
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui, |_| ()).response
    }
}

// ---------------------------------------------------------------------------
// HotkeyResponse
// ---------------------------------------------------------------------------

/// What [`MudHotkey::show`] did this frame.
pub struct HotkeyResponse<R> {
    /// The content's `Response` when visible; a zero-sized hover response when
    /// hidden. `HotkeyResponse` dereferences to it, so `out.hovered()` works.
    pub response: Response,
    /// `true` on the frame the hotkey was pressed. Mirrors the trigger for
    /// MudBlazor `OnHotkeyPressed`.
    pub pressed: bool,
    /// `true` when `pressed` is a repeat from a held key.
    pub repeat: bool,
    /// The current content visibility. `false` only when
    /// `hide_child_content_on_repress` is set and the toggle is in its hidden
    /// state.
    pub visible: bool,
    /// The content closure's value; `None` when hidden.
    pub inner: Option<R>,
}

impl<R> std::ops::Deref for HotkeyResponse<R> {
    type Target = Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (no `egui::Context` required)
// ---------------------------------------------------------------------------

/// `(pressed, repeat)` for the current frame.
///
/// `pressed` is `true` on the initial press *and* on every repeat while the
/// key is held; `repeat` distinguishes the two. Modifiers must match exactly
/// (`Modifiers::matches_logically`), so `Ctrl+K` does not fire on a bare `K`.
fn detect_press(ui: &Ui, key: Key, modifiers: Modifiers) -> (bool, bool) {
    ui.input(|i| {
        let mut pressed = false;
        let mut repeat = false;
        for event in &i.events {
            if let egui::Event::Key {
                key: k,
                pressed: true,
                repeat: r,
                modifiers: m,
                ..
            } = event
            {
                if *k == key && m.matches_logically(modifiers) {
                    pressed = true;
                    repeat |= *r;
                }
            }
        }
        (pressed, repeat)
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn run_ui<R>(events: Vec<egui::Event>, f: impl FnOnce(&mut Ui) -> R) -> R {
        let ctx = egui::Context::default();
        let mut f = Some(f);
        let mut out: Option<R> = None;
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            events,
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

    fn key_event(key: Key, modifiers: Modifiers, repeat: bool) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat,
            modifiers,
        }
    }

    // --- detect_press ----------------------------------------------------

    #[test]
    fn detect_press_matches_key_and_modifiers() {
        let (pressed, repeat) = run_ui(
            vec![key_event(Key::K, Modifiers::CTRL, false)],
            |ui| detect_press(ui, Key::K, Modifiers::CTRL),
        );
        assert!(pressed);
        assert!(!repeat);
    }

    #[test]
    fn detect_press_rejects_wrong_key() {
        let (pressed, _) = run_ui(
            vec![key_event(Key::J, Modifiers::CTRL, false)],
            |ui| detect_press(ui, Key::K, Modifiers::CTRL),
        );
        assert!(!pressed);
    }

    #[test]
    fn detect_press_rejects_wrong_modifiers() {
        let (pressed, _) = run_ui(
            vec![key_event(Key::K, Modifiers::NONE, false)],
            |ui| detect_press(ui, Key::K, Modifiers::CTRL),
        );
        assert!(!pressed);
    }

    #[test]
    fn detect_press_reports_repeat() {
        let (pressed, repeat) = run_ui(
            vec![key_event(Key::K, Modifiers::CTRL, true)],
            |ui| detect_press(ui, Key::K, Modifiers::CTRL),
        );
        assert!(pressed);
        assert!(repeat);
    }

    #[test]
    fn detect_press_ignores_releases() {
        let release = egui::Event::Key {
            key: Key::K,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: Modifiers::CTRL,
        };
        let (pressed, _) = run_ui(
            vec![release],
            |ui| detect_press(ui, Key::K, Modifiers::CTRL),
        );
        assert!(!pressed);
    }

    // --- consume_key -----------------------------------------------------

    #[test]
    fn consume_key_removes_the_event_for_later_widgets() {
        // Enter a K event, consume it, then check that the same `Ui` no longer
        // reports it. This is the guarantee `PreventEventPropagation` relies on.
        let consumed_before = run_ui(
            vec![key_event(Key::K, Modifiers::CTRL, false)],
            |ui| {
                ui.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::K));
                ui.input(|i| i.key_pressed(Key::K))
            },
        );
        assert!(!consumed_before, "consume_key should hide the event from later widgets");
    }

    // --- builders ---------------------------------------------------------

    #[test]
    fn defaults_match_mudblazor_9() {
        let h = MudHotkey::new(Key::Space);
        assert_eq!(h.key, Key::Space);
        assert_eq!(h.key_modifiers, Modifiers::NONE);
        assert!(h.on_hotkey_pressed.is_none());
        assert!(!h.disabled);
        assert!(!h.prevent_event_propagation);
        assert!(h.hide_child_content_on_repress);
        assert!(h.id_salt.is_none());
        assert!(h.tag.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let h = MudHotkey::new(Key::K)
            .key_modifiers(Modifiers::CTRL)
            .on_hotkey_pressed(|_| {})
            .disabled(true)
            .prevent_event_propagation(true)
            .hide_child_content_on_repress(false)
            .id_salt("search")
            .classes(["mt-2"])
            .tag("shortcut");

        assert_eq!(h.key, Key::K);
        assert_eq!(h.key_modifiers, Modifiers::CTRL);
        assert!(h.on_hotkey_pressed.is_some());
        assert!(h.disabled);
        assert!(h.prevent_event_propagation);
        assert!(!h.hide_child_content_on_repress);
        assert!(h.id_salt.is_some());
        assert!(!h.base.class_names().is_empty());
        assert_eq!(h.tag_ref::<&str>(), Some(&"shortcut"));
        assert!(h.tag_ref::<u32>().is_none());
    }

    #[test]
    fn with_modifiers_convenience() {
        let h = MudHotkey::with_modifiers(Key::S, Modifiers::CTRL);
        assert_eq!(h.key, Key::S);
        assert_eq!(h.key_modifiers, Modifiers::CTRL);
    }

    #[test]
    fn id_salt_is_stable_and_distinguishing() {
        let a = MudHotkey::new(Key::A).id_salt("a");
        let a2 = MudHotkey::new(Key::A).id_salt("a");
        let b = MudHotkey::new(Key::A).id_salt("b");
        assert_eq!(a.id_salt, a2.id_salt);
        assert_ne!(a.id_salt, b.id_salt);
    }
}