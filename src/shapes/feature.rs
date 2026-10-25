//! Features of a rounded polygon: its edges and corners.

use crate::shapes::cubic::Cubic;
use crate::shapes::point::Point;

/// A part of a polygon's outline: either a straight edge or a (rounded) corner.
///
/// # Examples
///
/// ```
/// use rounded_polygon_amane::shapes::cubic::Cubic;
/// use rounded_polygon_amane::shapes::feature::Feature;
///
/// let corner = Feature::corner(vec![Cubic::empty(0.0, 0.0)], true);
///
/// assert!(corner.is_convex_corner());
/// assert!(!corner.is_edge());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Feature {
    /// A straight run between two corners.
    Edge {
        /// The curves that make up the edge.
        cubics: Vec<Cubic>,
    },
    /// A corner, possibly rounded.
    Corner {
        /// The curves that make up the corner.
        cubics: Vec<Cubic>,
        /// Whether the corner points outwards (convex) or inwards (concave).
        convex: bool,
    },
}

impl Feature {
    /// Creates an edge from `cubics`.
    #[must_use]
    pub const fn edge(cubics: Vec<Cubic>) -> Self {
        Self::Edge { cubics }
    }

    /// Creates a corner from `cubics`, convex or concave.
    #[must_use]
    pub const fn corner(cubics: Vec<Cubic>, convex: bool) -> Self {
        Self::Corner { cubics, convex }
    }

    /// The curves that make up this feature.
    #[must_use]
    pub fn cubics(&self) -> &[Cubic] {
        match self {
            Self::Edge { cubics } | Self::Corner { cubics, .. } => cubics,
        }
    }

    /// Returns `true` for edges, which may be dropped without changing the
    /// shape's character.
    #[must_use]
    pub const fn is_ignorable_feature(&self) -> bool {
        matches!(self, Self::Edge { .. })
    }

    /// Returns `true` if this is an edge.
    #[must_use]
    pub const fn is_edge(&self) -> bool {
        matches!(self, Self::Edge { .. })
    }

    /// Returns `true` if this is a convex corner.
    #[must_use]
    pub const fn is_convex_corner(&self) -> bool {
        matches!(self, Self::Corner { convex: true, .. })
    }

    /// Returns `true` if this is a concave corner.
    #[must_use]
    pub const fn is_concave_corner(&self) -> bool {
        matches!(self, Self::Corner { convex: false, .. })
    }

    /// Maps every point of the feature through `f`, which receives each
    /// point's `x` and `y` and returns its replacement.
    #[must_use]
    pub fn transformed(&self, mut f: impl FnMut(f32, f32) -> Point) -> Self {
        let map = |cubics: &[Cubic], f: &mut dyn FnMut(f32, f32) -> Point| -> Vec<Cubic> {
            cubics.iter().map(|c| c.transformed(&mut *f)).collect()
        };
        match self {
            Self::Edge { cubics } => Self::Edge {
                cubics: map(cubics, &mut f),
            },
            Self::Corner { cubics, convex } => Self::Corner {
                cubics: map(cubics, &mut f),
                convex: *convex,
            },
        }
    }

    /// The same feature traversed in the opposite direction.
    ///
    /// Reversing a corner turns convex into concave and back.
    #[must_use]
    pub fn reversed(&self) -> Self {
        let reverse =
            |cubics: &[Cubic]| -> Vec<Cubic> { cubics.iter().map(Cubic::reverse).collect() };
        match self {
            Self::Edge { cubics } => Self::Edge {
                cubics: reverse(cubics),
            },
            Self::Corner { cubics, convex } => Self::Corner {
                cubics: reverse(cubics),
                convex: !convex,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line() -> Cubic {
        Cubic::straight_line(0.0, 0.0, 1.0, 0.0)
    }

    #[test]
    fn edge_is_an_edge() {
        assert!(Feature::edge(vec![line()]).is_edge());
    }

    #[test]
    fn edge_is_ignorable() {
        assert!(Feature::edge(vec![line()]).is_ignorable_feature());
    }

    #[test]
    fn edge_is_not_a_corner() {
        let edge = Feature::edge(vec![line()]);

        assert!(!edge.is_convex_corner() && !edge.is_concave_corner());
    }

    #[test]
    fn convex_corner_is_convex_and_not_concave() {
        let corner = Feature::corner(vec![line()], true);

        assert!(corner.is_convex_corner() && !corner.is_concave_corner());
    }

    #[test]
    fn concave_corner_is_concave_and_not_convex() {
        let corner = Feature::corner(vec![line()], false);

        assert!(corner.is_concave_corner() && !corner.is_convex_corner());
    }

    #[test]
    fn corner_is_neither_edge_nor_ignorable() {
        let corner = Feature::corner(vec![line()], true);

        assert!(!corner.is_edge() && !corner.is_ignorable_feature());
    }

    #[test]
    fn cubics_returns_the_curves_of_an_edge() {
        assert_eq!(Feature::edge(vec![line()]).cubics(), [line()]);
    }

    #[test]
    fn cubics_returns_the_curves_of_a_corner() {
        assert_eq!(
            Feature::corner(vec![line(), line()], true).cubics().len(),
            2
        );
    }

    #[test]
    fn transformed_maps_every_cubic_of_an_edge() {
        let edge = Feature::edge(vec![line()]).transformed(|x, y| Point::new(y, x));

        assert_eq!(edge.cubics()[0].anchor1_y(), 1.0);
    }

    #[test]
    fn transformed_keeps_a_corner_convexity() {
        let corner = Feature::corner(vec![line()], true).transformed(Point::new);

        assert!(corner.is_convex_corner());
    }

    #[test]
    fn transformed_maps_every_cubic_of_a_corner() {
        let corner =
            Feature::corner(vec![line(), line()], false).transformed(|x, y| Point::new(x + 1.0, y));

        assert!(corner.cubics().iter().all(|c| c.anchor0_x() == 1.0));
    }

    #[test]
    fn reversed_reverses_the_cubics_of_an_edge() {
        let edge = Feature::edge(vec![line()]).reversed();

        assert_eq!(edge.cubics()[0], line().reverse());
    }

    #[test]
    fn reversed_flips_a_convex_corner_to_concave() {
        assert!(Feature::corner(vec![line()], true)
            .reversed()
            .is_concave_corner());
    }

    #[test]
    fn reversed_flips_a_concave_corner_to_convex() {
        assert!(Feature::corner(vec![line()], false)
            .reversed()
            .is_convex_corner());
    }

    #[test]
    fn reversed_twice_returns_the_original() {
        let corner = Feature::corner(vec![line()], true);

        assert_eq!(corner.reversed().reversed(), corner);
    }
}

