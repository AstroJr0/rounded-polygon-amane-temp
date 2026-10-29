//! Easing curves, named like Qt Quick's `Easing` types.
//!
//! Each function gives the control points `[x1, y1, x2, y2]` of a cubic
//! Bézier, which is what [`ShapeCanvas`](super::super::ShapeCanvas) takes for
//! `morph_easing` and `easing_on_rotation`. The `back` curves return a
//! [`Back`], so you can set its `overshoot`; it converts to the same array.
//!
//! The curves are single-segment Bézier approximations of Qt's. They look
//! right, but they aren't numerically identical. The `out_in_*` curves are
//! the roughest, because Qt builds them from two segments. Elastic and bounce
//! curves wobble, which one Bézier can't do, so those are not here.
//!
//! ```rust,ignore
//! ShapeCanvas::new()
//!     .morph_easing(easing::out_back().overshoot(1.7))
//!     .easing_on_rotation(easing::out_cubic())
//! ```

/// Qt's default overshoot for the `back` curves.
const DEFAULT_OVERSHOOT: f32 = 1.70158;

/// Defines one function per curve, each returning its control points.
macro_rules! curves {
    ($($(#[$doc:meta])* $name:ident => [$x1:expr, $y1:expr, $x2:expr, $y2:expr];)*) => {$(
        $(#[$doc])*
        #[must_use]
        pub const fn $name() -> [f32; 4] {
            [$x1, $y1, $x2, $y2]
        }
    )*};
}

curves! {
    /// No easing: constant speed ofc.
    linear => [0.0, 0.0, 1.0, 1.0];
    // below are fixed approximations, they can not be done with
    // single arrays
    /// Starts slow, speeds up. Quadratic.
    in_quad => [0.11, 0.0, 0.5, 0.0];
    /// Starts fast, slows down. Quadratic.
    out_quad => [0.5, 1.0, 0.89, 1.0];
    /// Slow at both ends. Quadratic.
    in_out_quad => [0.45, 0.0, 0.55, 1.0];
    /// Fast at both ends, slow in the middle. Quadratic. An approximation.
    out_in_quad => [0.55, 1.0, 0.45, 0.0];

    /// Starts slow, speeds up. Cubic.
    in_cubic => [0.32, 0.0, 0.67, 0.0];
    /// Starts fast, slows down. Cubic.
    out_cubic => [0.33, 1.0, 0.68, 1.0];
    /// Slow at both ends. Cubic.
    in_out_cubic => [0.65, 0.0, 0.35, 1.0];
    /// Fast at both ends, slow in the middle. Cubic. An approximation.
    out_in_cubic => [0.33, 1.0, 0.67, 0.0];

    /// Starts slow, speeds up. Quartic.
    in_quart => [0.5, 0.0, 0.75, 0.0];
    /// Starts fast, slows down. Quartic.
    out_quart => [0.25, 1.0, 0.5, 1.0];
    /// Slow at both ends. Quartic.
    in_out_quart => [0.76, 0.0, 0.24, 1.0];
    /// Fast at both ends, slow in the middle. Quartic. An approximation.
    out_in_quart => [0.22, 1.0, 0.78, 0.0];

    /// Starts slow, speeds up. Quintic.
    in_quint => [0.64, 0.0, 0.78, 0.0];
    /// Starts fast, slows down. Quintic.
    out_quint => [0.22, 1.0, 0.36, 1.0];
    /// Slow at both ends. Quintic.
    in_out_quint => [0.83, 0.0, 0.17, 1.0];
    /// Fast at both ends, slow in the middle. Quintic. An approximation.
    out_in_quint => [0.14, 1.0, 0.86, 0.0];

    /// Starts slow, speeds up. Sine wave.
    in_sine => [0.12, 0.0, 0.39, 0.0];
    /// Starts fast, slows down. Sine wave.
    out_sine => [0.61, 1.0, 0.88, 1.0];
    /// Slow at both ends. Sine wave.
    in_out_sine => [0.37, 0.0, 0.63, 1.0];
    /// Fast at both ends, slow in the middle. Sine wave. An approximation.
    out_in_sine => [0.64, 1.0, 0.36, 0.0];

    /// Starts very slow, then shoots up. Exponential.
    in_expo => [0.7, 0.0, 0.84, 0.0];
    /// Shoots off, then settles very slowly. Exponential.
    out_expo => [0.16, 1.0, 0.3, 1.0];
    /// Very slow at both ends. Exponential.
    in_out_expo => [0.87, 0.0, 0.13, 1.0];
    /// Fast at both ends, slow in the middle. Exponential. An approximation.
    out_in_expo => [0.09, 1.0, 0.91, 0.0];

    /// Starts slow, speeds up along a circle's arc.
    in_circ => [0.55, 0.0, 1.0, 0.45];
    /// Starts fast, slows down along a circle's arc.
    out_circ => [0.0, 0.55, 0.45, 1.0];
    /// Slow at both ends, along a circle's arc.
    in_out_circ => [0.85, 0.0, 0.15, 1.0];
    /// Fast at both ends, slow in the middle. Along a circle's arc. An approximation.
    out_in_circ => [0.15, 1.0, 0.85, 0.0];
}

/// Where a [`Back`] curve overshoots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    In,
    Out,
    InOut,
    OutIn,
}

/// A `back` curve: it pulls back (or overshoots) before settling.
///
/// Made by [`in_back`], [`out_back`], [`in_out_back`] and [`out_in_back`]. Turns into
/// `[x1, y1, x2, y2]` with [`bezier`](Self::bezier) or `.into()`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Back {
    side: Side,
    overshoot: f32,
}

