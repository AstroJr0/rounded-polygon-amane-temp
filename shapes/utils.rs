//! Small geometry helpers shared by the shape modules, plus the point-list
//! building blocks used to define custom polygons.

use super::super::handlers::offset::Offset;
use super::corners::CornerRounding;
use super::points::Point;
use super::rounded_polygon::RoundedPolygon;

/// Tolerance used when comparing distances.
pub const DISTANCE_EPSILON: f32 = 1e-4;

/// Tolerance used when comparing angles, in radians.
pub const ANGLE_EPSILON: f32 = 1e-6;

/// Returns `true` if the corner at `current` turns clockwise when walking
/// `previous -> current -> next`.
#[must_use]
pub fn convex(previous: Point, current: Point, next: Point) -> bool {
    (current - previous).clockwise(next - current)
}

/// Linearly interpolates between `start` and `stop`.
///
/// A `fraction` of `0.0` yields `start` and `1.0` yields `stop`.
#[must_use]
pub fn interpolate(start: f32, stop: f32, fraction: f32) -> f32 {
    (1.0 - fraction) * start + fraction * stop
}

/// The unit vector pointing along `(x, y)`.
///
/// The zero vector has no direction, so its components are `NaN`.
#[must_use]
pub fn direction_vector(x: f32, y: f32) -> Point {
    Point::new(x, y).direction()
}

/// Length of the vector `(x, y)`.
#[must_use]
pub fn distance(x: f32, y: f32) -> f32 {
    x.hypot(y)
}

/// Squared length of the vector `(x, y)`.
#[must_use]
pub fn distance_squared(x: f32, y: f32) -> f32 {
    x * x + y * y
}

/// Converts polar coordinates around the origin to a [`Point`].
#[must_use]
pub fn radial_to_cartesian(radius: f32, angle_radians: f32) -> Point {
    radial_to_cartesian_around(radius, angle_radians, Point::default())
}

/// Converts polar coordinates around `center` to a [`Point`].
#[must_use]
pub fn radial_to_cartesian_around(radius: f32, angle_radians: f32, center: Point) -> Point {
    let (sin, cos) = angle_radians.sin_cos();
    Point::new(cos, sin) * radius + center
}

/// Clamps `value` to the range between `min` and `max`.
///
/// Unlike [`f32::clamp`], the bounds may be given in either order.
///
/// # Panics
///
/// Panics if either bound is `NaN`.
#[must_use]
pub fn coerce_in(value: f32, min: f32, max: f32) -> f32 {
    let (low, high) = if min <= max { (min, max) } else { (max, min) };
    value.clamp(low, high)
}

/// The remainder of `value / modulus`, always in `0..modulus` for a positive
/// `modulus`, even when `value` is negative.
#[must_use]
pub fn positive_modulo(value: f32, modulus: f32) -> f32 {
    ((value % modulus) + modulus) % modulus
}

/// A polygon vertex together with the rounding to apply to its corner.
///
/// # Examples
///
/// ```rust
/// use rounded_polygon_amane::handlers::offset::Offset;
/// use rounded_polygon_amane::shapes::corners::CornerRounding;
/// use rounded_polygon_amane::shapes::utils::PointNRound;
///
/// let rounded = PointNRound::new(Offset::new(1.0, 0.5), CornerRounding::from_radius(0.2));
/// let sharp = PointNRound::unrounded(Offset::new(0.0, 0.5));
///
/// assert_eq!(rounded.rounding.radius, 0.2);
/// assert_eq!(sharp.rounding, CornerRounding::UNROUNDED);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointNRound {
    /// Position of the vertex.
    pub offset: Offset,
    /// Rounding applied to the corner at this vertex.
    pub rounding: CornerRounding,
}

impl PointNRound {
    /// Creates a vertex at `offset` with the given `rounding`.
    #[must_use]
    pub const fn new(offset: Offset, rounding: CornerRounding) -> Self {
        Self { offset, rounding }
    }

    /// Creates a sharp (unrounded) vertex at `offset`.
    #[must_use]
    pub const fn unrounded(offset: Offset) -> Self {
        Self::new(offset, CornerRounding::UNROUNDED)
    }
}

