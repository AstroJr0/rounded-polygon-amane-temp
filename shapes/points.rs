//! A two-dimensional point, also used as a vector.
//!
//! Mapping progress values between two cyclic ranges.

use std::error::Error;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use super::utils::positive_modulo;

/// A point (or vector) in 2D space.
///
/// # Examples
///
/// ```
/// use rounded_polygon_amane::shapes::points::Point;
///
/// let a = Point::new(3.0, 4.0);
/// let b = Point::new(1.0, 1.0);
///
/// assert_eq!(a - b, Point::new(2.0, 3.0));
/// assert_eq!(a.distance(), 5.0);
/// assert_eq!(a.dot_product(b), 7.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    /// Horizontal component.
    pub x: f32,
    /// Vertical component.
    pub y: f32,
}

impl Point {
    /// Creates a point from its components.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Distance from the origin.
    #[must_use]
    pub fn distance(self) -> f32 {
        self.x.hypot(self.y)
    }

    /// Squared distance from the origin; avoids the square root of [`Point::distance`].
    #[must_use]
    pub fn distance_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Dot product of this vector with `other`.
    #[must_use]
    pub fn dot_product(self, other: Self) -> f32 {
        self.dot_product_xy(other.x, other.y)
    }

    /// Dot product of this vector with the vector `(x, y)`.
    #[must_use]
    pub fn dot_product_xy(self, x: f32, y: f32) -> f32 {
        self.x * x + self.y * y
    }

    /// Returns `true` if `other` lies clockwise from this vector.
    ///
    /// This is the sign of the 2D cross product, so it is `false` for parallel
    /// and anti-parallel vectors.
    #[must_use]
    pub fn clockwise(self, other: Self) -> bool {
        self.x * other.y - self.y * other.x > 0.0
    }

    /// The unit vector pointing the same way as this one.
    ///
    /// The zero vector has no direction, so its components are `NaN`.
    #[must_use]
    pub fn direction(self) -> Self {
        self / self.distance()
    }

    /// Linearly interpolates between `start` and `stop`.
    ///
    /// A `fraction` of `0.0` yields `start` and `1.0` yields `stop`. Values
    /// outside that range extrapolate.
    #[must_use]
    pub fn interpolate(start: Self, stop: Self, fraction: f32) -> Self {
        Self::new(
            start.x + (stop.x - start.x) * fraction,
            start.y + (stop.y - start.y) * fraction,
        )
    }

    /// Returns the point produced by `f`, which receives this point's `x` and `y`.
    #[must_use]
    pub fn transformed(self, f: impl FnOnce(f32, f32) -> Self) -> Self {
        f(self.x, self.y)
    }

    /// Rotates this vector by 90 degrees about the origin.
    #[must_use]
    pub fn rotate90(self) -> Self {
        Self::new(-self.y, self.x)
    }
}

impl Neg for Point {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f32> for Point {
    type Output = Self;

    fn mul(self, operand: f32) -> Self {
        Self::new(self.x * operand, self.y * operand)
    }
}

impl Div<f32> for Point {
    type Output = Self;

    fn div(self, operand: f32) -> Self {
        Self::new(self.x / operand, self.y / operand)
    }
}

impl Rem<f32> for Point {
    type Output = Self;

    fn rem(self, operand: f32) -> Self {
        Self::new(self.x % operand, self.y % operand)
    }
}

/// Segments shorter than this are treated as a single point when mapping.
const MIN_SEGMENT_SIZE: f32 = 0.001;

/// Why a value could not be mapped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingError {
    /// The value does not fall inside any segment of the mapping, for example
    /// because it is `NaN` or the mapping is empty.
    NoValidSegment,
}

impl fmt::Display for MappingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoValidSegment => f.write_str("no valid segment found"),
        }
    }
}

impl Error for MappingError {}

