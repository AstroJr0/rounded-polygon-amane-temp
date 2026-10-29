//! Polygons whose corners are rounded.

use std::f32::consts::PI;

use super::corners::{CornerRounding, RoundedCorner};
use super::features::Feature;
use super::points::Point;
use super::polygon_manager::Cubic;
use super::utils::{convex, distance, distance_squared, radial_to_cartesian};

/// A polygon with optionally rounded corners, stored as the curves of its outline.
///
/// # Examples
///
/// ```
/// use rounded_polygon_amane::shapes::corners::CornerRounding;
/// use rounded_polygon_amane::shapes::points::Point;
/// use rounded_polygon_amane::shapes::rounded_polygon::RoundedPolygon;
///
/// // A unit square centered on the origin, with rounded corners.
/// let square = RoundedPolygon::rectangle(
///     1.0,
///     1.0,
///     CornerRounding::from_radius(0.2),
///     None,
///     Point::default(),
/// );
///
/// assert_eq!(square.center(), Point::default());
/// assert!(!square.cubics().is_empty());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RoundedPolygon {
    features: Vec<Feature>,
    center: Point,
    cubics: Vec<Cubic>,
}

impl RoundedPolygon {
    /// Creates a polygon from its `features` and `center`.
    #[must_use]
    pub fn new(features: Vec<Feature>, center: Point) -> Self {
        let cubics = build_cubic_list(&features, center);
        Self {
            features,
            center,
            cubics,
        }
    }

    /// The edges and corners that make up the outline.
    #[must_use]
    pub fn features(&self) -> &[Feature] {
        &self.features
    }

    /// The center of the polygon.
    #[must_use]
    pub const fn center(&self) -> Point {
        self.center
    }

    /// X coordinate of the center.
    #[must_use]
    pub const fn center_x(&self) -> f32 {
        self.center.x
    }

    /// Y coordinate of the center.
    #[must_use]
    pub const fn center_y(&self) -> f32 {
        self.center.y
    }

    /// The curves of the outline, in order. They form a closed loop.
    #[must_use]
    pub fn cubics(&self) -> &[Cubic] {
        &self.cubics
    }

    /// Maps every point of the polygon, center included, through `f`, which
    /// receives each point's `x` and `y` and returns its replacement.
    #[must_use]
    pub fn transformed(&self, mut f: impl FnMut(f32, f32) -> Point) -> Self {
        let center = self.center.transformed(&mut f);
        let features = self
            .features
            .iter()
            .map(|x| x.transformed(&mut f))
            .collect();
        Self::new(features, center)
    }

    /// Scales and moves the polygon to fit a 1x1 square at the origin.
    ///
    /// The shape keeps its proportions and is centered along the shorter side.
    #[must_use]
    pub fn normalized(&self) -> Self {
        let [min_x, min_y, max_x, max_y] = self.calculate_bounds(true);
        let width = max_x - min_x;
        let height = max_y - min_y;
        let side = width.max(height);
        // Center the shape if the bounds are not a square.
        let offset_x = (side - width) / 2.0 - min_x;
        let offset_y = (side - height) / 2.0 - min_y;

        self.transformed(|x, y| Point::new((x + offset_x) / side, (y + offset_y) / side))
    }

    /// The bounds of the smallest square around the center that contains the
    /// whole polygon, as `[min_x, min_y, max_x, max_y]`.
    #[must_use]
    pub fn calculate_max_bounds(&self) -> [f32; 4] {
        let max_dist_squared = self
            .cubics
            .iter()
            .map(|cubic| {
                let anchor_distance = distance_squared(
                    cubic.anchor0_x() - self.center.x,
                    cubic.anchor0_y() - self.center.y,
                );
                let middle = cubic.point_on_curve(0.5);
                let middle_distance =
                    distance_squared(middle.x - self.center.x, middle.y - self.center.y);
                anchor_distance.max(middle_distance)
            })
            .fold(0.0, f32::max);

        let radius = max_dist_squared.sqrt();
        [
            self.center.x - radius,
            self.center.y - radius,
            self.center.x + radius,
            self.center.y + radius,
        ]
    }

