//! Main application layout: drawer sidebar + app bar + routed body.
//!
//! `Layout::show` receives only [`NavigationContext`], which is a read-only
//! data bundle — it does **not** carry a [`NavigationManager`]. Sidebar links
//! therefore queue their target into `PENDING_NAV`, which `App::ui` drains
//! next frame. See `app.rs` for the protocol.
//!
//! Every widget below is a `Mud*` control — no raw `egui` primitives in the
//! sidebar. `MudNavMenu` receives a throwaway `NavigationManager` so its
//! active-detection API is available; the links that are actually clickable
//! carry `href = None` and route through `on_click`.

use egui::{vec2, Align, Color32, Ui};
use mud_egui_components::buttons::icon_button::mud_icon_button::MudIconButton;
use mud_egui_components::inputs::text_field::{AdornmentEnd, MudTextField};
use mud_egui_components::layout::{
    DrawerState, LayoutState, MudAppBar, MudDrawer, MudLayout, MudMainContent,
};
use mud_egui_components::{MudChip, MudContainer, MudDivider, MudIcon, MudStack, MudText};
use mud_egui_components::navigation::nav_menu::{
    MudNavMenu, NavGroup, NavLink, NavMenuState,
};
use mud_egui_core::enums::{Color, DrawerVariant, Margin, MaxWidth, Size, Typo, Variant};
use mud_egui_icons_material::filled;
use mud_egui_icons_simple_icons::{brand_colors, brand_icons};
use mud_egui_navigation::navigation::NavigationManager;
use mud_egui_navigation::page::{Layout, NavigationContext};

/// Horizontal gap between the drawer and the body column, in points.
const DRAWER_GAP: f32 = 2.0;

/// Vertical breathing room at the top of the drawer.
const DRAWER_TOP_PADDING: f32 = 8.0;

/// Temp-data key shared with `app.rs`. Pushing a target here defers the
/// actual `navigate_to` call to `App::ui` on the next frame.
const PENDING_NAV: &str = "pending_nav";

// ─────────────────────────────────────────────────────────────────────────
//  Sidebar helpers
// ─────────────────────────────────────────────────────────────────────────

/// Build a `NavLink` that **defers** navigation via `PENDING_NAV`.
///
/// `MudNavMenu` requires a live `NavigationManager`, which `Layout::show`
/// does not have. We therefore pass a throwaway manager just for the
/// signature and route the actual navigation through `on_click`.
fn link<'a>(ctx: &egui::Context, href: &'a str, text: &'a str, icon: &'a str) -> NavLink<'a> {
    let ctx = ctx.clone();
    let target = href.to_string();
    let text_upper = text.to_uppercase();
    NavLink::label(text_upper)
        .icon(icon)
        .on_click(move || {
            ctx.data_mut(|d| {
                d.insert_temp(egui::Id::new(PENDING_NAV), target.clone());
            });
        })
}

/// Group header with a trailing count chip, via `title_content`.
fn group_header<'a>(
    id: &'a str,
    title: &'a str,
    icon: &'a str,
    count: &'a str,
) -> NavGroup<'a> {
    let title_owned = title.to_uppercase();
    let count_owned = count.to_string();
    let chip_id = format!("nav-group-chip-{id}"); // Unique per group to avoid ID collisions
    NavGroup::new(title_owned.clone())
        .id(id)
        .icon(icon)
        .title_content(move |ui| {
            MudStack::horizontal()
                .spacing(1)
                .align_items(mud_egui_core::enums::AlignItems::Center)
                .show(ui, |ui| {
                    ui.add(MudText::new(title_owned.clone()).uppercase(true).typo(Typo::Body2));
                    ui.add(
                        MudChip::new(count_owned.clone())
                            .id(chip_id.clone())
                            .size(Size::Small)
                            .variant(Variant::Text),
                    );
                });
        })
}

