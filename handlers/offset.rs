//! A two-dimensional offset: a point or a displacement in 2D space.

use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

/// An immutable 2D offset.
///
/// An offset is either *specified* (both components are real numbers or
/// infinities) or [`Offset::UNSPECIFIED`], which marks "no value" with `NaN`
/// components.
///
/// # Examples
///
/// ```rust
/// use rounded_polygon_amane::handlers::offset::Offset;
///
/// let a = Offset::new(3.0, 4.0);
/// let b = Offset::new(1.0, 1.0);
///
/// assert_eq!(a + b, Offset::new(4.0, 5.0));
/// assert_eq!(a.distance(), 5.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Offset {
    /// Horizontal component
    pub x: f32,
    /// Vertical component
    pub y: f32,
}

impl Offset {
    /// The origin, `(0, 0)`
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// An offset with both components set to positive infinity
    pub const INFINITE: Self = Self::new(f32::INFINITY, f32::INFINITY);

    /// The "no value" offset, with both components `NaN`
    ///
    /// `NaN != NaN`, so compare with [`Offset::is_unspecified`] rather than `==`
    pub const UNSPECIFIED: Self = Self::new(f32::NAN, f32::NAN);

    /// Creates an offset from its components
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Distance from the origin
    #[must_use]
    pub fn distance(self) -> f32 {
        self.x.hypot(self.y)
    }

    /// Squared distance from the origin; avoids the square root of [`Offset::distance`]
    #[must_use]
    pub fn distance_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Returns `true` if neither component is infinite or `NaN`
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// Returns `true` unless this is [`Offset::UNSPECIFIED`]
    ///
    /// Only an offset whose components are *both* `NaN` counts as unspecified
    #[must_use]
    pub fn is_specified(self) -> bool {
        !self.is_unspecified()
    }

    /// Returns `true` if both components are `NaN`
    #[must_use]
    pub fn is_unspecified(self) -> bool {
        self.x.is_nan() && self.y.is_nan()
    }

    /// **Linearly** interpolates between `start` and `stop`
    ///
    /// A `fraction` of `0.0` yields `start` and `1.0` yields `stop`. Values
    /// outside that range extrapolate.
    #[must_use]
    pub fn lerp(start: Self, stop: Self, fraction: f32) -> Self {
        Self::new(
            start.x + (stop.x - start.x) * fraction,
            start.y + (stop.y - start.y) * fraction,
        )
    }

    /// Returns `self` if it is specified, otherwise the result of `default`
    ///
    /// **`default`** is only called when `self` is unspecified
    #[must_use]
    pub fn take_or_else(self, default: impl FnOnce() -> Self) -> Self {
        if self.is_specified() {
            self
        } else {
            default()
        }
    }

    /// Angle of this offset from the positive x-axis, in **degrees**
    #[must_use]
    pub fn angle_degrees(self) -> f32 {
        self.y.atan2(self.x).to_degrees()
    }

    /// Rotates this offset by `angle` degrees about the origin
    #[must_use]
    pub fn rotate_degrees(self, angle: f32) -> Self {
        self.rotate_degrees_around(angle, Self::ZERO)
    }

    /// Rotates this offset by `angle` degrees about `center`
    #[must_use]
    pub fn rotate_degrees_around(self, angle: f32, center: Self) -> Self {
        let (sin, cos) = angle.to_radians().sin_cos();
        let off = self - center;
        Self::new(off.x * cos - off.y * sin, off.x * sin + off.y * cos) + center
    }
}

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_specified() {
            write!(f, "Offset({:.1}, {:.1})", self.x, self.y)
        } else {
            f.write_str("Offset.Unspecified")
        }
    }
}

impl Neg for Offset {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl Add for Offset {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Offset {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f32> for Offset {
    type Output = Self;

    fn mul(self, operand: f32) -> Self {
        Self::new(self.x * operand, self.y * operand)
    }
}

impl Div<f32> for Offset {
    type Output = Self;

    fn div(self, operand: f32) -> Self {
        Self::new(self.x / operand, self.y / operand)
    }
}

impl Rem<f32> for Offset {
    type Output = Self;

