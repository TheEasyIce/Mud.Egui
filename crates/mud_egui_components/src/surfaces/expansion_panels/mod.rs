//! Accordion container and panel. Ports MudBlazor's `MudExpansionPanels` and
//! `MudExpansionPanel`.

mod mud_expansion_panel;
mod mud_expansion_panels;

pub use mud_expansion_panel::{MudExpansionPanel, PanelResponse};
pub use mud_expansion_panels::{
    collapse_all, collapse_all_except, expand_all, ExpansionPanelsUi, MudExpansionPanels,
    PanelSetResponse,
};