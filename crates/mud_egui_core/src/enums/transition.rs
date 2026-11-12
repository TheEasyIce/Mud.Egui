// TODO: port to Rust / egui

// Source: Mud.Avalonia.Core/Enums/Transition.cs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Transition {
    None,
    Fade,
    Slide,
    Custom,
}

