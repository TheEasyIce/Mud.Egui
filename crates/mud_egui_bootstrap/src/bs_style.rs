//! `BsStyle`: combines the five Bootstrap caches into one `egui::Frame`.
//!
//! Add to your crate's `lib.rs`:
//! ```ignore
//! pub mod bs_background;
//! pub mod bs_border;
//! pub mod bs_margin;
//! pub mod bs_padding;
//! pub mod bs_rounded;
//! pub mod bs_style;
//! ```

use egui::{Align, Color32, Frame, Stroke, Ui, Margin, CornerRadius, Rect, Pos2, vec2};
use mud_egui_core::base::component_base::ComponentBase;

use crate::bs_background::{BgCache, BgClass};
use crate::bs_border::{BorderStyle, BsBorderCache};
use crate::bs_margin::BsMarginCache;
use crate::bs_padding::BsPaddingCache;
use crate::bs_rounded::BsRoundedCache;

// --- Geometry Helpers ---

/// Builds a closed, clockwise rounded-rect outline.
fn rounded_rect_path(rect: Rect, cr: CornerRadius, inset: f32) -> Vec<Pos2> {
    const STEPS: usize = 12;
    let max_r = rect.width().min(rect.height()) / 2.0;
    let r = |v: u8| ((v as f32) - inset).clamp(0.0, max_r);
    let (nw, ne, se, sw) = (r(cr.nw), r(cr.ne), r(cr.se), r(cr.sw));

    let mut pts = Vec::with_capacity(STEPS * 4 + 5);
    let mut arc = |center: Pos2, radius: f32, start_deg: f32| {
        if radius <= 0.0 {
            pts.push(center);
            return;
        }
        for i in 0..=STEPS {
            let a = (start_deg + 90.0 * i as f32 / STEPS as f32).to_radians();
            pts.push(center + radius * vec2(a.cos(), a.sin()));
        }
    };

    arc(Pos2::new(rect.left() + nw,  rect.top() + nw),     nw, 180.0); // top-left
    arc(Pos2::new(rect.right() - ne, rect.top() + ne),     ne, 270.0); // top-right
    arc(Pos2::new(rect.right() - se, rect.bottom() - se),  se,   0.0); // bottom-right
    arc(Pos2::new(rect.left() + sw,  rect.bottom() - sw),  sw,  90.0); // bottom-left

    pts.push(pts[0]); // close the loop
    pts
}

/// Removes the outer margin from a frame's response rect.
fn strip_margin(rect: Rect, m: Margin) -> Rect {
    Rect::from_min_max(
        rect.min + vec2(m.left as f32, m.top as f32),
        rect.max - vec2(m.right as f32, m.bottom as f32),
    )
}

// --- BsStyle ---

/// Holds one cache per utility. Keep this as a field on your component.
#[derive(Default)]
pub struct BsStyle {
    bg: BgCache,
    border: BsBorderCache,
    margin: BsMarginCache,
    padding: BsPaddingCache,
    rounded: BsRoundedCache,

    // Results that `egui::Frame` cannot express by itself.
    h_align: Option<Align>,
    v_align: Option<Align>,
    border_style: Option<BorderStyle>,
    border_stroke: Option<Stroke>,
    outer_margin: Margin,
    corner_radius: CornerRadius,
}

impl BsStyle {
    pub fn build_frame(
        &mut self,
        base: &ComponentBase,
        theme: &dyn Fn(&str) -> Option<Color32>,
    ) -> Frame {
        let mut frame = Frame::new();
        self.outer_margin = Margin::ZERO;
        self.corner_radius = CornerRadius::ZERO;

        if let Some(bg) = self.bg.resolve(base) {
            frame = frame.fill(match bg {
                BgClass::Literal(c) => c,
                BgClass::Resource { key, .. } => theme(&key).unwrap_or(Color32::TRANSPARENT),
            });
        }

        self.border_style = None;
        self.border_stroke = None;
        let mut dashed_border_width = 0.0;

        if let Some(b) = self.border.resolve(base) {
            self.border_style = Some(b.style);
            if b.width > 0.0 {
                let color = theme("MudPaletteLinesDefault").unwrap_or(Color32::GRAY);
                let stroke = Stroke::new(b.width, color);
                self.border_stroke = Some(stroke);

                if b.style == BorderStyle::Solid {
                    frame = frame.stroke(stroke);
                } else {
                    // Remember width to push content inward later
                    dashed_border_width = b.width;
                }
            }
        }

        self.h_align = None;
        self.v_align = None;
        if let Some(m) = self.margin.resolve(base) {
            frame = frame.outer_margin(m.margin);
            self.outer_margin = m.margin;
            self.h_align = m.h_align;
            self.v_align = m.v_align;
        }

        // Apply padding + dashed border width to simulate CSS layout
        let mut final_margin = self.padding.resolve(base).unwrap_or(Margin::ZERO);
        if dashed_border_width > 0.0 {
            let w = dashed_border_width as i8;
            final_margin.left = final_margin.left.saturating_add(w);
            final_margin.right = final_margin.right.saturating_add(w);
            final_margin.top = final_margin.top.saturating_add(w);
            final_margin.bottom = final_margin.bottom.saturating_add(w);
        }
        if final_margin != Margin::ZERO {
            frame = frame.inner_margin(final_margin);
        }

        if let Some(r) = self.rounded.resolve(base) {
            frame = frame.corner_radius(r);
            self.corner_radius = r;
        }

        frame
    }

    /// Draws dashed or dotted borders manually, tracing the rounded corners.
    pub fn paint_border(&self, ui: &Ui, rect: egui::Rect) {
        if let (Some(style), Some(stroke)) = (self.border_style, self.border_stroke) {
            let (dash_len, gap_len) = match style {
                BorderStyle::Dashed => (stroke.width * 3.0, stroke.width * 2.0),
                BorderStyle::Dotted => (stroke.width * 0.1, stroke.width * 2.0),
                _ => return,
            };

            let half = stroke.width * 0.5;
            let box_rect = strip_margin(rect, self.outer_margin).shrink(half);

            let path = rounded_rect_path(box_rect, self.corner_radius, half);
            ui.painter()
                .extend(egui::Shape::dashed_line(&path, stroke, dash_len, gap_len));
        }
    }

    pub fn h_align(&self) -> Option<Align> {
        self.h_align
    }

    pub fn v_align(&self) -> Option<Align> {
        self.v_align
    }

    pub fn border_style(&self) -> Option<BorderStyle> {
        self.border_style
    }
}