// TODO: port to Rust / egui

// Source: Mud.Avalonia.Core/Enums/Origin.cs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Origin {
    TopLeft = 0,
    TopCenter = 1,
    TopRight = 2,
    CenterLeft = 3,
    Center = 4,
    CenterRight = 5,
    BottomLeft = 6,
    BottomCenter = 7,
    BottomRight = 8,
}

