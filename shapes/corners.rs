//! Rounding configuration for a single polygon corner.
//!
//! The curves that round a single polygon corner.

use super::points::Point;
use super::polygon_manager::Cubic;
use super::utils::{direction_vector, DISTANCE_EPSILON};

/// How a single corner of a polygon is rounded.
///
/// # Examples
///
/// ```rust
/// use rounded_polygon_amane::shapes::corners::CornerRounding;
///
/// let rounding = CornerRounding::new(0.2, 0.5);
/// assert_eq!(rounding.radius, 0.2);
///
/// // Radius only, no smoothing.
/// assert_eq!(CornerRounding::from_radius(0.2).smoothing, 0.0);
/// ```

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CornerRounding {
    /// Radius of the circle that rounds the corner, relative to the size of the shape.
    pub radius: f32,
    /// How much the rounding blends into the adjacent edges: `0.0` is a plain
    /// circular arc, `1.0` is fully smoothed.
    pub smoothing: f32,
}

impl CornerRounding {
    /// No rounding at all: a sharp corner.
    pub const UNROUNDED: Self = Self::new(0.0, 0.0);

    /// Creates a rounding from a `radius` and a `smoothing` factor.
    #[must_use]
    pub const fn new(radius: f32, smoothing: f32) -> Self {
        Self { radius, smoothing }
    }

    /// Creates a rounding with the given `radius` and no smoothing.
    #[must_use]
    pub const fn from_radius(radius: f32) -> Self {
        Self::new(radius, 0.0)
    }
}

/// Below this sine the two sides are (anti-)parallel and no rounding is possible.
const MIN_SIN_ANGLE: f32 = 1e-3;

/// A circle that rounds a corner: its center and radius.
#[derive(Debug, Clone, Copy)]
struct Circle {
    center: Point,
    radius: f32,
}

/// The corner at `p1` of the path `p0 -> p1 -> p2`, together with the rounding
/// requested for it.
///
/// A corner reports how much of each adjacent side it would like to cut
/// ([`RoundedCorner::expected_round_cut`] and [`RoundedCorner::expected_cut`]).
/// The polygon decides how much it can actually give, and
/// [`RoundedCorner::cubics`] builds the curves for that amount.
///
/// # Examples
///
/// ```rust
/// use rounded_polygon_amane::shapes::corners::CornerRounding;
/// use rounded_polygon_amane::shapes::points::Point;
/// use rounded_polygon_amane::shapes::corners::RoundedCorner;
///
/// // A right angle at the origin, rounded with a radius of 0.2.
/// let corner = RoundedCorner::new(
///     Point::new(0.0, 1.0),
///     Point::new(0.0, 0.0),
///     Point::new(1.0, 0.0),
///     CornerRounding::from_radius(0.2),
/// );
///
/// // Rounding is cut from the sides: line, arc, line.
/// let cubics = corner.cubics(0.5, 0.5);
/// assert_eq!(cubics.len(), 3);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RoundedCorner {
    p0: Point,
    p1: Point,
    p2: Point,
    d1: Point,
    d2: Point,
    corner_radius: f32,
    smoothing: f32,
    expected_round_cut: f32,
}

