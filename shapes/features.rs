//! Features of a rounded polygon: its edges and corners.
//!
//! Matching the corners of one polygon to the corners of another, so a morph
//! between them lines up like with like.

use std::collections::HashSet;

use super::points::{progress_in_range, DoubleMapper, Point};
use super::polygon_manager::Cubic;
use super::utils::DISTANCE_EPSILON;

/// A part of a polygon's outline: either a straight edge or a (rounded) corner.
///
/// # Examples
///
/// ```
/// use rounded_polygon_amane::shapes::polygon_manager::Cubic;
/// use rounded_polygon_amane::shapes::features::Feature;
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

/// The mapping used when there is nothing to match: every value maps to itself.
const IDENTITY_MAPPING: [(f32, f32); 2] = [(0.0, 0.0), (0.5, 0.5)];

/// A [`Feature`] together with how far along the outline it sits, in `0..1`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressableFeature {
    /// Position along the outline, in `0..1`.
    pub progress: f32,
    /// The feature at that position.
    pub feature: Feature,
}

impl ProgressableFeature {
    /// Creates a feature at the given outline `progress`.
    #[must_use]
    pub const fn new(progress: f32, feature: Feature) -> Self {
        Self { progress, feature }
    }
}

/// A candidate pairing of two corners and how far apart they are.
#[derive(Debug, Clone, Copy)]
struct DistanceVertex {
    distance: f32,
    f1: usize,
    f2: usize,
}

/// Collects accepted corner pairings, kept sorted by the first progress value.
#[derive(Debug, Default)]
struct MappingHelper {
    mapping: Vec<(f32, f32)>,
    used_f1: HashSet<usize>,
    used_f2: HashSet<usize>,
}

impl MappingHelper {
    /// Adds the pairing of feature `f1` (at progress `p1`) with `f2` (at `p2`),
    /// unless either feature is already used, it would sit too close to a
    /// neighbouring pairing, or it would cross one.
    fn add_mapping(&mut self, f1: usize, p1: f32, f2: usize, p2: f32) {
        if self.used_f1.contains(&f1) || self.used_f2.contains(&f2) {
            return;
        }

        let Err(insertion_index) = self
            .mapping
            .binary_search_by(|(progress, _)| progress.total_cmp(&p1))
        else {
            // Another pairing already sits at exactly this progress.
            return;
        };

        let n = self.mapping.len();
        if n >= 1 {
            let (before1, before2) = self.mapping[(insertion_index + n - 1) % n];
            let (after1, after2) = self.mapping[insertion_index % n];

            if progress_distance(p1, before1) < DISTANCE_EPSILON
                || progress_distance(p1, after1) < DISTANCE_EPSILON
                || progress_distance(p2, before2) < DISTANCE_EPSILON
                || progress_distance(p2, after2) < DISTANCE_EPSILON
            {
                return;
            }

            if n > 1 && !progress_in_range(p2, before2, after2) {
                return;
            }
        }

        self.mapping.insert(insertion_index, (p1, p2));
        self.used_f1.insert(f1);
        self.used_f2.insert(f2);
    }
}

/// Builds the mapper that carries progress along `features1` to the matching
/// progress along `features2`, by pairing up their closest corners.
///
/// Edges are ignored. Corners are only paired with corners of the same
/// convexity.
#[must_use]
pub fn feature_mapper(
    features1: &[ProgressableFeature],
    features2: &[ProgressableFeature],
) -> DoubleMapper {
    let corners = |features: &[ProgressableFeature]| -> Vec<ProgressableFeature> {
        features
            .iter()
            .filter(|f| matches!(f.feature, Feature::Corner { .. }))
            .cloned()
            .collect()
    };

    DoubleMapper::new(do_mapping(&corners(features1), &corners(features2)))
}

