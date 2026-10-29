//! Measuring a polygon's outline, so it can be cut at a given progress.
//!
//! Morphing one rounded polygon into another.
//!
//! Cubic Bezier curves.

use std::array;
use std::ops::{Add, Div, Mul};

use super::features::{feature_mapper, Feature, ProgressableFeature};
use super::points::{MappingError, Point};
use super::rounded_polygon::RoundedPolygon;
use super::utils::{self, coerce_in, positive_modulo, ANGLE_EPSILON, DISTANCE_EPSILON};

/// Measures curves by the length of a few straight segments along them.
///
/// # Examples
///
/// ```
/// use rounded_polygon_amane::shapes::polygon_manager::Cubic;
/// use rounded_polygon_amane::shapes::polygon_manager::LengthMeasurer;
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
/// use rounded_polygon_amane::shapes::corners::CornerRounding;
/// use rounded_polygon_amane::shapes::points::Point;
/// use rounded_polygon_amane::shapes::polygon_manager::{LengthMeasurer, MeasuredPolygon};
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

// juicy file of the project

/// A transition between two rounded polygons.
///
/// The outlines of both polygons are cut so that they consist of the same
/// number of curves, with like corners lined up. Any progress between `0.0`
/// (the start polygon) and `1.0` (the end polygon) can then be read off by
/// interpolating curve by curve.
///
/// # Examples
///
/// ```
/// use rounded_polygon_amane::shapes::corners::CornerRounding;
/// use rounded_polygon_amane::shapes::polygon_manager::Morph;
/// use rounded_polygon_amane::shapes::points::Point;
/// use rounded_polygon_amane::shapes::rounded_polygon::RoundedPolygon;
///
/// let square = RoundedPolygon::rectangle(1.0, 1.0, CornerRounding::from_radius(0.2), None, Point::default());
/// let circle = RoundedPolygon::circle(8, 0.5, Point::default());
///
/// let morph = Morph::new(&square, &circle)?;
/// let halfway = morph.as_cubics(0.5);
///
/// assert!(!halfway.is_empty());
/// # Ok::<(), rounded_polygon_amane::shapes::points::MappingError>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Morph {
    morph_match: Vec<(Cubic, Cubic)>,
}

impl Morph {
    /// Prepares a morph from `start` to `end`.
    ///
    /// # Errors
    ///
    /// Returns a [`MappingError`] if the corners of the two polygons cannot be
    /// matched up, for example because a polygon has `NaN` coordinates.
    pub fn new(start: &RoundedPolygon, end: &RoundedPolygon) -> Result<Self, MappingError> {
        Ok(Self {
            morph_match: match_polygons(start, end)?,
        })
    }

    /// The pairs of start and end curves that are interpolated between.
    #[must_use]
    pub fn morph_match(&self) -> &[(Cubic, Cubic)] {
        &self.morph_match
    }

    /// The outline at `progress`, where `0.0` is the start polygon and `1.0`
    /// the end polygon. The curves form a closed loop.
    #[must_use]
    pub fn as_cubics(&self, progress: f32) -> Vec<Cubic> {
        let mut result = Vec::with_capacity(self.morph_match.len());

        // Tracking the first and last curve makes the loop end exactly where
        // it began. Anchors that are off by even a fraction of a pixel can
        // cause rendering artifacts.
        let mut first_cubic: Option<Cubic> = None;
        let mut last_cubic: Option<Cubic> = None;
        for (start, end) in &self.morph_match {
            let cubic = Cubic::interpolate(start, end, progress);
            if first_cubic.is_none() {
                first_cubic = Some(cubic.clone());
            }
            if let Some(last) = last_cubic.take() {
                result.push(last);
            }
            last_cubic = Some(cubic);
        }

        if let (Some(last), Some(first)) = (last_cubic, first_cubic) {
            result.push(Cubic::new([
                last.anchor0_x(),
                last.anchor0_y(),
                last.control0_x(),
                last.control0_y(),
                last.control1_x(),
                last.control1_y(),
                first.anchor0_x(),
                first.anchor0_y(),
            ]));
        }
        result
    }

    /// Calls `callback` with each interpolated curve at `progress`, in order.
    ///
    /// Unlike [`Morph::as_cubics`], the last curve is not adjusted to close the loop.
    pub fn for_each_cubic(&self, progress: f32, mut callback: impl FnMut(&Cubic)) {
        for (start, end) in &self.morph_match {
            callback(&Cubic::interpolate(start, end, progress));
        }
    }
}

