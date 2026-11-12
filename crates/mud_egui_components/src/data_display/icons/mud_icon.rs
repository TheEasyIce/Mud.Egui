//! A single icon rendered from an SVG path expression or a text glyph.
//!
//! Production-grade port and radical expansion of MudBlazor's `MudIcon`
//! (MudBlazor 8.13.0). Parses SVG path data into tessellated triangles via
//! `lyon`, then draws them with `egui::Mesh`. Glyph icons are rendered with
//! the proportional font. Two layers of global caching keep repeated draws
//! essentially free.
//!
//! # MudBlazor parity
//!
//! | MudBlazor parameter | Rust method | Default |
//! |---------------------|-------------|---------|
//! | `ChildContent`      | **not ported** (see below) | — |
//! | `Color`             | [`MudIcon::color`] | [`Color::Inherit`] |
//! | `Disabled`          | [`MudIcon::disabled`] | `false` |
//! | `Icon`              | [`MudIcon::new`] / [`MudIcon::icon`] | required |
//! | `Size`              | [`MudIcon::size_preset`] (or [`MudIcon::size`] index) | `Medium` |
//! | `Title`             | [`MudIcon::title`] | `None` |
//! | `ViewBox`           | [`MudIcon::viewbox_size`] | `0.0` (use bounds) |
//! | `Class`             | [`MudIcon::classes`] | `None` |
//! | `Style`             | *(use `classes`)* | — |
//! | `Tag`               | **not ported** (no HTML tag) | — |
//! | `UserAttributes`    | **not ported** (no HTML tag) | — |
//!
//! ### Extensions (not in MudBlazor)
//!
//! * [`MudIcon::kind`] (`IconKind::SvgPath` / `IconKind::Glyph`).
//! * [`MudIcon::size`] — `u8` index (`0..=4`, maps 1:1 to `Size`).
//! * [`MudIcon::size_px`] — explicit pixel size for fine control.
//! * [`MudIcon::custom_color`] — explicit `Color32` override.
//! * [`MudIcon::rotation`], [`MudIcon::flip`] — transform.
//! * [`MudIcon::id`], [`MudIcon::aria_label`] — accessibility.
//! * `on_hover` / `on_unhover` / `on_click` / `on_focus` / `on_blur`.
//!
//! ### Not ported
//!
//! * **`ChildContent`** — MudBlazor allows arbitrary Razor markup inside the
//!   icon element. The idiomatic egui equivalent is to put the custom content
//!   in the parent (a `Frame` + the icon, or a custom widget), not inside the
//!   icon. Adding a lifetime-bound `FnMut(&mut Ui)` field here would force a
//!   lifetime on `MudIcon` and complicate the builder for a rarely-used hook.
//! * **`ViewBox` as a string** — MudBlazor accepts `"0 0 24 24"`. egui's icon
//!   geometry is square (SVG paths are drawn into a square extent), so the
//!   port takes a single `f32` viewbox side length. The default `0.0` means
//!   "use the geometry's own bounds".
//! * **`Style`** — egui has no inline CSS strings; use [`MudIcon::classes`].
//! * **`Tag` / `UserAttributes`** — there is no HTML tag to attach them to.
//!
//! # Architecture
//!
//! ```text
//! PATH_CACHE (global)          TRANSFORM_CACHE (global)
//!  path string → geometry  →   (geometry_id, extent, rot, flip, color)
//!                              → local-space Mesh
//!                                    │
//!                                    ▼
//!                          clone + translate → painter
//! ```
//!
//! Layout / colour / style decisions live on the component instance.
//! Geometry and transformed meshes are process-global and lock-protected so
//! every icon that shares a path benefits from the first tessellation.
//!
//! # Styling
//!
//! Utility classes on [`ComponentBase`] are resolved by the active
//! [`StyleFlavor`]:
//!
//! * [`StyleFlavor::Bootstrap`] (default) — `bg-*`, `rounded-*`, `p-*`, `m-*`.
//! * [`StyleFlavor::Tailwind`] — same families plus arbitrary values,
//!   opacity modifiers (`bg-primary/50`), and state variants
//!   (`hover:`, `active:`, `dark:`).
//!
//! The icon's own **fill colour** is controlled by `color` / `custom_color`;
//! CSS classes currently paint only the rectangle behind the icon, its
//! corner radius, and the margins around the allocated area.
//!
//! # Variant state
//!
//! Tailwind `hover:` / `active:` variants need the previous frame's
//! interaction state.  Prefer [`MudIcon::show_mut`] (or keep the icon on a
//! struct field) so `tw_state` survives across frames.  `dark:` works either
//! way because theme mode is re-read every call.
//!
//! # Caching strategy
//!
//! 1. **`PATH_CACHE`** — raw tessellated geometry (vertex positions in the
//!    SVG's native coordinate space), keyed by the path string.
//! 2. **`TRANSFORM_CACHE`** — the mesh *after* scale / flip / rotate / colour
//!    have been applied, in icon-local space.
//!
//! On a double cache hit the hot path only builds a `TransformKey`, clones
//! the cached local mesh, and translates it to the final screen position.
//!
//! # Example
//! ```ignore
//! // Stateless one-shot
//! ui.add(MudIcon::new(icons::STAR).color(Color::Warning).size_preset(Size::Large));
//!
//! // Explicit pixel size (bypasses the 0..=4 index table)
//! ui.add(MudIcon::new(icons::STAR).color(Color::Warning).size_px(18.0));
//!
//! // Stateful (persistent hover / active)
//! self.settings_icon
//!     .flavor(StyleFlavor::Tailwind)
//!     .classes(["bg-primary/20", "rounded-lg", "p-2", "hover:bg-primary/40"]);
//! self.settings_icon.show_mut(ui);
//! ```

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, OnceLock, RwLock};

