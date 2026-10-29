//! Cubic Bezier curves.

use std::array;
use std::ops::{Add, Div, Mul};

use super::point::Point;
use super::utils::{self, DISTANCE_EPSILON};

/// A cubic Bézier curve made of two anchor points and two control points.
///
/// The eight coordinates are stored as
/// `[anchor0.x, anchor0.y, control0.x, control0.y, control1.x, control1.y, anchor1.x, anchor1.y]`.
///
/// # Examples
///
/// ```rut
/// use rounded_polygon_amane::shapes::cubic::Cubic;
/// use rounded_polygon_amane::shapes::point::Point;
///
/// let line = Cubic::straight_line(0.0, 0.0, 3.0, 3.0);
///
/// assert_eq!(line.point_on_curve(0.5), Point::new(1.5, 1.5));
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Cubic {
    /// The eight coordinates, in the order described on [`Cubic`].
    pub points: [f32; 8],
}

impl Cubic {
    /// Creates a curve from its eight coordinates.
    #[must_use]
    pub const fn new(points: [f32; 8]) -> Self {
        Self { points }
    }

    /// Creates a curve from its two anchor and two control points.
    #[must_use]
    pub const fn from_points(
        anchor0: Point,
        control0: Point,
        control1: Point,
        anchor1: Point,
    ) -> Self {
        Self::new([
            anchor0.x, anchor0.y, control0.x, control0.y, control1.x, control1.y, anchor1.x,
            anchor1.y,
        ])
    }

    /// X coordinate of the start anchor.
    #[must_use]
    pub const fn anchor0_x(&self) -> f32 {
        self.points[0]
    }

    /// Y coordinate of the start anchor.
    #[must_use]
    pub const fn anchor0_y(&self) -> f32 {
        self.points[1]
    }

    /// X coordinate of the control point next to the start anchor.
    #[must_use]
    pub const fn control0_x(&self) -> f32 {
        self.points[2]
    }

    /// Y coordinate of the control point next to the start anchor.
    #[must_use]
    pub const fn control0_y(&self) -> f32 {
        self.points[3]
    }

    /// X coordinate of the control point next to the end anchor.
    #[must_use]
    pub const fn control1_x(&self) -> f32 {
        self.points[4]
    }

    /// Y coordinate of the control point next to the end anchor.
    #[must_use]
    pub const fn control1_y(&self) -> f32 {
        self.points[5]
    }

    /// X coordinate of the end anchor.
    #[must_use]
    pub const fn anchor1_x(&self) -> f32 {
        self.points[6]
    }

    /// Y coordinate of the end anchor.
    #[must_use]
    pub const fn anchor1_y(&self) -> f32 {
        self.points[7]
    }

    /// A straight segment from `(x0, y0)` to `(x1, y1)`, with control points a
    /// third of the way along.
    #[must_use]
    pub fn straight_line(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self::new([
            x0,
            y0,
            utils::interpolate(x0, x1, 1.0 / 3.0),
            utils::interpolate(y0, y1, 1.0 / 3.0),
            utils::interpolate(x0, x1, 2.0 / 3.0),
            utils::interpolate(y0, y1, 2.0 / 3.0),
            x1,
            y1,
        ])
    }

    /// An arc of the circle around `(center_x, center_y)` from `(x0, y0)` to
    /// `(x1, y1)`, approximated by one cubic.
    ///
    /// Arcs that are nearly straight fall back to [`Cubic::straight_line`].
    #[must_use]
    pub fn circular_arc(center_x: f32, center_y: f32, x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        let p0d = utils::direction_vector(x0 - center_x, y0 - center_y);
        let p1d = utils::direction_vector(x1 - center_x, y1 - center_y);
        let rotated_p0 = p0d.rotate90();
        let rotated_p1 = p1d.rotate90();
        let clockwise = rotated_p0.dot_product_xy(x1 - center_x, y1 - center_y) >= 0.0;
        let cosa = p0d.dot_product(p1d);

        if cosa > 0.999 {
            return Self::straight_line(x0, y0, x1, y1);
        }

        let direction = if clockwise { 1.0 } else { -1.0 };
        // Rounding can push `cosa` just past -1, which would make the root NaN.
        let sina = (1.0 - cosa * cosa).max(0.0).sqrt();
        let k = utils::distance(x0 - center_x, y0 - center_y) * 4.0 / 3.0
            * ((2.0 * (1.0 - cosa)).sqrt() - sina)
            / (1.0 - cosa)
            * direction;

        Self::new([
            x0,
            y0,
            x0 + rotated_p0.x * k,
            y0 + rotated_p0.y * k,
            x1 - rotated_p1.x * k,
            y1 - rotated_p1.y * k,
            x1,
            y1,
        ])
    }