    fn rem(self, operand: f32) -> Self {
        Self::new(self.x % operand, self.y % operand)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    #[test]
    fn new_stores_components() {
        let offset = Offset::new(1.5, -2.5);

        assert_eq!((offset.x, offset.y), (1.5, -2.5));
    }

    #[test]
    fn default_is_zero() {
        assert_eq!(Offset::default(), Offset::ZERO);
    }

    #[test]
    fn distance_of_three_four_is_five() {
        assert_eq!(Offset::new(3.0, 4.0).distance(), 5.0);
    }

    #[test]
    fn distance_squared_of_three_four_is_twenty_five() {
        assert_eq!(Offset::new(3.0, 4.0).distance_squared(), 25.0);
    }

    #[test]
    fn is_finite_is_true_for_regular_values() {
        assert!(Offset::new(1.0, -1.0).is_finite());
    }

    #[test]
    fn is_finite_is_false_for_infinite() {
        assert!(!Offset::INFINITE.is_finite());
    }

    #[test]
    fn is_finite_is_false_for_unspecified() {
        assert!(!Offset::UNSPECIFIED.is_finite());
    }

    #[test]
    fn zero_is_specified() {
        assert!(Offset::ZERO.is_specified());
    }

    #[test]
    fn unspecified_is_not_specified() {
        assert!(Offset::UNSPECIFIED.is_unspecified());
    }

    #[test]
    fn single_nan_component_is_still_specified() {
        assert!(Offset::new(f32::NAN, 1.0).is_specified());
    }

    #[test]
    fn neg_flips_both_components() {
        assert_eq!(-Offset::new(1.0, -2.0), Offset::new(-1.0, 2.0));
    }

    #[test]
    fn add_sums_components() {
        assert_eq!(
            Offset::new(1.0, 2.0) + Offset::new(3.0, 4.0),
            Offset::new(4.0, 6.0)
        );
    }

    #[test]
    fn sub_subtracts_components() {
        assert_eq!(
            Offset::new(5.0, 7.0) - Offset::new(2.0, 3.0),
            Offset::new(3.0, 4.0)
        );
    }

    #[test]
    fn mul_scales_components() {
        assert_eq!(Offset::new(1.0, -2.0) * 3.0, Offset::new(3.0, -6.0));
    }

    #[test]
    fn div_scales_components_down() {
        assert_eq!(Offset::new(4.0, -6.0) / 2.0, Offset::new(2.0, -3.0));
    }

    #[test]
    fn rem_takes_remainder_of_each_component() {
        assert_eq!(Offset::new(5.0, -5.0) % 3.0, Offset::new(2.0, -2.0));
    }

    #[test]
    fn display_formats_specified_with_one_decimal() {
        assert_eq!(Offset::new(1.5, -2.0).to_string(), "Offset(1.5, -2.0)");
    }

    #[test]
    fn display_formats_unspecified() {
        assert_eq!(Offset::UNSPECIFIED.to_string(), "Offset.Unspecified");
    }

    #[test]
    fn lerp_at_zero_returns_start() {
        let start = Offset::new(1.0, 2.0);
        let stop = Offset::new(5.0, 10.0);

        assert_eq!(Offset::lerp(start, stop, 0.0), start);
    }

    #[test]
    fn lerp_at_one_returns_stop() {
        let start = Offset::new(1.0, 2.0);
        let stop = Offset::new(5.0, 10.0);

        assert_eq!(Offset::lerp(start, stop, 1.0), stop);
    }

    #[test]
    fn lerp_at_half_returns_midpoint() {
        let start = Offset::new(0.0, 0.0);
        let stop = Offset::new(4.0, 8.0);

        assert_eq!(Offset::lerp(start, stop, 0.5), Offset::new(2.0, 4.0));
    }

    #[test]
    fn take_or_else_keeps_specified_value_without_calling_default() {
        let offset = Offset::new(1.0, 2.0);

        assert_eq!(offset.take_or_else(|| unreachable!()), offset);
    }

    #[test]
    fn take_or_else_returns_default_when_unspecified() {
        let fallback = Offset::new(7.0, 8.0);

        assert_eq!(Offset::UNSPECIFIED.take_or_else(|| fallback), fallback);
    }

    #[test]
    fn angle_degrees_of_positive_y_axis_is_ninety() {
        let angle = Offset::new(0.0, 1.0).angle_degrees();

        assert!((angle - 90.0).abs() < EPSILON);
    }

    #[test]
    fn rotate_degrees_quarter_turn_maps_x_axis_to_y_axis() {
        let rotated = Offset::new(1.0, 0.0).rotate_degrees(90.0);

        assert!(rotated.x.abs() < EPSILON && (rotated.y - 1.0).abs() < EPSILON);
    }

    #[test]
    fn rotate_degrees_around_center_half_turn_mirrors_through_center() {
        let rotated = Offset::new(2.0, 1.0).rotate_degrees_around(180.0, Offset::new(1.0, 1.0));

        assert!(rotated.x.abs() < EPSILON && (rotated.y - 1.0).abs() < EPSILON);
    }
}