impl RoundedCorner {
    /// Creates the corner at `p1` between the sides towards `p0` and `p2`.
    ///
    /// If either side has no length, the corner cannot be rounded and its
    /// expected cut is zero.
    #[must_use]
    pub fn new(p0: Point, p1: Point, p2: Point, rounding: CornerRounding) -> Self {
        let v01 = p0 - p1;
        let v21 = p2 - p1;
        let d01 = v01.distance();
        let d21 = v21.distance();

        if d01 > 0.0 && d21 > 0.0 {
            let d1 = v01 / d01;
            let d2 = v21 / d21;

            // The cosine of the angle at p1 is the dot product of the unit
            // vectors towards the other two vertices.
            let cos_angle = d1.dot_product(d2);
            // From sin^2 + cos^2 = 1. Rounding can push the cosine just past
            // 1, so keep the root real.
            let sin_angle = (1.0 - cos_angle * cos_angle).max(0.0).sqrt();

            // How far along a side the rounding circle touches it, from
            // tan(A/2) = sin(A) / (1 + cos(A)) = radius / cut.
            let expected_round_cut = if sin_angle > MIN_SIN_ANGLE {
                rounding.radius * (cos_angle + 1.0) / sin_angle
            } else {
                0.0
            };

            Self {
                p0,
                p1,
                p2,
                d1,
                d2,
                corner_radius: rounding.radius,
                smoothing: rounding.smoothing,
                expected_round_cut,
            }
        } else {
            Self {
                p0,
                p1,
                p2,
                d1: Point::default(),
                d2: Point::default(),
                corner_radius: 0.0,
                smoothing: 0.0,
                expected_round_cut: 0.0,
            }
        }
    }

    /// How much of each adjacent side the circular rounding needs.
    #[must_use]
    pub fn expected_round_cut(&self) -> f32 {
        self.expected_round_cut
    }

    /// How much of each adjacent side the rounding and its smoothing need.
    #[must_use]
    pub fn expected_cut(&self) -> f32 {
        (1.0 + self.smoothing) * self.expected_round_cut
    }

    /// Builds the curves that round this corner.
    ///
    /// `allowed_cut0` and `allowed_cut1` are how much of the sides towards `p0`
    /// and `p2` the polygon can give up. The smaller one limits the radius; any
    /// extra room on a side goes to smoothing.
    ///
    /// Returns one zero-length curve at `p1` when there is nothing to round,
    /// otherwise a flanking curve, the circular arc and a second flanking curve.
    #[must_use]
    pub fn cubics(&self, allowed_cut0: f32, allowed_cut1: f32) -> Vec<Cubic> {
        // The smaller cut sets the radius; spare room on one side can still
        // be used for smoothing.
        let allowed_cut = allowed_cut0.min(allowed_cut1);

        if self.expected_round_cut < DISTANCE_EPSILON
            || allowed_cut < DISTANCE_EPSILON
            || self.corner_radius < DISTANCE_EPSILON
        {
            return vec![Cubic::straight_line(
                self.p1.x, self.p1.y, self.p1.x, self.p1.y,
            )];
        }

        let actual_round_cut = allowed_cut.min(self.expected_round_cut);

        // Rounding is served first; smoothing only gets what is left over,
        // separately for each side.
        let actual_smoothing0 = self.actual_smoothing(allowed_cut0);
        let actual_smoothing1 = self.actual_smoothing(allowed_cut1);

        let actual_radius = self.corner_radius * actual_round_cut / self.expected_round_cut;

        let center_distance = actual_radius.hypot(actual_round_cut);
        let center = self.p1 + ((self.d1 + self.d2) / 2.0).direction() * center_distance;
        let circle = Circle {
            center,
            radius: actual_radius,
        };

        let circle_intersection0 = self.p1 + self.d1 * actual_round_cut;
        let circle_intersection2 = self.p1 + self.d2 * actual_round_cut;

        let flanking0 = self.flanking_curve(
            actual_round_cut,
            actual_smoothing0,
            self.p0,
            circle_intersection0,
            circle_intersection2,
            circle,
        );
        let flanking2 = self
            .flanking_curve(
                actual_round_cut,
                actual_smoothing1,
                self.p2,
                circle_intersection2,
                circle_intersection0,
                circle,
            )
            .reverse();

        let arc = Cubic::circular_arc(
            center.x,
            center.y,
            flanking0.anchor1_x(),
            flanking0.anchor1_y(),
            flanking2.anchor0_x(),
            flanking2.anchor0_y(),
        );

        vec![flanking0, arc, flanking2]
    }

