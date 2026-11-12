use egui::Ui;
use mud_egui_components::buttons::button::mud_button::MudButton;
use mud_egui_components::data_display::simple_table::MudSimpleTable;
use mud_egui_components::surfaces::card::MudCard;
use mud_egui_components::{MudContainer, MudCodeBlock, MudChip, MudDivider, MudGrid, MudItem, MudStack, MudText};
use mud_egui_core::enums::max_width::MaxWidth;
use mud_egui_core::enums::{AlignItems, Color, Justify, Size, Typo, Variant, Wrap};
use mud_egui_navigation::navigation::NavigationManager;
use mud_egui_navigation::page::{NavigationContext, Page};

pub struct ContainerPage {
    max_width: MaxWidth,
    fixed: bool,
    gutters: bool,
    fluid: bool,
    elevation: u8,
    copy_flash_until: f64,
}

impl Default for ContainerPage {
    fn default() -> Self {
        Self { max_width: MaxWidth::Md, fixed: false, gutters: true, fluid: false, elevation: 0, copy_flash_until: 0.0 }
    }
}

const ALL_WIDTHS: &[MaxWidth] = &[MaxWidth::Xs, MaxWidth::Sm, MaxWidth::Md, MaxWidth::Lg, MaxWidth::Xl, MaxWidth::Xxl, MaxWidth::False];

fn max_width_label(w: MaxWidth) -> &'static str {
    match w { MaxWidth::Xs => "Xs", MaxWidth::Sm => "Sm", MaxWidth::Md => "Md", MaxWidth::Lg => "Lg", MaxWidth::Xl => "Xl", MaxWidth::Xxl => "Xxl", MaxWidth::False => "∞" }
}
fn max_width_pt(w: MaxWidth) -> &'static str {
    match w { MaxWidth::Xs => "444 pt", MaxWidth::Sm => "600 pt", MaxWidth::Md => "960 pt", MaxWidth::Lg => "1280 pt", MaxWidth::Xl => "1920 pt", MaxWidth::Xxl => "2560 pt", MaxWidth::False => "unbounded" }
}
fn max_width_variant(w: MaxWidth) -> &'static str {
    match w { MaxWidth::Xs => "MaxWidth::Xs", MaxWidth::Sm => "MaxWidth::Sm", MaxWidth::Md => "MaxWidth::Md", MaxWidth::Lg => "MaxWidth::Lg", MaxWidth::Xl => "MaxWidth::Xl", MaxWidth::Xxl => "MaxWidth::Xxl", MaxWidth::False => "MaxWidth::False" }
}

fn toggle_row(ui: &mut Ui, id: &str, label: &str, sublabel: &str, value: &mut bool) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.add(MudText::new(label).typo(Typo::Body2));
            ui.add(MudText::new(sublabel).typo(Typo::Body2).color(Color::TextSecondary));
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let chip = MudChip::new(if *value { "ON" } else { "OFF" })
                .id(id)
                .variant(Variant::Filled)
                .size(Size::Small)
                .color(if *value { Color::Primary } else { Color::Default });
            if ui.add(chip).clicked() { *value = !*value; }
        });
    });
}

