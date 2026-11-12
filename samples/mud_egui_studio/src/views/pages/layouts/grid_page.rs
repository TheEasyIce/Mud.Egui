use egui::Ui;
use mud_egui_components::buttons::button::mud_button::MudButton;
use mud_egui_components::data_display::simple_table::MudSimpleTable;
use mud_egui_components::surfaces::card::MudCard;
use mud_egui_components::{MudChip, MudDivider, MudGrid, MudItem, MudStack, MudText};
use mud_egui_core::enums::{AlignItems, Color, Justify, Size, Typo, Variant, Wrap};
use mud_egui_navigation::navigation::NavigationManager;
use mud_egui_navigation::page::{NavigationContext, Page};
use mud_egui_theming::theme::MudTheme;

#[derive(Default)]
pub struct GridPage;

impl Page for GridPage {
    fn show(&mut self, ui: &mut Ui, _ctx: &NavigationContext, _nav: &mut NavigationManager) {
        ui.vertical(|ui| {
            // ── Breadcrumb & Meta ────────────────────────────────────────
            ui.horizontal(|ui| {
                MudStack::horizontal().spacing(1).align_items(AlignItems::Center).show(ui, |s| {
                    s.item(|ui| { ui.add(MudText::new("Docs").color(Color::TextSecondary).typo(Typo::Body2)); });
                    s.item(|ui| { ui.add(MudText::new("/").color(Color::TextSecondary).typo(Typo::Body2)); });
                    s.item(|ui| { ui.add(MudText::new("Components").color(Color::TextSecondary).typo(Typo::Body2)); });
                    s.item(|ui| { ui.add(MudText::new("/").color(Color::TextSecondary).typo(Typo::Body2)); });
                    s.item(|ui| { ui.add(MudText::new("Layout & Containers").color(Color::TextSecondary).typo(Typo::Body2)); });
                    s.item(|ui| { ui.add(MudText::new("/").color(Color::TextSecondary).typo(Typo::Body2)); });
                    s.item(|ui| { ui.add(MudText::new("MudGrid").typo(Typo::Body2).bold(true)); });
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    MudStack::horizontal().spacing(1).align_items(AlignItems::Center).show(ui, |s| {
                        s.item(|ui| { ui.add(MudText::new("Architecture: Native AOT").typo(Typo::Body2).color(Color::TextSecondary)); });
                        s.item(|ui| { ui.add(MudText::new("•").typo(Typo::Body2).color(Color::TextSecondary)); });
                        s.item(|ui| { ui.add(MudText::new("Avalonia v11.2 Core").typo(Typo::Body2).color(Color::TextSecondary)); });
                        s.item(|ui| { ui.add(MudText::new("•").typo(Typo::Body2).color(Color::TextSecondary)); });
                        s.item(|ui| { ui.add(MudText::new("● SkiaSharp 2.88.8").typo(Typo::Body2).color(Color::Success)); });
                    });
                });
            });

            ui.add_space(16.0);

            // ── Header Block ──────────────────────────────────────────────
            ui.horizontal_top(|ui| {
                let total = ui.available_width();
                let gap = ui.spacing().item_spacing.x;
                let left_w = ((total - gap) * 0.7).max(0.0);
                let right_w = (total - gap - left_w).max(0.0);

                ui.allocate_ui_with_layout(egui::vec2(left_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.set_min_width(left_w);
                    MudStack::horizontal().spacing(2).align_items(AlignItems::Center).show(ui, |s| {
                        s.item(|ui| { ui.add(MudText::new("MudGrid").typo(Typo::H3)); });
                        s.item(|ui| { ui.add(MudChip::new("SHIPPED v1.0.0").variant(Variant::Filled).size(Size::Small).color(Color::Success)); });
                        s.item(|ui| { ui.add(MudChip::new("Mud.Avalonia.Controls.Layout").variant(Variant::Filled).size(Size::Small).color(Color::Default)); });
                    });
                    ui.add_space(4.0);
                    ui.add(MudText::new("12-column responsive flex-grid system with breakpoint-aware spanning (xs, sm, md, lg, xl, xxl), auto-wrapping, and configurable column/row spacing for Avalonia desktop and cross-platform layouts.").typo(Typo::Body1).color(Color::TextSecondary));
                });

                ui.allocate_ui_with_layout(egui::vec2(right_w, 0.0), egui::Layout::top_down(egui::Align::Max), |ui| {
                    ui.set_min_width(right_w);
                    MudStack::horizontal().spacing(2).justify(Justify::End).show(ui, |s| {
                        s.item(|ui| { ui.add(MudButton::new("Copy XAML").size(Size::Small).variant(Variant::Filled).color(Color::TextSecondary)); });
                        s.item(|ui| { ui.add(MudButton::new("Open in Studio Preview").size(Size::Small).variant(Variant::Filled).color(Color::Primary)); });
                    });
                });
            });

            ui.add_space(24.0);

            // ── Section 01 Header ────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.add(MudText::new("SECTION 01 / CANVAS").typo(Typo::Body2).color(Color::Primary).bold(true));
                    ui.add(MudText::new("Interactive 12-Column Responsive Grid Sandbox").typo(Typo::H5));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    MudCard::new().elevation(0).show(ui, |ui| {
                        MudStack::horizontal().spacing(1).align_items(AlignItems::Center).show(ui, |s| {
                            s.item(|ui| { ui.add(MudChip::new("xs\n0px").variant(Variant::Text).size(Size::Small)); });
                            s.item(|ui| { ui.add(MudChip::new("sm\n600").variant(Variant::Text).size(Size::Small)); });
                            s.item(|ui| { ui.add(MudChip::new("md\n960").variant(Variant::Text).size(Size::Small)); });
                            s.item(|ui| { ui.add(MudChip::new("lg\n1280").variant(Variant::Filled).size(Size::Small).color(Color::Primary)); });
                            s.item(|ui| { ui.add(MudChip::new("xl\n1920").variant(Variant::Text).size(Size::Small)); });
                            s.item(|ui| { ui.add(MudChip::new("xxl\n2560").variant(Variant::Text).size(Size::Small)); });
                        });
                    });
                });
            });

            ui.add_space(12.0);

            // ── Section 01 Body (Canvas + Inspector) ─────────────────────
            ui.horizontal_top(|ui| {
                let total = ui.available_width();
                let gap = ui.spacing().item_spacing.x;
                let left_w = ((total - gap) * 0.67).max(0.0);
                let right_w = (total - gap - left_w).max(0.0);

                // Left Canvas
                ui.allocate_ui_with_layout(egui::vec2(left_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.set_min_width(left_w);
                    MudCard::new().elevation(1).show(ui, |ui| {
                        ui.vertical(|ui| {
                            // Canvas Header
                            ui.horizontal(|ui| {
                                MudStack::horizontal().spacing(2).align_items(AlignItems::Center).show(ui, |s| {
                                    s.item(|ui| { ui.add(MudText::new("Visual Layout Surface").typo(Typo::Subtitle1)); });
                                    s.item(|ui| { ui.add(MudChip::new("Viewport: lg (1280px+)").variant(Variant::Filled).size(Size::Small).color(Color::Default)); });
                                });
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    MudStack::horizontal().spacing(2).align_items(AlignItems::Center).show(ui, |s| {
                                        s.item(|ui| { ui.add(MudText::new("Gutter: 12px (Spacing 3)").typo(Typo::Body2).color(Color::TextSecondary)); });
                                        s.item(|ui| { ui.add(MudText::new("•").typo(Typo::Body2).color(Color::TextSecondary)); });
                                        s.item(|ui| { ui.add(MudText::new("Active Cols: 12").typo(Typo::Body2).color(Color::TextSecondary)); });
                                        s.item(|ui| { ui.add(MudText::new("•").typo(Typo::Body2).color(Color::TextSecondary)); });
                                        s.item(|ui| { ui.add(MudText::new("Skia Measure: 0.04 ms").typo(Typo::Body2).color(Color::Success)); });
                                    });
                                });
                            });

                            ui.add_space(12.0);

                            // Sandbox Row 1
                            MudGrid::new().spacing(3).show(ui, |g| {
                                g.item(MudItem::new().xs(12), |ui| {
                                    MudCard::new().elevation(0).classes(["bg-surface-container-high"]).show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.add(MudChip::new("col-span-12").variant(Variant::Filled).size(Size::Small).color(Color::Primary));
                                            ui.add(MudText::new("MudItem xs=\"12\" (Full Screen Span)").typo(Typo::Body2));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui.add(MudText::new("100% width").typo(Typo::Body2).color(Color::TextSecondary));
                                            });
                                        });
                                    });
                                });
                            });
                            ui.add_space(12.0);

                            // Sandbox Row 2
                            MudGrid::new().spacing(3).show(ui, |g| {
                                g.item(MudItem::new().xs(12).md(6), |ui| {
                                    MudCard::new().elevation(0).classes(["bg-surface-container-high"]).show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.add(MudChip::new("col-span-6").variant(Variant::Filled).size(Size::Small).color(Color::Secondary));
                                            ui.add(MudText::new("xs=\"12\" md=\"6\"").typo(Typo::Body2).color(Color::TextSecondary));
                                        });
                                        ui.add(MudText::new("Dual column split layout card A").typo(Typo::Body2).color(Color::TextSecondary));
                                    });
                                });
                                g.item(MudItem::new().xs(12).md(6), |ui| {
                                    MudCard::new().elevation(0).classes(["bg-surface-container-high"]).show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.add(MudChip::new("col-span-6").variant(Variant::Filled).size(Size::Small).color(Color::Secondary));
                                            ui.add(MudText::new("xs=\"12\" md=\"6\"").typo(Typo::Body2).color(Color::TextSecondary));
                                        });
                                        ui.add(MudText::new("Dual column split layout card B").typo(Typo::Body2).color(Color::TextSecondary));
                                    });
                                });
                            });
                            ui.add_space(12.0);

                            // Sandbox Row 3
                            MudGrid::new().spacing(3).show(ui, |g| {
                                g.item(MudItem::new().xs(12).md(4), |ui| {
                                    MudCard::new().elevation(0).classes(["bg-surface-container-high"]).show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.add(MudChip::new("col-span-4").variant(Variant::Filled).size(Size::Small).color(Color::Default));
                                            ui.add(MudText::new("md=\"4\"").typo(Typo::Body2).color(Color::TextSecondary));
                                        });
                                        ui.add(MudText::new("Panel Alpha metrics").typo(Typo::Body2).color(Color::TextSecondary));
                                    });
                                });
                                g.item(MudItem::new().xs(12).md(4), |ui| {
                                    MudCard::new().elevation(0).classes(["bg-surface-container-high"]).show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.add(MudChip::new("col-span-4").variant(Variant::Filled).size(Size::Small).color(Color::Default));
                                            ui.add(MudText::new("md=\"4\"").typo(Typo::Body2).color(Color::TextSecondary));
                                        });
                                        ui.add(MudText::new("Panel Beta render stats").typo(Typo::Body2).color(Color::TextSecondary));
                                    });
                                });
                                g.item(MudItem::new().xs(12).md(4), |ui| {
                                    MudCard::new().elevation(0).classes(["bg-surface-container-high"]).show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.add(MudChip::new("col-span-4").variant(Variant::Filled).size(Size::Small).color(Color::Default));
                                            ui.add(MudText::new("md=\"4\"").typo(Typo::Body2).color(Color::TextSecondary));
                                        });
                                        ui.add(MudText::new("Panel Gamma diagnostics").typo(Typo::Body2).color(Color::TextSecondary));
                                    });
                                });
                            });
                            ui.add_space(12.0);

                            // Sandbox Row 4
                            MudGrid::new().spacing(3).show(ui, |g| {
                                g.item(MudItem::new().xs(6).lg(3), |ui| {
                                    MudCard::new().elevation(0).classes(["bg-surface-container-high"]).show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.add(MudText::new("col-3").typo(Typo::Body2).color(Color::Primary).bold(true));
                                            ui.add(MudText::new("Quarter 1").typo(Typo::Caption).color(Color::TextSecondary));
                                        });
                                    });
                                });
                                g.item(MudItem::new().xs(6).lg(3), |ui| {
                                    MudCard::new().elevation(0).classes(["bg-surface-container-high"]).show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.add(MudText::new("col-3").typo(Typo::Body2).color(Color::Primary).bold(true));
                                            ui.add(MudText::new("Quarter 2").typo(Typo::Caption).color(Color::TextSecondary));
                                        });
                                    });
                                });
                                g.item(MudItem::new().xs(6).lg(3), |ui| {
                                    MudCard::new().elevation(0).classes(["bg-surface-container-high"]).show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.add(MudText::new("col-3").typo(Typo::Body2).color(Color::Primary).bold(true));
                                            ui.add(MudText::new("Quarter 3").typo(Typo::Caption).color(Color::TextSecondary));
                                        });
                                    });
                                });
                                g.item(MudItem::new().xs(6).lg(3), |ui| {
                                    MudCard::new().elevation(0).classes(["bg-surface-container-high"]).show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.add(MudText::new("col-3").typo(Typo::Body2).color(Color::Primary).bold(true));
                                            ui.add(MudText::new("Quarter 4").typo(Typo::Caption).color(Color::TextSecondary));
                                        });
                                    });
                                });
                            });
                            ui.add_space(12.0);

                            // Sandbox Row 5 (Offset)
                            MudGrid::new().spacing(3).show(ui, |g| {
                                g.item(MudItem::new().xs(12).md(8), |ui| {
                                    MudCard::new().elevation(0).classes(["bg-primary-container"]).show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            MudStack::horizontal().spacing(1).show(ui, |s| {
                                                s.item(|ui| { ui.add(MudChip::new("col-span-8").variant(Variant::Filled).size(Size::Small).color(Color::Primary)); });
                                                s.item(|ui| { ui.add(MudChip::new("offset-2 (sm:col-start-3)").variant(Variant::Filled).size(Size::Small).color(Color::Default)); });
                                            });
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui.add(MudText::new("Centered Content Shell").typo(Typo::Body2).color(Color::Success));
                                            });
                                        });
                                        ui.add(MudText::new("Ideal for form wizards, focused inspector sheets, or modal body templates in desktop Avalonia applications.").typo(Typo::Body2).color(Color::TextSecondary));
                                    });
                                });
                            });

                            ui.add_space(16.0);

                            // Grid Guide Visualizer
                            ui.vertical(|ui| {
                                MudGrid::new().spacing(1).show(ui, |g| {
                                    for i in 1..=12 {
                                        // ⬇ FIX: `move` so `i` (i32: Copy) is captured by value,
                                        //    satisfying the `'static` bound on the closure.
                                        g.item(MudItem::new().xs(1), move |ui| {
                                            ui.vertical_centered(|ui| {
                                                ui.add(MudText::new(format!("{}", i)).typo(Typo::Caption).color(Color::TextSecondary));
                                            });
                                        });
                                    }
                                });
                                ui.add_space(4.0);
                                MudGrid::new().spacing(1).show(ui, |g| {
                                    for _ in 0..12 {
                                        g.item(MudItem::new().xs(1), |ui| {
                                            ui.add_space(4.0);
                                            // Just a visual bar
                                            MudCard::new().elevation(0).classes(["bg-primary"]).show(ui, |ui| {
                                                ui.allocate_space(egui::vec2(ui.available_width(), 4.0));
                                            });
                                        });
                                    }
                                });
                            });
                        });
                    });
                });

                // Right Inspector
                ui.allocate_ui_with_layout(egui::vec2(right_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.set_min_width(right_w);
                    MudCard::new().elevation(1).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.add(MudText::new("Property Inspector").typo(Typo::Subtitle1));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add(MudText::new("StyledProperties").typo(Typo::Body2).color(Color::TextSecondary));
                                });
                            });

                            ui.add_space(12.0);

                            // Spacing Slider (Static Visual)
                            ui.horizontal(|ui| {
                                ui.add(MudText::new("Spacing (Gutter)").typo(Typo::Body2));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add(MudChip::new("3 (12px)").variant(Variant::Filled).size(Size::Small).color(Color::Default));
                                });
                            });
                            ui.add_space(4.0);
                            // Purely visual slider representation
                            ui.vertical(|ui| {
                                ui.add_space(6.0);
                                // `Color::resolve` needs the MudTheme palette, not egui's Visuals.
                                let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
                                let palette = theme.current_palette();
                                let track_color = Color::TextSecondary
                                    .resolve(&palette)
                                    .unwrap_or_else(|| ui.visuals().weak_text_color());
                                let knob_color = Color::Primary
                                    .resolve(&palette)
                                    .unwrap_or(ui.visuals().selection.bg_fill);
                                ui.painter().line_segment(
                                    [egui::pos2(ui.cursor().left(), ui.cursor().top()), egui::pos2(ui.cursor().left() + ui.available_width(), ui.cursor().top())],
                                    egui::Stroke::new(4.0, track_color),
                                );
                                ui.painter().circle_filled(
                                    egui::pos2(ui.cursor().left() + ui.available_width() * 0.3, ui.cursor().top()),
                                    6.0,
                                    knob_color,
                                );
                                ui.allocate_space(egui::vec2(ui.available_width(), 12.0));
                            });
                            ui.horizontal(|ui| {
                                ui.add(MudText::new("0 (0px)").typo(Typo::Caption).color(Color::TextSecondary));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add(MudText::new("10 (40px)").typo(Typo::Caption).color(Color::TextSecondary));
                                });
                            });

                            ui.add_space(12.0);

                            // Justify Content
                            ui.add(MudText::new("Justify (Main Axis)").typo(Typo::Body2));
                            MudGrid::new().spacing(1).show(ui, |g| {
                                g.item(MudItem::new().xs(6), |ui| { ui.add_sized([ui.available_width(), 24.0], MudButton::new("FlexStart").variant(Variant::Filled).color(Color::Primary).size(Size::Small)); });
                                g.item(MudItem::new().xs(6), |ui| { ui.add_sized([ui.available_width(), 24.0], MudButton::new("Center").variant(Variant::Text).size(Size::Small)); });
                                g.item(MudItem::new().xs(6), |ui| { ui.add_sized([ui.available_width(), 24.0], MudButton::new("FlexEnd").variant(Variant::Text).size(Size::Small)); });
                                g.item(MudItem::new().xs(6), |ui| { ui.add_sized([ui.available_width(), 24.0], MudButton::new("SpaceBetween").variant(Variant::Text).size(Size::Small)); });
                                g.item(MudItem::new().xs(6), |ui| { ui.add_sized([ui.available_width(), 24.0], MudButton::new("SpaceAround").variant(Variant::Text).size(Size::Small)); });
                                g.item(MudItem::new().xs(6), |ui| { ui.add_sized([ui.available_width(), 24.0], MudButton::new("SpaceEvenly").variant(Variant::Text).size(Size::Small)); });
                            });

                            ui.add_space(12.0);

                            // AlignItems
                            ui.add(MudText::new("AlignItems (Cross Axis)").typo(Typo::Body2));
                            MudGrid::new().spacing(1).show(ui, |g| {
                                g.item(MudItem::new().xs(4), |ui| { ui.add_sized([ui.available_width(), 24.0], MudButton::new("Stretch").variant(Variant::Filled).color(Color::Primary).size(Size::Small)); });
                                g.item(MudItem::new().xs(4), |ui| { ui.add_sized([ui.available_width(), 24.0], MudButton::new("Start").variant(Variant::Text).size(Size::Small)); });
                                g.item(MudItem::new().xs(4), |ui| { ui.add_sized([ui.available_width(), 24.0], MudButton::new("Center").variant(Variant::Text).size(Size::Small)); });
                            });

                            ui.add_space(12.0);

                            // Column Resolution Mode
                            ui.add(MudText::new("Column Resolution Mode").typo(Typo::Body2));
                            MudCard::new().elevation(0).classes(["bg-surface-container-lowest", "p-1"]).show(ui, |ui| {
                                MudStack::horizontal().spacing(1).show(ui, |s| {
                                    s.item(|ui| { ui.add_sized([ui.available_width() * 0.5 - 4.0, 24.0], MudButton::new("Manual (12-Col)").variant(Variant::Filled).color(Color::Default).size(Size::Small)); });
                                    s.item(|ui| { ui.add_sized([ui.available_width(), 24.0], MudButton::new("Auto-Fit Columns").variant(Variant::Text).size(Size::Small)); });
                                });
                            });

                            ui.add_space(16.0);

                            // Generated Code Preview
                            MudCard::new().elevation(0).classes(["bg-surface-container-lowest"]).show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.add(MudText::new("Current XAML Evaluator").typo(Typo::Caption).color(Color::TextSecondary));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.add(MudText::new("SYNTAX VALID").typo(Typo::Caption).color(Color::Success));
                                        });
                                    });
                                    ui.add_space(4.0);
                                    ui.add(MudText::new("<mud:MudGrid Spacing=\"3\" Justify=\"Justify.FlexStart\">").monospace(true).color(Color::Primary));
                                });
                            });

                            ui.add_space(16.0);

                            // Architecture Card
                            MudCard::new().elevation(1).classes(["bg-surface-container-high"]).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.add(MudText::new("🧠").typo(Typo::H5));
                                    ui.vertical(|ui| {
                                        ui.add(MudText::new("Hardware Accelerated Layout Pass").typo(Typo::Subtitle2));
                                        ui.add(MudText::new("MudGrid delegates measured bounds directly to Avalonia's LayoutManager, bypassing redundant WPF-style measure-arrange invalidate loops.").typo(Typo::Caption).color(Color::TextSecondary));
                                    });
                                });
                            });
                        });
                    });
                });
            });

            ui.add_space(24.0);

            // ── Section 02: Declarative Code ─────────────────────────────
            ui.vertical(|ui| {
                ui.add(MudText::new("SECTION 02 / CODE PATTERNS").typo(Typo::Body2).color(Color::Primary).bold(true));
                ui.add(MudText::new("Declarative XAML & MVVM Implementation").typo(Typo::H5));
            });

            ui.add_space(12.0);

            MudGrid::new().spacing(3).show(ui, |g| {
                g.item(MudItem::new().xs(12).lg(6), |ui| {
                    MudCard::new().elevation(1).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.add(MudText::new("DashboardOverviewView.axaml").typo(Typo::Body2));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add(MudChip::new("XAML 2009 / Avalonia 11").variant(Variant::Filled).size(Size::Small).color(Color::TextSecondary));
                                });
                            });
                            ui.add(MudDivider::new());
                            ui.add(MudText::new(
                                r#"<!-- 12-Column Responsive MudGrid Container -->
<mud:MudGrid Spacing="3" 
             Justify="Justify.SpaceBetween"
             AlignItems="AlignItems.Stretch"
             Classes="dashboard-root-grid">

  <!-- Metric Widget 1: Responsive Col Breakdown -->
  <mud:MudItem xs="12" sm="6" md="4" lg="3">
    <mud:MudCard Elevation="2" Classes="p-3 rounded-lg">
      <mud:MudCardHeader>
        <mud:MudText Typo="Typo.h6" 
                     Text="{Binding TotalThroughput}" />
      </mud:MudCardHeader>
    </mud:MudCard>
  </mud:MudItem>

  <!-- Central Feed with Adaptive Offset -->
  <mud:MudItem xs="12" md="8" Offset="Breakpoint.Md, 2">
    <mud:MudDataGrid ItemsSource="{Binding MetricTimeSeries}" />
  </mud:MudItem>

</mud:MudGrid>"#).monospace(true).color(Color::TextSecondary));
                        });
                    });
                });

                g.item(MudItem::new().xs(12).lg(6), |ui| {
                    MudCard::new().elevation(1).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.add(MudText::new("DashboardOverviewViewModel.cs").typo(Typo::Body2));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add(MudChip::new("CommunityToolkit.Mvvm").variant(Variant::Filled).size(Size::Small).color(Color::TextSecondary));
                                });
                            });
                            ui.add(MudDivider::new());
                            ui.add(MudText::new(
                                r#"using CommunityToolkit.Mvvm.ComponentModel;
using Mud.Avalonia.Controls.Layout;
using System.Collections.ObjectModel;

public partial class DashboardOverviewViewModel : ObservableObject
{
    [ObservableProperty]
    private int _gridSpacing = 3;

    [ObservableProperty]
    private Breakpoint _currentBreakpoint = Breakpoint.Lg;

    public ObservableCollection<WidgetMetricCard> Cards { get; } = new();

    [RelayCommand]
    public void OnWindowResized(double width)
    {
        CurrentBreakpoint = width switch
        {
            < 600 => Breakpoint.Xs,
            < 960 => Breakpoint.Sm,
            < 1280 => Breakpoint.Md,
            < 1920 => Breakpoint.Lg,
            _ => Breakpoint.Xl
        };
    }
}"#).monospace(true).color(Color::TextSecondary));
                        });
                    });
                });
            });

            ui.add_space(24.0);

            // ── Section 03: API Reference ────────────────────────────────
            ui.vertical(|ui| {
                ui.add(MudText::new("SECTION 03 / SPECIFICATIONS").typo(Typo::Body2).color(Color::Primary).bold(true));
                ui.add(MudText::new("API & StyledProperties Reference").typo(Typo::H5));
            });

            ui.add_space(12.0);

            MudCard::new().elevation(1).show(ui, |ui| {
                MudSimpleTable::new().bordered(true).striped(true).show(ui, |t| {
                    t.header(|h| {
                        h.cell("StyledProperty");
                        h.cell("Type");
                        h.cell("Default");
                        h.cell("Description");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("Spacing").color(Color::Primary).bold(true)); });
                        r.cell("int");
                        r.cell_ui(|ui| { ui.add(MudText::new("3 (12px)").color(Color::TextSecondary)); });
                        r.cell("Spacing multiplier along row and column channels (multiplied by the 4px base layout grid).");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("Justify").color(Color::Primary).bold(true)); });
                        r.cell("Justify");
                        r.cell_ui(|ui| { ui.add(MudText::new("FlexStart").color(Color::TextSecondary)); });
                        r.cell("Defines horizontal alignment of columns within excess container width (FlexStart, Center, FlexEnd, SpaceBetween, SpaceAround, SpaceEvenly).");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("AlignItems").color(Color::Primary).bold(true)); });
                        r.cell("AlignItems");
                        r.cell_ui(|ui| { ui.add(MudText::new("Stretch").color(Color::TextSecondary)); });
                        r.cell("Defines vertical cross-axis stretch and alignment behavior for child MudItems within each row segment.");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("Breakpoint").color(Color::Primary).bold(true)); });
                        r.cell("Breakpoint");
                        r.cell_ui(|ui| { ui.add(MudText::new("Auto (Window)").color(Color::TextSecondary)); });
                        r.cell("Allows pinning or overriding responsive width calculations to a specific resolution threshold.");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("MaxWidth").color(Color::Primary).bold(true)); });
                        r.cell("MaxWidth");
                        r.cell_ui(|ui| { ui.add(MudText::new("False").color(Color::TextSecondary)); });
                        r.cell("Constrains grid horizontal extent automatically to standard container dimensions (e.g., MaxWidth.Medium = 960px).");
                    });
                    t.row(|r| {
                        r.cell_ui(|ui| { ui.add(MudText::new("RowSpacing").color(Color::Primary).bold(true)); });
                        r.cell("int?");
                        r.cell_ui(|ui| { ui.add(MudText::new("null").color(Color::TextSecondary)); });
                        r.cell("Independent vertical row spacing override. Inherits from Spacing when null.");
                    });
                });
            });

            ui.add_space(24.0);

            // ── Section 04: Ergonomics ───────────────────────────────────
            ui.vertical(|ui| {
                ui.add(MudText::new("SECTION 04 / ERGONOMICS & COMPLIANCE").typo(Typo::Body2).color(Color::Primary).bold(true));
                ui.add(MudText::new("Desktop Window Resizing Ergonomics & Accessibility (WCAG 2.2)").typo(Typo::H5));
            });

            ui.add_space(12.0);

            MudGrid::new().spacing(3).show(ui, |g| {
                g.item(MudItem::new().xs(12).md(4), |ui| {
                    MudCard::new().elevation(1).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.add(MudText::new("📐").typo(Typo::H3));
                            ui.add(MudText::new("Zero-Thrash Reflow").typo(Typo::Subtitle1));
                            ui.add(MudText::new("During continuous Win32/X11 window resizing, MudGrid debounces breakpoint dispatchers across high DPI monitors, avoiding catastrophic layout cascades and dropped Skia presentation frames.").typo(Typo::Body2).color(Color::TextSecondary));
                            ui.add_space(8.0);
                            ui.add(MudText::new("✓ 60 FPS Continuous Drag").typo(Typo::Body2).color(Color::Success));
                        });
                    });
                });
                g.item(MudItem::new().xs(12).md(4), |ui| {
                    MudCard::new().elevation(1).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.add(MudText::new("⌨").typo(Typo::H3));
                            ui.add(MudText::new("Preserved Keyboard Traversal").typo(Typo::Subtitle1));
                            ui.add(MudText::new("Maintains bidirectional tab indexing. Even when elements wrap across multiple row channels at smaller desktop breakpoints, sequential focus sequence strictly matches DOM and XAML visual order.").typo(Typo::Body2).color(Color::TextSecondary));
                            ui.add_space(8.0);
                            ui.add(MudText::new("✓ WCAG 2.4.3 Compliant").typo(Typo::Body2).color(Color::Success));
                        });
                    });
                });
                g.item(MudItem::new().xs(12).md(4), |ui| {
                    MudCard::new().elevation(1).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.add(MudText::new("🗣").typo(Typo::H3));
                            ui.add(MudText::new("Assistive Automation Node").typo(Typo::Subtitle1));
                            ui.add(MudText::new("Exposes an AutomationPeer representing grid structure semantics directly to UI Automation (UIA) on Windows and AT-SPI on Linux, without nested layout noise.").typo(Typo::Body2).color(Color::TextSecondary));
                            ui.add_space(8.0);
                            ui.add(MudText::new("✓ Native UIA Tree Mapping").typo(Typo::Body2).color(Color::Success));
                        });
                    });
                });
            });
        });
    }
}