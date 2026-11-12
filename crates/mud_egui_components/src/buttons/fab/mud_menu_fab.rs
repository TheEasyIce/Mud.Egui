//! `MudMenuFab` — a floating action button that expands into a vertical menu
//! of actions.
//!
//! Port of `MudMenuFab.razor.cs` (MudBlazor 8.5+). Pass 1.

use egui::{vec2, Response, Sense, Ui, Vec2};
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::{AnchorOrigin, Color, Size, TransformOrigin, Variant};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

use crate::data_display::icons::mud_icon::MudIcon;
use crate::feedback::popover::MudPopover;

use super::fab_menu_action::FabMenuAction;
use super::menu_fab_state::MenuFabState;
use super::mud_fab::{resolve_colors, MudFab};

/// Vertical gap between action buttons.
pub const ACTION_GAP: f32 = 8.0;

/// Distance between the trigger and the first action.
pub const TRIGGER_OFFSET: f32 = 12.0;

/// Action buttons are this many points smaller than the trigger.
pub const ACTION_INSET: f32 = 8.0;

/// The result of one [`MudMenuFab::show`] frame.
#[derive(Debug)]
pub struct MenuFabResponse {
    /// The trigger FAB's `Response`.
    pub response: Response,
    /// Whether the action menu is open after this frame.
    pub open: bool,
    /// `true` on the frame the action menu opened.
    pub opened: bool,
    /// `true` on the frame the action menu closed.
    pub closed: bool,
    /// The index of the action that was fired this frame, if any.
    pub triggered: Option<usize>,
}

impl MenuFabResponse {
    /// `true` when the action menu is open after this frame.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.open
    }
}

/// A floating action button with a vertical action menu.
#[must_use = "MudMenuFab does nothing until you call `.show(ui, state)`"]
pub struct MudMenuFab<'a> {
    pub icon: String,
    pub actions: Vec<FabMenuAction<'a>>,
    pub size: Size,
    pub color: Color,
    pub variant: Variant,
    pub disabled: bool,
    pub drop_shadow: bool,
    pub fixed: bool,
    pub bottom: Option<f32>,
    pub right: Option<f32>,
    pub on_click: Option<Box<dyn FnMut() + 'a>>,
    pub child: Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    pub aria_label: Option<String>,
    pub base: ComponentBase,
}

impl<'a> Default for MudMenuFab<'a> {
    fn default() -> Self {
        Self {
            icon: String::new(),
            actions: Vec::new(),
            size: Size::Large,
            color: Color::Default,
            variant: Variant::Filled,
            disabled: false,
            drop_shadow: true,
            fixed: false,
            bottom: None,
            right: None,
            on_click: None,
            child: None,
            aria_label: None,
            base: ComponentBase::new(),
        }
    }
}