impl ContainerPage {
    fn render_canvas(&self, ui: &mut Ui) {
        MudCard::new().elevation(1).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let w = ui.available_width();
                    ui.add(MudText::new(format!("Container Preview · {:.0} pt available", w)).monospace(true));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(MudText::new("Points · Anchor: Center").monospace(true).color(Color::TextSecondary));
                    });
                });
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    for pt in ["0", "600", "960", "1280", "1920", "2560"] {
                        ui.add(MudText::new(format!("{pt} pt")).monospace(true).color(Color::TextSecondary));
                        ui.add_space(8.0);
                    }
                });
                ui.add_space(8.0);

                let avail = ui.available_width();
                let mut c = MudContainer::new()
                    .max_width(self.max_width)
                    .fixed(self.fixed)
                    .gutters(self.gutters)
                    .fluid(self.fluid)
                    .elevation(self.elevation)
                    .classes(["bg-surface", "p-4", "rounded-lg"]);

                c.show(ui, |ui| {
                    MudGrid::new().show(ui, |g| {
                        g.item(MudItem::new().xs(12).md(4), |ui| {
                            MudCard::new().elevation(0).outlined(true).show(ui, |ui| {
                                MudStack::new().spacing(1).show(ui, |s| {
                                    s.item(|ui| { ui.add(MudText::new("Data Telemetry").typo(Typo::Subtitle1)); });
                                    s.item(|ui| { ui.add(MudText::new("Real-time throughput metrics stream.").typo(Typo::Body2).color(Color::TextSecondary)); });
                                    s.item(|ui| { ui.add(MudText::new("████████░░").monospace(true).color(Color::Primary)); });
                                });
                            });
                        });
                        g.item(MudItem::new().xs(12).md(4), |ui| {
                            MudCard::new().elevation(0).outlined(true).show(ui, |ui| {
                                MudStack::new().spacing(1).show(ui, |s| {
                                    s.item(|ui| { ui.add(MudText::new("Worker Nodes").typo(Typo::Subtitle1)); });
                                    s.item(|ui| { ui.add(MudText::new("8 cluster engines synchronized.").typo(Typo::Body2).color(Color::TextSecondary)); });
                                    s.item(|ui| { ui.add(MudText::new("● Healthy 99.98%").monospace(true).color(Color::Success)); });
                                });
                            });
                        });
                        g.item(MudItem::new().xs(12).md(4), |ui| {
                            MudCard::new().elevation(0).outlined(true).show(ui, |ui| {
                                MudStack::new().spacing(1).show(ui, |s| {
                                    s.item(|ui| { ui.add(MudText::new("Memory Bounds").typo(Typo::Subtitle1)); });
                                    s.item(|ui| { ui.add(MudText::new("Allocated: 14.2 GB / 32 GB.").typo(Typo::Body2).color(Color::TextSecondary)); });
                                    s.item(|ui| { ui.add(MudText::new("██████░░░░").monospace(true).color(Color::Success)); });
                                });
                            });
                        });
                    });
                });

                if let Some(layout) = c.last_layout() {
                    let margins = ((avail - layout.outer_width) / 2.0).max(0.0);
                    ui.add_space(8.0);
                    MudCard::new().elevation(0).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(MudText::new(format!(
                                "MaxWidth: {} ({}) · Content: {:.0} pt · Gutters: {:.0} pt · Margins: {:.0} pt",
                                max_width_label(layout.effective_max), max_width_pt(layout.effective_max),
                                layout.content_width, layout.gutter, margins,
                            )).monospace(true));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.add(MudText::new(format!("Breakpoint: {} · Auto Gutter Centering: Active", max_width_label(layout.breakpoint))).monospace(true).color(Color::TextSecondary));
                            });
                        });
                    });
                }
            });
        });
    }

    fn render_inspector(&mut self, ui: &mut Ui) {
        MudCard::new().elevation(1).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.add(MudText::new("Inspector Controls").typo(Typo::H6));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(MudButton::new("Reset").size(Size::Small).variant(Variant::Text).color(Color::TextSecondary)).clicked() {
                            self.max_width = MaxWidth::Md;
                            self.fixed = false;
                            self.gutters = true;
                            self.fluid = false;
                            self.elevation = 0;
                        }
                    });
                });
                ui.add_space(12.0);

                ui.add(MudText::new("MaxWidth Constraint").typo(Typo::Body2).color(Color::TextSecondary));
                ui.horizontal_wrapped(|ui| {
                    for &v in ALL_WIDTHS {
                        let sel = self.max_width == v;
                        let chip = MudChip::new(max_width_label(v))
                            .id(format!("mw-{}", max_width_label(v)))
                            .variant(if sel { Variant::Filled } else { Variant::Text })
                            .size(Size::Small)
                            .color(if sel { Color::Primary } else { Color::TextSecondary });
                        if ui.add(chip).clicked() { self.max_width = v; }
                    }
                });
                ui.add_space(12.0);

                toggle_row(ui, "fixed-toggle", "Fixed", "Snap to breakpoint min", &mut self.fixed);
                ui.add_space(8.0);
                toggle_row(ui, "gutters-toggle", "Gutters", "16 / 24 pt automatic", &mut self.gutters);
                ui.add_space(8.0);
                toggle_row(ui, "fluid-toggle", "Fluid", "Fill offered width", &mut self.fluid);
                ui.add_space(12.0);

                ui.add(MudText::new("Elevation").typo(Typo::Body2).color(Color::TextSecondary));
                ui.horizontal_wrapped(|ui| {
                    for &e in &[0u8, 1, 2, 4, 8, 16, 24] {
                        let sel = self.elevation == e;
                        let chip = MudChip::new(format!("{e}"))
                            .id(format!("elev-{}", e))
                            .variant(if sel { Variant::Filled } else { Variant::Text })
                            .size(Size::Small)
                            .color(if sel { Color::Primary } else { Color::TextSecondary });
                        if ui.add(chip).clicked() { self.elevation = e; }
                    }
                });
                ui.add_space(12.0);

                MudCard::new().elevation(0).outlined(true).show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.add(MudText::new("DIAGNOSTIC NOTICE").monospace(true).color(Color::TextSecondary));
                        ui.add(MudText::new("Content outside bounded max-width safely yields viewport real estate to OS margins, keeping text line-lengths under ~90 CPL on ultrawide monitors.").typo(Typo::Body2).color(Color::TextSecondary));
                    });
                });
            });
        });
    }

    fn generate_code(&self) -> String {
        format!(
            r#"use mud_egui_components::layout::container::MudContainer;
use mud_egui_core::enums::max_width::MaxWidth;

let mut container = MudContainer::new()
    .max_width({})
    .fixed({})
    .gutters({})
    .fluid({})
    .elevation({})
    .classes(["bg-surface", "p-4", "rounded-lg"])
    .on_breakpoint_change(|bp| log::info!("breakpoint: {{bp:?}}"));

container.show(ui, |ui| {{
    ui.heading("Centered content");
    ui.label("At most {} wide.");
}});"#,
            max_width_variant(self.max_width), self.fixed, self.gutters,
            self.fluid, self.elevation, max_width_pt(self.max_width),
        )
    }
}