    /// The smoothing that fits in `allowed_cut` once rounding has been served.
    fn actual_smoothing(&self, allowed_cut: f32) -> f32 {
        if allowed_cut > self.expected_cut() {
            self.smoothing
        } else if allowed_cut > self.expected_round_cut {
            self.smoothing * (allowed_cut - self.expected_round_cut)
                / (self.expected_cut() - self.expected_round_cut)
        } else {
            0.0
        }
    }

    /// The curve that leads from a side into the rounding circle.
    ///
    /// `side_start` is the far end of the side. The two intersections are where
    /// the circle meets this side and the other one.
    fn flanking_curve(
        &self,
        actual_round_cut: f32,
        actual_smoothing: f32,
        side_start: Point,
        circle_segment_intersection: Point,
        other_circle_segment_intersection: Point,
        circle: Circle,
    ) -> Cubic {
        let corner = self.p1;
        let side_direction = (side_start - corner).direction();
        let curve_start = corner + side_direction * (actual_round_cut * (1.0 + actual_smoothing));

        // Cut away a part of the circle section proportional to the smoothing:
        // none at 0, all of it at 1. This is an approximation.
        let p = Point::interpolate(
            circle_segment_intersection,
            (circle_segment_intersection + other_circle_segment_intersection) / 2.0,
            actual_smoothing,
        );

        // The flanking curve ends on the circle.
        let curve_end = circle.center
            + direction_vector(p.x - circle.center.x, p.y - circle.center.y) * circle.radius;

        // The end anchor sits where the circle's tangent at the curve end
        // meets the side.
        let circle_tangent = (curve_end - circle.center).rotate90();
        let anchor_end = line_intersection(side_start, side_direction, curve_end, circle_tangent)
            .unwrap_or(circle_segment_intersection);

        // 2/3 of the way from the end anchor back to the start, as design
        // tools do.
        let anchor_start = (curve_start + anchor_end * 2.0) / 3.0;

        Cubic::from_points(curve_start, anchor_start, anchor_end, curve_end)
    }
}

/// The point where the line through `p0` along `d0` meets the line through
/// `p1` along `d1`, or `None` if the lines are (nearly) parallel.
fn line_intersection(p0: Point, d0: Point, p1: Point, d1: Point) -> Option<Point> {
    let rotated_d1 = d1.rotate90();
    let den = d0.dot_product(rotated_d1);
    if den.abs() < DISTANCE_EPSILON {
        return None;
    }

    let num = (p1 - p0).dot_product(rotated_d1);
    // Equivalent to abs(den / num) < DISTANCE_EPSILON, without the division.
    if den.abs() < DISTANCE_EPSILON * num.abs() {
        return None;
    }

    Some(p0 + d0 * (num / den))
}

#[cfg(test)]
mod corner_rounding_tests {
    use super::*;

    #[test]
    fn new_stores_radius_and_smoothing() {
        let rounding = CornerRounding::new(0.25, 0.75);

        assert_eq!((rounding.radius, rounding.smoothing), (0.25, 0.75));
    }

    #[test]
    fn from_radius_has_no_smoothing() {
        assert_eq!(
            CornerRounding::from_radius(0.5),
            CornerRounding::new(0.5, 0.0)
        );
    }

    #[test]
    fn unrounded_has_zero_radius_and_smoothing() {
        assert_eq!(CornerRounding::UNROUNDED, CornerRounding::new(0.0, 0.0));
    }

    #[test]
    fn default_is_unrounded() {
        assert_eq!(CornerRounding::default(), CornerRounding::UNROUNDED);
    }
}

#[cfg(test)]
mod rounded_corner_tests {
    use super::*;

    const EPSILON: f32 = 1e-4;

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn near_point(a: Point, b: Point) -> bool {
        near(a.x, b.x) && near(a.y, b.y)
    }

    fn start(cubic: &Cubic) -> Point {
        Point::new(cubic.anchor0_x(), cubic.anchor0_y())
    }

    fn end(cubic: &Cubic) -> Point {
        Point::new(cubic.anchor1_x(), cubic.anchor1_y())
    }