/// Maps progress values between two cyclic ranges, in both directions.
///
/// The mapper is built from `(source, target)` pairs. Values between two
/// neighbouring pairs are mapped linearly, and the last pair wraps around to
/// the first, so every value in `0..1` has a segment.
///
/// # Examples
///
/// ```rust
/// use rounded_polygon_amane::shapes::points::DoubleMapper;
///
/// let mapper = DoubleMapper::new([(0.0, 0.0), (0.5, 0.25)]);
///
/// assert_eq!(mapper.map(0.25), Ok(0.125));
/// assert_eq!(mapper.map_back(0.125), Ok(0.25));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DoubleMapper {
    source_values: Vec<f32>,
    target_values: Vec<f32>,
}

impl DoubleMapper {
    /// Creates a mapper from `(source, target)` pairs.
    #[must_use]
    pub fn new(mappings: impl IntoIterator<Item = (f32, f32)>) -> Self {
        let (source_values, target_values) = mappings.into_iter().unzip();
        Self {
            source_values,
            target_values,
        }
    }

    /// A mapper that returns every value unchanged.
    #[must_use]
    pub fn identity() -> Self {
        Self::new([(0.0, 0.0), (0.5, 0.5)])
    }

    /// Maps a source value to its target value.
    ///
    /// # Errors
    ///
    /// Returns [`MappingError::NoValidSegment`] if `x` is in no segment.
    pub fn map(&self, x: f32) -> Result<f32, MappingError> {
        linear_map(&self.source_values, &self.target_values, x)
    }

    /// Maps a target value back to its source value.
    ///
    /// # Errors
    ///
    /// Returns [`MappingError::NoValidSegment`] if `x` is in no segment.
    pub fn map_back(&self, x: f32) -> Result<f32, MappingError> {
        linear_map(&self.target_values, &self.source_values, x)
    }
}

fn linear_map(x_values: &[f32], y_values: &[f32], x: f32) -> Result<f32, MappingError> {
    let len = x_values.len();
    let start = (0..len)
        .find(|&i| progress_in_range(x, x_values[i], x_values[(i + 1) % len]))
        .ok_or(MappingError::NoValidSegment)?;
    let end = (start + 1) % len;

    let segment_size_x = positive_modulo(x_values[end] - x_values[start], 1.0);
    let segment_size_y = positive_modulo(y_values[end] - y_values[start], 1.0);

    let position_in_segment = if segment_size_x < MIN_SEGMENT_SIZE {
        0.5
    } else {
        positive_modulo(x - x_values[start], 1.0) / segment_size_x
    };

    Ok(positive_modulo(
        y_values[start] + segment_size_y * position_in_segment,
        1.0,
    ))
}

/// Returns `true` if `progress` lies from `progress_from` to `progress_to`,
/// both ends included.
///
/// When `progress_to` is smaller than `progress_from`, the range wraps around
/// the end of the cycle.
#[must_use]
pub fn progress_in_range(progress: f32, progress_from: f32, progress_to: f32) -> bool {
    if progress_to >= progress_from {
        progress >= progress_from && progress <= progress_to
    } else {
        progress >= progress_from || progress <= progress_to
    }
}

#[cfg(test)]
mod point_tests {
    use super::*;

    #[test]
    fn new_stores_components() {
        let point = Point::new(1.5, -2.5);

        assert_eq!((point.x, point.y), (1.5, -2.5));
    }

    #[test]
    fn default_is_the_origin() {
        assert_eq!(Point::default(), Point::new(0.0, 0.0));
    }

    #[test]
    fn distance_of_three_four_is_five() {
        assert_eq!(Point::new(3.0, 4.0).distance(), 5.0);
    }

    #[test]
    fn distance_squared_of_three_four_is_twenty_five() {
        assert_eq!(Point::new(3.0, 4.0).distance_squared(), 25.0);
    }

    #[test]
    fn dot_product_multiplies_and_sums_components() {
        assert_eq!(Point::new(1.0, 2.0).dot_product(Point::new(3.0, 4.0)), 11.0);
    }