impl Page for ContainerPage {
    fn show(&mut self, ui: &mut Ui, _ctx: &NavigationContext, _nav: &mut NavigationManager) {
        let is_copied = ui.input(|i| i.time) < self.copy_flash_until;

        ui.vertical(|ui| {
            // Breadcrumb
            ui.horizontal(|ui| {
                ui.add(MudText::new("Docs").color(Color::TextSecondary).typo(Typo::Body2));
                ui.add(MudText::new("/").color(Color::TextSecondary).typo(Typo::Body2));
                ui.add(MudText::new("Components").color(Color::TextSecondary).typo(Typo::Body2));
                ui.add(MudText::new("/").color(Color::TextSecondary).typo(Typo::Body2));
                ui.add(MudText::new("Layout & Containers").color(Color::TextSecondary).typo(Typo::Body2));
                ui.add(MudText::new("/").color(Color::TextSecondary).typo(Typo::Body2));
                ui.add(MudText::new("MudContainer").typo(Typo::Body2).bold(true));
            });

            ui.add_space(16.0);

            // Header
            ui.horizontal_top(|ui| {
                let total = ui.available_width();
                let gap = ui.spacing().item_spacing.x;
                let left_w = ((total - gap) * 0.67).max(0.0);
                let right_w = (total - gap - left_w).max(0.0);

                ui.allocate_ui_with_layout(egui::vec2(left_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.set_max_width(left_w);
                    ui.set_min_width(left_w);
                    MudStack::horizontal().spacing(3).align_items(AlignItems::Center).show(ui, |s| {
                        s.item(|ui| { ui.add(MudText::new("MudContainer").typo(Typo::H3)); });
                        s.item(|ui| { ui.add(MudChip::new("SHIPPED v1.0.0").variant(Variant::Filled).size(Size::Small).color(Color::Success)); });
                        s.item(|ui| { ui.add(MudChip::new("egui 0.31 Ready").variant(Variant::Filled).size(Size::Small).color(Color::Default)); });
                    });

                    ui.add(MudText::new("mud_egui_components::layout::container::MudContainer").color(Color::Primary).monospace(true));
                    ui.add(MudText::new("Responsive, centered, width-limiting container for egui. Caps content at a Material Design breakpoint, centers it horizontally, applies responsive side gutters, and resolves Bootstrap/Tailwind utility classes into an egui::Frame.").typo(Typo::Body1).color(Color::TextSecondary));
                });

                ui.allocate_ui_with_layout(egui::vec2(right_w, 0.0), egui::Layout::top_down(egui::Align::Max), |ui| {
                    ui.set_max_width(right_w);
                    ui.set_min_width(right_w);
                    ui.horizontal(|ui| {
                        if ui.push_id("copy-rust-header", |ui| {
                            ui.add(MudButton::new(if is_copied { "Copied!" } else { "Copy Rust" }).size(Size::Small).variant(Variant::Filled).color(Color::TextSecondary))
                        }).inner.clicked() {
                            ui.ctx().copy_text(self.generate_code());
                            self.copy_flash_until = ui.input(|i| i.time) + 1.5;
                        }
                        if ui.push_id("open-preview", |ui| {
                            ui.add(MudButton::new("Open in Studio Preview").size(Size::Small).variant(Variant::Filled).color(Color::Primary))
                        }).inner.clicked() {
                            ui.ctx().copy_text(self.generate_code());
                            self.copy_flash_until = ui.input(|i| i.time) + 1.5;
                        }
                    });
                });
            });

            ui.add_space(24.0);

            // Section 01 header
            ui.horizontal(|ui| {
                ui.add(MudText::new("01").monospace(true).color(Color::Primary));
                ui.add(MudDivider::new().vertical(true));
                ui.add(MudText::new("Interactive Viewport Centering & MaxWidth Inspector").typo(Typo::H5));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(MudText::new("egui Viewport Emulation").monospace(true).color(Color::TextSecondary));
                });
            });

            ui.add_space(12.0);

            // Section 01 body: canvas + inspector
            ui.horizontal_top(|ui| {
                let total = ui.available_width();
                let gap = ui.spacing().item_spacing.x;
                let left_w = ((total - gap) * 0.75).max(0.0);
                let right_w = (total - gap - left_w).max(0.0);

                ui.allocate_ui_with_layout(egui::vec2(left_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.set_max_width(left_w);
                    ui.set_min_width(left_w);
                    self.render_canvas(ui);
                });

                ui.allocate_ui_with_layout(egui::vec2(right_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.set_max_width(right_w);
                    ui.set_min_width(right_w);
                    self.render_inspector(ui);
                });
            });

            ui.add_space(24.0);

            // Section 02 header
            ui.horizontal(|ui| {
                ui.add(MudText::new("02").monospace(true).color(Color::Primary));
                ui.add(MudDivider::new().vertical(true));
                ui.add(MudText::new("Declarative Rust & Builder Implementation").typo(Typo::H5));
            });

            ui.add_space(12.0);

            // Section 02 body: live code (MudCodeBlock has its own header + Copy button)
            MudCodeBlock::new(self.generate_code())
                .class("bg-surface")
                .height(260.0)
                .language("rust")
                .line_numbers(true)
                .id_salt("container-usage")
                .show(ui);

            ui.add_space(24.0);

            // Section 03 header
            ui.horizontal(|ui| {
                ui.add(MudText::new("03").monospace(true).color(Color::Primary));
                ui.add(MudDivider::new().vertical(true));
                ui.add(MudText::new("API & Events Reference").typo(Typo::H5));
            });

            ui.add_space(12.0);

            // Section 03 body: API table
            MudCard::new().elevation(1).show(ui, |ui| {
                MudSimpleTable::new().bordered(true).striped(true).show(ui, |t| {
                    t.header(|h| {
                        h.cell_width("Property", 170.0);
                        h.cell_width("Type", 130.0);
                        h.cell_width("Default", 90.0);
                        h.cell_width("Kind", 100.0);
                        h.cell("Description");          // flexible: gets the rest (~470 px)
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("MaxWidth").color(Color::Primary).bold(true)); });
                        r.cell("MaxWidth");
                        r.cell_ui(|ui| { ui.add(MudText::new("Lg").color(Color::Success)); });
                        r.cell("builder");
                        r.cell("Breakpoint ceiling — Xs 444, Sm 600, Md 960, Lg 1280, Xl 1920, Xxl 2560, False ∞.");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("Fixed").color(Color::Primary).bold(true)); });
                        r.cell("bool");
                        r.cell_ui(|ui| { ui.add(MudText::new("false").color(Color::Success)); });
                        r.cell("builder");
                        r.cell("Snap the ceiling to the current viewport breakpoint's min width.");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("Gutters").color(Color::Primary).bold(true)); });
                        r.cell("bool");
                        r.cell_ui(|ui| { ui.add(MudText::new("true").color(Color::Success)); });
                        r.cell("builder");
                        r.cell("Responsive side padding: 16 pt below 600 pt, 24 pt otherwise.");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("Fluid").color(Color::Primary).bold(true)); });
                        r.cell("bool");
                        r.cell_ui(|ui| { ui.add(MudText::new("false").color(Color::Success)); });
                        r.cell("extension");
                        r.cell("Fills the offered width unconditionally. Wins over fixed.");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("Disabled").color(Color::Primary).bold(true)); });
                        r.cell("bool");
                        r.cell_ui(|ui| { ui.add(MudText::new("false").color(Color::Success)); });
                        r.cell("extension");
                        r.cell("Disables children via Ui::disable, fades frame, silences callbacks.");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("Elevation").color(Color::Primary).bold(true)); });
                        r.cell("u8");
                        r.cell_ui(|ui| { ui.add(MudText::new("0").color(Color::Success)); });
                        r.cell("extension");
                        r.cell("Shadow depth, clamped to 0..=24.");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("FocusRing").color(Color::Primary).bold(true)); });
                        r.cell("bool");
                        r.cell_ui(|ui| { ui.add(MudText::new("true").color(Color::Success)); });
                        r.cell("extension");
                        r.cell("Animated 2 pt focus ring on interactive containers.");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("on_hover").color(Color::Primary).bold(true)); });
                        r.cell("FnMut()");
                        r.cell_ui(|ui| { ui.add(MudText::new("None").color(Color::Success)); });
                        r.cell("event");
                        r.cell("Fires on the rising edge of pointer-hover.");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("on_unhover").color(Color::Primary).bold(true)); });
                        r.cell("FnMut()");
                        r.cell_ui(|ui| { ui.add(MudText::new("None").color(Color::Success)); });
                        r.cell("event");
                        r.cell("Fires on the falling edge of pointer-hover.");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("on_click").color(Color::Primary).bold(true)); });
                        r.cell("FnMut()");
                        r.cell_ui(|ui| { ui.add(MudText::new("None").color(Color::Success)); });
                        r.cell("event");
                        r.cell("Primary click. Upgrades Sense to click().");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("on_focus").color(Color::Primary).bold(true)); });
                        r.cell("FnMut()");
                        r.cell_ui(|ui| { ui.add(MudText::new("None").color(Color::Success)); });
                        r.cell("event");
                        r.cell("Fires on the rising edge of keyboard focus.");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("on_blur").color(Color::Primary).bold(true)); });
                        r.cell("FnMut()");
                        r.cell_ui(|ui| { ui.add(MudText::new("None").color(Color::Success)); });
                        r.cell("event");
                        r.cell("Fires on the falling edge of keyboard focus.");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("on_breakpoint_change").color(Color::Primary).bold(true)); });
                        r.cell("FnMut(MaxWidth)");
                        r.cell_ui(|ui| { ui.add(MudText::new("None").color(Color::Success)); });
                        r.cell("event");
                        r.cell("Fires on the first frame and every viewport bucket crossing.");
                    });
                });
            });

            ui.add_space(24.0);

            // Section 04 header
            ui.horizontal(|ui| {
                ui.add(MudText::new("04").monospace(true).color(Color::Primary));
                ui.add(MudDivider::new().vertical(true));
                ui.add(MudText::new("Desktop Ultra-Wide Ergonomics & Accessibility").typo(Typo::H5));
            });

            ui.add_space(12.0);

            // Section 04 body: ergonomics cards
            MudGrid::new().show(ui, |grid| {
                grid.item(MudItem::new().xs(12).md(4), |ui| {
                    MudCard::new().elevation(1).show(ui, |ui| {
                        MudStack::new().spacing(2).show(ui, |s| {
                            s.item(|ui| { ui.add(MudText::new("👁").typo(Typo::H3)); });
                            s.item(|ui| { ui.add(MudText::new("Ultrawide Eye-Strain Prevention").typo(Typo::H6)); });
                            s.item(|ui| { ui.add(MudText::new("On 34-inch and 49-inch ultrawide monitors, unbounded paragraphs stretch across thousands of points. MudContainer enforces the typographic standard of 65 to 85 characters per line, eliminating extreme neck and saccadic eye strain.").typo(Typo::Body2).color(Color::TextSecondary)); });
                            s.item(|ui| { ui.add(MudText::new("✓ Optimal CPL Enforced").color(Color::Success).typo(Typo::Body2)); });
                        });
                    });
                });
                grid.item(MudItem::new().xs(12).md(4), |ui| {
                    MudCard::new().elevation(1).show(ui, |ui| {
                        MudStack::new().spacing(2).show(ui, |s| {
                            s.item(|ui| { ui.add(MudText::new("🖥").typo(Typo::H3)); });
                            s.item(|ui| { ui.add(MudText::new("Point-Scale Invariance").typo(Typo::H6)); });
                            s.item(|ui| { ui.add(MudText::new("egui lays out in resolution-independent points, not pixels. MudContainer breakpoints stay stable across mixed-DPI setups without visual jitter or layout thrashing.").typo(Typo::Body2).color(Color::TextSecondary)); });
                            s.item(|ui| { ui.add(MudText::new("✓ Mixed-DPI Calibrated").color(Color::Secondary).typo(Typo::Body2)); });
                        });
                    });
                });
                grid.item(MudItem::new().xs(12).md(4), |ui| {
                    MudCard::new().elevation(1).show(ui, |ui| {
                        MudStack::new().spacing(2).show(ui, |s| {
                            s.item(|ui| { ui.add(MudText::new("🔍").typo(Typo::H3)); });
                            s.item(|ui| { ui.add(MudText::new("WCAG 2.2 200% Zoom Reflow").typo(Typo::H6)); });
                            s.item(|ui| { ui.add(MudText::new("Fully compliant with WCAG 2.2 Reflow Criterion 1.4.10. At 200% OS scaling or window shrink, MudContainer fluidly downscales to lower breakpoint bounds, avoiding horizontal scrolling on primary text.").typo(Typo::Body2).color(Color::TextSecondary)); });
                            s.item(|ui| { ui.add(MudText::new("✓ WCAG 2.2 Level AA Verified").color(Color::Success).typo(Typo::Body2)); });
                        });
                    });
                });
            });
        });
    }
}