    /// A right angle at the origin: one side runs up the y axis, the other along the x axis.
    fn right_angle(rounding: CornerRounding) -> RoundedCorner {
        RoundedCorner::new(
            Point::new(0.0, 1.0),
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            rounding,
        )
    }

    #[test]
    fn expected_round_cut_of_a_right_angle_equals_the_radius() {
        let corner = right_angle(CornerRounding::from_radius(0.2));

        assert!(near(corner.expected_round_cut(), 0.2));
    }

    #[test]
    fn expected_round_cut_grows_as_the_corner_gets_sharper() {
        let sharp = RoundedCorner::new(
            Point::new(1.0, 0.2),
            Point::new(0.0, 0.0),
            Point::new(1.0, -0.2),
            CornerRounding::from_radius(0.2),
        );

        assert!(sharp.expected_round_cut() > 0.2);
    }

    #[test]
    fn expected_cut_adds_the_smoothing() {
        let corner = right_angle(CornerRounding::new(0.2, 0.5));

        assert!(near(corner.expected_cut(), 0.3));
    }

    #[test]
    fn expected_cut_without_smoothing_equals_the_round_cut() {
        let corner = right_angle(CornerRounding::from_radius(0.2));

        assert!(near(corner.expected_cut(), corner.expected_round_cut()));
    }

    #[test]
    fn expected_round_cut_is_zero_when_unrounded() {
        assert_eq!(
            right_angle(CornerRounding::UNROUNDED).expected_round_cut(),
            0.0
        );
    }

    #[test]
    fn expected_round_cut_is_zero_when_a_side_has_no_length() {
        let corner = RoundedCorner::new(
            Point::new(0.0, 0.0),
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            CornerRounding::from_radius(0.2),
        );

        assert_eq!(corner.expected_round_cut(), 0.0);
    }

    #[test]
    fn expected_round_cut_is_zero_when_the_sides_are_collinear() {
        let corner = RoundedCorner::new(
            Point::new(-1.0, 0.0),
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            CornerRounding::from_radius(0.2),
        );

        assert_eq!(corner.expected_round_cut(), 0.0);
    }

    #[test]
    fn expected_round_cut_is_zero_when_the_sides_fold_back_on_themselves() {
        let corner = RoundedCorner::new(
            Point::new(1.0, 0.0),
            Point::new(0.0, 0.0),
            Point::new(2.0, 0.0),
            CornerRounding::from_radius(0.2),
        );

        assert_eq!(corner.expected_round_cut(), 0.0);
    }

    #[test]
    fn cubics_of_an_unrounded_corner_is_one_curve() {
        let cubics = right_angle(CornerRounding::UNROUNDED).cubics(1.0, 1.0);

        assert_eq!(cubics.len(), 1);
    }