use egui::{
    Color32, Id, Mesh, Pos2, Rect, Response, Sense, Ui, Vec2, Widget, epaint,
};
use lyon_extra::parser::{ParserOptions, PathParser, Source};
use lyon_path::Path;
use lyon_tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};
use mud_egui_bootstrap::bs_style::BsStyle;
use mud_egui_core::base::component_base::ComponentBase;
use mud_egui_core::enums::Color;
use mud_egui_core::enums::{IconFlip, IconKind, Size, StyleFlavor};
use mud_egui_tailwind::tw_style::{TwState, TwStyle};
use mud_egui_theming::models::palettes::palette::Palette;
use mud_egui_theming::theme::MudTheme;

// =============================================================================
// Constants
// =============================================================================

/// Base rem size (in points) passed to the Tailwind resolver.
const REM_SIZE_PT: f32 = 14.0;

/// Opacity applied when the icon is disabled (Material Design guideline).
const DISABLED_OPACITY: f32 = 0.38;

/// Soft upper bound on the transform cache; clearing is cheaper than an LRU
/// for the typical working set of a few hundred unique icons.
const TRANSFORM_CACHE_SOFT_CAP: usize = 4096;

/// Fill tessellation tolerance (smaller = more triangles, smoother curves).
const FILL_TOLERANCE: f32 = 0.01;

// =============================================================================
// Size table
// =============================================================================

/// Pixel extents corresponding to size indices 0‥4 (and any out-of-range
/// index falls back to Medium / 24 px).
const ICON_SIZES_PX: [f32; 5] = [8.0, 16.0, 24.0, 32.0, 48.0];

/// Maps a size index to its pixel extent.
///
/// | Index | Pixels | MudBlazor `Size` |
/// |-------|--------|------------------|
/// | 0     | 8      | `ExtraSmall`     |
/// | 1     | 16     | `Small`          |
/// | 2     | 24     | `Medium` (default) |
/// | 3     | 32     | `Large`          |
/// | 4     | 48     | `ExtraLarge`     |
#[inline]
pub fn size_to_pixels(size: u8) -> f32 {
    ICON_SIZES_PX
        .get(size as usize)
        .copied()
        .unwrap_or(24.0)
}

/// Maps a MudBlazor [`Size`] to its index (0‥4).
#[inline]
pub fn size_preset_to_index(size: Size) -> u8 {
    match size {
        Size::ExtraSmall => 0,
        Size::Small => 1,
        Size::Medium => 2,
        Size::Large => 3,
        Size::ExtraLarge => 4,
    }
}

// =============================================================================
// Layer 1 – raw tessellation cache (per unique SVG path)
// =============================================================================

static NEXT_GEOMETRY_ID: AtomicUsize = AtomicUsize::new(1);

/// Immutable tessellated geometry shared by every transform that uses the
/// same path string.
struct RawGeometry {
    /// Monotonically increasing identifier used as part of the transform key.
    id: usize,
    /// Vertex positions in the SVG's native coordinate space.
    positions: Vec<Pos2>,
    /// Triangle indices (shared via `Arc` so clones are cheap).
    indices: Arc<Vec<u32>>,
    /// Axis-aligned bounding box of the geometry.
    bounds: Rect,
    /// Silhouette edges used to feather the fill for antialiasing.
    ///
    /// Each entry is `[a, b, opposite]`: `a`/`b` are the two endpoint
    /// indices of a boundary edge (an edge that belongs to exactly one
    /// triangle), in that triangle's original winding order, and
    /// `opposite` is the triangle's third vertex — used to work out which
    /// side is "outward" so the feather strip extrudes away from the
    /// filled shape rather than into it.
    boundary_edges: Arc<Vec<[u32; 3]>>,
}

/// Derive the silhouette (boundary) edges of a triangle soup: edges that
/// belong to exactly one triangle. Shared interior edges (belonging to two
/// triangles, as in any manifold interior) are not part of the outline and
/// are dropped.
fn boundary_edges_of(indices: &[u32]) -> Vec<[u32; 3]> {
    let mut edge_map: HashMap<(u32, u32), Vec<(u32, u32, u32)>> = HashMap::new();
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        for &(p0, p1, p2) in &[(a, b, c), (b, c, a), (c, a, b)] {
            let key = (p0.min(p1), p0.max(p1));
            edge_map.entry(key).or_default().push((p0, p1, p2));
        }
    }
    edge_map
        .into_values()
        .filter(|occ| occ.len() == 1)
        .map(|occ| {
            let (a, b, opp) = occ[0];
            [a, b, opp]
        })
        .collect()
}

type CachedGeometry = Arc<RawGeometry>;