fn do_mapping(
    features1: &[ProgressableFeature],
    features2: &[ProgressableFeature],
) -> Vec<(f32, f32)> {
    let mut distance_vertices: Vec<DistanceVertex> = features1
        .iter()
        .enumerate()
        .flat_map(|(i1, f1)| {
            features2.iter().enumerate().filter_map(move |(i2, f2)| {
                feature_dist_squared(&f1.feature, &f2.feature).map(|distance| DistanceVertex {
                    distance,
                    f1: i1,
                    f2: i2,
                })
            })
        })
        .collect();

    distance_vertices.sort_by(|a, b| a.distance.total_cmp(&b.distance));

    match distance_vertices.as_slice() {
        [] => IDENTITY_MAPPING.to_vec(),
        [only] => {
            let p1 = features1[only.f1].progress;
            let p2 = features2[only.f2].progress;
            vec![(p1, p2), ((p1 + 0.5) % 1.0, (p2 + 0.5) % 1.0)]
        }
        _ => {
            let mut helper = MappingHelper::default();
            for vertex in &distance_vertices {
                helper.add_mapping(
                    vertex.f1,
                    features1[vertex.f1].progress,
                    vertex.f2,
                    features2[vertex.f2].progress,
                );
            }
            helper.mapping
        }
    }
}

/// Squared distance between two features, or `None` if they cannot be paired
/// because they are corners of opposite convexity.
fn feature_dist_squared(f1: &Feature, f2: &Feature) -> Option<f32> {
    let opposite_convexity = matches!(
        (f1, f2),
        (Feature::Corner { convex: a, .. }, Feature::Corner { convex: b, .. }) if a != b
    );
    if opposite_convexity {
        return None;
    }
    Some((representative_point(f1) - representative_point(f2)).distance_squared())
}

/// The midpoint between where a feature starts and where it ends.
///
/// A feature without curves has no position, so it sits at the origin.
fn representative_point(feature: &Feature) -> Point {
    let cubics = feature.cubics();
    let (Some(first), Some(last)) = (cubics.first(), cubics.last()) else {
        return Point::default();
    };
    Point::new(
        (first.anchor0_x() + last.anchor1_x()) / 2.0,
        (first.anchor0_y() + last.anchor1_y()) / 2.0,
    )
}

/// Distance between two progress values on the cycle, in `0..=0.5`.
fn progress_distance(p1: f32, p2: f32) -> f32 {
    let direct = (p1 - p2).abs();
    direct.min(1.0 - direct)
}

#[cfg(test)]
mod feature_tests {
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

#[cfg(test)]
mod feature_mapping_tests {
    use super::super::polygon_manager::Cubic;
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn near(result: Result<f32, super::super::points::MappingError>, expected: f32) -> bool {
        result.is_ok_and(|value| (value - expected).abs() < EPSILON)
    }

    /// A corner made of one zero-length curve at `(x, y)`.
    fn corner(progress: f32, x: f32, y: f32, convex: bool) -> ProgressableFeature {
        ProgressableFeature::new(progress, Feature::corner(vec![Cubic::empty(x, y)], convex))
    }

    fn edge(progress: f32) -> ProgressableFeature {
        ProgressableFeature::new(progress, Feature::edge(vec![Cubic::empty(0.0, 0.0)]))
    }

    #[test]
    fn progress_distance_of_equal_values_is_zero() {
        assert_eq!(progress_distance(0.3, 0.3), 0.0);
    }

    #[test]
    fn progress_distance_takes_the_short_way_around_the_cycle() {
        assert!((progress_distance(0.05, 0.95) - 0.1).abs() < EPSILON);
    }

    #[test]
    fn progress_distance_never_exceeds_a_half() {
        assert!((progress_distance(0.0, 0.5) - 0.5).abs() < EPSILON);
    }

    #[test]
    fn feature_mapper_without_features_is_the_identity() {
        assert!(near(feature_mapper(&[], &[]).map(0.3), 0.3));
    }

    #[test]
    fn feature_mapper_ignores_edges() {
        let mapper = feature_mapper(&[edge(0.1)], &[edge(0.4)]);

        assert!(near(mapper.map(0.3), 0.3));
    }

    #[test]
    fn feature_mapper_with_opposite_convexity_is_the_identity() {
        let mapper = feature_mapper(
            &[corner(0.1, 0.0, 0.0, true)],
            &[corner(0.4, 0.0, 0.0, false)],
        );

        assert!(near(mapper.map(0.3), 0.3));
    }