/// `(id, title, icon, count, [(href, text, icon), …])` for the docs groups.
const DOCS_GROUPS: &[(&str, &str, &str, &str, &[(&str, &str, &str)])] = &[
    (
        "layout-containers", "Layout & Containers", filled::GRID_VIEW, "8",
        &[
            ("/docs/layout-containers/mudcontainer", "MudContainer", filled::CROP_SQUARE),
            ("/docs/layout-containers/mudgrid",      "MudGrid",      filled::VIEW_MODULE),
            ("/docs/layout-containers/mudstack",     "MudStack",     filled::VIEW_AGENDA),
            ("/docs/layout-containers/mudpaper",     "MudPaper",     filled::DESCRIPTION),
            ("/docs/layout-containers/muddivider",   "MudDivider",   filled::HORIZONTAL_RULE),
            ("/docs/layout-containers/mudhidden",    "MudHidden",    filled::VISIBILITY_OFF),
            ("/docs/layout-containers/mudtoolbar",   "MudToolBar",   filled::CONSTRUCTION),
            ("/docs/layout-containers/mudcollapse",  "MudCollapse",  filled::UNFOLD_LESS),
        ],
    ),
    (
        "buttons-actions", "Buttons & Actions", filled::SMART_BUTTON, "6",
        &[
            ("/docs/buttons-actions/mudbutton",           "MudButton",           filled::TOUCH_APP),
            ("/docs/buttons-actions/mudiconbutton",       "MudIconButton",       filled::STAR),
            ("/docs/buttons-actions/mudtoggleiconbutton", "MudToggleIconButton", filled::TOGGLE_ON),
            ("/docs/buttons-actions/mudbuttongroup",      "MudButtonGroup",      filled::VIEW_COLUMN),
            ("/docs/buttons-actions/mudfab",              "MudFab",              filled::ADD_CIRCLE),
            ("/docs/buttons-actions/mudfabmenu",          "MudFabMenu",          filled::MORE_VERT),
        ],
    ),
    (
        "inputs-selection", "Inputs & Selection", filled::CHECK_BOX, "9",
        &[
            ("/docs/inputs-selection/mudtextfield",   "MudTextField",    filled::TEXT_FIELDS),
            ("/docs/inputs-selection/mudautocomplete", "MudAutocomplete", filled::MANAGE_SEARCH),
            ("/docs/inputs-selection/mudselect",      "MudSelect",       filled::ARROW_DROP_DOWN_CIRCLE),
            ("/docs/inputs-selection/mudcheckbox",    "MudCheckBox",     filled::CHECK_BOX),
            ("/docs/inputs-selection/mudswitch",      "MudSwitch",       filled::TOGGLE_OFF),
            ("/docs/inputs-selection/mudradiogroup",  "MudRadioGroup",   filled::RADIO_BUTTON_CHECKED),
            ("/docs/inputs-selection/mudslider",      "MudSlider",       filled::TUNE),
            ("/docs/inputs-selection/mudrating",      "MudRating",       filled::STAR_RATE),
            ("/docs/inputs-selection/mudtogglegroup", "MudToggleGroup",  filled::VIEW_WEEK),
        ],
    ),
    (
        "pickers", "Pickers", filled::CALENDAR_MONTH, "4",
        &[
            ("/docs/pickers/muddatepicker",      "MudDatePicker",      filled::EVENT),
            ("/docs/pickers/muddaterangepicker", "MudDateRangePicker", filled::DATE_RANGE),
            ("/docs/pickers/mudtimepicker",      "MudTimePicker",      filled::SCHEDULE),
            ("/docs/pickers/mudcolorpicker",     "MudColorPicker",     filled::PALETTE),
        ],
    ),
    (
        "forms-validation", "Forms & Validation", filled::VERIFIED, "1",
        &[("/docs/forms-validation/mudform", "MudForm", filled::ASSIGNMENT)],
    ),
    (
        "navigation", "Navigation", filled::MENU, "8",
        &[
            ("/docs/navigation/mudnavmenu",     "MudNavMenu",     filled::MENU),
            ("/docs/navigation/mudnavgroup",    "MudNavGroup",    filled::UNFOLD_MORE),
            ("/docs/navigation/mudtabs",        "MudTabs",        filled::TAB),
            ("/docs/navigation/muddynamictabs", "MudDynamicTabs", filled::TAB_UNSELECTED),
            ("/docs/navigation/mudpagination",  "MudPagination",  filled::LAST_PAGE),
            ("/docs/navigation/mudstepper",     "MudStepper",     filled::LINEAR_SCALE),
            ("/docs/navigation/mudbreadcrumbs", "MudBreadcrumbs", filled::NAVIGATE_NEXT),
            ("/docs/navigation/mudlink",        "MudLink",        filled::LINK),
        ],
    ),
    (
        "display-feedback", "Display & Feedback", filled::NOTIFICATIONS_ACTIVE, "6",
        &[
            ("/docs/display-feedback/mudtext",    "MudText",    filled::TITLE),
            ("/docs/display-feedback/mudavatar",  "MudAvatar",  filled::ACCOUNT_CIRCLE),
            ("/docs/display-feedback/mudchip",    "MudChip",    filled::LABEL),
            ("/docs/display-feedback/mudchipset", "MudChipSet", filled::SELL),
            ("/docs/display-feedback/mudicon",    "MudIcon",    filled::CATEGORY),
            ("/docs/display-feedback/mudimage",   "MudImage",   filled::IMAGE),
        ],
    ),
    (
        "cards-media", "Cards & Media", filled::FEATURED_PLAY_LIST, "5",
        &[
            ("/docs/cards-media/mudcard",        "MudCard",        filled::CREDIT_CARD),
            ("/docs/cards-media/mudcardheader",  "MudCardHeader",  filled::VIEW_HEADLINE),
            ("/docs/cards-media/mudcardmedia",   "MudCardMedia",   filled::PERM_MEDIA),
            ("/docs/cards-media/mudcardcontent", "MudCardContent", filled::SUBJECT),
            ("/docs/cards-media/mudcardactions", "MudCardActions", filled::CALL_TO_ACTION),
        ],
    ),
    (
        "data-display", "Data Display", filled::TABLE_CHART, "4",
        &[
            ("/docs/data-display/mudhighlighter", "MudHighlighter", filled::HIGHLIGHT),
            ("/docs/data-display/mudavatargroup", "MudAvatarGroup", filled::GROUPS),
            ("/docs/data-display/mudbadge",       "MudBadge",       filled::BADGE),
            ("/docs/data-display/mudskeleton",    "MudSkeleton",    filled::HOURGLASS_EMPTY),
        ],
    ),
    (
        "overlays-feedback", "Overlays & Feedback", filled::LAYERS, "4",
        &[
            ("/docs/overlays-feedback/mudalert",            "MudAlert",            filled::ERROR_OUTLINE),
            ("/docs/overlays-feedback/mudtooltip",          "MudTooltip",          filled::COMMENT),
            ("/docs/overlays-feedback/mudprogresslinear",   "MudProgressLinear",   filled::TIMELINE),
            ("/docs/overlays-feedback/mudprogresscircular", "MudProgressCircular", filled::AUTORENEW),
        ],
    ),
    (
        "trees-timelines", "Trees & Timelines", filled::ACCOUNT_TREE, "4",
        &[
            ("/docs/trees-timelines/treeview",     "TreeView",     filled::ACCOUNT_TREE),
            ("/docs/trees-timelines/treeviewitem", "TreeViewItem", filled::SUBDIRECTORY_ARROW_RIGHT),
            ("/docs/trees-timelines/timeline",     "Timeline",     filled::TIMELINE),
            ("/docs/trees-timelines/timelineitem", "TimelineItem", filled::FIBER_MANUAL_RECORD),
        ],
    ),
    (
        "charts-advanced", "Charts & Advanced", filled::INSIGHTS, "4",
        &[
            ("/docs/charts-advanced/mudcarousel",   "MudCarousel",   filled::VIEW_CAROUSEL),
            ("/docs/charts-advanced/mudsplitpanel", "MudSplitPanel", filled::VERTICAL_SPLIT),
            ("/docs/charts-advanced/mudhotkey",     "MudHotkey",     filled::KEYBOARD),
            ("/docs/charts-advanced/mudfileupload", "MudFileUpload", filled::UPLOAD_FILE),
        ],
    ),
];

