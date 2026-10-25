//! Measuring a polygon's outline, so it can be cut at a given progress.

use crate::shapes::cubic::Cubic;
use crate::shapes::feature::Feature;
use crate::shapes::feature_mapping::ProgressableFeature;
use crate::shapes::point::Point;
use crate::shapes::rounded_polygon::RoundedPolygon;
use crate::shapes::utils::{coerce_in, positive_modulo, DISTANCE_EPSILON};

/// Measures curves by the length of a few straight segments along them.
///
/// # Examples
///
/// ```
/// use rounded_polygon_amane::shapes::cubic::Cubic;
/// use rounded_polygon_amane::shapes::polygon_measure::LengthMeasurer;
///
/// let measurer = LengthMeasurer::default();
/// let short = measurer.measure_cubic(&Cubic::straight_line(0.0, 0.0, 1.0, 0.0));
/// let long = measurer.measure_cubic(&Cubic::straight_line(0.0, 0.0, 2.0, 0.0));
///
/// assert!(long > short);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LengthMeasurer {
    segments: u32,
}

impl Default for LengthMeasurer {
    /// A measurer that uses 3 segments per curve.
    fn default() -> Self {
        Self { segments: 3 }
    }
}

impl LengthMeasurer {
    /// The approximate length of `cubic`.
    #[must_use]
    pub fn measure_cubic(&self, cubic: &Cubic) -> f32 {
        self.closest_progress_to(cubic, f32::INFINITY).1
    }

    /// The curve parameter at which `cubic` has reached the length `measure`.
    #[must_use]
    pub fn find_cubic_cut_point(&self, cubic: &Cubic, measure: f32) -> f32 {
        self.closest_progress_to(cubic, measure).0
    }

    /// Walks along `cubic` and returns `(parameter, length)`: the parameter at
    /// which `threshold` length has been covered together with `threshold`, or
    /// `(1.0, total length)` if the curve is shorter than that.
    fn closest_progress_to(&self, cubic: &Cubic, threshold: f32) -> (f32, f32) {
        let segments = self.segments as f32;
        let mut total = 0.0;
        let mut remainder = threshold;
        let mut previous = Point::new(cubic.anchor0_x(), cubic.anchor0_y());

        for i in 1..self.segments {
            let progress = i as f32 / segments;
            let point = cubic.point_on_curve(progress);
            let segment = (point - previous).distance();

            if segment >= remainder {
                return (progress - (1.0 - remainder / segment) / segments, threshold);
            }

            remainder -= segment;
            total += segment;
            previous = point;
        }

        (1.0, total)
    }
}

/// A curve together with the stretch of the polygon's outline it covers.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasuredCubic {
    /// The curve.
    pub cubic: Cubic,
    /// Outline progress at the start of the curve, in `0..1`.
    pub start_outline_progress: f32,
    /// Outline progress at the end of the curve, in `0..1`.
    pub end_outline_progress: f32,
    /// The measured length of the curve.
    pub measured_size: f32,
}

impl MeasuredCubic {
    /// Measures `cubic` and records the outline progress range it covers.
    #[must_use]
    pub fn new(
        measurer: &LengthMeasurer,
        cubic: Cubic,
        start_outline_progress: f32,
        end_outline_progress: f32,
    ) -> Self {
        let measured_size = measurer.measure_cubic(&cubic);
        Self {
            cubic,
            start_outline_progress,
            end_outline_progress,
            measured_size,
        }
    }

    /// Cuts the curve at `cut_outline_progress`, which is limited to the range
    /// the curve covers, and returns the part before and the part after.
    #[must_use]
    pub fn cut_at_progress(
        &self,
        measurer: &LengthMeasurer,
        cut_outline_progress: f32,
    ) -> (Self, Self) {
        let bounded = coerce_in(
            cut_outline_progress,
            self.start_outline_progress,
            self.end_outline_progress,
        );
        let outline_progress_size = self.end_outline_progress - self.start_outline_progress;
        let progress_from_start = bounded - self.start_outline_progress;

        let relative_progress = progress_from_start / outline_progress_size;
        let t = measurer.find_cubic_cut_point(&self.cubic, relative_progress * self.measured_size);

        let (before, after) = self.cubic.split(t);
        (
            Self::new(measurer, before, self.start_outline_progress, bounded),
            Self::new(measurer, after, bounded, self.end_outline_progress),
        )
    }
}