    /// The bounding box of the polygon as `[min_x, min_y, max_x, max_y]`.
    ///
    /// With `approximate`, control points are counted as part of the shape,
    /// which is cheaper but can be larger than the outline.
    #[must_use]
    pub fn calculate_bounds(&self, approximate: bool) -> [f32; 4] {
        self.cubics
            .iter()
            .map(|cubic| cubic.calculate_bounds(approximate))
            .fold(
                [
                    f32::INFINITY,
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                    f32::NEG_INFINITY,
                ],
                |acc, bounds| {
                    [
                        acc[0].min(bounds[0]),
                        acc[1].min(bounds[1]),
                        acc[2].max(bounds[2]),
                        acc[3].max(bounds[3]),
                    ]
                },
            )
    }

    /// The average of the points in `vertices`, which holds `x, y` pairs.
    #[must_use]
    pub fn calculate_center(vertices: &[f32]) -> Point {
        let count = (vertices.len() / 2) as f32;
        let (pairs, _) = vertices.as_chunks::<2>();
        let (sum_x, sum_y) = pairs
            .iter()
            .fold((0.0, 0.0), |(x, y), pair| (x + pair[0], y + pair[1]));
        Point::new(sum_x / count, sum_y / count)
    }

    /// The `x, y` pairs of a regular polygon's corners, starting on the
    /// positive x axis and going round the center.
    #[must_use]
    pub fn vertices_from_num_verts(num_vertices: usize, radius: f32, center: Point) -> Vec<f32> {
        (0..num_vertices)
            .flat_map(|i| {
                let angle = PI / num_vertices as f32 * 2.0 * i as f32;
                let vertex = radial_to_cartesian(radius, angle) + center;
                [vertex.x, vertex.y]
            })
            .collect()
    }

    /// A regular polygon with `num_vertices` corners.
    ///
    /// `per_vertex_rounding` overrides `rounding` for the corners it covers.
    #[must_use]
    pub fn from_num_vertices(
        num_vertices: usize,
        radius: f32,
        center: Point,
        rounding: CornerRounding,
        per_vertex_rounding: Option<&[CornerRounding]>,
    ) -> Self {
        Self::from_vertices(
            &Self::vertices_from_num_verts(num_vertices, radius, center),
            rounding,
            per_vertex_rounding,
            Some(center),
        )
    }

    /// A polygon through the points in `vertices`, which holds `x, y` pairs.
    ///
    /// Every corner gets `rounding`, except those covered by
    /// `per_vertex_rounding`. The center is `center`, or the average of the
    /// vertices if that is `None`.
    #[must_use]
    pub fn from_vertices(
        vertices: &[f32],
        rounding: CornerRounding,
        per_vertex_rounding: Option<&[CornerRounding]>,
        center: Option<Point>,
    ) -> Self {
        let n = vertices.len() / 2;
        let vertex = |i: usize| Point::new(vertices[i * 2], vertices[i * 2 + 1]);

        let rounded_corners: Vec<RoundedCorner> = (0..n)
            .map(|i| {
                let vertex_rounding = per_vertex_rounding
                    .and_then(|all| all.get(i))
                    .copied()
                    .unwrap_or(rounding);
                RoundedCorner::new(
                    vertex((i + n - 1) % n),
                    vertex(i),
                    vertex((i + 1) % n),
                    vertex_rounding,
                )
            })
            .collect();

        // For each side, check there is room for the cuts its two corners
        // need, and if not split the room: rounding first, then smoothing.
        // Side `i` runs from corner `i` to corner `i + 1`. The pair holds how
        // much of the expected round cut, and how much of the expected
        // smoothing, can be used.
        let cut_adjusts: Vec<(f32, f32)> = (0..n)
            .map(|ix| {
                let next = (ix + 1) % n;
                let expected_round_cut = rounded_corners[ix].expected_round_cut()
                    + rounded_corners[next].expected_round_cut();
                let expected_cut =
                    rounded_corners[ix].expected_cut() + rounded_corners[next].expected_cut();
                let side_size =
                    distance(vertex(ix).x - vertex(next).x, vertex(ix).y - vertex(next).y);

                // Rounding of both corners is served before any smoothing.
                if expected_round_cut > side_size {
                    // Not enough room for full rounding.
                    (side_size / expected_round_cut, 0.0)
                } else if expected_cut > side_size {
                    // Full rounding fits, but not full smoothing.
                    (
                        1.0,
                        (side_size - expected_round_cut) / (expected_cut - expected_round_cut),
                    )
                } else {
                    // Room for rounding and smoothing.
                    (1.0, 1.0)
                }
            })
            .collect();

        // The curves of every corner. `allowed_cut(0)` is for the side from the
        // previous corner to this one, `allowed_cut(1)` for the side from this
        // corner to the next.
        let corners: Vec<Vec<Cubic>> = rounded_corners
            .iter()
            .enumerate()
            .map(|(i, corner)| {
                let allowed_cut = |delta: usize| {
                    let (round_cut_ratio, cut_ratio) = cut_adjusts[(i + n - 1 + delta) % n];
                    corner.expected_round_cut() * round_cut_ratio
                        + (corner.expected_cut() - corner.expected_round_cut()) * cut_ratio
                };
                corner.cubics(allowed_cut(0), allowed_cut(1))
            })
            .collect();

        let mut features = Vec::with_capacity(n * 2);
        for (i, corner) in corners.iter().enumerate() {
            let next_corner = &corners[(i + 1) % n];
            let is_convex = convex(vertex((i + n - 1) % n), vertex(i), vertex((i + 1) % n));
            features.push(Feature::corner(corner.clone(), is_convex));

            // A corner always has at least one curve; the edge joins its last
            // curve to the first curve of the next corner.
            let (Some(last), Some(next_first)) = (corner.last(), next_corner.first()) else {
                continue;
            };
            features.push(Feature::edge(vec![Cubic::straight_line(
                last.anchor1_x(),
                last.anchor1_y(),
                next_first.anchor0_x(),
                next_first.anchor0_y(),
            )]));
        }

        let center = center.unwrap_or_else(|| Self::calculate_center(vertices));
        Self::from_features(features, Some(center))
    }