/// Pairs up pieces of the two outlines.
///
/// Both outlines are measured and their corners matched. The second outline is
/// rotated so its matching corner comes first, then both are walked together,
/// cutting whichever curve is longer so that every piece on one side has a
/// piece on the other covering the same stretch.
fn match_polygons(
    p1: &RoundedPolygon,
    p2: &RoundedPolygon,
) -> Result<Vec<(Cubic, Cubic)>, MappingError> {
    let measurer = LengthMeasurer::default();
    let measured_polygon1 = MeasuredPolygon::measure_polygon(measurer, p1);
    let measured_polygon2 = MeasuredPolygon::measure_polygon(measurer, p2);

    let double_mapper = feature_mapper(measured_polygon1.features(), measured_polygon2.features());

    // Where the second outline has to be cut so its start matches the first.
    let polygon2_cut_point = double_mapper.map(0.0)?;

    let bs1 = measured_polygon1;
    let bs2 = measured_polygon2.cut_and_shift(polygon2_cut_point);

    let mut result = Vec::new();

    let mut i1 = 0;
    let mut i2 = 0;
    let mut b1 = bs1.cubics().get(i1).cloned();
    i1 += 1;
    let mut b2 = bs2.cubics().get(i2).cloned();
    i2 += 1;

    while let (Some(current1), Some(current2)) = (b1.take(), b2.take()) {
        // How far along the outline each current curve reaches, measured on
        // the first outline's scale.
        let b1a = if i1 == bs1.cubics().len() {
            1.0
        } else {
            current1.end_outline_progress
        };
        let b2a = if i2 == bs2.cubics().len() {
            1.0
        } else {
            double_mapper.map_back(positive_modulo(
                current2.end_outline_progress + polygon2_cut_point,
                1.0,
            ))?
        };
        let min_end = b1a.min(b2a);

        // The curve reaching further is cut at the shorter one's end; the
        // other is used whole and the next curve takes its place.
        let (segment1, next1) = if b1a > min_end + ANGLE_EPSILON {
            let (segment, rest) = current1.cut_at_progress(&measurer, min_end);
            (segment, Some(rest))
        } else {
            let next = bs1.cubics().get(i1).cloned();
            i1 += 1;
            (current1, next)
        };
        let (segment2, next2) = if b2a > min_end + ANGLE_EPSILON {
            let cut = positive_modulo(double_mapper.map(min_end)? - polygon2_cut_point, 1.0);
            let (segment, rest) = current2.cut_at_progress(&measurer, cut);
            (segment, Some(rest))
        } else {
            let next = bs2.cubics().get(i2).cloned();
            i2 += 1;
            (current2, next)
        };

        result.push((segment1.cubic, segment2.cubic));
        b1 = next1;
        b2 = next2;
    }

    Ok(result)
}

/// A cubic Bézier curve made of two anchor points and two control points.
///
/// The eight coordinates are stored as
/// `[anchor0.x, anchor0.y, control0.x, control0.y, control1.x, control1.y, anchor1.x, anchor1.y]`.
///
/// # Examples
///
/// ```rut
/// use rounded_polygon_amane::shapes::polygon_manager::Cubic;
/// use rounded_polygon_amane::shapes::points::Point;
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
mod polygon_measure_tests {
    use super::super::corners::CornerRounding;
    use super::*;

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

#[cfg(test)]
mod morph_tests {
    use super::super::corners::CornerRounding;
    use super::super::features::Feature;
    use super::super::points::Point;
    use super::*;

    const EPSILON: f32 = 1e-3;

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn is_closed_loop(cubics: &[Cubic]) -> bool {
        cubics.iter().enumerate().all(|(i, cubic)| {
            let next = &cubics[(i + 1) % cubics.len()];
            near(cubic.anchor1_x(), next.anchor0_x()) && near(cubic.anchor1_y(), next.anchor0_y())
        })
    }

    fn near_bounds(actual: [f32; 4], expected: [f32; 4]) -> bool {
        actual.iter().zip(expected).all(|(a, b)| near(*a, b))
    }

    fn exact_bounds(cubics: &[Cubic]) -> [f32; 4] {
        cubics.iter().map(|c| c.calculate_bounds(false)).fold(
            [
                f32::INFINITY,
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::NEG_INFINITY,
            ],
            |a, b| {
                [
                    a[0].min(b[0]),
                    a[1].min(b[1]),
                    a[2].max(b[2]),
                    a[3].max(b[3]),
                ]
            },
        )
    }

    fn morph(start: &RoundedPolygon, end: &RoundedPolygon) -> Morph {
        Morph::new(start, end).expect("the polygons' corners can be matched")
    }

    fn rounded_square() -> RoundedPolygon {
        RoundedPolygon::rectangle(
            1.0,
            1.0,
            CornerRounding::from_radius(0.2),
            None,
            Point::default(),
        )
    }

    fn circle() -> RoundedPolygon {
        RoundedPolygon::circle(8, 0.5, Point::default())
    }