/// A polygon whose outline has been measured: each curve knows which stretch
/// of the outline, in `0..1`, it covers.
///
/// # Examples
///
/// ```
/// use rounded_polygon_amane::shapes::corner_rounding::CornerRounding;
/// use rounded_polygon_amane::shapes::point::Point;
/// use rounded_polygon_amane::shapes::polygon_measure::{LengthMeasurer, MeasuredPolygon};
/// use rounded_polygon_amane::shapes::rounded_polygon::RoundedPolygon;
///
/// let square = RoundedPolygon::rectangle(1.0, 1.0, CornerRounding::UNROUNDED, None, Point::default());
/// let measured = MeasuredPolygon::measure_polygon(LengthMeasurer::default(), &square);
///
/// assert_eq!(measured.cubics().len(), 4);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MeasuredPolygon {
    measurer: LengthMeasurer,
    features: Vec<ProgressableFeature>,
    cubics: Vec<MeasuredCubic>,
    outline_progress: Vec<f32>,
}

impl MeasuredPolygon {
    /// Creates a measured polygon from `cubics` and the outline progress at
    /// each of their boundaries.
    ///
    /// `outline_progress` holds one more value than `cubics`: the progress at
    /// the start of the first curve, then at the end of every curve. Curves
    /// that cover (almost) no outline are dropped, and the last curve is
    /// stretched to end at exactly `1.0`.
    #[must_use]
    pub fn new(
        measurer: LengthMeasurer,
        features: Vec<ProgressableFeature>,
        cubics: &[Cubic],
        outline_progress: Vec<f32>,
    ) -> Self {
        let mut measured_cubics: Vec<MeasuredCubic> = Vec::new();
        let mut start_outline_progress = 0.0;
        for (cubic, ends) in cubics.iter().zip(outline_progress.windows(2)) {
            let end = ends[1];
            if end - ends[0] > DISTANCE_EPSILON {
                measured_cubics.push(MeasuredCubic::new(
                    &measurer,
                    cubic.clone(),
                    start_outline_progress,
                    end,
                ));
                // The next curve starts exactly where this one ends.
                start_outline_progress = end;
            }
        }

        if let Some(last) = measured_cubics.last_mut() {
            last.end_outline_progress = 1.0;
        }

        Self {
            measurer,
            features,
            cubics: measured_cubics,
            outline_progress,
        }
    }

    /// The corners of the polygon and where each sits along the outline.
    #[must_use]
    pub fn features(&self) -> &[ProgressableFeature] {
        &self.features
    }

    /// The measured curves, in outline order.
    #[must_use]
    pub fn cubics(&self) -> &[MeasuredCubic] {
        &self.cubics
    }

    /// The outline progress at each curve boundary, as passed to [`MeasuredPolygon::new`].
    #[must_use]
    pub fn outline_progress(&self) -> &[f32] {
        &self.outline_progress
    }

    /// The measurer used for this polygon.
    #[must_use]
    pub const fn measurer(&self) -> LengthMeasurer {
        self.measurer
    }

    /// Cuts the outline at `cutting_point` and makes that point the new start.
    ///
    /// Returns an unchanged copy if the point is at the very start, or lies in
    /// no curve.
    #[must_use]
    pub fn cut_and_shift(&self, cutting_point: f32) -> Self {
        if cutting_point < DISTANCE_EPSILON {
            return self.clone();
        }

        // Find the curve to cut.
        let Some(target_index) = self.cubics.iter().position(|it| {
            cutting_point >= it.start_outline_progress && cutting_point <= it.end_outline_progress
        }) else {
            return self.clone();
        };
        let (before, after) =
            self.cubics[target_index].cut_at_progress(&self.measurer, cutting_point);

        // The cubics we need, in order: the part of the target after the cut,
        // every curve after the target (wrapping round to those before it),
        // and finally the part of the target before the cut.
        let count = self.cubics.len();
        let mut cubics = vec![after.cubic];
        cubics.extend((1..count).map(|i| self.cubics[(i + target_index) % count].cubic.clone()));
        cubics.push(before.cubic);

        // The outline progress shifts so the cut becomes 0. For curves ending
        // at [0.3, 0.8, 1.0], cut at 0.6, that is 0.0, 0.2, 0.4, 0.7, 1.0:
        // the cut itself, then 0.8 - 0.6, 1.0 - 0.6, 0.3 - 0.6 + 1, and the
        // cut again.
        let mut outline_progress = vec![0.0];
        outline_progress.extend((1..=count).map(|i| {
            let cubic_index = (target_index + i - 1) % count;
            positive_modulo(
                self.cubics[cubic_index].end_outline_progress - cutting_point,
                1.0,
            )
        }));
        outline_progress.push(1.0);

        let features = self
            .features
            .iter()
            .map(|f| {
                ProgressableFeature::new(
                    positive_modulo(f.progress - cutting_point, 1.0),
                    f.feature.clone(),
                )
            })
            .collect();

        Self::new(self.measurer, features, &cubics, outline_progress)
    }