    /// A polygon made of `features`.
    ///
    /// The center is `center`, or the average of where the features' curves
    /// start if that is `None`.
    #[must_use]
    pub fn from_features(features: Vec<Feature>, center: Option<Point>) -> Self {
        let center = center.unwrap_or_else(|| {
            let vertices: Vec<f32> = features
                .iter()
                .flat_map(|feature| feature.cubics())
                .flat_map(|cubic| [cubic.anchor0_x(), cubic.anchor0_y()])
                .collect();
            Self::calculate_center(&vertices)
        });
        Self::new(features, center)
    }

    /// An approximation of a circle by a polygon of `num_vertices` rounded corners.
    #[must_use]
    pub fn circle(num_vertices: usize, radius: f32, center: Point) -> Self {
        // Half the angle between two neighbouring vertices.
        let theta = PI / num_vertices as f32;
        // Radius of the underlying polygon that gives the circle its radius.
        let polygon_radius = radius / theta.cos();
        Self::from_num_vertices(
            num_vertices,
            polygon_radius,
            center,
            CornerRounding::from_radius(radius),
            None,
        )
    }

    /// A `width` by `height` rectangle around `center`.
    #[must_use]
    pub fn rectangle(
        width: f32,
        height: f32,
        rounding: CornerRounding,
        per_vertex_rounding: Option<&[CornerRounding]>,
        center: Point,
    ) -> Self {
        let left = center.x - width / 2.0;
        let top = center.y - height / 2.0;
        let right = center.x + width / 2.0;
        let bottom = center.y + height / 2.0;

        Self::from_vertices(
            &[right, bottom, left, bottom, left, top, right, top],
            rounding,
            per_vertex_rounding,
            Some(center),
        )
    }

    /// A star with `num_vertices_per_radius` points.
    ///
    /// Outer corners get `rounding` and inner corners get `inner_rounding`, or
    /// `rounding` if that is `None`. `per_vertex_rounding` overrides both.
    #[must_use]
    pub fn star(
        num_vertices_per_radius: usize,
        radius: f32,
        inner_radius: f32,
        rounding: CornerRounding,
        inner_rounding: Option<CornerRounding>,
        per_vertex_rounding: Option<&[CornerRounding]>,
        center: Point,
    ) -> Self {
        let alternating: Vec<CornerRounding>;
        let per_vertex = match (per_vertex_rounding, inner_rounding) {
            (Some(explicit), _) => Some(explicit),
            (None, Some(inner)) => {
                alternating = (0..num_vertices_per_radius)
                    .flat_map(|_| [rounding, inner])
                    .collect();
                Some(alternating.as_slice())
            }
            (None, None) => None,
        };

        Self::from_vertices(
            &Self::star_vertices_from_num_verts(
                num_vertices_per_radius,
                radius,
                inner_radius,
                center,
            ),
            rounding,
            per_vertex,
            Some(center),
        )
    }

