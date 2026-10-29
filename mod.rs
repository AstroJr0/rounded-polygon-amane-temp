//! Morphing rounded polygons, built for the amane shell.
//!
//! [`ShapeCanvas`] draws a [`RoundedPolygon`] in an amane view and morphs
//! between polygons when the one you hand it changes.
//!
//! # Examples
//!
//! ```rust,ignore
//! use amane::{App, LayerWindow};
//! use rounded_polygon_amane::{custom_polygon, Anchor, CornerRounding, Offset, PointNRound, ShapeCanvas};
//!
//! fn view() -> LayerWindow {
//!     let corner = PointNRound::new(Offset::new(1.0, 0.5), CornerRounding::from_radius(0.2));
//!
//!     LayerWindow::new().width(96.0).height(96.0).child(
//!         ShapeCanvas::new()
//!             .rounded_polygon(|| custom_polygon(&[corner], 4, Offset::new(0.5, 0.5), false))
//!             .color("#685496")
//!             .size(96.0, 96.0)
//!             .rotate_on_morph(true)
//!             .build(),
//!     )
//! }
//! ```

#![deny(missing_docs)]
// Drop-in module: most of the API is unused by any one shell.
#![allow(dead_code, unused_imports)]

pub mod geometry;
pub mod graphics;
pub mod presets;
pub mod shapes;

pub use geometry::offset::Offset;
pub use graphics::matrix::Matrix;
pub use shapes::corner_rounding::CornerRounding;
pub use shapes::morph::Morph;
pub use shapes::point::Point;
pub use shapes::rounded_polygon::RoundedPolygon;
pub use shapes::utils::{custom_polygon, do_repeat, PointNRound};

use std::collections::HashMap;
use std::panic::Location;
use std::sync::{LazyLock, Mutex, PoisonError};
use std::time::Instant;

use amane::{request_frame, Canvas, Cap, Color, Path, Rectangle, Shape};

use presets::easing;
use shapes::cubic::Cubic;

/// Radius of the dots drawn by [`ShapeCanvas::debug`], in pixels.
const DEBUG_DOT: f32 = 4.0;

/// Where the polygon sits inside a canvas larger than the polygon, on one axis.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Anchor {
    /// Top, or left.
    Start,
    /// Middle.
    #[default]
    Center,
    /// Bottom, or right.
    End,
}

impl Anchor {
    /// How far along the leftover space the polygon sits, in `0..=1`.
    const fn fraction(self) -> f32 {
        match self {
            Self::Start => 0.0,
            Self::Center => 0.5,
            Self::End => 1.0,
        }
    }
}

/// A rotation that eases from one angle to another.
#[derive(Debug, Clone, Copy)]
struct Spin {
    from: f32,
    to: f32,
    start: Instant,
}

impl Spin {
    fn is_running(&self, p: f32) -> bool {
        self.from != self.to && p < 1.0
    }

    fn angle(&self, eased: f32) -> f32 {
        self.from + (self.to - self.from) * eased
    }
}

/// What a canvas remembers between frames.
struct State {
    target: RoundedPolygon,
    /// `None` when there is nothing to morph from, or the shapes can't be matched.
    morph: Option<Morph>,
    morph_start: Instant,
    spin: Spin,
}

/// One frame of a canvas, ready to draw.
struct Frame {
    cubics: Vec<Cubic>,
    /// Polygon bounds as `[min_x, min_y, max_x, max_y]`.
    bounds: [f32; 4],
    angle: f32,
    animating: bool,
}

/// Canvas states by id. A view is rebuilt on every redraw, so the morph
/// in flight has to live outside it.
static STATES: LazyLock<Mutex<HashMap<String, State>>> = LazyLock::new(Mutex::default);

/// Draws a [`RoundedPolygon`] and morphs it when the polygon changes.
///
/// Build it, then call [`build`](Self::build) to get a widget. The polygon is
/// remembered between redraws by the place `new` is called from; give canvases
/// created in a loop, or shared by several windows, their own
/// [`id`](Self::id).
pub struct ShapeCanvas {
    id: String,
    polygon: Option<RoundedPolygon>,
    color: Color,
    border: Option<(f32, Color)>,
    width: f32,
    height: f32,
    anchor: (Anchor, Anchor),
    offset: (f32, f32),
    normalized: bool,
    debug: bool,
    morph_easing: [f32; 4],
    morph_duration: f32,
    rotate_on_morph: bool,
    rotation_on_morph: f32,
    easing_on_rotation: [f32; 4],
    view: bool,
}

impl ShapeCanvas {
    /// Starts a canvas: 100 by 100 pixels, filled `#685496`, no border,
    /// centered, 300 ms morphs eased with "out back", no rotation.
    #[must_use]
    #[track_caller]
    pub fn new() -> Self {
        let at = Location::caller();
        Self {
            id: format!("{}:{}:{}", at.file(), at.line(), at.column()),
            polygon: None,
            color: Color::from("#685496"),
            border: None,
            width: 100.0,
            height: 100.0,
            anchor: (Anchor::Center, Anchor::Center),
            offset: (0.0, 0.0),
            normalized: true,
            debug: false,
            morph_easing: easing::out_back().bezier(),
            morph_duration: 300.0,
            rotate_on_morph: false,
            rotation_on_morph: 360.0,
            easing_on_rotation: easing::out_cubic(),
            view: true,
        }
    }

