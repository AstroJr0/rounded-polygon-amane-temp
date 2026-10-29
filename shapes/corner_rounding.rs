//! Rounding configuration for a single polygon corner.

/// How a single corner of a polygon is rounded.
///
/// # Examples
///
/// ```rust
/// use rounded_polygon_amane::shapes::corner_rounding::CornerRounding;
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

#[cfg(test)]
mod tests {
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