    /// A zero-length curve sitting at `(x0, y0)`.
    #[must_use]
    pub const fn empty(x0: f32, y0: f32) -> Self {
        Self::new([x0, y0, x0, y0, x0, y0, x0, y0])
    }

    /// Interpolates every coordinate between `from` and `to`.
    ///
    /// A `progress` of `0.0` yields `from` and `1.0` yields `to`.
    #[must_use]
    pub fn interpolate(from: &Self, to: &Self, progress: f32) -> Self {
        Self::new(array::from_fn(|i| {
            utils::interpolate(from.points[i], to.points[i], progress)
        }))
    }

    /// The point on the curve at parameter `t`, where `0.0` is the start
    /// anchor and `1.0` the end anchor.
    #[must_use]
    pub fn point_on_curve(&self, t: f32) -> Point {
        let u = 1.0 - t;
        Point::new(
            self.anchor0_x() * (u * u * u)
                + self.control0_x() * (3.0 * t * u * u)
                + self.control1_x() * (3.0 * t * t * u)
                + self.anchor1_x() * (t * t * t),
            self.anchor0_y() * (u * u * u)
                + self.control0_y() * (3.0 * t * u * u)
                + self.control1_y() * (3.0 * t * t * u)
                + self.anchor1_y() * (t * t * t),
        )
    }

    /// Returns `true` if the two anchors coincide (within [`DISTANCE_EPSILON`]).
    #[must_use]
    pub fn zero_length(&self) -> bool {
        (self.anchor0_x() - self.anchor1_x()).abs() < DISTANCE_EPSILON
            && (self.anchor0_y() - self.anchor1_y()).abs() < DISTANCE_EPSILON
    }

    /// Returns `true` if the corner where this curve meets `next` turns clockwise.
    #[must_use]
    pub fn convex_to(&self, next: &Self) -> bool {
        let previous = Point::new(self.anchor0_x(), self.anchor0_y());
        let current = Point::new(self.anchor1_x(), self.anchor1_y());
        let next = Point::new(next.anchor1_x(), next.anchor1_y());
        utils::convex(previous, current, next)
    }

    /// The bounding box of the curve as `[min_x, min_y, max_x, max_y]`.
    ///
    /// With `approximate`, the box of all four points is returned, which is
    /// cheaper but can be larger than the curve. Without it, the box is exact.
    #[must_use]
    pub fn calculate_bounds(&self, approximate: bool) -> [f32; 4] {
        if self.zero_length() {
            return [
                self.anchor0_x(),
                self.anchor0_y(),
                self.anchor0_x(),
                self.anchor0_y(),
            ];
        }

        let mut min_x = self.anchor0_x().min(self.anchor1_x());
        let mut min_y = self.anchor0_y().min(self.anchor1_y());
        let mut max_x = self.anchor0_x().max(self.anchor1_x());
        let mut max_y = self.anchor0_y().max(self.anchor1_y());

        if approximate {
            return [
                min_x.min(self.control0_x().min(self.control1_x())),
                min_y.min(self.control0_y().min(self.control1_y())),
                max_x.max(self.control0_x().max(self.control1_x())),
                max_y.max(self.control0_y().max(self.control1_y())),
            ];
        }

        let x_params = extremum_params(
            self.anchor0_x(),
            self.control0_x(),
            self.control1_x(),
            self.anchor1_x(),
        );
        for t in x_params.into_iter().flatten() {
            let x = self.point_on_curve(t).x;
            min_x = min_x.min(x);
            max_x = max_x.max(x);
        }

        let y_params = extremum_params(
            self.anchor0_y(),
            self.control0_y(),
            self.control1_y(),
            self.anchor1_y(),
        );
        for t in y_params.into_iter().flatten() {
            let y = self.point_on_curve(t).y;
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }

        [min_x, min_y, max_x, max_y]
    }

    /// Splits the curve at parameter `t` into the part before and the part after.
    #[must_use]
    pub fn split(&self, t: f32) -> (Self, Self) {
        let u = 1.0 - t;
        let point = self.point_on_curve(t);

        let before = Self::new([
            self.anchor0_x(),
            self.anchor0_y(),
            self.anchor0_x() * u + self.control0_x() * t,
            self.anchor0_y() * u + self.control0_y() * t,
            self.anchor0_x() * (u * u)
                + self.control0_x() * (2.0 * u * t)
                + self.control1_x() * (t * t),
            self.anchor0_y() * (u * u)
                + self.control0_y() * (2.0 * u * t)
                + self.control1_y() * (t * t),
            point.x,
            point.y,
        ]);
        let after = Self::new([
            point.x,
            point.y,
            self.control0_x() * (u * u)
                + self.control1_x() * (2.0 * u * t)
                + self.anchor1_x() * (t * t),
            self.control0_y() * (u * u)
                + self.control1_y() * (2.0 * u * t)
                + self.anchor1_y() * (t * t),
            self.control1_x() * u + self.anchor1_x() * t,
            self.control1_y() * u + self.anchor1_y() * t,
            self.anchor1_x(),
            self.anchor1_y(),
        ]);
        (before, after)
    }