    /// Names this canvas, so its morph state is kept apart from others
    /// created at the same place in the code.
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Sets the polygon to draw. When it differs from the previous one, the
    /// canvas morphs from the old shape to the new.
    #[must_use]
    pub fn rounded_polygon(mut self, polygon: impl FnOnce() -> RoundedPolygon) -> Self {
        self.polygon = Some(polygon());
        self
    }

    /// Sets the fill color. A hex string or a [`Color`].
    #[must_use]
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    /// Draws an outline of `width` pixels along the polygon's edge.
    #[must_use]
    pub fn border(mut self, width: f32, color: impl Into<Color>) -> Self {
        self.border = Some((width, color.into()));
        self
    }

    /// Sets the canvas size in pixels.
    #[must_use]
    pub const fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Places the polygon inside the canvas, horizontally then vertically.
    /// Only matters when the polygon doesn't fill the canvas.
    #[must_use]
    pub const fn anchor(mut self, horizontal: Anchor, vertical: Anchor) -> Self {
        self.anchor = (horizontal, vertical);
        self
    }

    /// Moves the polygon, in pixels, after anchoring.
    #[must_use]
    pub const fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset = (x, y);
        self
    }

    /// Whether the polygon spans `0..1` and should be scaled up to the
    /// canvas (the default). Turn off for polygons already in pixels.
    #[must_use]
    pub const fn normalized(mut self, normalized: bool) -> Self {
        self.normalized = normalized;
        self
    }

    /// Marks every anchor point of the outline with a red dot.
    #[must_use]
    pub const fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Sets the morph easing as cubic Bézier control points
    /// `[x1, y1, x2, y2]`, or a curve from [`presets::easing`]. Defaults to "out back".
    #[must_use]
    pub fn morph_easing(mut self, easing: impl Into<[f32; 4]>) -> Self {
        self.morph_easing = easing.into();
        self
    }

    /// Sets how long a morph takes, in milliseconds. Defaults to `300.0`.
    #[must_use]
    pub const fn morph_duration(mut self, milliseconds: f32) -> Self {
        self.morph_duration = milliseconds;
        self
    }

    /// Whether the canvas turns while it morphs. Defaults to `false`.
    #[must_use]
    pub const fn rotate_on_morph(mut self, rotate: bool) -> Self {
        self.rotate_on_morph = rotate;
        self
    }

    /// Sets how many degrees each morph adds to the rotation, clockwise.
    /// Turns add up. Defaults to `360.0`.
    #[must_use]
    pub const fn rotation_on_morph(mut self, degrees: f32) -> Self {
        self.rotation_on_morph = degrees;
        self
    }

    /// Sets the rotation easing as cubic Bézier control points
    /// `[x1, y1, x2, y2]`, or a curve from [`presets::easing`]. Defaults to "out cubic".
    #[must_use]
    pub fn easing_on_rotation(mut self, easing: impl Into<[f32; 4]>) -> Self {
        self.easing_on_rotation = easing.into();
        self
    }

    /// Shows or hides the polygon. A hidden canvas keeps its size.
    #[must_use]
    pub const fn view(mut self, show: bool) -> Self {
        self.view = show;
        self
    }

    /// Makes the widget. Call it on every redraw; it advances the morph and
    /// asks amane for another frame while one is running.
    #[must_use]
    pub fn build(mut self) -> Rectangle {
        let now = Instant::now();

        let frame = self.polygon.take().map(|polygon| {
            let mut states = STATES.lock().unwrap_or_else(PoisonError::into_inner);
            let state = states
                .entry(self.id.clone())
                .or_insert_with(|| State::new(polygon.clone(), now));
            if state.target.cubics() != polygon.cubics() {
                state.retarget(polygon, &self, now);
            }
            state.frame(&self, now)
        });

        let Some(frame) = frame else {
            return self.rectangle(&[], [0.0, 0.0, 1.0, 1.0], 0.0);
        };
        if frame.animating {
            request_frame();
        }
        let cubics = if self.view { frame.cubics } else { Vec::new() };
        self.rectangle(&cubics, frame.bounds, frame.angle)
    }

    fn rectangle(self, cubics: &[Cubic], bounds: [f32; 4], angle: f32) -> Rectangle {
        let [min_x, min_y, max_x, max_y] = bounds;
        let scale = if self.normalized {
            self.width.min(self.height)
        } else {
            1.0
        };
        let x0 = (self.width - (max_x - min_x) * scale) * self.anchor.0.fraction() - min_x * scale
            + self.offset.0;
        let y0 = (self.height - (max_y - min_y) * scale) * self.anchor.1.fraction() - min_y * scale
            + self.offset.1;
        let map = |x: f32, y: f32| (x * scale + x0, y * scale + y0);

        let mut body = outline(cubics, map).fill(self.color);
        if let Some((width, color)) = self.border.filter(|(width, _)| *width > 0.0) {
            body = body.stroke(width, color);
        }
        let dots = if self.debug {
            dots(cubics, map)
        } else {
            Path::new()
        };

        let canvas = Canvas::new()
            .width(self.width)
            .height(self.height)
            .shapes(amane::shapes![body, dots]);

        Rectangle::new()
            .width(self.width)
            .height(self.height)
            .rotate(angle)
            .child(canvas)
    }
}

