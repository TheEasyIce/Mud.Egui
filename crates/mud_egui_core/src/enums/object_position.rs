// TODO: port to Rust / egui

// Source: Mud.Avalonia.Core/Enums/ObjectPosition.cs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectPosition {
    Center,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