    /// The same curve traversed in the opposite direction.
    #[must_use]
    pub fn reverse(&self) -> Self {
        Self::new([
            self.anchor1_x(),
            self.anchor1_y(),
            self.control1_x(),
            self.control1_y(),
            self.control0_x(),
            self.control0_y(),
            self.anchor0_x(),
            self.anchor0_y(),
        ])
    }

    /// Maps every point of the curve through `f`, which receives each point's
    /// `x` and `y` and returns its replacement.
    #[must_use]
    pub fn transformed(&self, mut f: impl FnMut(f32, f32) -> Point) -> Self {
        let mut points = self.points;
        let (pairs, _) = points.as_chunks_mut::<2>();
        for pair in pairs {
            let mapped = f(pair[0], pair[1]);
            pair[0] = mapped.x;
            pair[1] = mapped.y;
        }
        Self::new(points)
    }
}

/// Parameters in `0..=1` where one coordinate of the curve has a turning point.
fn extremum_params(a0: f32, c0: f32, c1: f32, a1: f32) -> [Option<f32>; 2] {
    let a = -a0 + 3.0 * c0 - 3.0 * c1 + a1;
    let b = 2.0 * a0 - 4.0 * c0 + 2.0 * c1;
    let c = -a0 + c0;
    let in_unit_range = |t: f32| (0.0..=1.0).contains(&t).then_some(t);

    if a.abs() < DISTANCE_EPSILON {
        if b == 0.0 {
            [None, None]
        } else {
            [in_unit_range(2.0 * c / (-2.0 * b)), None]
        }
    } else {
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            [None, None]
        } else {
            let root = discriminant.sqrt();
            [
                in_unit_range((-b + root) / (2.0 * a)),
                in_unit_range((-b - root) / (2.0 * a)),
            ]
        }
    }
}

impl Add for Cubic {
    type Output = Self;

    /// Adds the two curves coordinate by coordinate.
    fn add(self, other: Self) -> Self {
        Self::new(array::from_fn(|i| self.points[i] + other.points[i]))
    }
}

impl Mul<f32> for Cubic {
    type Output = Self;

    /// Scales every coordinate by `operand`.
    fn mul(self, operand: f32) -> Self {
        Self::new(self.points.map(|value| value * operand))
    }
}

impl Div<f32> for Cubic {
    type Output = Self;