/// Repeats `points` around `center` to build a symmetric vertex list.
///
/// Without `mirroring`, the whole list is copied `reps` times, each copy
/// rotated a further `360 / reps` degrees around `center`.
///
/// With `mirroring`, the list is laid out over `reps * 2` sectors. Every other
/// sector holds the points in reverse order, reflected about the ray through
/// the first point; the first point is not repeated in those reversed sectors.
///
/// # Examples
///
/// ```rust
/// use rounded_polygon_amane::handlers::offset::Offset;
/// use rounded_polygon_amane::shapes::utils::{do_repeat, PointNRound};
///
/// let points = [PointNRound::unrounded(Offset::new(1.0, 0.5))];
/// let repeated = do_repeat(&points, 4, Offset::new(0.5, 0.5), false);
///
/// assert_eq!(repeated.len(), 4);
/// ```
#[must_use]
pub fn do_repeat(
    points: &[PointNRound],
    reps: usize,
    center: Offset,
    mirroring: bool,
) -> Vec<PointNRound> {
    if mirroring {
        repeat_mirrored(points, reps, center)
    } else {
        repeat_rotated(points, reps, center)
    }
}

/// Builds a polygon whose corners are `points`, repeated as by [`do_repeat`].
///
/// Each point brings its own rounding. The polygon is centered on `center`.
///
/// # Examples
///
/// ```rust
/// use rounded_polygon_amane::handlers::offset::Offset;
/// use rounded_polygon_amane::shapes::corners::CornerRounding;
/// use rounded_polygon_amane::shapes::utils::{custom_polygon, PointNRound};
///
/// // A rounded triangle: one corner, repeated three times around the center.
/// let corner = PointNRound::new(Offset::new(0.5, 1.0), CornerRounding::from_radius(0.1));
/// let triangle = custom_polygon(&[corner], 3, Offset::new(0.5, 0.5), false);
///
/// assert_eq!(triangle.features().len(), 6);
/// ```
#[must_use]
pub fn custom_polygon(
    points: &[PointNRound],
    reps: usize,
    center: Offset,
    mirroring: bool,
) -> RoundedPolygon {
    let repeated = do_repeat(points, reps, center, mirroring);
    let vertices: Vec<f32> = repeated
        .iter()
        .flat_map(|p| [p.offset.x, p.offset.y])
        .collect();
    let per_vertex_rounding: Vec<CornerRounding> = repeated.iter().map(|p| p.rounding).collect();

    RoundedPolygon::from_vertices(
        &vertices,
        CornerRounding::UNROUNDED,
        Some(&per_vertex_rounding),
        Some(Point::new(center.x, center.y)),
    )
}

fn repeat_rotated(points: &[PointNRound], reps: usize, center: Offset) -> Vec<PointNRound> {
    let count = points.len();

    (0..count * reps)
        .map(|i| {
            let point = points[i % count];
            let angle = (i / count) as f32 * 360.0 / reps as f32;
            PointNRound::new(
                point.offset.rotate_degrees_around(angle, center),
                point.rounding,
            )
        })
        .collect()
}