impl<'a> MudMenuFab<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline] pub fn icon(mut self, s: impl Into<String>) -> Self { self.icon = s.into(); self }
    #[inline] pub fn actions(mut self, actions: impl IntoIterator<Item = FabMenuAction<'a>>) -> Self { self.actions = actions.into_iter().collect(); self }
    #[inline] pub fn size(mut self, s: Size) -> Self { self.size = s; self }
    #[inline] pub fn color(mut self, c: Color) -> Self { self.color = c; self }
    #[inline] pub fn variant(mut self, v: Variant) -> Self { self.variant = v; self }
    #[inline] pub fn disabled(mut self, b: bool) -> Self { self.disabled = b; self }
    #[inline] pub fn drop_shadow(mut self, b: bool) -> Self { self.drop_shadow = b; self }
    #[inline] pub fn fixed(mut self, b: bool) -> Self { self.fixed = b; self }
    #[inline] pub fn bottom(mut self, px: f32) -> Self { self.bottom = Some(px); self }
    #[inline] pub fn right(mut self, px: f32) -> Self { self.right = Some(px); self }
    #[inline] pub fn on_click(mut self, f: impl FnMut() + 'a) -> Self { self.on_click = Some(Box::new(f)); self }
    #[inline] pub fn child(mut self, f: impl FnMut(&mut Ui) + 'a) -> Self { self.child = Some(Box::new(f)); self }
    #[inline] pub fn aria_label(mut self, s: impl Into<String>) -> Self { self.aria_label = Some(s.into()); self }
    #[inline] pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self { self.base = self.base.classes(classes); self }

    /// Draws the trigger and, when open, the action menu.
    pub fn show(mut self, ui: &mut Ui, state: &mut MenuFabState) -> MenuFabResponse {
        let was_open = state.popover.open;

        // 1. Render the trigger.
        let mut trigger = MudFab::new()
            .size(self.size)
            .color(self.color)
            .variant(self.variant)
            .disabled(self.disabled)
            .drop_shadow(self.drop_shadow)
            .fixed(self.fixed);
        if let Some(b) = self.bottom { trigger = trigger.bottom(b); }
        if let Some(r) = self.right { trigger = trigger.right(r); }
        if let Some(aria) = self.aria_label.as_deref() { trigger = trigger.title(aria); }
        trigger = if let Some(child) = self.child.take() {
            trigger.child(child)
        } else {
            trigger.icon(self.icon.clone())
        };
        let trigger_resp = trigger.show(ui);

        // 2. Trigger interaction.
        if !self.disabled && trigger_resp.clicked() {
            state.popover.toggle();
            if let Some(cb) = self.on_click.as_mut() { cb(); }
        }

        // 3. Action menu.
        //
        // The render data is materialised into an owned `Vec` *before* the
        // popover, so the popover's closure borrows nothing from `self`. The
        // callbacks fire afterwards via `self.actions.get_mut`.
        let mut clicked_index: Option<usize> = None;
        let mut popover_rect: Option<egui::Rect> = None;

        if state.popover.open && !self.actions.is_empty() {
            let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
            let palette = theme.current_palette();
            let action_size = action_side(self.size);
            let size_hint = vec2(action_size, total_height(self.actions.len(), action_size));

            let popover = MudPopover::new()
                .anchor_origin(AnchorOrigin::Top)
                .transform_origin(TransformOrigin::BottomCenter)
                .offset(TRIGGER_OFFSET)
                .paper(false)
                .elevation(0)
                .dismiss_on_outside_click(false)
                .size_hint(size_hint);

            let action_data: Vec<(String, Option<String>)> = self
                .actions
                .iter()
                .map(|a| (a.icon.clone(), a.tooltip.clone()))
                .collect();

            let color = self.color;
            let variant = self.variant;
            let disabled = self.disabled;
            let drop_shadow = self.drop_shadow;
            let triggered_cell = &mut clicked_index;

            let pop = popover.show(ui, &mut state.popover, trigger_resp.rect, |pop_ui| {
                pop_ui.spacing_mut().item_spacing.y = ACTION_GAP;
                for (i, (icon, tooltip)) in action_data.iter().enumerate() {
                    let resp = render_action_button(
                        pop_ui,
                        icon,
                        action_size,
                        color,
                        variant,
                        disabled,
                        drop_shadow,
                        &palette,
                    );
                    if !disabled && resp.clicked() {
                        *triggered_cell = Some(i);
                    }
                    if let Some(tip) = tooltip.as_deref() {
                        resp.on_hover_text(tip);
                    }
                }
            });
            popover_rect = Some(pop.0.response.rect);
        }

        // 4. Fire the callback (outside the popover closure).
        let triggered = clicked_index;
        if let Some(i) = triggered {
            if let Some(cb) = self
                .actions
                .get_mut(i)
                .and_then(|a| a.on_click.as_mut())
            {
                cb();
            }
            state.popover.open = false;
        }

        // 5. Close on outside click (excluding the trigger).
        if state.popover.open {
            if let Some(prect) = popover_rect {
                let clicked_outside = ui.ctx().input(|i| {
                    i.pointer.any_click()
                        && i.pointer
                        .interact_pos()
                        .map(|p| !prect.contains(p) && !trigger_resp.rect.contains(p))
                        .unwrap_or(false)
                });
                if clicked_outside {
                    state.popover.open = false;
                }
            }
        }

        let now_open = state.popover.open;
        MenuFabResponse {
            response: trigger_resp,
            open: now_open,
            opened: !was_open && now_open,
            closed: was_open && !now_open,
            triggered,
        }
    }
}

/// The action button side, in points. Eight points smaller than the trigger.
#[inline]
pub fn action_side(size: Size) -> f32 {
    match size {
        Size::ExtraSmall | Size::Small => 32.0,
        Size::Medium => 40.0,
        Size::Large | Size::ExtraLarge => 48.0,
    }
}

