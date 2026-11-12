//! Which utility-class system a widget resolves its classes against.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StyleFlavor {
    /// Bootstrap-style utilities (`bg-primary`, `p-3`, `border-dashed-2`, …).
    #[default]
    Bootstrap,
    /// Tailwind-style utilities (`bg-primary/90`, `hover:bg-primary-dark`, …).
    Tailwind,
}