/// MudBlazor typography variants (H1-H6, Body, Subtitle, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Typo {
    #[default]
    Default,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Subtitle1,
    Subtitle2,
    Body1,
    Body2,
    Button,
    Caption,
    Overline,
}

impl Typo {
    /// Variants that MudBlazor renders with `text-transform: uppercase`.
    pub const fn is_uppercase_by_default(self) -> bool {
        matches!(self, Typo::Button | Typo::Overline)
    }
}