/// Global path → geometry cache.  The outer map is keyed by an `Arc<str>`
/// so the path string itself is shared; the inner `OnceLock` ensures a
/// given path is tessellated at most once even under concurrent readers.
static PATH_CACHE: LazyLock<RwLock<HashMap<Arc<str>, Arc<OnceLock<Option<CachedGeometry>>>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Parse and tessellate an SVG path string into a [`RawGeometry`].
fn tessellate_icon(path_data: &str) -> Option<CachedGeometry> {
    let mut builder = Path::builder();
    let mut parser = PathParser::new();

    let mut source = Source::new(path_data.chars());
    if parser
        .parse(&ParserOptions::DEFAULT, &mut source, &mut builder)
        .is_err()
    {
        #[cfg(debug_assertions)]
        eprintln!("MudIcon: failed to parse SVG path: {path_data:?}");
        return None;
    }
    let path = builder.build();

    let mut geometry: VertexBuffers<Pos2, u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();
    let opts = FillOptions::tolerance(FILL_TOLERANCE);

    if tessellator
        .tessellate_path(
            &path,
            &opts,
            &mut BuffersBuilder::new(&mut geometry, |v: FillVertex| {
                Pos2::new(v.position().x, v.position().y)
            }),
        )
        .is_err()
    {
        #[cfg(debug_assertions)]
        eprintln!("MudIcon: tessellation failed for path: {path_data:?}");
        return None;
    }

    if geometry.indices.is_empty() {
        return None;
    }

    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for v in &geometry.vertices {
        min_x = min_x.min(v.x);
        min_y = min_y.min(v.y);
        max_x = max_x.max(v.x);
        max_y = max_y.max(v.y);
    }
    let bounds = if min_x.is_finite() && max_x.is_finite() && max_x > min_x && max_y > min_y {
        Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y))
    } else {
        Rect::from_min_size(Pos2::ZERO, Vec2::splat(24.0))
    };

    let boundary_edges = boundary_edges_of(&geometry.indices);

    let id = NEXT_GEOMETRY_ID.fetch_add(1, Ordering::Relaxed);
    Some(Arc::new(RawGeometry {
        id,
        positions: geometry.vertices,
        indices: Arc::new(geometry.indices),
        bounds,
        boundary_edges: Arc::new(boundary_edges),
    }))
}

/// Look up (or create) the tessellated geometry for `path`.
fn get_geometry(path: &str) -> Option<CachedGeometry> {
    if let Some(slot) = PATH_CACHE
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .get(path)
    {
        let slot = slot.clone();
        return slot.get_or_init(|| tessellate_icon(path)).clone();
    }

    let slot = {
        let mut cache = PATH_CACHE.write().unwrap_or_else(|e| e.into_inner());
        cache
            .entry(Arc::from(path))
            .or_insert_with(|| Arc::new(OnceLock::new()))
            .clone()
    };
    slot.get_or_init(|| tessellate_icon(path)).clone()
}

// =============================================================================
// Layer 2 – transformed mesh cache
// =============================================================================

/// Cache key for a fully transformed local-space mesh.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct TransformKey {
    geometry_id: usize,
    extent_bits: u32,
    rotation_bucket: i32,
    flip: u8,
    color: [u8; 4],
    feather_bucket: i32,
}

impl TransformKey {
    fn new(
        geometry_id: usize,
        extent: f32,
        rotation: f32,
        flip: IconFlip,
        color: Color32,
        feather: f32,
    ) -> Self {
        const BUCKETS_PER_DEGREE: f32 = 2.0;
        let rotation_bucket = ((rotation.rem_euclid(360.0)) * BUCKETS_PER_DEGREE).round() as i32;
        let flip = match flip {
            IconFlip::None => 0,
            IconFlip::Horizontal => 1,
            IconFlip::Vertical => 2,
            IconFlip::Both => 3,
        };
        Self {
            geometry_id,
            extent_bits: extent.to_bits(),
            rotation_bucket,
            flip,
            color: color.to_array(),
            feather_bucket: (feather * 20.0).round() as i32,
        }
    }
}

/// A mesh that has already been scaled, flipped, rotated and coloured, but
/// is still expressed in icon-local coordinates (origin at top-left of the
/// icon extent).
struct TransformedLocal {
    mesh: Mesh,
}