    #[test]
    fn dot_product_xy_matches_dot_product() {
        assert_eq!(Point::new(1.0, 2.0).dot_product_xy(3.0, 4.0), 11.0);
    }

    #[test]
    fn dot_product_of_perpendicular_vectors_is_zero() {
        assert_eq!(Point::new(1.0, 0.0).dot_product(Point::new(0.0, 1.0)), 0.0);
    }

    #[test]
    fn clockwise_is_true_when_other_is_a_positive_cross_product_away() {
        assert!(Point::new(1.0, 0.0).clockwise(Point::new(0.0, 1.0)));
    }

    #[test]
    fn clockwise_is_false_when_other_is_a_negative_cross_product_away() {
        assert!(!Point::new(0.0, 1.0).clockwise(Point::new(1.0, 0.0)));
    }

    #[test]
    fn clockwise_is_false_for_parallel_vectors() {
        assert!(!Point::new(1.0, 0.0).clockwise(Point::new(2.0, 0.0)));
    }

    #[test]
    fn direction_of_three_four_is_a_unit_vector() {
        assert_eq!(Point::new(3.0, 4.0).direction(), Point::new(0.6, 0.8));
    }

    #[test]
    fn direction_of_zero_vector_is_nan() {
        let direction = Point::default().direction();

        assert!(direction.x.is_nan() && direction.y.is_nan());
    }

    #[test]
    fn neg_flips_both_components() {
        assert_eq!(-Point::new(1.0, -2.0), Point::new(-1.0, 2.0));
    }

    #[test]
    fn add_sums_components() {
        assert_eq!(
            Point::new(1.0, 2.0) + Point::new(3.0, 4.0),
            Point::new(4.0, 6.0)
        );
    }

    #[test]
    fn sub_subtracts_components() {
        assert_eq!(
            Point::new(5.0, 7.0) - Point::new(2.0, 3.0),
            Point::new(3.0, 4.0)
        );
    }

    #[test]
    fn mul_scales_components() {
        assert_eq!(Point::new(1.0, -2.0) * 3.0, Point::new(3.0, -6.0));
    }

    #[test]
    fn div_scales_components_down() {
        assert_eq!(Point::new(4.0, -6.0) / 2.0, Point::new(2.0, -3.0));
    }

    #[test]
    fn rem_takes_remainder_of_each_component() {
        assert_eq!(Point::new(5.0, -5.0) % 3.0, Point::new(2.0, -2.0));
    }

    #[test]
    fn interpolate_at_zero_returns_start() {
        let start = Point::new(1.0, 2.0);
        let stop = Point::new(5.0, 10.0);

        assert_eq!(Point::interpolate(start, stop, 0.0), start);
    }

    #[test]
    fn interpolate_at_one_returns_stop() {
        let start = Point::new(1.0, 2.0);
        let stop = Point::new(5.0, 10.0);

        assert_eq!(Point::interpolate(start, stop, 1.0), stop);
    }

    #[test]
    fn interpolate_at_half_returns_midpoint() {
        let start = Point::new(0.0, 0.0);
        let stop = Point::new(4.0, 8.0);

        assert_eq!(Point::interpolate(start, stop, 0.5), Point::new(2.0, 4.0));
    }

    #[test]
    fn transformed_passes_components_to_the_function() {
        let swapped = Point::new(2.0, 3.0).transformed(|x, y| Point::new(y, x));

        assert_eq!(swapped, Point::new(3.0, 2.0));
    }

    #[test]
    fn rotate90_turns_x_axis_into_y_axis() {
        assert_eq!(Point::new(1.0, 0.0).rotate90(), Point::new(0.0, 1.0));
    }

    #[test]
    fn rotate90_maps_one_two_to_negative_two_one() {
        assert_eq!(Point::new(1.0, 2.0).rotate90(), Point::new(-2.0, 1.0));
    }

