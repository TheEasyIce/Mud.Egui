# MudSplitPanel Component

A resizable split-view container for `egui`.

`MudSplitPanel` is the direct port of MudBlazor's `MudSplitPanel` (and Avalonia's `MudSplitPanel.cs`). It divides a container into two resizable panels separated by a draggable splitter. It is the perfect foundation for IDE layouts, file explorers, chat applications, and master-detail views.

---

## Quick Start

The split panel requires two closures: one for the left/top panel, and one for the right/bottom panel.

```rust
use mud_egui_components::MudSplitPanel;

MudSplitPanel::new()
    .id_salt("my_editor_layout") // IMPORTANT: See below!
    .show(
        ui,
        |ui| {
            // First Panel (Left or Top)
            ui.label("Sidebar");
        },
        |ui| {
            // Second Panel (Right or Bottom)
            ui.label("Main Content");
        },
    );