    /// Measures the outline of `polygon`.
    #[must_use]
    pub fn measure_polygon(measurer: LengthMeasurer, polygon: &RoundedPolygon) -> Self {
        let mut cubics: Vec<Cubic> = Vec::new();
        // For each corner: the corner and the index of its middle curve.
        let mut feature_to_cubic: Vec<(&Feature, usize)> = Vec::new();

        for feature in polygon.features() {
            for (cubic_index, cubic) in feature.cubics().iter().enumerate() {
                if matches!(feature, Feature::Corner { .. })
                    && cubic_index == feature.cubics().len() / 2
                {
                    feature_to_cubic.push((feature, cubics.len()));
                }
                cubics.push(cubic.clone());
            }
        }

        let mut measures = vec![0.0];
        let mut total_measure = 0.0;
        for cubic in &cubics {
            total_measure += measurer.measure_cubic(cubic);
            measures.push(total_measure);
        }

        let outline_progress: Vec<f32> = measures.iter().map(|m| m / total_measure).collect();

        let features = feature_to_cubic
            .into_iter()
            .map(|(feature, ix)| {
                ProgressableFeature::new(
                    positive_modulo((outline_progress[ix] + outline_progress[ix + 1]) / 2.0, 1.0),
                    feature.clone(),
                )
            })
            .collect();

        Self::new(measurer, features, &cubics, outline_progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::corner_rounding::CornerRounding;

    const EPSILON: f32 = 1e-4;

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn unit_square() -> RoundedPolygon {
        RoundedPolygon::rectangle(1.0, 1.0, CornerRounding::UNROUNDED, None, Point::default())
    }

    fn measured_square() -> MeasuredPolygon {
        MeasuredPolygon::measure_polygon(LengthMeasurer::default(), &unit_square())
    }

    #[test]
    fn measure_cubic_of_a_longer_line_is_larger() {
        let measurer = LengthMeasurer::default();
        let short = measurer.measure_cubic(&Cubic::straight_line(0.0, 0.0, 1.0, 0.0));
        let long = measurer.measure_cubic(&Cubic::straight_line(0.0, 0.0, 2.0, 0.0));

        assert!(long > short);
    }

    #[test]
    fn measure_cubic_scales_with_the_length_of_the_line() {
        let measurer = LengthMeasurer::default();
        let short = measurer.measure_cubic(&Cubic::straight_line(0.0, 0.0, 1.0, 0.0));
        let long = measurer.measure_cubic(&Cubic::straight_line(0.0, 0.0, 2.0, 0.0));

        assert!(near(long, short * 2.0));
    }

    #[test]
    fn measure_cubic_of_a_zero_length_curve_is_zero() {
        // Evaluating a degenerate curve leaves a few ulps of rounding error.
        assert!(near(
            LengthMeasurer::default().measure_cubic(&Cubic::empty(1.0, 1.0)),
            0.0
        ));
    }

    #[test]
    fn find_cubic_cut_point_at_zero_is_the_start() {
        let line = Cubic::straight_line(0.0, 0.0, 3.0, 0.0);

        assert!(near(
            LengthMeasurer::default().find_cubic_cut_point(&line, 0.0),
            0.0
        ));
    }

    #[test]
    fn find_cubic_cut_point_grows_with_the_measure() {
        let measurer = LengthMeasurer::default();
        let line = Cubic::straight_line(0.0, 0.0, 3.0, 0.0);

        assert!(
            measurer.find_cubic_cut_point(&line, 1.5) > measurer.find_cubic_cut_point(&line, 0.5)
        );
    }

    #[test]
    fn find_cubic_cut_point_past_the_end_is_one() {
        let line = Cubic::straight_line(0.0, 0.0, 3.0, 0.0);

        assert_eq!(
            LengthMeasurer::default().find_cubic_cut_point(&line, 100.0),
            1.0
        );
    }

    #[test]
    fn measure_polygon_finds_one_feature_per_corner() {
        assert_eq!(measured_square().features().len(), 4);
    }

    #[test]
    fn measure_polygon_drops_the_zero_length_corner_curves() {
        assert_eq!(measured_square().cubics().len(), 4);
    }

    #[test]
    fn measure_polygon_spreads_a_square_evenly_over_the_outline() {
        let progress: Vec<f32> = measured_square()
            .features()
            .iter()
            .map(|f| f.progress)
            .collect();

        assert!(progress
            .iter()
            .zip([0.0, 0.25, 0.5, 0.75])
            .all(|(a, b)| near(*a, b)));
    }

    #[test]
    fn measure_polygon_starts_at_zero_and_ends_at_one() {
        let measured = measured_square();

        assert_eq!(measured.cubics()[0].start_outline_progress, 0.0);
        assert_eq!(measured.cubics()[3].end_outline_progress, 1.0);
    }

    #[test]
    fn measure_polygon_gives_neighbouring_curves_matching_boundaries() {
        let measured = measured_square();

        assert!(measured
            .cubics()
            .windows(2)
            .all(|w| w[0].end_outline_progress == w[1].start_outline_progress));
    }

    #[test]
    fn measure_polygon_records_the_outline_progress_of_every_curve_boundary() {
        // 4 edges and 4 zero-length corner curves, plus the start.
        assert_eq!(measured_square().outline_progress().len(), 9);
    }

    #[test]
    fn measure_polygon_assigns_a_rounded_corner_the_progress_of_its_middle_curve() {
        let rounded = RoundedPolygon::rectangle(
            1.0,
            1.0,
            CornerRounding::from_radius(0.2),
            None,
            Point::default(),
        );

        let measured = MeasuredPolygon::measure_polygon(LengthMeasurer::default(), &rounded);

        assert_eq!(measured.features().len(), 4);
    }

    #[test]
    fn cut_and_shift_at_the_start_changes_nothing() {
        let measured = measured_square();

        assert_eq!(measured.cut_and_shift(0.0), measured);
    }

    #[test]
    fn cut_and_shift_keeps_every_curve_and_adds_the_cut_one() {
        let measured = measured_square();

        // One curve is cut in two, so there is one more than before.
        assert_eq!(measured.cut_and_shift(0.6).cubics().len(), 5);
    }

    #[test]
    fn cut_and_shift_starts_at_zero_and_ends_at_one() {
        let shifted = measured_square().cut_and_shift(0.6);

        assert_eq!(shifted.cubics()[0].start_outline_progress, 0.0);
        assert_eq!(
            shifted.cubics().last().map(|c| c.end_outline_progress),
            Some(1.0)
        );
    }

    #[test]
    fn cut_and_shift_shifts_the_feature_progress_by_the_cut() {
        let shifted = measured_square().cut_and_shift(0.6);
        let progress: Vec<f32> = shifted.features().iter().map(|f| f.progress).collect();

        // 0.0, 0.25, 0.5, 0.75 minus 0.6, wrapped into 0..1.
        assert!(progress
            .iter()
            .zip([0.4, 0.65, 0.9, 0.15])
            .all(|(a, b)| near(*a, b)));
    }

    #[test]
    fn cut_and_shift_keeps_the_outline_closed() {
        let shifted = measured_square().cut_and_shift(0.6);

        assert!(shifted
            .cubics()
            .windows(2)
            .all(|w| near(w[0].end_outline_progress, w[1].start_outline_progress)));
    }

    #[test]
    fn measured_cubic_cut_at_progress_splits_the_range_at_the_cut() {
        let measurer = LengthMeasurer::default();
        let cubic = MeasuredCubic::new(
            &measurer,
            Cubic::straight_line(0.0, 0.0, 3.0, 0.0),
            0.2,
            0.6,
        );

        let (before, after) = cubic.cut_at_progress(&measurer, 0.4);

        assert_eq!(
            (
                before.start_outline_progress,
                before.end_outline_progress,
                after.start_outline_progress,
                after.end_outline_progress
            ),
            (0.2, 0.4, 0.4, 0.6)
        );
    }

    #[test]
    fn measured_cubic_cut_at_progress_limits_the_cut_to_its_own_range() {
        let measurer = LengthMeasurer::default();
        let cubic = MeasuredCubic::new(
            &measurer,
            Cubic::straight_line(0.0, 0.0, 3.0, 0.0),
            0.2,
            0.6,
        );

        let (before, _) = cubic.cut_at_progress(&measurer, 0.9);

        assert_eq!(before.end_outline_progress, 0.6);
    }

    #[test]
    fn measured_cubic_cut_at_progress_pieces_meet_at_the_cut() {
        let measurer = LengthMeasurer::default();
        let cubic = MeasuredCubic::new(
            &measurer,
            Cubic::straight_line(0.0, 0.0, 3.0, 0.0),
            0.0,
            1.0,
        );

        let (before, after) = cubic.cut_at_progress(&measurer, 0.5);

        assert!(near(before.cubic.anchor1_x(), after.cubic.anchor0_x()));
    }

    #[test]
    fn new_drops_curves_that_cover_no_outline() {
        let cubics = [
            Cubic::empty(0.0, 0.0),
            Cubic::straight_line(0.0, 0.0, 1.0, 0.0),
        ];

        let measured = MeasuredPolygon::new(
            LengthMeasurer::default(),
            vec![],
            &cubics,
            vec![0.0, 0.0, 1.0],
        );

        assert_eq!(measured.cubics().len(), 1);
    }

    #[test]
    fn new_stretches_the_last_curve_to_the_end_of_the_outline() {
        let cubics = [Cubic::straight_line(0.0, 0.0, 1.0, 0.0)];

        let measured = MeasuredPolygon::new(
            LengthMeasurer::default(),
            vec![],
            &cubics,
            vec![0.0, 0.9999],
        );

        assert_eq!(measured.cubics()[0].end_outline_progress, 1.0);
    }
}