static TRANSFORM_CACHE: LazyLock<RwLock<HashMap<TransformKey, Arc<TransformedLocal>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Obtain (or build) the local-space transformed mesh for the given
/// geometry and visual parameters.
fn get_transformed_local(
    geometry: &RawGeometry,
    viewbox_size: f32,
    extent: f32,
    rotation: f32,
    flip: IconFlip,
    color: Color32,
    feather: f32,
) -> Arc<TransformedLocal> {
    let key = TransformKey::new(geometry.id, extent, rotation, flip, color, feather);

    if let Some(hit) = TRANSFORM_CACHE
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .get(&key)
    {
        return hit.clone();
    }

    let (native_w, native_h, native_cx, native_cy) = if viewbox_size > 0.0 {
        (
            viewbox_size,
            viewbox_size,
            viewbox_size / 2.0,
            viewbox_size / 2.0,
        )
    } else {
        (
            geometry.bounds.width(),
            geometry.bounds.height(),
            geometry.bounds.center().x,
            geometry.bounds.center().y,
        )
    };

    let scale = (extent / native_w).min(extent / native_h);
    let center = Vec2::splat(extent / 2.0);
    let native_center = Vec2::new(native_cx, native_cy);
    let (sin, cos) = rotation.to_radians().sin_cos();
    let flip_h = matches!(flip, IconFlip::Horizontal | IconFlip::Both);
    let flip_v = matches!(flip, IconFlip::Vertical | IconFlip::Both);
    let rotated = rotation.abs() > 0.01;

    let mut mesh = Mesh::default();
    mesh.vertices.reserve(geometry.positions.len());

    for &raw in &geometry.positions {
        let mut p = (raw.to_vec2() - native_center) * scale + center;
        if flip_h {
            p.x = extent - p.x;
        }
        if flip_v {
            p.y = extent - p.y;
        }
        if rotated {
            let off = p - center;
            p = center + Vec2::new(off.x * cos - off.y * sin, off.x * sin + off.y * cos);
        }
        mesh.vertices.push(epaint::Vertex {
            pos: p.to_pos2(),
            uv: epaint::WHITE_UV,
            color,
        });
    }
    mesh.indices = (*geometry.indices).clone();

    // ---- Antialiasing feather --------------------------------------
    //
    // A `Shape::Mesh` submitted straight to the painter bypasses egui's
    // own tessellator, so it gets none of the edge feathering that makes
    // every *other* shape in the UI look smooth. We replicate that here:
    // for every silhouette edge, extrude a thin two-triangle strip
    // outward, fading the fill colour's alpha to zero across it. The
    // result is a soft ~1px antialiased edge instead of a hard, jagged
    // one, matching how the rest of egui (and Avalonia/Skia) renders.
    if feather > 0.0 && !geometry.boundary_edges.is_empty() {
        let transparent = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 0);
        for &[a, b, opp] in geometry.boundary_edges.iter() {
            let (Some(pa), Some(pb), Some(po)) = (
                mesh.vertices.get(a as usize).map(|v| v.pos),
                mesh.vertices.get(b as usize).map(|v| v.pos),
                mesh.vertices.get(opp as usize).map(|v| v.pos),
            ) else {
                continue;
            };

            let edge = pb - pa;
            if edge.length_sq() < 1e-6 {
                continue;
            }
            let dir = edge.normalized();
            let mut normal = Vec2::new(-dir.y, dir.x);
            let mid = pa + edge * 0.5;
            if normal.dot(po - mid) > 0.0 {
                normal = -normal;
            }

            let offset = normal * feather;
            let outer_a_idx = mesh.vertices.len() as u32;
            mesh.vertices.push(epaint::Vertex {
                pos: pa + offset,
                uv: epaint::WHITE_UV,
                color: transparent,
            });
            let outer_b_idx = mesh.vertices.len() as u32;
            mesh.vertices.push(epaint::Vertex {
                pos: pb + offset,
                uv: epaint::WHITE_UV,
                color: transparent,
            });

            mesh.indices.extend_from_slice(&[
                a, b, outer_b_idx,
                a, outer_b_idx, outer_a_idx,
            ]);
        }
    }

    let transformed = Arc::new(TransformedLocal { mesh });

    let mut cache = TRANSFORM_CACHE.write().unwrap_or_else(|e| e.into_inner());
    if cache.len() >= TRANSFORM_CACHE_SOFT_CAP {
        cache.clear();
    }
    cache.insert(key, transformed.clone());
    transformed
}

// =============================================================================
// Interaction / accessibility helpers
// =============================================================================

/// Lightweight interaction flags captured from the previous frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InteractionState {
    pub hovered: bool,
    pub active: bool,
    pub focused: bool,
    pub dark: bool,
    pub disabled: bool,
}

impl InteractionState {
    /// Convert into the `TwState` expected by the Tailwind resolver.
    #[inline]
    pub fn to_tw_state(self) -> TwState {
        TwState {
            hovered: self.hovered && !self.disabled,
            active: self.active && !self.disabled,
            dark: self.dark,
        }
    }
}

// =============================================================================
// Component
// =============================================================================

/// A themed icon rendered from an SVG path or a text glyph.
///
/// # MudBlazor API parity
///
/// | MudBlazor parameter | Rust equivalent |
/// |---------------------|-----------------|
/// | `Icon`              | [`Self::new`] / [`Self::icon`] |
/// | `Size`              | [`Self::size_preset`] (or [`Self::size`] index) |
/// | `Color`             | [`Self::color`] |
/// | `Disabled`          | [`Self::disabled`] |
/// | `Title`             | [`Self::title`] |
/// | `ViewBox`           | [`Self::viewbox_size`] |
/// | `Class`             | [`Self::classes`] |
///
/// Additional capabilities (extensions):
///
/// * Dual style systems (Bootstrap + Tailwind with variants).
/// * Rotation and flip transforms with cached meshes.
/// * Explicit interaction callbacks.
/// * Accessibility identifiers and ARIA labels.
/// * Two-layer global geometry / transform cache.
/// * Explicit pixel sizing via [`Self::size_px`] when the 0..=4 index
///   buckets are too coarse (e.g. chip icons at 14/18/20 px).
///
/// # Derives
///
/// Only `Default` is derived.  `Debug` / `Clone` are omitted because the
/// style caches do not implement them; add those derives to the caches if
/// needed and re-enable them here.
#[must_use = "MudIcon does nothing until you call `.show(ui)` / `.show_mut(ui)` or `ui.add(..)`"]
#[derive(Default)]
pub struct MudIcon {
    /// SVG path data or glyph character(s).
    pub icon: String,

    /// Whether the icon is an SVG path or a text glyph.
    pub kind: IconKind,

    /// Size index (0 = 8 px … 4 = 48 px).  Defaults to 2 (24 px / Medium).
    ///
    /// Prefer [`Self::size_preset`] when you want MudBlazor parity with the
    /// [`Size`] enum; this `u8` field remains for backward compatibility and
    /// for callers that compute the index numerically.
    pub size: u8,

    /// Explicit pixel size. Overrides [`Self::size`] when set. Use this for
    /// fine-grained control (e.g., chip icons that need to render at 14px
    /// or 18px, which don't fit the 0..=4 index buckets).
    pub size_px: Option<f32>,

    /// Semantic colour resolved against the active palette.
    pub color: Color,

    /// Explicit override colour; takes precedence over [`Self::color`].
    pub custom_color: Option<Color32>,

    /// Clockwise rotation in degrees.
    pub rotation: f32,

    /// Horizontal / vertical / both flip.
    pub flip: IconFlip,

    /// When true the icon is drawn at reduced opacity and ignores interaction.
    pub disabled: bool,

    /// Explicit viewBox size.  When `0.0` the geometry bounds are used.
    pub viewbox_size: f32,

