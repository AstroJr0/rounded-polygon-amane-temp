//! Morphing one rounded polygon into another.
// juicy file of the project

use crate::shapes::cubic::Cubic;
use crate::shapes::feature_mapping::feature_mapper;
use crate::shapes::float_mapping::MappingError;
use crate::shapes::polygon_measure::{LengthMeasurer, MeasuredPolygon};
use crate::shapes::rounded_polygon::RoundedPolygon;
use crate::shapes::utils::{positive_modulo, ANGLE_EPSILON};

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
/// use rounded_polygon_amane::shapes::corner_rounding::CornerRounding;
/// use rounded_polygon_amane::shapes::morph::Morph;
/// use rounded_polygon_amane::shapes::point::Point;
/// use rounded_polygon_amane::shapes::rounded_polygon::RoundedPolygon;
///
/// let square = RoundedPolygon::rectangle(1.0, 1.0, CornerRounding::from_radius(0.2), None, Point::default());
/// let circle = RoundedPolygon::circle(8, 0.5, Point::default());
///
/// let morph = Morph::new(&square, &circle)?;
/// let halfway = morph.as_cubics(0.5);
///
/// assert!(!halfway.is_empty());
/// # Ok::<(), rounded_polygon_amane::shapes::float_mapping::MappingError>(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::corner_rounding::CornerRounding;
    use crate::shapes::feature::Feature;
    use crate::shapes::point::Point;

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
          