impl Default for ShapeCanvas {
    #[track_caller]
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    fn new(target: RoundedPolygon, now: Instant) -> Self {
        Self {
            target,
            morph: None,
            morph_start: now,
            spin: Spin {
                from: 0.0,
                to: 0.0,
                start: now,
            },
        }
    }

    /// Starts morphing from the shape we were heading to, to `next`.
    fn retarget(&mut self, next: RoundedPolygon, cfg: &ShapeCanvas, now: Instant) {
        self.morph = Morph::new(&self.target, &next).ok();
        self.target = next;
        self.morph_start = now;

        if cfg.rotate_on_morph {
            let p = progress(self.spin.start, now, cfg.morph_duration);
            self.spin = Spin {
                from: self.spin.angle(ease(cfg.easing_on_rotation, p)),
                to: self.spin.to + cfg.rotation_on_morph,
                start: now,
            };
        }
    }

    fn frame(&self, cfg: &ShapeCanvas, now: Instant) -> Frame {
        let morph_p = progress(self.morph_start, now, cfg.morph_duration);
        let cubics = match &self.morph {
            Some(morph) => morph.as_cubics(ease(cfg.morph_easing, morph_p)),
            None => self.target.cubics().to_vec(),
        };

        let spin_p = progress(self.spin.start, now, cfg.morph_duration);
        let bounds = if cfg.normalized {
            [0.0, 0.0, 1.0, 1.0]
        } else {
            self.target.calculate_bounds(false)
        };

        Frame {
            cubics,
            bounds,
            angle: self.spin.angle(ease(cfg.easing_on_rotation, spin_p)),
            animating: (self.morph.is_some() && morph_p < 1.0) || self.spin.is_running(spin_p),
        }
    }
}

/// How far an animation that began at `start` has run, in `0..=1`.
fn progress(start: Instant, now: Instant, duration_ms: f32) -> f32 {
    if duration_ms <= 0.0 {
        return 1.0;
    }
    (now.duration_since(start).as_secs_f32() * 1000.0 / duration_ms).min(1.0)
}

/// Eases `x` in `0..=1` along the cubic Bézier `[x1, y1, x2, y2]`, like CSS
/// `cubic-bezier`. The result may leave `0..=1` when `y1` or `y2` does.
fn ease([x1, y1, x2, y2]: [f32; 4], x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let axis = |a: f32, b: f32, t: f32| {
        let u = 1.0 - t;
        3.0 * a * u * u * t + 3.0 * b * u * t * t + t * t * t
    };
    // With x1, x2 in 0..=1 the curve's x only grows, so bisect for the t at x.
    let (x1, x2) = (x1.clamp(0.0, 1.0), x2.clamp(0.0, 1.0));
    let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
    for _ in 0..24 {
        let mid = f32::midpoint(lo, hi);
        if axis(x1, x2, mid) < x {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    axis(y1, y2, f32::midpoint(lo, hi))
}

/// The closed path through `cubics`, with every point passed through `map`.
fn outline(cubics: &[Cubic], map: impl Fn(f32, f32) -> (f32, f32)) -> Path {
    let Some(first) = cubics.first() else {
        return Path::new();
    };
    let (x, y) = map(first.anchor0_x(), first.anchor0_y());

    cubics
        .iter()
        .fold(Path::new().move_to(x, y), |path, c| {
            let (c0x, c0y) = map(c.control0_x(), c.control0_y());
            let (c1x, c1y) = map(c.control1_x(), c.control1_y());
            let (a1x, a1y) = map(c.anchor1_x(), c.anchor1_y());
            path.cubic_to(c0x, c0y, c1x, c1y, a1x, a1y)
        })
        .close()
}

/// A round dot on every anchor point of `cubics`, as zero-length round-capped lines.
fn dots(cubics: &[Cubic], map: impl Fn(f32, f32) -> (f32, f32)) -> Path {
    let starts = cubics.first().map(|c| (c.anchor0_x(), c.anchor0_y()));
    let ends = cubics.iter().map(|c| (c.anchor1_x(), c.anchor1_y()));

    starts
        .into_iter()
        .chain(ends)
        .fold(Path::new(), |path, (x, y)| {
            let (x, y) = map(x, y);
            path.move_to(x, y).line_to(x, y)
        })
        .stroke(DEBUG_DOT, Color::RED)
        .cap(Cap::Round)
}