// ─────────────────────────────────────────────────────────────────────────
//  Layout
// ─────────────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct MainLayout {
    layout: LayoutState,
    drawer: DrawerState,
    /// Bound buffer for the quick-search field.
    search: String,
    /// Group expansion flags — shared by all four `MudNavMenu` instances.
    nav_state: NavMenuState,
}

impl Layout for MainLayout {
    fn show(
        &mut self,
        ui: &mut Ui,
        _ctx: &NavigationContext,
        body: &mut dyn FnMut(&mut Ui),
    ) {
        let Self { layout, drawer, search, nav_state } = self;

        // A throwaway manager, created once per frame, only so `MudNavMenu`
        // can satisfy its signature and run active-detection. The clicks
        // themselves are routed through `on_click` → `PENDING_NAV`.
        let mut nav_mgr = NavigationManager::new("/", "app://mud.egui/", 1);

        MudLayout::new()
            .classes(["bg-background"])
            .show(ui, layout, |ui, layout| {
                ui.horizontal(|ui| {
                    MudDrawer::new()
                        .width(270.0)
                        .classes(["p-2"])
                        .elevation(0)
                        .variant(DrawerVariant::Responsive)
                        .show(ui, drawer, layout, |ui| {
                            ui.add_space(DRAWER_TOP_PADDING);

                            MudStack::new().show(ui, |ui| {
                                // ── Brand ────────────────────────────────
                                MudStack::horizontal().spacing(2).show(ui, |ui| {
                                    ui.add(
                                        MudIcon::new(brand_icons::SPARKPOST)
                                            .size(4)
                                            .custom_color(hex_to_color32(
                                                brand_colors::SPARKPOST,
                                            )),
                                    );
                                    MudStack::new().spacing(0).show(ui, |ui| {
                                        ui.add(
                                            MudText::new("Mud.Egui")
                                                .bold(true)
                                                .typo(Typo::H5),
                                        );
                                        ui.add(
                                            MudText::new("Ported, not approximated.")
                                                .typo(Typo::Subtitle2),
                                        );
                                    });
                                });

                                // ── Search ───────────────────────────────
                                MudTextField::new(search)
                                    .margin(Margin::Dense)
                                    .variant(Variant::Outlined)
                                    .placeholder("Quick Search")
                                    .adornment_end(AdornmentEnd::icon(filled::SEARCH))
                                    .show(ui);

                                ui.add(MudDivider::new());

                                // ── Nav ──────────────────────────────────
                                let ecx = ui.ctx().clone();

                                // GETTING STARTED
                                ui.add(MudText::new("GETTING STARTED").typo(Typo::Overline));
                                MudNavMenu::new()
                                    .color(Color::Primary)
                                    .dense(true)
                                    .show(ui, &mut nav_mgr, nav_state, |menu| {
                                        menu.link(link(&ecx, "/installation", "Installation", filled::DOWNLOAD));
                                        menu.link(link(&ecx, "/overview",     "Overview",     filled::EXPLORE));
                                        menu.link(link(&ecx, "/architecture", "Architecture", filled::SCHEMA));
                                    });
                                ui.add_space(6.0);

                                // MAIN
                                ui.add(MudText::new("MAIN").typo(Typo::Overline));
                                MudNavMenu::new()
                                    .color(Color::Primary)
                                    .dense(true)
                                    .show(ui, &mut nav_mgr, nav_state, |menu| {
                                        menu.link(link(&ecx, "/dashboard", "Dashboard", filled::DASHBOARD));
                                        menu.link(link(&ecx, "/all-components", "All Components", filled::WIDGETS));
                                        menu.link(link(&ecx, "/studio-playground", "Studio / Playground", filled::CODE));
                                        menu.link(link(&ecx, "/theme-customizer", "Theme Customizer", filled::PALETTE));
                                    });
                                ui.add_space(6.0);

                                // DOCUMENTATION
                                ui.add(MudText::new("DOCUMENTATION").typo(Typo::Overline));
                                MudNavMenu::new()
                                    .color(Color::Primary)
                                    .dense(true)
                                    .show(ui, &mut nav_mgr, nav_state, |menu| {
                                        for (id, title, icon, count, items) in DOCS_GROUPS {
                                            let mut g = group_header(id, title, icon, count);
                                            for (href, text, icon) in *items {
                                                g = g.link(link(&ecx, href, text, icon));
                                            }
                                            menu.group(g);
                                        }
                                    });
                                ui.add_space(6.0);

                                // TOOLS & SYSTEM
                                ui.add(MudText::new("TOOLS & SYSTEM").typo(Typo::Overline));
                                MudNavMenu::new()
                                    .color(Color::Primary)
                                    .dense(true)
                                    .show(ui, &mut nav_mgr, nav_state, |menu| {
                                        menu.link(link(&ecx, "/themes", "Themes", filled::FORMAT_PAINT));
                                        menu.link(link(&ecx, "/changelog", "Changelog", filled::HISTORY));
                                    });
                            });
                        });

                    // Small gap between the drawer and the body column.
                    ui.add_space(DRAWER_GAP);

                    // The body column: app bar on top, main content below.
                    let body_w = ui.available_width();
                    let body_h = ui
                        .available_height()
                        .max(layout.available_row_height());
                    ui.allocate_ui_with_layout(
                        vec2(body_w, body_h),
                        egui::Layout::top_down(Align::Min),
                        |ui| {
                            MudAppBar::new()
                                .elevation(0)
                                .dense(true)
                                .show(ui, layout, |ui| {
                                    let clicked = ui
                                        .add(
                                            MudIconButton::new(filled::MENU)
                                                .variant(Variant::Text)
                                                .color(Color::Inherit)
                                                .size(Size::Small),
                                        )
                                        .clicked();
                                    if clicked {
                                        drawer.toggle();
                                    }
                                    ui.add(MudText::new("MY APP"));
                                });

                            MudMainContent::new().show(ui, layout, |ui| {
                                // Sticky rule at the top of the content area.
                                ui.add(MudDivider::new());

                                egui::ScrollArea::vertical()
                                    .id_salt("main-content-scroll")
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        MudContainer::new()
                                            .classes(["py-3"])
                                            .gutters(true)
                                            .max_width(MaxWidth::Lg)
                                            .show(ui, |ui| {
                                                body(ui);
                                            });
                                    });
                            });
                        },
                    );
                });
            });
    }
}

/// Parse a `#RGB`, `#RGBA`, `#RRGGBB`, or `#RRGGBBAA` hex string into a
/// [`Color32`]. Returns [`Color32::GRAY`] when the input is not a valid hex
/// colour.
fn hex_to_color32(hex: &str) -> Color32 {
    let s = hex.trim_start_matches('#');
    let parse = |r: std::ops::Range<usize>| -> u8 { u8::from_str_radix(&s[r], 16).unwrap_or(0) };
    match s.len() {
        3 => Color32::from_rgb(parse(0..1) * 17, parse(1..2) * 17, parse(2..3) * 17),
        4 => Color32::from_rgba_unmultiplied(
            parse(0..1) * 17,
            parse(1..2) * 17,
            parse(2..3) * 17,
            parse(3..4) * 17,
        ),
        6 => Color32::from_rgb(parse(0..2), parse(2..4), parse(4..6)),
        8 => Color32::from_rgba_unmultiplied(parse(0..2), parse(2..4), parse(4..6), parse(6..8)),
        _ => Color32::GRAY,
    }
}