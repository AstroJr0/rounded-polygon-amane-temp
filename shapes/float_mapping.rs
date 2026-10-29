//! Mapping progress values between two cyclic ranges.

use std::error::Error;
use std::fmt;

use super::utils::positive_modulo;

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
/// use rounded_polygon_amane::shapes::float_mapping::DoubleMapper;
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
mod tests {
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