    #[test]
    fn feature_mapper_with_one_pair_maps_the_pair_onto_each_other() {
        let mapper = feature_mapper(
            &[corner(0.25, 0.0, 0.0, true)],
            &[corner(0.5, 1.0, 1.0, true)],
        );

        assert!(near(mapper.map(0.25), 0.5));
    }

    #[test]
    fn feature_mapper_with_one_pair_also_maps_the_opposite_side() {
        let mapper = feature_mapper(
            &[corner(0.25, 0.0, 0.0, true)],
            &[corner(0.5, 1.0, 1.0, true)],
        );

        assert!(near(mapper.map(0.5), 0.75));
    }

    #[test]
    fn feature_mapper_pairs_each_corner_with_its_nearest_corner() {
        let first = [corner(0.0, 0.0, 0.0, true), corner(0.5, 1.0, 1.0, true)];
        let second = [corner(0.1, 0.0, 0.1, true), corner(0.6, 1.0, 1.1, true)];

        let mapper = feature_mapper(&first, &second);

        assert!(near(mapper.map(0.5), 0.6));
    }

    #[test]
    fn feature_mapper_interpolates_between_paired_corners() {
        let first = [corner(0.0, 0.0, 0.0, true), corner(0.5, 1.0, 1.0, true)];
        let second = [corner(0.1, 0.0, 0.1, true), corner(0.6, 1.0, 1.1, true)];

        let mapper = feature_mapper(&first, &second);

        assert!(near(mapper.map(0.25), 0.35));
    }

    #[test]
    fn feature_mapper_ignores_edges_among_corners() {
        let first = [
            corner(0.0, 0.0, 0.0, true),
            edge(0.25),
            corner(0.5, 1.0, 1.0, true),
        ];
        let second = [corner(0.1, 0.0, 0.1, true), corner(0.6, 1.0, 1.1, true)];

        let mapper = feature_mapper(&first, &second);

        assert!(near(mapper.map(0.5), 0.6));
    }

    #[test]
    fn add_mapping_keeps_the_mapping_sorted_by_first_progress() {
        let mut helper = MappingHelper::default();

        helper.add_mapping(0, 0.5, 0, 0.5);
        helper.add_mapping(1, 0.25, 1, 0.25);

        assert_eq!(helper.mapping, [(0.25, 0.25), (0.5, 0.5)]);
    }

    #[test]
    fn add_mapping_skips_a_feature_that_is_already_used() {
        let mut helper = MappingHelper::default();

        helper.add_mapping(0, 0.2, 0, 0.2);
        helper.add_mapping(0, 0.6, 1, 0.6);

        assert_eq!(helper.mapping.len(), 1);
    }

    #[test]
    fn add_mapping_skips_a_pairing_too_close_to_a_neighbour() {
        let mut helper = MappingHelper::default();

        helper.add_mapping(0, 0.2, 0, 0.2);
        helper.add_mapping(1, 0.200_001, 1, 0.6);

        assert_eq!(helper.mapping.len(), 1);
    }

    #[test]
    fn add_mapping_skips_a_pairing_that_would_cross_existing_ones() {
        let mut helper = MappingHelper::default();

        helper.add_mapping(0, 0.0, 0, 0.0);
        helper.add_mapping(1, 0.5, 1, 0.5);
        // Between the two above on the first side, but outside them on the second.
        helper.add_mapping(2, 0.25, 2, 0.75);

        assert_eq!(helper.mapping.len(), 2);
    }

    #[test]
    fn add_mapping_accepts_a_pairing_between_existing_ones() {
        let mut helper = MappingHelper::default();

        helper.add_mapping(0, 0.0, 0, 0.0);
        helper.add_mapping(1, 0.5, 1, 0.5);
        helper.add_mapping(2, 0.25, 2, 0.25);

        assert_eq!(helper.mapping, [(0.0, 0.0), (0.25, 0.25), (0.5, 0.5)]);
    }
}