    #[test]
    fn cubics_of_an_unrounded_corner_is_a_zero_length_curve_at_the_corner() {
        let cubics = right_angle(CornerRounding::UNROUNDED).cubics(1.0, 1.0);

        assert_eq!(cubics[0], Cubic::straight_line(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn cubics_with_no_room_is_a_zero_length_curve_at_the_corner() {
        let cubics = right_angle(CornerRounding::from_radius(0.2)).cubics(0.0, 1.0);

        assert_eq!(cubics.len(), 1);
    }

    #[test]
    fn cubics_of_a_rounded_corner_is_flank_arc_flank() {
        let cubics = right_angle(CornerRounding::from_radius(0.2)).cubics(1.0, 1.0);

        assert_eq!(cubics.len(), 3);
    }

    #[test]
    fn cubics_start_on_the_first_side_at_the_cut_distance() {
        let cubics = right_angle(CornerRounding::from_radius(0.2)).cubics(1.0, 1.0);

        assert!(near_point(start(&cubics[0]), Point::new(0.0, 0.2)));
    }

    #[test]
    fn cubics_end_on_the_second_side_at_the_cut_distance() {
        let cubics = right_angle(CornerRounding::from_radius(0.2)).cubics(1.0, 1.0);

        assert!(near_point(end(&cubics[2]), Point::new(0.2, 0.0)));
    }

    #[test]
    fn cubics_arc_runs_between_the_two_touch_points() {
        let cubics = right_angle(CornerRounding::from_radius(0.2)).cubics(1.0, 1.0);

        assert!(near_point(start(&cubics[1]), Point::new(0.0, 0.2)));
        assert!(near_point(end(&cubics[1]), Point::new(0.2, 0.0)));
    }

    #[test]
    fn cubics_arc_midpoint_lies_on_the_rounding_circle() {
        let cubics = right_angle(CornerRounding::from_radius(0.2)).cubics(1.0, 1.0);

        // The circle of radius 0.2 touching both sides is centered at (0.2, 0.2).
        let midpoint = cubics[1].point_on_curve(0.5);

        assert!(((midpoint - Point::new(0.2, 0.2)).distance() - 0.2).abs() < 1e-3);
    }

    #[test]
    fn cubics_shrink_the_radius_when_the_allowed_cut_is_smaller() {
        let cubics = right_angle(CornerRounding::from_radius(0.2)).cubics(0.1, 1.0);

        assert!(near_point(start(&cubics[1]), Point::new(0.0, 0.1)));
        assert!(near_point(end(&cubics[1]), Point::new(0.1, 0.0)));
    }

    #[test]
    fn cubics_use_the_smaller_cut_of_the_two_sides_for_the_radius() {
        let cubics = right_angle(CornerRounding::from_radius(0.2)).cubics(1.0, 0.1);

        assert!(near_point(start(&cubics[1]), Point::new(0.0, 0.1)));
    }

    #[test]
    fn cubics_with_smoothing_start_further_along_the_side() {
        let cubics = right_angle(CornerRounding::new(0.2, 1.0)).cubics(1.0, 1.0);

        // Full smoothing doubles the cut: 0.2 * (1 + 1).
        assert!(near_point(start(&cubics[0]), Point::new(0.0, 0.4)));
    }

    #[test]
    fn cubics_with_smoothing_end_further_along_the_other_side() {
        let cubics = right_angle(CornerRounding::new(0.2, 1.0)).cubics(1.0, 1.0);

        assert!(near_point(end(&cubics[2]), Point::new(0.4, 0.0)));
    }

    #[test]
    fn cubics_with_smoothing_join_up_without_gaps() {
        let cubics = right_angle(CornerRounding::new(0.2, 1.0)).cubics(1.0, 1.0);

        assert!(near_point(end(&cubics[0]), start(&cubics[1])));
        assert!(near_point(end(&cubics[1]), start(&cubics[2])));
    }

    #[test]
    fn cubics_skip_smoothing_when_there_is_no_room_beyond_the_round_cut() {
        let smoothed = right_angle(CornerRounding::new(0.2, 1.0)).cubics(0.2, 0.2);
        let plain = right_angle(CornerRounding::from_radius(0.2)).cubics(0.2, 0.2);

        assert!(near_point(start(&smoothed[0]), start(&plain[0])));
    }

    #[test]
    fn cubics_apply_partial_smoothing_with_partial_room() {
        // Room for half of the extra cut: 0.2 + 0.5 * (0.4 - 0.2) = 0.3.
        let cubics = right_angle(CornerRounding::new(0.2, 1.0)).cubics(0.3, 0.3);

        assert!(near_point(start(&cubics[0]), Point::new(0.0, 0.3)));
    }

    #[test]
    fn line_intersection_finds_where_two_lines_cross() {
        let crossing = line_intersection(
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(2.0, 1.0),
            Point::new(0.0, 1.0),
        );

        assert!(crossing.is_some_and(|p| near_point(p, Point::new(2.0, 0.0))));
    }

    #[test]
    fn line_intersection_of_parallel_lines_is_none() {
        let crossing = line_intersection(
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 1.0),
            Point::new(1.0, 0.0),
        );

        assert_eq!(crossing, None);
    }
}