    fn triangle() -> RoundedPolygon {
        RoundedPolygon::from_num_vertices(
            3,
            0.6,
            Point::default(),
            CornerRounding::from_radius(0.1),
            None,
        )
    }

    #[test]
    fn new_pairs_every_start_curve_with_an_end_curve() {
        let morph = Morph::new(&rounded_square(), &circle());

        assert!(morph.is_ok_and(|m| !m.morph_match().is_empty()));
    }

    #[test]
    fn as_cubics_of_identical_polygons_has_one_curve_per_visible_feature_curve() {
        let square = rounded_square();
        let visible_curves = square
            .features()
            .iter()
            .flat_map(Feature::cubics)
            .filter(|cubic| !cubic.zero_length())
            .count();

        let morph = morph(&square, &square);

        assert_eq!(morph.as_cubics(0.5).len(), visible_curves);
    }

    #[test]
    fn as_cubics_of_identical_polygons_reproduces_the_polygon() {
        let square = rounded_square();

        let morph = morph(&square, &square);

        assert!(near_bounds(
            exact_bounds(&morph.as_cubics(0.5)),
            square.calculate_bounds(false)
        ));
    }

    #[test]
    fn as_cubics_at_zero_traces_the_start_polygon() {
        let (start, end) = (rounded_square(), circle());

        let morph = morph(&start, &end);

        assert!(near_bounds(
            exact_bounds(&morph.as_cubics(0.0)),
            start.calculate_bounds(false)
        ));
    }

    #[test]
    fn as_cubics_at_one_traces_the_end_polygon() {
        let (start, end) = (rounded_square(), circle());

        let morph = morph(&start, &end);

        assert!(near_bounds(
            exact_bounds(&morph.as_cubics(1.0)),
            end.calculate_bounds(false)
        ));
    }

    #[test]
    fn as_cubics_halfway_lies_between_the_two_shapes() {
        let (start, end) = (rounded_square(), circle());

        let morph = morph(&start, &end);
        let bounds = exact_bounds(&morph.as_cubics(0.5));

        // The square reaches 0.5 and the circle 0.5, so halfway cannot exceed that by much.
        assert!(bounds[2] <= start.calculate_bounds(false)[2] + EPSILON);
    }

    #[test]
    fn as_cubics_is_a_closed_loop_at_every_progress() {
        let morph = morph(&rounded_square(), &triangle());

        assert!([0.0, 0.25, 0.5, 0.75, 1.0]
            .iter()
            .all(|&progress| is_closed_loop(&morph.as_cubics(progress))));
    }

    #[test]
    fn as_cubics_is_a_closed_loop_between_square_and_circle() {
        let morph = morph(&rounded_square(), &circle());

        assert!(is_closed_loop(&morph.as_cubics(0.5)));
    }

    #[test]
    fn as_cubics_is_a_closed_loop_between_circle_and_triangle() {
        let morph = morph(&circle(), &triangle());

        assert!(is_closed_loop(&morph.as_cubics(0.3)));
    }

    #[test]
    fn as_cubics_has_as_many_curves_as_the_match() {
        let morph = morph(&rounded_square(), &triangle());

        assert_eq!(morph.as_cubics(0.5).len(), morph.morph_match().len());
    }

    #[test]
    fn for_each_cubic_visits_every_matched_pair() {
        let morph = morph(&rounded_square(), &circle());
        let mut count = 0;

        morph.for_each_cubic(0.5, |_| count += 1);

        assert_eq!(count, morph.morph_match().len());
    }

    #[test]
    fn for_each_cubic_at_zero_passes_the_start_curves() {
        let morph = morph(&rounded_square(), &circle());
        let mut curves = Vec::new();

        morph.for_each_cubic(0.0, |c| curves.push(c.clone()));

        assert_eq!(curves[0], morph.morph_match()[0].0);
    }

    #[test]
    fn matched_pairs_each_join_up_on_both_outlines() {
        let morph = morph(&rounded_square(), &triangle());
        let starts: Vec<Cubic> = morph.morph_match().iter().map(|(a, _)| a.clone()).collect();
        let ends: Vec<Cubic> = morph.morph_match().iter().map(|(_, b)| b.clone()).collect();

        assert!(is_closed_loop(&starts) && is_closed_loop(&ends));
    }

    #[test]
    fn morphing_a_circle_into_a_star_works() {
        let star = RoundedPolygon::star(
            5,
            0.5,
            0.25,
            CornerRounding::from_radius(0.05),
            None,
            None,
            Point::default(),
        );

        let morph = morph(&circle(), &star);

        assert!(is_closed_loop(&morph.as_cubics(0.5)));
    }
}

#[cfg(test)]
mod cubic_tests {
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