/// Total height of the action column (N buttons + (N − 1) gaps).
#[inline]
pub fn total_height(count: usize, action_size: f32) -> f32 {
    if count == 0 {
        return 0.0;
    }
    let n = count as f32;
    n * action_size + (n - 1.0) * ACTION_GAP
}

fn render_action_button(
    ui: &mut Ui,
    icon: &str,
    size: f32,
    color: Color,
    variant: Variant,
    disabled: bool,
    drop_shadow: bool,
    palette: &Palette,
) -> Response {
    let fallback = ui.visuals().text_color();
    let colors = resolve_colors(variant, color, disabled, fallback, palette);
    let sense = if disabled { Sense::hover() } else { Sense::click() };
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), sense);

    if ui.is_rect_visible(rect) {
        super::mud_fab::paint_fab_bg(
            ui,
            rect,
            &colors,
            size * 0.5,
            drop_shadow && matches!(variant, Variant::Filled),
        );
        if !disabled && response.hovered() {
            ui.painter().rect_filled(
                rect,
                egui::CornerRadius::same((size * 0.5) as u8),
                egui::Color32::from_rgba_unmultiplied(
                    colors.ink.r(),
                    colors.ink.g(),
                    colors.ink.b(),
                    18,
                ),
            );
        }
        let icon_px = size * 0.5;
        ui.put(
            egui::Rect::from_center_size(rect.center(), Vec2::splat(icon_px)),
            MudIcon::new(icon.to_owned())
                .size_px(icon_px)
                .custom_color(colors.ink),
        );
    }

    if disabled {
        response.on_hover_cursor(egui::CursorIcon::NotAllowed)
    } else {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mud_egui_icons_material::filled;

    #[test]
    fn action_side_is_eight_points_smaller_than_the_trigger() {
        assert_eq!(action_side(Size::Small), 32.0);
        assert_eq!(action_side(Size::Medium), 40.0);
        assert_eq!(action_side(Size::Large), 48.0);
        assert_eq!(action_side(Size::ExtraSmall), 32.0);
        assert_eq!(action_side(Size::ExtraLarge), 48.0);
    }

    #[test]
    fn total_height_accounts_for_gaps() {
        assert_eq!(total_height(0, 40.0), 0.0);
        assert_eq!(total_height(1, 40.0), 40.0);
        assert_eq!(total_height(2, 40.0), 40.0 + ACTION_GAP + 40.0);
        assert_eq!(total_height(3, 40.0), 3.0 * 40.0 + 2.0 * ACTION_GAP);
    }

    #[test]
    fn defaults_match_mudblazor() {
        let f: MudMenuFab = MudMenuFab::new();
        assert!(f.icon.is_empty());
        assert!(f.actions.is_empty());
        assert!(matches!(f.size, Size::Large));
        assert!(matches!(f.color, Color::Default));
        assert!(matches!(f.variant, Variant::Filled));
        assert!(!f.disabled);
        assert!(f.drop_shadow);
        assert!(!f.fixed);
        assert!(f.bottom.is_none() && f.right.is_none());
        assert!(f.on_click.is_none());
        assert!(f.child.is_none());
        assert!(f.aria_label.is_none());
    }

    #[test]
    fn builders_set_every_field() {
        let f: MudMenuFab = MudMenuFab::new()
            .icon("I")
            .actions([
                FabMenuAction::new(filled::CHECK).tooltip("ok"),
                FabMenuAction::new(filled::CANCEL).tooltip("no"),
            ])
            .size(Size::Medium)
            .color(Color::Primary)
            .variant(Variant::Outlined)
            .disabled(true)
            .drop_shadow(false)
            .fixed(true)
            .bottom(24.0)
            .right(12.0)
            .on_click(|| {})
            .child(|_| {})
            .aria_label("Actions")
            .classes(["cls"]);

        assert_eq!(f.icon, "I");
        assert_eq!(f.actions.len(), 2);
        assert!(matches!(f.size, Size::Medium));
        assert!(matches!(f.color, Color::Primary));
        assert!(matches!(f.variant, Variant::Outlined));
        assert!(f.disabled && !f.drop_shadow);
        assert!(f.fixed);
        assert_eq!(f.bottom, Some(24.0));
        assert_eq!(f.right, Some(12.0));
        assert!(f.on_click.is_some());
        assert!(f.child.is_some());
        assert_eq!(f.aria_label.as_deref(), Some("Actions"));
        assert!(f.base.has_class("cls"));
    }
}