fn repeat_mirrored(points: &[PointNRound], reps: usize, center: Offset) -> Vec<PointNRound> {
    let angles: Vec<f32> = points
        .iter()
        .map(|p| (p.offset - center).angle_degrees())
        .collect();
    let distances: Vec<f32> = points
        .iter()
        .map(|p| (p.offset - center).distance())
        .collect();
    let first_angle = angles.first().copied().unwrap_or_default();

    let sectors = reps * 2;
    let section_angle = 360.0 / sectors as f32;
    let mut result = Vec::with_capacity(points.len() * sectors);

    for sector in 0..sectors {
        let forward = sector % 2 == 0;
        for index in 0..points.len() {
            let i = if forward {
                index
            } else {
                points.len() - 1 - index
            };
            if i == 0 && !forward {
                continue;
            }

            let mirrored_angle = if forward {
                angles[i]
            } else {
                2.0 * first_angle - angles[i]
            };
            let angle = sector as f32 * section_angle + mirrored_angle;
            let (sin, cos) = angle.to_radians().sin_cos();
            let offset = Offset::new(center.x + distances[i] * cos, center.y + distances[i] * sin);
            result.push(PointNRound::new(offset, points[i].rounding));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn near(a: Offset, b: Offset) -> bool {
        (a.x - b.x).abs() < EPSILON && (a.y - b.y).abs() < EPSILON
    }

    fn square_points(rounding: CornerRounding) -> [PointNRound; 4] {
        [
            PointNRound::new(Offset::new(0.0, 0.0), rounding),
            PointNRound::new(Offset::new(1.0, 0.0), rounding),
            PointNRound::new(Offset::new(1.0, 1.0), rounding),
            PointNRound::new(Offset::new(0.0, 1.0), rounding),
        ]
    }

    #[test]
    fn custom_polygon_makes_a_corner_and_an_edge_per_point() {
        let square = custom_polygon(
            &square_points(CornerRounding::UNROUNDED),
            1,
            Offset::new(0.5, 0.5),
            false,
        );

        assert_eq!(square.features().len(), 8);
    }

    #[test]
    fn custom_polygon_is_centered_on_the_given_center() {
        let square = custom_polygon(
            &square_points(CornerRounding::UNROUNDED),
            1,
            Offset::new(0.5, 0.5),
            false,
        );

        assert_eq!(square.center(), Point::new(0.5, 0.5));
    }

    #[test]
    fn custom_polygon_uses_each_points_own_rounding() {
        let sharp = custom_polygon(
            &square_points(CornerRounding::UNROUNDED),
            1,
            Offset::new(0.5, 0.5),
            false,
        );
        let rounded = custom_polygon(
            &square_points(CornerRounding::from_radius(0.1)),
            1,
            Offset::new(0.5, 0.5),
            false,
        );

        assert!(rounded.cubics().len() > sharp.cubics().len());
    }

    #[test]
    fn custom_polygon_repeats_the_points_around_the_center() {
        let corner = PointNRound::unrounded(Offset::new(1.0, 0.5));

        let diamond = custom_polygon(&[corner], 4, Offset::new(0.5, 0.5), false);

        assert_eq!(diamond.features().len(), 8);
    }

    #[test]
    fn custom_polygon_of_a_repeated_corner_is_symmetric_about_its_center() {
        let corner = PointNRound::unrounded(Offset::new(1.0, 0.5));

        let diamond = custom_polygon(&[corner], 4, Offset::new(0.5, 0.5), false);
        let [min_x, min_y, max_x, max_y] = diamond.calculate_bounds(false);

        assert!(near(Offset::new(min_x, min_y), Offset::new(0.0, 0.0)));
        assert!(near(Offset::new(max_x, max_y), Offset::new(1.0, 1.0)));
    }

    #[test]
    fn custom_polygon_with_mirroring_lays_out_the_mirrored_points() {
        let points = [
            PointNRound::unrounded(Offset::new(1.0, 0.0)),
            PointNRound::unrounded(Offset::new(2.0, 2.0)),
        ];

        let polygon = custom_polygon(&points, 1, Offset::ZERO, true);

        // 2 points on the first sector plus 1 mirrored point.
        assert_eq!(polygon.features().len(), 6);
    }

    #[test]
    fn custom_polygon_of_no_points_is_a_single_point_at_the_center() {
        let polygon = custom_polygon(&[], 1, Offset::new(0.5, 0.5), false);

        assert_eq!(
            polygon.cubics(),
            [super::super::polygon_manager::Cubic::empty(0.5, 0.5)]
        );
    }

    #[test]
    fn convex_is_true_for_a_clockwise_turn() {
        let previous = Point::new(0.0, 0.0);
        let current = Point::new(1.0, 0.0);
        let next = Point::new(1.0, 1.0);

        assert!(convex(previous, current, next));
    }

    #[test]
    fn convex_is_false_for_a_counter_clockwise_turn() {
        let previous = Point::new(0.0, 0.0);
        let current = Point::new(1.0, 0.0);
        let next = Point::new(1.0, -1.0);

        assert!(!convex(previous, current, next));
    }

    #[test]
    fn convex_is_false_for_collinear_points() {
        let previous = Point::new(0.0, 0.0);
        let current = Point::new(1.0, 0.0);
        let next = Point::new(2.0, 0.0);

        assert!(!convex(previous, current, next));
    }

    #[test]
    fn interpolate_at_zero_returns_start() {
        assert_eq!(interpolate(2.0, 10.0, 0.0), 2.0);
    }

    #[test]
    fn interpolate_at_one_returns_stop() {
        assert_eq!(interpolate(2.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn interpolate_at_half_returns_midpoint() {
        assert_eq!(interpolate(2.0, 10.0, 0.5), 6.0);
    }

    #[test]
    fn direction_vector_of_three_four_is_a_unit_vector() {
        assert_eq!(direction_vector(3.0, 4.0), Point::new(0.6, 0.8));
    }

    #[test]
    fn distance_of_three_four_is_five() {
        assert_eq!(distance(3.0, 4.0), 5.0);
    }

    #[test]
    fn distance_squared_of_three_four_is_twenty_five() {
        assert_eq!(distance_squared(3.0, 4.0), 25.0);
    }

    #[test]
    fn radial_to_cartesian_at_zero_radians_points_along_the_x_axis() {
        let point = radial_to_cartesian(2.0, 0.0);

        assert_eq!(point, Point::new(2.0, 0.0));
    }

    #[test]
    fn radial_to_cartesian_at_half_pi_points_along_the_y_axis() {
        let point = radial_to_cartesian(2.0, std::f32::consts::FRAC_PI_2);

        assert!(point.x.abs() < EPSILON && (point.y - 2.0).abs() < EPSILON);
    }

    #[test]
    fn radial_to_cartesian_around_offsets_the_result_by_the_center() {
        let point = radial_to_cartesian_around(2.0, 0.0, Point::new(1.0, 3.0));

        assert_eq!(point, Point::new(3.0, 3.0));
    }

    #[test]
    fn coerce_in_keeps_a_value_inside_the_range() {
        assert_eq!(coerce_in(0.5, 0.0, 1.0), 0.5);
    }

    #[test]
    fn coerce_in_raises_a_value_below_the_range() {
        assert_eq!(coerce_in(-1.0, 0.0, 1.0), 0.0);
    }

    #[test]
    fn coerce_in_lowers_a_value_above_the_range() {
        assert_eq!(coerce_in(2.0, 0.0, 1.0), 1.0);
    }

    #[test]
    fn coerce_in_accepts_bounds_in_reverse_order() {
        assert_eq!(coerce_in(2.0, 1.0, 0.0), 1.0);
    }

    #[test]
    fn coerce_in_keeps_nan_values() {
        assert!(coerce_in(f32::NAN, 0.0, 1.0).is_nan());
    }

    #[test]
    fn positive_modulo_keeps_a_value_already_in_range() {
        assert_eq!(positive_modulo(0.25, 1.0), 0.25);
    }

    #[test]
    fn positive_modulo_wraps_a_negative_value_into_range() {
        assert_eq!(positive_modulo(-0.25, 1.0), 0.75);
    }

    #[test]
    fn positive_modulo_wraps_a_value_above_range() {
        assert_eq!(positive_modulo(2.5, 1.0), 0.5);
    }

    #[test]
    fn positive_modulo_of_an_exact_multiple_is_zero() {
        assert_eq!(positive_modulo(-2.0, 1.0), 0.0);
    }

    #[test]
    fn point_n_round_new_stores_offset_and_rounding() {
        let rounding = CornerRounding::new(0.2, 0.5);
        let point = PointNRound::new(Offset::new(1.0, 2.0), rounding);

        assert_eq!(
            (point.offset, point.rounding),
            (Offset::new(1.0, 2.0), rounding)
        );
    }

    #[test]
    fn point_n_round_unrounded_uses_no_rounding() {
        let point = PointNRound::unrounded(Offset::new(1.0, 2.0));

        assert_eq!(point.rounding, CornerRounding::UNROUNDED);
    }

    #[test]
    fn do_repeat_once_without_mirroring_returns_the_same_points() {
        let points = [
            PointNRound::unrounded(Offset::new(1.0, 0.5)),
            PointNRound::new(Offset::new(0.5, 1.0), CornerRounding::from_radius(0.1)),
        ];

        let repeated = do_repeat(&points, 1, Offset::new(0.5, 0.5), false);

        assert_eq!(repeated, points);
    }

    #[test]
    fn do_repeat_without_mirroring_yields_points_times_reps() {
        let points = [
            PointNRound::unrounded(Offset::new(1.0, 0.5)),
            PointNRound::unrounded(Offset::new(0.5, 1.0)),
        ];

        let repeated = do_repeat(&points, 3, Offset::new(0.5, 0.5), false);

        assert_eq!(repeated.len(), 6);
    }

    #[test]
    fn do_repeat_without_mirroring_rotates_each_copy_around_the_center() {
        let points = [PointNRound::unrounded(Offset::new(1.0, 0.5))];

        let repeated = do_repeat(&points, 4, Offset::new(0.5, 0.5), false);

        assert!(near(repeated[1].offset, Offset::new(0.5, 1.0)));
    }

    #[test]
    fn do_repeat_without_mirroring_rotates_a_half_turn_for_two_reps() {
        let points = [PointNRound::unrounded(Offset::new(1.0, 0.5))];

        let repeated = do_repeat(&points, 2, Offset::new(0.5, 0.5), false);

        assert!(near(repeated[1].offset, Offset::new(0.0, 0.5)));
    }

    #[test]
    fn do_repeat_without_mirroring_keeps_each_points_rounding() {
        let rounding = CornerRounding::new(0.3, 0.4);
        let points = [PointNRound::new(Offset::new(1.0, 0.5), rounding)];

        let repeated = do_repeat(&points, 3, Offset::new(0.5, 0.5), false);

        assert!(repeated.iter().all(|p| p.rounding == rounding));
    }

    #[test]
    fn do_repeat_with_zero_reps_returns_nothing() {
        let points = [PointNRound::unrounded(Offset::new(1.0, 0.5))];

        assert!(do_repeat(&points, 0, Offset::ZERO, false).is_empty());
    }

    #[test]
    fn do_repeat_with_zero_reps_and_mirroring_returns_nothing() {
        let points = [PointNRound::unrounded(Offset::new(1.0, 0.5))];

        assert!(do_repeat(&points, 0, Offset::ZERO, true).is_empty());
    }

    #[test]
    fn do_repeat_of_no_points_returns_nothing() {
        assert!(do_repeat(&[], 3, Offset::ZERO, false).is_empty());
    }

    #[test]
    fn do_repeat_of_no_points_with_mirroring_returns_nothing() {
        assert!(do_repeat(&[], 3, Offset::ZERO, true).is_empty());
    }

    #[test]
    fn do_repeat_with_mirroring_skips_the_first_point_on_reversed_sectors() {
        let points = [
            PointNRound::unrounded(Offset::new(1.0, 0.0)),
            PointNRound::unrounded(Offset::new(2.0, 2.0)),
            PointNRound::unrounded(Offset::new(0.0, 3.0)),
        ];

        let repeated = do_repeat(&points, 2, Offset::ZERO, true);

        // 4 sectors x 3 points, minus the first point in each of the 2 reversed sectors
        assert_eq!(repeated.len(), 10);
    }

    #[test]
    fn do_repeat_with_mirroring_keeps_the_original_points_in_the_first_sector() {
        let points = [
            PointNRound::unrounded(Offset::new(1.0, 0.0)),
            PointNRound::unrounded(Offset::new(2.0, 2.0)),
        ];

        let repeated = do_repeat(&points, 1, Offset::ZERO, true);

        assert!(near(repeated[0].offset, points[0].offset));
        assert!(near(repeated[1].offset, points[1].offset));
    }

    #[test]
    fn do_repeat_with_mirroring_reflects_points_across_the_y_axis_for_one_rep() {
        let points = [
            PointNRound::unrounded(Offset::new(1.0, 0.0)),
            PointNRound::unrounded(Offset::new(2.0, 2.0)),
        ];

        let repeated = do_repeat(&points, 1, Offset::ZERO, true);

        assert!(near(repeated[2].offset, Offset::new(-2.0, 2.0)));
    }

    #[test]
    fn do_repeat_with_mirroring_keeps_the_rounding_of_the_source_point() {
        let rounding = CornerRounding::from_radius(0.3);
        let points = [
            PointNRound::unrounded(Offset::new(1.0, 0.0)),
            PointNRound::new(Offset::new(2.0, 2.0), rounding),
        ];

        let repeated = do_repeat(&points, 1, Offset::ZERO, true);

        assert_eq!(repeated[2].rounding, rounding);
    }
}
