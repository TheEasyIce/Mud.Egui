// TODO: port to Rust / egui

// Source: Mud.Avalonia.Components/Feedback/Snackbar/Snackbar.cs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SnackbarDuplicatesBehavior {
    Allow,
    Prevent,
    Overwrite,
}