    #[test]
    fn rotate90_four_times_returns_the_original() {
        let point = Point::new(1.0, 2.0);

        assert_eq!(point.rotate90().rotate90().rotate90().rotate90(), point);
    }
}

#[cfg(test)]
mod float_mapping_tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn near(result: Result<f32, MappingError>, expected: f32) -> bool {
        result.is_ok_and(|value| (value - expected).abs() < EPSILON)
    }

    #[test]
    fn progress_in_range_is_true_inside_a_plain_range() {
        assert!(progress_in_range(0.5, 0.25, 0.75));
    }

    #[test]
    fn progress_in_range_is_false_outside_a_plain_range() {
        assert!(!progress_in_range(0.9, 0.25, 0.75));
    }

    #[test]
    fn progress_in_range_includes_both_ends() {
        assert!(progress_in_range(0.25, 0.25, 0.75) && progress_in_range(0.75, 0.25, 0.75));
    }

    #[test]
    fn progress_in_range_wraps_past_the_end_of_the_cycle() {
        assert!(progress_in_range(0.95, 0.9, 0.1));
    }

    #[test]
    fn progress_in_range_wraps_to_the_start_of_the_cycle() {
        assert!(progress_in_range(0.05, 0.9, 0.1));
    }

    #[test]
    fn progress_in_range_is_false_in_the_gap_of_a_wrapping_range() {
        assert!(!progress_in_range(0.5, 0.9, 0.1));
    }

    #[test]
    fn identity_maps_a_value_in_the_first_segment_to_itself() {
        assert!(near(DoubleMapper::identity().map(0.25), 0.25));
    }

    #[test]
    fn identity_maps_a_value_in_the_wrapping_segment_to_itself() {
        assert!(near(DoubleMapper::identity().map(0.75), 0.75));
    }

    #[test]
    fn identity_maps_back_to_the_same_value() {
        assert!(near(DoubleMapper::identity().map_back(0.6), 0.6));
    }

    #[test]
    fn map_scales_linearly_inside_a_segment() {
        let mapper = DoubleMapper::new([(0.0, 0.0), (0.5, 0.25)]);

        assert!(near(mapper.map(0.25), 0.125));
    }

    #[test]
    fn map_interpolates_across_the_wrapping_segment() {
        let mapper = DoubleMapper::new([(0.0, 0.0), (0.5, 0.25)]);

        assert!(near(mapper.map(0.75), 0.625));
    }

    #[test]
    fn map_hits_the_given_pairs_exactly() {
        let mapper = DoubleMapper::new([(0.0, 0.1), (0.5, 0.6)]);

        assert!(near(mapper.map(0.5), 0.6));
    }

    #[test]
    fn map_back_inverts_map() {
        let mapper = DoubleMapper::new([(0.0, 0.0), (0.5, 0.25)]);

        assert!(near(mapper.map_back(0.125), 0.25));
    }

    #[test]
    fn map_back_inverts_map_in_the_wrapping_segment() {
        let mapper = DoubleMapper::new([(0.0, 0.0), (0.5, 0.25)]);

        assert!(near(mapper.map_back(0.625), 0.75));
    }

    #[test]
    fn map_uses_the_segment_midpoint_when_the_source_segment_is_tiny() {
        let mapper = DoubleMapper::new([(0.0, 0.0), (0.0005, 0.5)]);

        assert!(near(mapper.map(0.0002), 0.25));
    }

    #[test]
    fn map_of_nan_has_no_valid_segment() {
        assert_eq!(
            DoubleMapper::identity().map(f32::NAN),
            Err(MappingError::NoValidSegment)
        );
    }

    #[test]
    fn map_on_an_empty_mapper_has_no_valid_segment() {
        assert_eq!(
            DoubleMapper::new([]).map(0.5),
            Err(MappingError::NoValidSegment)
        );
    }

    #[test]
    fn mapping_error_displays_a_message() {
        assert_eq!(
            MappingError::NoValidSegment.to_string(),
            "no valid segment found"
        );
    }
}