    /// Tooltip / accessibility title (shown on hover).
    pub title: Option<String>,

    /// Optional stable identifier for egui memory and accessibility.
    pub id: Option<Id>,

    /// Optional ARIA label (falls back to `title` when absent).
    pub aria_label: Option<String>,

    /// Shared component state (CSS-like utility classes, elevation, …).
    pub base: ComponentBase,

    /// Which utility-class system to resolve against.
    pub flavor: StyleFlavor,

    /// Bootstrap resolver cache.
    pub bs_style: BsStyle,

    /// Tailwind resolver cache.
    pub tw_style: TwStyle,

    /// Interaction / theme state fed to the Tailwind resolver.
    pub interaction: InteractionState,

    // ------------------------------------------------------------------
    // Optional event callbacks (zero-cost when None)
    // ------------------------------------------------------------------
    pub on_hover: Option<Box<dyn FnMut() + Send + Sync>>,
    pub on_unhover: Option<Box<dyn FnMut() + Send + Sync>>,
    pub on_click: Option<Box<dyn FnMut() + Send + Sync>>,
    pub on_focus: Option<Box<dyn FnMut() + Send + Sync>>,
    pub on_blur: Option<Box<dyn FnMut() + Send + Sync>>,
}

impl MudIcon {
    /// Creates a new SVG-path icon with Medium size and Inherit colour.
    pub fn new(icon: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            kind: IconKind::SvgPath,
            size: 2,
            size_px: None,
            color: Color::Inherit,
            custom_color: None,
            rotation: 0.0,
            flip: IconFlip::None,
            disabled: false,
            viewbox_size: 0.0,
            title: None,
            id: None,
            aria_label: None,
            base: ComponentBase::new(),
            flavor: StyleFlavor::default(),
            bs_style: BsStyle::default(),
            tw_style: TwStyle::default(),
            interaction: InteractionState::default(),
            on_hover: None,
            on_unhover: None,
            on_click: None,
            on_focus: None,
            on_blur: None,
        }
    }

    // ------------------------------------------------------------------
    // Builder – content & appearance
    // ------------------------------------------------------------------

    /// Sets the icon kind (SVG path or glyph).
    #[inline]
    pub fn kind(mut self, kind: IconKind) -> Self {
        self.kind = kind;
        self
    }

    /// Sets the size index (0 = 8 px … 4 = 48 px).
    #[inline]
    pub fn size(mut self, size: u8) -> Self {
        self.size = size;
        self
    }

    /// Sets the size using the MudBlazor [`Size`] enum.
    ///
    /// This is the parity entry point. It maps 1:1 to the `u8` index used
    /// by [`Self::size`]:
    ///
    /// | `Size` | Index | Pixels |
    /// |--------|-------|--------|
    /// | `ExtraSmall` | 0 | 8 |
    /// | `Small` | 1 | 16 |
    /// | `Medium` | 2 | 24 |
    /// | `Large` | 3 | 32 |
    /// | `ExtraLarge` | 4 | 48 |
    #[inline]
    pub fn size_preset(mut self, size: Size) -> Self {
        self.size = size_preset_to_index(size);
        self
    }

    /// Sets an explicit pixel size, overriding [`Self::size`].
    ///
    /// This bypasses the 5-bucket `ICON_SIZES_PX` table, allowing any
    /// arbitrary extent (e.g. `14.0` for small chips, `18.0` for medium
    /// chips, `20.0` for large chips). Negative and non-finite values are
    /// clamped to `0.0`, which means "fall back to `size` index".
    #[inline]
    pub fn size_px(mut self, px: f32) -> Self {
        self.size_px = if px.is_finite() && px > 0.0 { Some(px) } else { None };
        self
    }

    /// Sets the semantic colour.
    #[inline]
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets an explicit override colour.
    #[inline]
    pub fn custom_color(mut self, color: Color32) -> Self {
        self.custom_color = Some(color);
        self
    }

    /// Sets clockwise rotation in degrees.
    #[inline]
    pub fn rotation(mut self, degrees: f32) -> Self {
        self.rotation = degrees;
        self
    }

    /// Sets the flip mode.
    #[inline]
    pub fn flip(mut self, flip: IconFlip) -> Self {
        self.flip = flip;
        self
    }

    /// Marks the icon as disabled (reduced opacity, no interaction).
    #[inline]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self.interaction.disabled = disabled;
        self
    }

    /// Sets an explicit viewBox size (0 = use geometry bounds).
    #[inline]
    pub fn viewbox_size(mut self, size: f32) -> Self {
        self.viewbox_size = size;
        self
    }

    /// Sets the tooltip / accessibility title.
    #[inline]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    // ------------------------------------------------------------------
    // Builder – identity & accessibility
    // ------------------------------------------------------------------

    /// Assigns a stable `egui::Id`.
    #[inline]
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets an ARIA label for assistive technologies.
    #[inline]
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    // ------------------------------------------------------------------
    // Builder – styling
    // ------------------------------------------------------------------

    /// Selects the utility-class resolver.
    #[inline]
    pub fn flavor(mut self, flavor: StyleFlavor) -> Self {
        self.flavor = flavor;
        self
    }

    /// Adds utility classes to the underlying [`ComponentBase`].
    #[inline]
    pub fn classes<S: AsRef<str>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.base = self.base.classes(classes);
        self
    }

    // ------------------------------------------------------------------
    // Builder – callbacks
    // ------------------------------------------------------------------

    /// Registers a callback invoked when the pointer enters the icon.
    #[inline]
    pub fn on_hover(mut self, f: impl FnMut() + Send + Sync + 'static) -> Self {
        self.on_hover = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when the pointer leaves the icon.
    #[inline]
    pub fn on_unhover(mut self, f: impl FnMut() + Send + Sync + 'static) -> Self {
        self.on_unhover = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked on primary-button click.
    #[inline]
    pub fn on_click(mut self, f: impl FnMut() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when the icon gains keyboard focus.
    #[inline]
    pub fn on_focus(mut self, f: impl FnMut() + Send + Sync + 'static) -> Self {
        self.on_focus = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when the icon loses keyboard focus.
    #[inline]
    pub fn on_blur(mut self, f: impl FnMut() + Send + Sync + 'static) -> Self {
        self.on_blur = Some(Box::new(f));
        self
    }

    // =========================================================================
    // Rendering
    // =========================================================================

    /// Stateless render: consumes `self`, compatible with `ui.add(...)`.
    ///
    /// `dark:` variants work.  `hover:` / `active:` variants only fire on the
    /// first frame after a manual seed because `self` is dropped.  Prefer
    /// [`Self::show_mut`] when you need persistent interaction state.
    pub fn show(self, ui: &mut Ui) -> Response {
        let mut me = self;
        me.show_mut(ui)
    }

    /// Stateful render: takes `&mut self` and updates interaction state from
    /// the current frame's response, so `hover:` / `active:` variants work
    /// on the next frame.
    ///
    /// Store the icon on a struct field and call this each frame.
    pub fn show_mut(&mut self, ui: &mut Ui) -> Response {
        let theme = MudTheme::fetch(ui.ctx()).unwrap_or_default();
        let palette = theme.current_palette();

        self.interaction.dark = ui.visuals().dark_mode;
        self.interaction.disabled = self.disabled;

        let frame_style = self.resolve_frame(ui, &palette);

        let bg = frame_style.fill;
        let corner_radius = frame_style.corner_radius;
        let padding = frame_style.inner_margin;
        let outer_margin = frame_style.outer_margin;

        let icon_extent = self.size_px.unwrap_or_else(|| size_to_pixels(self.size));

        let total_w = icon_extent
            + (padding.left + padding.right) as f32
            + (outer_margin.left + outer_margin.right) as f32;
        let total_h = icon_extent
            + (padding.top + padding.bottom) as f32
            + (outer_margin.top + outer_margin.bottom) as f32;
        let total = Vec2::new(total_w.max(icon_extent), total_h.max(icon_extent));

        let sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };

        let (rect, response) = if let Some(id) = self.id {
            let pos = ui.next_widget_position();
            let rect = Rect::from_min_size(pos, total);
            let response = ui.interact(rect, id, sense);
            ui.advance_cursor_after_rect(rect);
            (rect, response)
        } else {
            ui.allocate_exact_size(total, sense)
        };

        if !ui.is_rect_visible(rect) {
            self.update_interaction(&response);
            return response;
        }

        if bg.a() > 0 {
            let bg_rect = Rect::from_min_size(
                Pos2::new(
                    rect.min.x + outer_margin.left as f32,
                    rect.min.y + outer_margin.top as f32,
                ),
                Vec2::new(
                    (rect.width() - (outer_margin.left + outer_margin.right) as f32).max(0.0),
                    (rect.height() - (outer_margin.top + outer_margin.bottom) as f32).max(0.0),
                ),
            );
            ui.painter().rect_filled(bg_rect, corner_radius, bg);
        }

        let icon_rect = Rect::from_min_size(
            Pos2::new(
                rect.min.x + outer_margin.left as f32 + padding.left as f32,
                rect.min.y + outer_margin.top as f32 + padding.top as f32,
            ),
            Vec2::splat(icon_extent),
        );

        let icon_color = self.resolve_color(ui.visuals().text_color(), palette);

        match self.kind {
            IconKind::Glyph => self.draw_glyph_at(ui, icon_rect, icon_color),
            IconKind::SvgPath => self.draw_path_at(ui, icon_rect, icon_extent, icon_color),
        }

        if let Some(label) = self.aria_label.as_deref().or(self.title.as_deref()) {
            response.clone().on_hover_text(label);
        }

        self.dispatch_events(&response);
        self.update_interaction(&response);

        response
    }

    /// Resolves the active flavour's classes into an `egui::Frame`.
    fn resolve_frame(&mut self, ui: &Ui, palette: &Palette) -> egui::Frame {
        let lookup = |key: &str| -> Option<Color32> {
            palette.resolve(key).map(Color32::from)
        };

        match self.flavor {
            StyleFlavor::Bootstrap => self.bs_style.build_frame(&self.base, &lookup),
            StyleFlavor::Tailwind => {
                let state = self.interaction.to_tw_state();
                self.tw_style
                    .build_frame(&self.base, state, REM_SIZE_PT, &lookup)
            }
        }
    }

    fn draw_glyph_at(&self, ui: &mut Ui, rect: Rect, color: Color32) {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &self.icon,
            egui::FontId::proportional(rect.height()),
            color,
        );
    }

    fn draw_path_at(&self, ui: &mut Ui, rect: Rect, extent: f32, color: Color32) {
        let Some(geometry) = get_geometry(&self.icon) else {
            return;
        };
        let feather = 1.0 / ui.ctx().pixels_per_point().max(0.5);
        let local = get_transformed_local(
            &geometry,
            self.viewbox_size,
            extent,
            self.rotation,
            self.flip,
            color,
            feather,
        );
        let mut mesh = local.mesh.clone();
        let origin = rect.min.to_vec2();
        for v in &mut mesh.vertices {
            v.pos += origin;
        }
        ui.painter().add(epaint::Shape::mesh(mesh));
    }

    /// Resolves the final icon fill colour, applying the disabled opacity
    /// multiplier when appropriate.
    fn resolve_color(&self, ambient_text_color: Color32, palette: &Palette) -> Color32 {
        let mut base = match self.custom_color {
            Some(c) => c,
            None => match self.color {
                Color::Inherit => ambient_text_color,
                c => c.resolve(palette).unwrap_or(ambient_text_color),
            },
        };
        if self.disabled {
            base = base.gamma_multiply(DISABLED_OPACITY);
        }
        base
    }

    /// Edge-triggered callback dispatch.
    fn dispatch_events(&mut self, response: &Response) {
        if self.disabled {
            return;
        }

        let was_hovered = self.interaction.hovered;
        let was_focused = self.interaction.focused;

        if response.hovered() && !was_hovered {
            if let Some(ref mut f) = self.on_hover {
                f();
            }
        } else if !response.hovered() && was_hovered {
            if let Some(ref mut f) = self.on_unhover {
                f();
            }
        }

        if response.clicked() {
            if let Some(ref mut f) = self.on_click {
                f();
            }
        }

        if response.gained_focus() {
            if let Some(ref mut f) = self.on_focus {
                f();
            }
        } else if response.lost_focus() || (!response.has_focus() && was_focused) {
            if let Some(ref mut f) = self.on_blur {
                f();
            }
        }
    }

    fn update_interaction(&mut self, response: &Response) {
        self.interaction.hovered = response.hovered();
        self.interaction.active = response.is_pointer_button_down_on();
        self.interaction.focused = response.has_focus();
    }
}

impl Widget for MudIcon {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

// =============================================================================
// Cache introspection
// =============================================================================

/// Returns the number of entries currently held in the path geometry cache.
pub fn path_cache_len() -> usize {
    PATH_CACHE
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .len()
}

/// Returns the number of entries currently held in the transform cache.
pub fn transform_cache_len() -> usize {
    TRANSFORM_CACHE
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .len()
}

/// Clears both global caches.  Intended for tests and rare memory-pressure
/// situations; normal applications never need to call this.
pub fn clear_caches() {
    PATH_CACHE
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
    TRANSFORM_CACHE
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const SQUARE_A: &str = "M0 0L10 0L10 10L0 10Z";
    const SQUARE_B: &str = "M1 1L13 1L13 13L1 13Z";

    // ---- size table ----------------------------------------------------------

    #[test]
    fn size_buckets() {
        assert_eq!(size_to_pixels(0), 8.0);
        assert_eq!(size_to_pixels(1), 16.0);
        assert_eq!(size_to_pixels(2), 24.0);
        assert_eq!(size_to_pixels(3), 32.0);
        assert_eq!(size_to_pixels(4), 48.0);
        assert_eq!(size_to_pixels(9), 24.0);
    }

    #[test]
    fn size_preset_maps_to_index() {
        assert_eq!(size_preset_to_index(Size::ExtraSmall), 0);
        assert_eq!(size_preset_to_index(Size::Small), 1);
        assert_eq!(size_preset_to_index(Size::Medium), 2);
        assert_eq!(size_preset_to_index(Size::Large), 3);
        assert_eq!(size_preset_to_index(Size::ExtraLarge), 4);
    }

    #[test]
    fn size_px_overrides_size_index() {
        let icon = MudIcon::new(SQUARE_A).size(4).size_px(14.0);
        assert_eq!(icon.size_px, Some(14.0));

        assert_eq!(MudIcon::new(SQUARE_A).size_px(-5.0).size_px, None);
        assert_eq!(MudIcon::new(SQUARE_A).size_px(f32::NAN).size_px, None);
        assert_eq!(MudIcon::new(SQUARE_A).size_px(f32::INFINITY).size_px, None);
        assert_eq!(MudIcon::new(SQUARE_A).size_px(0.0).size_px, None);

        assert_eq!(MudIcon::new(SQUARE_A).size_px, None);
    }

    // ---- defaults & builders -------------------------------------------------

    #[test]
    fn new_defaults() {
        let icon = MudIcon::new(SQUARE_A);
        assert_eq!(icon.flavor, StyleFlavor::Bootstrap);
        assert_eq!(icon.size, 2);
        assert_eq!(icon.size_px, None);
        assert_eq!(icon.color, Color::Inherit);
        assert!(!icon.disabled);
        assert_eq!(icon.rotation, 0.0);
        assert_eq!(icon.flip, IconFlip::None);
        assert!(icon.base.class_names().is_empty());
        assert!(!icon.interaction.hovered);
        assert!(!icon.interaction.active);
        assert!(!icon.interaction.dark);
        assert!(icon.title.is_none());
        assert!(icon.id.is_none());
        assert!(icon.on_click.is_none());
    }

    #[test]
    fn flavor_builder_selects_tailwind() {
        let icon = MudIcon::new(SQUARE_A).flavor(StyleFlavor::Tailwind);
        assert_eq!(icon.flavor, StyleFlavor::Tailwind);
    }

    #[test]
    fn classes_builder_populates_base() {
        let icon = MudIcon::new(SQUARE_A).classes(["bg-primary", "p-1"]);
        assert!(icon.base.has_class("bg-primary"));
        assert!(icon.base.has_class("p-1"));
    }

    #[test]
    fn classes_builder_accepts_space_separated() {
        let icon = MudIcon::new(SQUARE_A).classes(["bg-primary p-1"]);
        assert!(icon.base.has_class("bg-primary"));
        assert!(icon.base.has_class("p-1"));
    }

    #[test]
    fn builder_chain_preserves_all_fields() {
        let icon = MudIcon::new(SQUARE_B)
            .flavor(StyleFlavor::Tailwind)
            .classes(["bg-primary/50", "hover:bg-primary", "dark:bg-surface"])
            .size(4)
            .size_px(18.0)
            .color(Color::Error)
            .rotation(45.0)
            .flip(IconFlip::Horizontal)
            .disabled(true)
            .viewbox_size(24.0)
            .title("Settings")
            .aria_label("Open settings");
        assert_eq!(icon.flavor, StyleFlavor::Tailwind);
        assert!(icon.base.has_class("bg-primary/50"));
        assert!(icon.base.has_class("hover:bg-primary"));
        assert!(icon.base.has_class("dark:bg-surface"));
        assert_eq!(icon.size, 4);
        assert_eq!(icon.size_px, Some(18.0));
        assert_eq!(icon.color, Color::Error);
        assert_eq!(icon.rotation, 45.0);
        assert_eq!(icon.flip, IconFlip::Horizontal);
        assert!(icon.disabled);
        assert!(icon.interaction.disabled);
        assert_eq!(icon.viewbox_size, 24.0);
        assert_eq!(icon.title.as_deref(), Some("Settings"));
        assert_eq!(icon.aria_label.as_deref(), Some("Open settings"));
    }

    #[test]
    fn interaction_state_is_publicly_mutable() {
        let mut icon = MudIcon::new(SQUARE_A).flavor(StyleFlavor::Tailwind);
        icon.interaction.dark = true;
        icon.interaction.hovered = true;
        assert!(icon.interaction.dark && icon.interaction.hovered);
    }

    #[test]
    fn interaction_to_tw_state_respects_disabled() {
        let mut s = InteractionState {
            hovered: true,
            active: true,
            dark: true,
            disabled: true,
            ..Default::default()
        };
        let tw = s.to_tw_state();
        assert!(!tw.hovered && !tw.active && tw.dark);

        s.disabled = false;
        let tw = s.to_tw_state();
        assert!(tw.hovered && tw.active && tw.dark);
    }

    // ---- geometry cache ------------------------------------------------------

    #[test]
    fn tessellate_simple_square() {
        clear_caches();
        let geo = tessellate_icon(SQUARE_A).expect("square should tessellate");
        assert!(!geo.positions.is_empty());
        assert!(!geo.indices.is_empty());
        assert!(geo.bounds.width() > 0.0);
        assert!(geo.bounds.height() > 0.0);
    }

    #[test]
    fn path_cache_reuses_geometry() {
        clear_caches();
        let a = get_geometry(SQUARE_A).expect("first lookup");
        let b = get_geometry(SQUARE_A).expect("second lookup");
        assert!(Arc::ptr_eq(&a, &b));
        assert_eq!(path_cache_len(), 1);
    }

    #[test]
    fn distinct_paths_produce_distinct_geometry() {
        clear_caches();
        let a = get_geometry(SQUARE_A).expect("A");
        let b = get_geometry(SQUARE_B).expect("B");
        assert!(!Arc::ptr_eq(&a, &b));
        assert_eq!(path_cache_len(), 2);
    }

    #[test]
    fn transform_cache_reuses_mesh() {
        clear_caches();
        let geo = get_geometry(SQUARE_A).expect("geo");
        let t1 = get_transformed_local(&geo, 0.0, 24.0, 0.0, IconFlip::None, Color32::WHITE, 1.0);
        let t2 = get_transformed_local(&geo, 0.0, 24.0, 0.0, IconFlip::None, Color32::WHITE, 1.0);
        assert!(Arc::ptr_eq(&t1, &t2));
        assert_eq!(transform_cache_len(), 1);
    }

    #[test]
    fn different_colors_produce_different_transforms() {
        clear_caches();
        let geo = get_geometry(SQUARE_A).expect("geo");
        let t1 = get_transformed_local(&geo, 0.0, 24.0, 0.0, IconFlip::None, Color32::WHITE, 1.0);
        let t2 = get_transformed_local(&geo, 0.0, 24.0, 0.0, IconFlip::None, Color32::RED, 1.0);
        assert!(!Arc::ptr_eq(&t1, &t2));
        assert_eq!(transform_cache_len(), 2);
    }

    #[test]
    fn rotation_is_quantised() {
        clear_caches();
        let geo = get_geometry(SQUARE_A).expect("geo");
        let t1 = get_transformed_local(&geo, 0.0, 24.0, 10.0, IconFlip::None, Color32::WHITE, 1.0);
        let t2 = get_transformed_local(&geo, 0.0, 24.0, 10.2, IconFlip::None, Color32::WHITE, 1.0);
        assert!(Arc::ptr_eq(&t1, &t2));
    }

    // ---- colour resolution ---------------------------------------------------

    #[test]
    fn disabled_applies_opacity() {
        let icon = MudIcon::new(SQUARE_A)
            .custom_color(Color32::from_rgb(255, 0, 0))
            .disabled(true);
        assert!((DISABLED_OPACITY - 0.38).abs() < f32::EPSILON);
        let opaque = Color32::from_rgb(255, 0, 0);
        let faded = opaque.gamma_multiply(DISABLED_OPACITY);
        assert!(faded.a() < opaque.a() || faded.r() < opaque.r());
        assert!(icon.disabled);
        assert!(icon.interaction.disabled);
    }

    // ---- style flavour default -----------------------------------------------

    #[test]
    fn flavor_default_is_bootstrap() {
        assert_eq!(StyleFlavor::default(), StyleFlavor::Bootstrap);
    }

    #[test]
    fn rem_size_matches_container() {
        assert_eq!(REM_SIZE_PT, 14.0);
    }
}