    /// The `x, y` pairs of a star's corners, alternating outer and inner.
    #[must_use]
    pub fn star_vertices_from_num_verts(
        num_vertices_per_radius: usize,
        radius: f32,
        inner_radius: f32,
        center: Point,
    ) -> Vec<f32> {
        let step = PI / num_vertices_per_radius as f32;
        (0..num_vertices_per_radius)
            .flat_map(|i| {
                let outer = radial_to_cartesian(radius, step * 2.0 * i as f32) + center;
                let inner = radial_to_cartesian(inner_radius, step * (2 * i + 1) as f32) + center;
                [outer.x, outer.y, inner.x, inner.y]
            })
            .collect()
    }
}

/// The outline curves for `features`, as one closed loop.
fn build_cubic_list(features: &[Feature], center: Point) -> Vec<Cubic> {
    let mut result = Vec::new();

    // Tracking the first and last curve makes the loop end exactly where it
    // began. Anchors that are off by even a fraction of a pixel can cause
    // rendering artifacts.
    let mut first_cubic: Option<Cubic> = None;
    let mut last_cubic: Option<Cubic> = None;

    // If the first feature is a three-curve corner, its middle curve is split
    // in two so the loop can start and end halfway through that corner.
    let (split_start, split_end) = match features.first().map(Feature::cubics) {
        Some([before, middle, after]) => {
            let (start, end) = middle.split(0.5);
            (
                Some(vec![before.clone(), start]),
                Some(vec![end, after.clone()]),
            )
        }
        _ => (None, None),
    };

    // Going one past the last feature appends the first feature's initial half.
    for i in 0..=features.len() {
        let feature_cubics: &[Cubic] = if i == 0 && split_end.is_some() {
            split_end.as_deref().unwrap_or_default()
        } else if i == features.len() {
            match split_start.as_deref() {
                Some(cubics) => cubics,
                None => break,
            }
        } else {
            features[i].cubics()
        };

        for cubic in feature_cubics {
            // Zero-length curves add nothing and can cause rendering artifacts.
            if cubic.zero_length() {
                // Dropping several nearly-zero curves in a row can add up to
                // a visible gap, so the last curve always ends at the latest
                // anchor.
                if let Some(last) = last_cubic.as_mut() {
                    last.points[6] = cubic.anchor1_x();
                    last.points[7] = cubic.anchor1_y();
                }
            } else {
                if let Some(last) = last_cubic.take() {
                    result.push(last);
                }
                last_cubic = Some(cubic.clone());
                if first_cubic.is_none() {
                    first_cubic = Some(cubic.clone());
                }
            }
        }
    }

    match (last_cubic, first_cubic) {
        (Some(last), Some(first)) => result.push(Cubic::new([
            last.anchor0_x(),
            last.anchor0_y(),
            last.control0_x(),
            last.control0_y(),
            last.control1_x(),
            last.control1_y(),
            first.anchor0_x(),
            first.anchor0_y(),
        ])),
        // An empty or zero-size polygon.
        _ => result.push(Cubic::empty(center.x, center.y)),
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-3;

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn near_bounds(actual: [f32; 4], expected: [f32; 4]) -> bool {
        actual.iter().zip(expected).all(|(a, b)| near(*a, b))
    }

    fn unit_square(rounding: CornerRounding) -> RoundedPolygon {
        RoundedPolygon::rectangle(1.0, 1.0, rounding, None, Point::default())
    }

    fn is_closed_loop(cubics: &[Cubic]) -> bool {
        cubics.iter().enumerate().all(|(i, cubic)| {
            let next = &cubics[(i + 1) % cubics.len()];
            near(cubic.anchor1_x(), next.anchor0_x()) && near(cubic.anchor1_y(), next.anchor0_y())
        })
    }

    #[test]
    fn calculate_center_averages_the_vertices() {
        let center = RoundedPolygon::calculate_center(&[0.0, 0.0, 2.0, 0.0, 2.0, 2.0, 0.0, 2.0]);

        assert_eq!(center, Point::new(1.0, 1.0));
    }

    #[test]
    fn vertices_from_num_verts_starts_on_the_positive_x_axis() {
        let vertices = RoundedPolygon::vertices_from_num_verts(4, 1.0, Point::default());

        assert!(near(vertices[0], 1.0) && near(vertices[1], 0.0));
    }

    #[test]
    fn vertices_from_num_verts_goes_a_quarter_turn_per_vertex_for_four() {
        let vertices = RoundedPolygon::vertices_from_num_verts(4, 1.0, Point::default());

        assert!(near(vertices[2], 0.0) && near(vertices[3], 1.0));
    }

    #[test]
    fn vertices_from_num_verts_offsets_by_the_center() {
        let vertices = RoundedPolygon::vertices_from_num_verts(4, 1.0, Point::new(5.0, 6.0));

        assert!(near(vertices[0], 6.0) && near(vertices[1], 6.0));
    }

    #[test]
    fn star_vertices_alternate_outer_and_inner_radius() {
        let vertices = RoundedPolygon::star_vertices_from_num_verts(4, 1.0, 0.5, Point::default());

        let outer = distance(vertices[0], vertices[1]);
        let inner = distance(vertices[2], vertices[3]);

        assert!(near(outer, 1.0) && near(inner, 0.5));
    }

    #[test]
    fn from_vertices_makes_a_corner_and_an_edge_for_each_vertex() {
        let square = unit_square(CornerRounding::UNROUNDED);

        assert_eq!(square.features().len(), 8);
    }

    #[test]
    fn from_vertices_alternates_corners_and_edges() {
        let square = unit_square(CornerRounding::UNROUNDED);

        assert!(square
            .features()
            .iter()
            .enumerate()
            .all(|(i, f)| f.is_edge() == (i % 2 == 1)));
    }

    #[test]
    fn from_vertices_marks_the_corners_of_a_square_convex() {
        let square = unit_square(CornerRounding::UNROUNDED);

        assert!(square
            .features()
            .iter()
            .step_by(2)
            .all(Feature::is_convex_corner));
    }

    #[test]
    fn from_vertices_computes_the_center_when_none_is_given() {
        let polygon = RoundedPolygon::from_vertices(
            &[0.0, 0.0, 2.0, 0.0, 2.0, 2.0, 0.0, 2.0],
            CornerRounding::UNROUNDED,
            None,
            None,
        );

        assert_eq!(polygon.center(), Point::new(1.0, 1.0));
    }

    #[test]
    fn from_vertices_uses_the_given_center() {
        let polygon = RoundedPolygon::from_vertices(
            &[0.0, 0.0, 2.0, 0.0, 2.0, 2.0, 0.0, 2.0],
            CornerRounding::UNROUNDED,
            None,
            Some(Point::new(7.0, 8.0)),
        );

        assert_eq!(polygon.center(), Point::new(7.0, 8.0));
    }

    #[test]
    fn from_vertices_per_vertex_rounding_overrides_the_default() {
        let rounding = [CornerRounding::from_radius(0.2)];
        let plain = unit_square(CornerRounding::UNROUNDED);
        let mixed = RoundedPolygon::from_vertices(
            &[0.5, 0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5],
            CornerRounding::UNROUNDED,
            Some(&rounding),
            Some(Point::default()),
        );

        assert!(mixed.cubics().len() > plain.cubics().len());
    }

    #[test]
    fn from_vertices_shrinks_rounding_that_does_not_fit_on_a_side() {
        // A radius of 1 on a unit square cannot fit; the outline must still close.
        let square = unit_square(CornerRounding::from_radius(1.0));

        assert!(is_closed_loop(square.cubics()));
    }

    #[test]
    fn unrounded_square_has_one_curve_per_side() {
        assert_eq!(unit_square(CornerRounding::UNROUNDED).cubics().len(), 4);
    }

    #[test]
    fn unrounded_square_is_a_closed_loop() {
        assert!(is_closed_loop(
            unit_square(CornerRounding::UNROUNDED).cubics()
        ));
    }

    #[test]
    fn rounded_square_is_a_closed_loop() {
        assert!(is_closed_loop(
            unit_square(CornerRounding::from_radius(0.2)).cubics()
        ));
    }

    #[test]
    fn rounded_square_with_smoothing_is_a_closed_loop() {
        assert!(is_closed_loop(
            unit_square(CornerRounding::new(0.2, 1.0)).cubics()
        ));
    }

    #[test]
    fn rounded_square_has_more_curves_than_an_unrounded_one() {
        let rounded = unit_square(CornerRounding::from_radius(0.2));

        assert!(rounded.cubics().len() > 4);
    }

    #[test]
    fn rectangle_bounds_match_its_size() {
        let rectangle = RoundedPolygon::rectangle(
            2.0,
            1.0,
            CornerRounding::UNROUNDED,
            None,
            Point::new(10.0, 20.0),
        );

        assert!(near_bounds(
            rectangle.calculate_bounds(false),
            [9.0, 19.5, 11.0, 20.5]
        ));
    }

    #[test]
    fn rounding_a_square_keeps_its_exact_bounds() {
        let rounded = unit_square(CornerRounding::from_radius(0.2));

        assert!(near_bounds(
            rounded.calculate_bounds(false),
            [-0.5, -0.5, 0.5, 0.5]
        ));
    }

    #[test]
    fn approximate_bounds_contain_the_exact_bounds() {
        let rounded = unit_square(CornerRounding::from_radius(0.2));
        let exact = rounded.calculate_bounds(false);
        let approximate = rounded.calculate_bounds(true);

        assert!(approximate[0] <= exact[0] + EPSILON && approximate[2] >= exact[2] - EPSILON);
    }

    #[test]
    fn calculate_max_bounds_is_a_square_around_the_center() {
        let rectangle =
            RoundedPolygon::rectangle(2.0, 2.0, CornerRounding::UNROUNDED, None, Point::default());

        // The corners are sqrt(2) from the center.
        let radius = 2.0_f32.sqrt();

        assert!(near_bounds(
            rectangle.calculate_max_bounds(),
            [-radius, -radius, radius, radius]
        ));
    }

    #[test]
    fn circle_has_the_requested_radius() {
        let circle = RoundedPolygon::circle(8, 1.0, Point::default());

        assert!(near_bounds(
            circle.calculate_bounds(false),
            [-1.0, -1.0, 1.0, 1.0]
        ));
    }

    #[test]
    fn circle_is_centered_where_asked() {
        let circle = RoundedPolygon::circle(8, 1.0, Point::new(3.0, 4.0));

        assert_eq!(circle.center(), Point::new(3.0, 4.0));
    }

    #[test]
    fn circle_is_a_closed_loop() {
        assert!(is_closed_loop(
            RoundedPolygon::circle(8, 1.0, Point::default()).cubics()
        ));
    }

    #[test]
    fn from_num_vertices_makes_one_corner_per_vertex() {
        let triangle = RoundedPolygon::from_num_vertices(
            3,
            1.0,
            Point::default(),
            CornerRounding::UNROUNDED,
            None,
        );

        assert_eq!(triangle.features().len(), 6);
    }

    #[test]
    fn star_makes_a_corner_for_every_outer_and_inner_vertex() {
        let star = RoundedPolygon::star(
            5,
            1.0,
            0.5,
            CornerRounding::UNROUNDED,
            None,
            None,
            Point::default(),
        );

        assert_eq!(star.features().len(), 20);
    }

    #[test]
    fn star_has_concave_inner_corners() {
        let star = RoundedPolygon::star(
            5,
            1.0,
            0.5,
            CornerRounding::UNROUNDED,
            None,
            None,
            Point::default(),
        );

        assert!(star
            .features()
            .iter()
            .skip(2)
            .step_by(4)
            .all(Feature::is_concave_corner));
    }

    #[test]
    fn star_applies_the_inner_rounding_to_the_inner_corners() {
        let plain = RoundedPolygon::star(
            4,
            1.0,
            0.5,
            CornerRounding::UNROUNDED,
            None,
            None,
            Point::default(),
        );
        let rounded_inside = RoundedPolygon::star(
            4,
            1.0,
            0.5,
            CornerRounding::UNROUNDED,
            Some(CornerRounding::from_radius(0.1)),
            None,
            Point::default(),
        );

        assert!(rounded_inside.cubics().len() > plain.cubics().len());
    }

    #[test]
    fn star_per_vertex_rounding_takes_precedence_over_inner_rounding() {
        let all_sharp = vec![CornerRounding::UNROUNDED; 8];
        let star = RoundedPolygon::star(
            4,
            1.0,
            0.5,
            CornerRounding::UNROUNDED,
            Some(CornerRounding::from_radius(0.1)),
            Some(&all_sharp),
            Point::default(),
        );

        assert_eq!(star.cubics().len(), 8);
    }

    #[test]
    fn transformed_moves_the_center_too() {
        let square = unit_square(CornerRounding::UNROUNDED);

        let moved = square.transformed(|x, y| Point::new(x + 2.0, y + 3.0));

        assert_eq!(moved.center(), Point::new(2.0, 3.0));
    }

    #[test]
    fn transformed_moves_the_bounds() {
        let square = unit_square(CornerRounding::UNROUNDED);

        let moved = square.transformed(|x, y| Point::new(x + 2.0, y + 3.0));

        assert!(near_bounds(
            moved.calculate_bounds(false),
            [1.5, 2.5, 2.5, 3.5]
        ));
    }

    #[test]
    fn normalized_fits_the_shape_in_a_unit_square() {
        let wide = RoundedPolygon::rectangle(
            4.0,
            2.0,
            CornerRounding::UNROUNDED,
            None,
            Point::new(10.0, 10.0),
        );

        assert!(near_bounds(
            wide.normalized().calculate_bounds(false),
            [0.0, 0.25, 1.0, 0.75]
        ));
    }

    #[test]
    fn normalized_keeps_a_square_a_square() {
        let normalized = unit_square(CornerRounding::UNROUNDED).normalized();

        assert!(near_bounds(
            normalized.calculate_bounds(false),
            [0.0, 0.0, 1.0, 1.0]
        ));
    }

    #[test]
    fn empty_polygon_is_a_single_zero_length_curve_at_the_center() {
        let polygon = RoundedPolygon::new(vec![], Point::new(1.0, 2.0));

        assert_eq!(polygon.cubics(), [Cubic::empty(1.0, 2.0)]);
    }

    #[test]
    fn from_features_computes_the_center_when_none_is_given() {
        let features = vec![Feature::edge(vec![Cubic::straight_line(
            0.0, 0.0, 2.0, 0.0,
        )])];

        let polygon = RoundedPolygon::from_features(features, None);

        assert_eq!(polygon.center(), Point::new(0.0, 0.0));
    }

    #[test]
    fn zero_length_curves_move_the_previous_curves_end_anchor() {
        let features = vec![
            Feature::edge(vec![Cubic::straight_line(0.0, 0.0, 1.0, 0.0)]),
            Feature::edge(vec![Cubic::new([
                1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.000_05, 0.0,
            ])]),
            Feature::edge(vec![Cubic::straight_line(1.0, 0.0, 1.0, 1.0)]),
        ];

        let polygon = RoundedPolygon::new(features, Point::default());

        assert_eq!(polygon.cubics()[0].anchor1_x(), 1.000_05);
    }

    #[test]
    fn last_curve_ends_exactly_where_the_first_begins() {
        let features = vec![
            Feature::edge(vec![Cubic::straight_line(0.0, 0.0, 1.0, 0.0)]),
            Feature::edge(vec![Cubic::straight_line(1.0, 0.0, 1.0, 1.0)]),
            Feature::edge(vec![Cubic::straight_line(1.0, 1.0, 0.1, 0.1)]),
        ];

        let polygon = RoundedPolygon::new(features, Point::default());

        let last = &polygon.cubics()[polygon.cubics().len() - 1];
        assert_eq!((last.anchor1_x(), last.anchor1_y()), (0.0, 0.0));
    }

    #[test]
    fn first_three_curve_corner_is_split_so_the_loop_starts_mid_corner() {
        let corner = Feature::corner(
            vec![
                Cubic::straight_line(0.0, 1.0, 0.0, 0.5),
                Cubic::circular_arc(0.5, 0.5, 0.0, 0.5, 0.5, 0.0),
                Cubic::straight_line(0.5, 0.0, 1.0, 0.0),
            ],
            true,
        );

        let polygon = RoundedPolygon::new(vec![corner], Point::new(0.5, 0.5));

        // Start of loop is the middle of the arc, not the corner's first curve.
        let arc_midpoint = Cubic::circular_arc(0.5, 0.5, 0.0, 0.5, 0.5, 0.0).point_on_curve(0.5);
        let first = &polygon.cubics()[0];
        assert!(near(first.anchor0_x(), arc_midpoint.x) && near(first.anchor0_y(), arc_midpoint.y));
    }
}