    /// Divides every coordinate by `operand`.
    fn div(self, operand: f32) -> Self {
        self * (1.0 / operand)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-4;

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn near_point(a: Point, b: Point) -> bool {
        near(a.x, b.x) && near(a.y, b.y)
    }

    fn sample() -> Cubic {
        Cubic::new([0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
    }

    #[test]
    fn new_stores_the_points_in_order() {
        assert_eq!(sample().points, [0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn accessors_read_the_matching_coordinates() {
        let cubic = Cubic::new([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);

        assert_eq!(
            [
                cubic.anchor0_x(),
                cubic.anchor0_y(),
                cubic.control0_x(),
                cubic.control0_y(),
                cubic.control1_x(),
                cubic.control1_y(),
                cubic.anchor1_x(),
                cubic.anchor1_y(),
            ],
            [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]
        );
    }

    #[test]
    fn from_points_orders_anchor_control_control_anchor() {
        let cubic = Cubic::from_points(
            Point::new(1.0, 2.0),
            Point::new(3.0, 4.0),
            Point::new(5.0, 6.0),
            Point::new(7.0, 8.0),
        );

        assert_eq!(cubic.points, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    }

    #[test]
    fn straight_line_places_control_points_at_thirds() {
        let line = Cubic::straight_line(0.0, 0.0, 3.0, 6.0);

        assert!(near(line.control0_x(), 1.0) && near(line.control0_y(), 2.0));
        assert!(near(line.control1_x(), 2.0) && near(line.control1_y(), 4.0));
    }

    #[test]
    fn straight_line_keeps_its_endpoints() {
        let line = Cubic::straight_line(1.0, 2.0, 3.0, 4.0);

        assert_eq!(
            (
                line.anchor0_x(),
                line.anchor0_y(),
                line.anchor1_x(),
                line.anchor1_y()
            ),
            (1.0, 2.0, 3.0, 4.0)
        );
    }

    #[test]
    fn circular_arc_keeps_its_endpoints() {
        let arc = Cubic::circular_arc(0.0, 0.0, 1.0, 0.0, 0.0, 1.0);

        assert_eq!(
            (
                arc.anchor0_x(),
                arc.anchor0_y(),
                arc.anchor1_x(),
                arc.anchor1_y()
            ),
            (1.0, 0.0, 0.0, 1.0)
        );
    }

    #[test]
    fn circular_arc_quarter_turn_midpoint_lies_on_the_circle() {
        let arc = Cubic::circular_arc(0.0, 0.0, 1.0, 0.0, 0.0, 1.0);

        assert!((arc.point_on_curve(0.5).distance() - 1.0).abs() < 1e-3);
    }

    #[test]
    fn circular_arc_quarter_turn_has_the_standard_control_offset() {
        let arc = Cubic::circular_arc(0.0, 0.0, 1.0, 0.0, 0.0, 1.0);

        // 4/3 * tan(pi/8) is the usual handle length for a quarter circle.
        assert!(near(arc.control0_y(), 0.552_284_8));
    }

    #[test]
    fn circular_arc_counter_clockwise_bends_the_other_way() {
        let arc = Cubic::circular_arc(0.0, 0.0, 0.0, 1.0, 1.0, 0.0);

        assert!((arc.point_on_curve(0.5).distance() - 1.0).abs() < 1e-3);
    }

    #[test]
    fn circular_arc_half_turn_has_finite_coordinates() {
        let arc = Cubic::circular_arc(0.0, 0.0, 1.0, 0.0, -1.0, 0.0);

        assert!(arc.points.iter().all(|value| value.is_finite()));
    }

    #[test]
    fn circular_arc_of_nearly_equal_points_is_a_straight_line() {
        let arc = Cubic::circular_arc(0.0, 0.0, 1.0, 0.0, 1.0, 0.001);

        assert_eq!(arc, Cubic::straight_line(1.0, 0.0, 1.0, 0.001));
    }

    #[test]
    fn empty_repeats_the_point_in_every_slot() {
        assert_eq!(
            Cubic::empty(2.0, 3.0).points,
            [2.0, 3.0, 2.0, 3.0, 2.0, 3.0, 2.0, 3.0]
        );
    }

    #[test]
    fn interpolate_at_zero_returns_the_first_curve() {
        let from = sample();
        let to = sample() * 2.0;

        assert_eq!(Cubic::interpolate(&from, &to, 0.0), from);
    }

    #[test]
    fn interpolate_at_one_returns_the_second_curve() {
        let from = sample();
        let to = sample() * 2.0;

        assert_eq!(Cubic::interpolate(&from, &to, 1.0), to);
    }

    #[test]
    fn interpolate_at_half_returns_the_average() {
        let from = Cubic::empty(0.0, 0.0);
        let to = Cubic::empty(4.0, 8.0);

        assert_eq!(Cubic::interpolate(&from, &to, 0.5), Cubic::empty(2.0, 4.0));
    }

    #[test]
    fn point_on_curve_at_zero_is_the_start_anchor() {
        assert_eq!(sample().point_on_curve(0.0), Point::new(0.0, 0.0));
    }

    #[test]
    fn point_on_curve_at_one_is_the_end_anchor() {
        assert_eq!(sample().point_on_curve(1.0), Point::new(5.0, 6.0));
    }

    #[test]
    fn point_on_curve_at_half_of_a_straight_line_is_the_midpoint() {
        let line = Cubic::straight_line(0.0, 0.0, 3.0, 3.0);

        assert!(near_point(line.point_on_curve(0.5), Point::new(1.5, 1.5)));
    }

    #[test]
    fn zero_length_is_true_when_the_anchors_coincide() {
        assert!(Cubic::new([1.0, 1.0, 5.0, 5.0, 6.0, 6.0, 1.0, 1.0]).zero_length());
    }

    #[test]
    fn zero_length_is_false_when_the_anchors_differ() {
        assert!(!Cubic::straight_line(0.0, 0.0, 1.0, 0.0).zero_length());
    }

    #[test]
    fn convex_to_is_true_for_a_clockwise_turn() {
        let first = Cubic::straight_line(0.0, 0.0, 1.0, 0.0);
        let second = Cubic::straight_line(1.0, 0.0, 1.0, 1.0);

        assert!(first.convex_to(&second));
    }

    #[test]
    fn convex_to_is_false_for_a_counter_clockwise_turn() {
        let first = Cubic::straight_line(0.0, 0.0, 1.0, 0.0);
        let second = Cubic::straight_line(1.0, 0.0, 1.0, -1.0);

        assert!(!first.convex_to(&second));
    }

    #[test]
    fn calculate_bounds_of_a_zero_length_curve_is_its_anchor() {
        assert_eq!(
            Cubic::empty(2.0, 3.0).calculate_bounds(false),
            [2.0, 3.0, 2.0, 3.0]
        );
    }

    #[test]
    fn calculate_bounds_of_a_straight_line_spans_its_endpoints() {
        let line = Cubic::straight_line(0.0, 0.0, 3.0, 3.0);

        assert_eq!(line.calculate_bounds(false), [0.0, 0.0, 3.0, 3.0]);
    }

    #[test]
    fn calculate_bounds_approximate_includes_the_control_points() {
        let bulge = Cubic::new([0.0, 0.0, 0.0, 2.0, 1.0, 2.0, 1.0, 0.0]);

        assert_eq!(bulge.calculate_bounds(true), [0.0, 0.0, 1.0, 2.0]);
    }

    #[test]
    fn calculate_bounds_exact_finds_the_curve_peak() {
        let bulge = Cubic::new([0.0, 0.0, 0.0, 2.0, 1.0, 2.0, 1.0, 0.0]);
        let bounds = bulge.calculate_bounds(false);

        // The curve peaks at 3/4 of the control height, so y reaches 1.5, not 2.
        assert!(near(bounds[3], 1.5));
    }

    #[test]
    fn calculate_bounds_exact_keeps_the_anchor_extent_on_the_other_axis() {
        let bulge = Cubic::new([0.0, 0.0, 0.0, 2.0, 1.0, 2.0, 1.0, 0.0]);
        let bounds = bulge.calculate_bounds(false);

        assert_eq!((bounds[0], bounds[1], bounds[2]), (0.0, 0.0, 1.0));
    }

    #[test]
    fn split_at_half_meets_at_the_curve_midpoint() {
        let line = Cubic::straight_line(0.0, 0.0, 3.0, 3.0);

        let (before, after) = line.split(0.5);

        assert!(near_point(
            Point::new(before.anchor1_x(), before.anchor1_y()),
            Point::new(after.anchor0_x(), after.anchor0_y())
        ));
    }

    #[test]
    fn split_keeps_the_outer_anchors() {
        let cubic = sample();

        let (before, after) = cubic.split(0.3);

        assert_eq!(
            (
                before.anchor0_x(),
                before.anchor0_y(),
                after.anchor1_x(),
                after.anchor1_y()
            ),
            (0.0, 0.0, 5.0, 6.0)
        );
    }

    #[test]
    fn split_pieces_follow_the_original_curve() {
        let cubic = sample();

        let (before, _) = cubic.split(0.5);

        // Halfway along the first piece is a quarter of the way along the original.
        assert!(near_point(
            before.point_on_curve(0.5),
            cubic.point_on_curve(0.25)
        ));
    }

    #[test]
    fn reverse_swaps_the_anchors_and_the_controls() {
        assert_eq!(
            sample().reverse().points,
            [5.0, 6.0, 3.0, 4.0, 1.0, 2.0, 0.0, 0.0]
        );
    }

    #[test]
    fn reverse_twice_returns_the_original() {
        assert_eq!(sample().reverse().reverse(), sample());
    }

    #[test]
    fn add_sums_coordinates() {
        assert_eq!(sample() + sample(), sample() * 2.0);
    }

    #[test]
    fn mul_scales_every_coordinate() {
        assert_eq!(Cubic::empty(1.0, 2.0) * 3.0, Cubic::empty(3.0, 6.0));
    }

    #[test]
    fn div_scales_every_coordinate_down() {
        assert_eq!(Cubic::empty(4.0, 8.0) / 2.0, Cubic::empty(2.0, 4.0));
    }

    #[test]
    fn transformed_passes_each_point_to_the_function() {
        let swapped = Cubic::new([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0])
            .transformed(|x, y| Point::new(y, x));

        assert_eq!(swapped.points, [2.0, 1.0, 4.0, 3.0, 6.0, 5.0, 8.0, 7.0]);
    }

    #[test]
    fn transformed_leaves_the_original_unchanged() {
        let original = sample();

        let _ = original.transformed(|x, y| Point::new(x + 1.0, y + 1.0));

        assert_eq!(original, sample());
    }
}
