#![allow(dead_code, unused, non_camel_case_types)]

pub mod buttons;
pub mod data_display;
pub mod feedback;
pub mod foundation;
pub mod inputs;
pub mod layout;
pub mod navigation;
pub mod surfaces;
pub mod typography;

// =============================================================================
// Component Re-exports
// =============================================================================
// This allows users to write:
// use mud_egui_components::MudText;
// use mud_egui_components::MudStack;

pub use mud_egui_core::base::component_base;

pub use typography::mud_text::MudText;
pub use data_display::code::mud_code_block::{MudCodeBlock};

pub use layout::container::mud_container::{MudContainer, HorizontalLayout,MudContainerMetrics};
pub use layout::stack::mud_stack::MudStack;
pub use layout::spacer::mud_spacer::MudSpacer;
pub use layout::grid::mud_grid::MudGrid;
pub use layout::grid::mud_item::MudItem;
pub use layout::grid::mud_grid_span::MudGridSpan;
pub use layout::divider::mud_divider::MudDivider;
pub use layout::split_panel::mud_split_panel::MudSplitPanel;
pub use layout::tool_bar::mud_tool_bar::MudToolBar;

pub use data_display::chip::mud_chip::{MudChip};
pub use data_display::chip::mud_chipset::{MudChipSet};
pub use data_display::badge::mud_badge::MudBadge;
pub use data_display::avatar::mud_avatar::{MudAvatar};
pub use data_display::avatar::mud_avatar_group::{MudAvatarGroup, AvatarItem, AvatarContentInput, MudAvatarGroupCtx};
pub use data_display::icons::mud_icon::MudIcon;
pub use data_display::image::mud_image::MudImage;

// pub use inputs::check_box::mud_check_box::MudCheckBox;
// pub use inputs::hotkey::mud_hotkey::MudHotkey;
// pub use inputs::hotkey::mud_hotkey::HotkeyEventArgs;
// pub use inputs::radio::mud_radio_group::MudRadioGroup;
// pub use inputs::radio::mud_radio_group::MudRadio;
// pub use inputs::radio::mud_radio_group::RadioGroupUi;
// pub use inputs::rating::mud_rating::MudRating;
// pub use inputs::slider::mud_slider::MudSlider;
// pub use inputs::switch::mud_switch::MudSwitch;
// pub use inputs::text_field::mud_text_field::MudTextField;
pub use inputs::toggle::mud_toggle_group::{MudToggleGroup, MudToggleItem};