impl Back {
    const fn new(side: Side) -> Self {
        Self {
            side,
            overshoot: DEFAULT_OVERSHOOT,
        }
    }

    /// Sets how far the curve overshoots. Qt's default is `1.70158`, about a
    /// 10% overshoot. `0.0` gives a plain cubic ease.
    #[must_use]
    pub const fn overshoot(mut self, overshoot: f32) -> Self {
        self.overshoot = overshoot;
        self
    }

    /// The control points `[x1, y1, x2, y2]` of this curve.
    #[must_use]
    pub fn bezier(self) -> [f32; 4] {
        let k = self.overshoot / DEFAULT_OVERSHOOT;
        match self.side {
            Side::In => [0.36, 0.0, 0.66, -0.56 * k],
            Side::Out => [0.34, 1.0 + 0.56 * k, 0.64, 1.0],
            Side::InOut => [0.68, -0.6 * k, 0.32, 1.0 + 0.6 * k],
            Side::OutIn => [0.34 - 0.1 * k, 1.0 + 0.3 * k, 0.66 + 0.1 * k, -0.3 * k],
        }
    }
}

impl From<Back> for [f32; 4] {
    fn from(back: Back) -> Self {
        back.bezier()
    }
}

/// Pulls back first, then speeds up. Set the pull with `.overshoot(..)`.
#[must_use]
pub const fn in_back() -> Back {
    Back::new(Side::In)
}

/// Overshoots the end, then settles back. Set the overshoot with `.overshoot(..)`.
#[must_use]
pub const fn out_back() -> Back {
    Back::new(Side::Out)
}

/// Pulls back at the start and overshoots at the end. Set both with `.overshoot(..)`.
#[must_use]
pub const fn in_out_back() -> Back {
    Back::new(Side::InOut)
}

/// Overshoots, comes back to the middle, pulls back, then finishes. Set it
/// with `.overshoot(..)`. A rough approximation.
#[must_use]
pub const fn out_in_back() -> Back {
    Back::new(Side::OutIn)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXED: [[f32; 4]; 29] = [
        linear(),
        in_quad(),
        out_quad(),
        in_out_quad(),
        out_in_quad(),
        in_cubic(),
        out_cubic(),
        in_out_cubic(),
        out_in_cubic(),
        in_quart(),
        out_quart(),
        in_out_quart(),
        out_in_quart(),
        in_quint(),
        out_quint(),
        in_out_quint(),
        out_in_quint(),
        in_sine(),
        out_sine(),
        in_out_sine(),
        out_in_sine(),
        in_expo(),
        out_expo(),
        in_out_expo(),
        out_in_expo(),
        in_circ(),
        out_circ(),
        in_out_circ(),
        out_in_circ(),
    ];

    fn near(a: [f32; 4], b: [f32; 4]) -> bool {
        a.iter().zip(b).all(|(a, b)| (a - b).abs() < 1e-5)
    }

    #[test]
    fn every_x_is_inside_zero_to_one() {
        for curve in FIXED {
            assert!((0.0..=1.0).contains(&curve[0]) && (0.0..=1.0).contains(&curve[2]));
        }
    }

    #[test]
    fn every_y_stays_inside_zero_to_one() {
        for curve in FIXED {
            assert!((0.0..=1.0).contains(&curve[1]) && (0.0..=1.0).contains(&curve[3]));
        }
    }

    #[test]
    fn out_curves_start_fast_and_in_curves_start_slow() {
        assert!(out_cubic()[1] > in_cubic()[1]);
    }

    #[test]
    fn out_back_by_default_is_the_common_ease_out_back() {
        assert!(near(out_back().bezier(), [0.34, 1.56, 0.64, 1.0]));
    }

    #[test]
    fn in_back_by_default_is_the_common_ease_in_back() {
        assert!(near(in_back().bezier(), [0.36, 0.0, 0.66, -0.56]));
    }

    #[test]
    fn in_out_back_by_default_is_the_common_ease_in_out_back() {
        assert!(near(in_out_back().bezier(), [0.68, -0.6, 0.32, 1.6]));
    }

    #[test]
    fn a_bigger_overshoot_pushes_the_curve_further() {
        assert!(out_back().overshoot(3.0).bezier()[1] > out_back().bezier()[1]);
    }

    #[test]
    fn no_overshoot_stays_inside_the_unit_square() {
        assert!(near(
            out_back().overshoot(0.0).bezier(),
            [0.34, 1.0, 0.64, 1.0]
        ));
    }

    #[test]
    fn out_in_back_by_default_dips_below_zero_in_its_second_control_point() {
        assert!(near(out_in_back().bezier(), [0.24, 1.3, 0.76, -0.3]));
    }

    #[test]
    fn out_in_back_without_overshoot_is_an_out_in_cubic() {
        assert!(near(
            out_in_back().overshoot(0.0).bezier(),
            [0.34, 1.0, 0.66, 0.0]
        ));
    }

    #[test]
    fn out_in_curves_start_fast_and_end_fast() {
        let curve = out_in_cubic();

        assert!(curve[1] > curve[0] && curve[3] < curve[2]);
    }

    #[test]
    fn back_turns_into_an_array() {
        let curve: [f32; 4] = out_back().into();

        assert!(near(curve, out_back().bezier()));
    }
}
