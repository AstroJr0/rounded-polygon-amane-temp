//! The Material 3 Expressive shape set, as ready-made [`RoundedPolygon`]s.
//!
//! Every function returns a polygon normalized to fit `0..1`. Each shape is
//! built once, on first use, and cloned out after that.
//!
//! ```rust,ignore
//! ShapeCanvas::new().rounded_polygon(material_shapes::heart)
//! ```

use std::sync::LazyLock;

use super::super::handlers::matrix::Matrix;
use super::super::handlers::offset::Offset;
use super::super::shapes::corners::CornerRounding;
use super::super::shapes::points::Point;
use super::super::shapes::rounded_polygon::RoundedPolygon;
use super::super::shapes::utils::{custom_polygon, PointNRound};

const ROUND_15: CornerRounding = CornerRounding::from_radius(0.15);
const ROUND_20: CornerRounding = CornerRounding::from_radius(0.2);
const ROUND_30: CornerRounding = CornerRounding::from_radius(0.3);
const ROUND_50: CornerRounding = CornerRounding::from_radius(0.5);
const ROUND_100: CornerRounding = CornerRounding::from_radius(1.0);

/// A corner at `(x, y)` with no rounding.
fn sharp(x: f32, y: f32) -> PointNRound {
    PointNRound::unrounded(Offset::new(x, y))
}

/// A corner at `(x, y)` rounded by `radius`.
fn round(x: f32, y: f32, radius: f32) -> PointNRound {
    PointNRound::new(Offset::new(x, y), CornerRounding::from_radius(radius))
}

/// A corner at `(x, y)` rounded by `radius`, smoothed by `smoothing`.
fn smooth(x: f32, y: f32, radius: f32, smoothing: f32) -> PointNRound {
    PointNRound::new(Offset::new(x, y), CornerRounding::new(radius, smoothing))
}

/// A corner at `(x, y)` with the given rounding.
fn with(x: f32, y: f32, rounding: CornerRounding) -> PointNRound {
    PointNRound::new(Offset::new(x, y), rounding)
}

/// A polygon through `points`, repeated `reps` times around `(0.5, 0.5)`.
fn custom(points: &[PointNRound], reps: usize) -> RoundedPolygon {
    custom_polygon(points, reps, Offset::new(0.5, 0.5), false)
}

fn rotation(degrees: f32) -> Matrix {
    let mut matrix = Matrix::IDENTITY;
    matrix.rotate_z(degrees);
    matrix
}

fn scaling(x: f32, y: f32) -> Matrix {
    let mut matrix = Matrix::IDENTITY;
    matrix.scale(x, y);
    matrix
}

/// Moves every point of `polygon` through `matrix`.
fn mapped(polygon: &RoundedPolygon, matrix: &Matrix) -> RoundedPolygon {
    polygon.transformed(|x, y| {
        let mapped = matrix.map(Offset::new(x, y));
        Point::new(mapped.x, mapped.y)
    })
}

fn rotated(polygon: &RoundedPolygon, degrees: f32) -> RoundedPolygon {
    mapped(polygon, &rotation(degrees))
}

/// A rectangle of the given width: bottom corners rounded by 20%, top by 100%.
fn arched(width: f32) -> RoundedPolygon {
    RoundedPolygon::rectangle(
        width,
        1.0,
        CornerRounding::UNROUNDED,
        Some(&[ROUND_20, ROUND_20, ROUND_100, ROUND_100]),
        Point::default(),
    )
}

/// Defines one cached preset function per entry.
macro_rules! presets {
    ($($(#[$doc:meta])* $name:ident => $build:expr;)*) => {$(
        $(#[$doc])*
        #[must_use]
        pub fn $name() -> RoundedPolygon {
            static CELL: LazyLock<RoundedPolygon> = LazyLock::new(|| $build);
            CELL.clone()
        }
    )*};
}

presets! {
    /// A circle.
    circle => rotated(&RoundedPolygon::circle(10, 1.0, Point::default()), 45.0).normalized();

    /// A square with soft corners.
    square => RoundedPolygon::rectangle(
        1.0, 1.0, ROUND_30, None, Point::default(),
    ).normalized();

    /// A leaning parallelogram.
    slanted => custom(&[
        smooth(0.926, 0.970, 0.189, 0.811),
        smooth(-0.021, 0.967, 0.187, 0.057),
    ], 2).normalized();

    /// A square with a rounded top.
    arch => arched(1.0).normalized();

    /// A quarter fan.
    fan => custom(&[
        smooth(1.004, 1.000, 0.148, 0.417),
        round(0.000, 1.000, 0.151),
        round(0.000, -0.003, 0.148),
        round(0.978, 0.020, 0.803),
    ], 1).normalized();

    /// An arrowhead pointing up.
    arrow => custom(&[
        round(1.225, 1.060, 0.211),
        round(0.500, 0.892, 0.313),
        round(-0.216, 1.050, 0.207),
        smooth(0.499, -0.160, 0.215, 1.000),
    ], 1).normalized();

    /// A half circle.
    semi_circle => arched(1.6).normalized();

    /// A tilted oval.
    oval => {
        let circle = RoundedPolygon::circle(8, 1.0, Point::default());
        let circle = rotated(&circle, -90.0);
        let circle = mapped(&circle, &scaling(1.0, 0.64));
        rotated(&circle, 135.0).normalized()
    };

    /// A rounded capsule, tilted.
    pill => rotated(
        &custom(&[
            round(0.428, -0.001, 0.426),
            round(0.961, 0.039, 0.426),
            sharp(1.001, 0.428),
            round(1.000, 0.609, 1.000),
        ], 2),
        180.0,
    ).normalized();

    /// A triangle with soft corners.
    triangle => rotated(
        &RoundedPolygon::from_num_vertices(3, 1.0, Point::new(0.5, 0.5), ROUND_20, None),
        30.0,
    ).normalized();

    /// A diamond with soft corners.
    diamond => custom(&[
        smooth(0.500, 1.096, 0.151, 0.524),
        round(0.040, 0.500, 0.159),
    ], 2).normalized();

    /// A clam shell.
    clam_shell => custom(&[
        round(0.829, 0.841, 0.159),
        round(0.171, 0.841, 0.159),
        round(-0.020, 0.500, 0.140),
    ], 2).normalized();

    /// A five-sided shape with soft corners.
    pentagon => custom(&[
        round(0.828, 0.970, 0.169),
        round(0.172, 0.970, 0.169),
        round(-0.030, 0.365, 0.164),
        round(0.500, -0.009, 0.172),
        round(1.030, 0.365, 0.164),
    ], 1).normalized();

    /// A gemstone.
    gem => custom(&[
        round(1.005, 0.792, 0.208),
        smooth(0.5, 1.023, 0.241, 0.778),
        round(-0.005, 0.792, 0.208),
        round(0.073, 0.258, 0.228),
        smooth(0.5, 0.000, 0.241, 0.778),
        round(0.927, 0.258, 0.228),
    ], 1).normalized();

    /// A sun with eight short rays.
    sunny => rotated(
        &RoundedPolygon::star(8, 1.0, 0.8, ROUND_15, None, None, Point::default()),
        45.0,
    ).normalized();

    /// A sun with eight sharp rays.
    very_sunny => rotated(
        &custom(&[
            round(0.500, 1.080, 0.085),
            round(0.358, 0.843, 0.085),
        ], 8),
        -45.0,
    ).normalized();

    /// A cookie with 4 bumps.
    cookie_4_sided => custom(&[
        round(1.237, 1.236, 0.258),
        round(0.500, 0.918, 0.233),
    ], 4).normalized();

    /// A cookie with 6 bumps.
    cookie_6_sided => custom(&[
        round(0.723, 0.884, 0.394),
        round(0.500, 1.099, 0.398),
    ], 6).normalized();

    /// A cookie with 7 bumps.
    cookie_7_sided => rotated(
        &RoundedPolygon::star(7, 1.0, 0.75, ROUND_50, None, None, Point::default()).normalized(),
        // Five turns of a 28th of a circle.
        5.0 * 360.0 / 28.0,
    ).normalized();

    /// A cookie with 9 bumps.
    cookie_9_sided => rotated(
        &RoundedPolygon::star(9, 1.0, 0.8, ROUND_50, None, None, Point::default()),
        30.0,
    ).normalized();

    /// A cookie with 12 bumps.
    cookie_12_sided => rotated(
        &RoundedPolygon::star(12, 1.0, 0.8, ROUND_50, None, None, Point::default()),
        30.0,
    ).normalized();

    /// A ghost with a wavy hem.
    ghostish => custom(&[
        smooth(1.000, 1.140, 0.254, 0.106),
        round(0.575, 0.906, 0.253),
        round(0.425, 0.906, 0.253),
        smooth(0.000, 1.140, 0.254, 0.106),
        with(0.000, 0.000, ROUND_100),
        with(0.500, 0.000, ROUND_100),
        with(1.000, 0.000, ROUND_100),
    ], 1).normalized();

    /// A four-leaf clover.
    clover_4_leaf => custom(&[
        round(1.099, 0.725, 0.476),
        round(0.725, 1.099, 0.476),
        sharp(0.500, 0.926),
    ], 4).normalized();

    /// An eight-leaf clover.
    clover_8_leaf => custom(&[
        round(0.758, 1.101, 0.209),
        sharp(0.500, 0.964),
    ], 8).normalized();

    /// A twelve-point burst.
    burst => rotated(
        &custom(&[
            round(0.592, 0.842, 0.006),
            round(0.500, 1.006, 0.006),
        ], 12),
        -60.0,
    ).normalized();

    /// A ten-point burst with soft tips.
    soft_burst => rotated(
        &custom(&[
            round(0.193, 0.277, 0.053),
            round(0.176, 0.055, 0.053),
        ], 10),
        180.0,
    ).normalized();

    /// A fifteen-point explosion.
    boom => rotated(
        &custom(&[
            round(0.457, 0.296, 0.007),
            round(0.500, -0.051, 0.007),
        ], 15),
        120.0,
    ).normalized();

    /// A sixteen-point explosion with soft tips.
    soft_boom => rotated(
        &custom(&[
            sharp(0.733, 0.454),
            round(0.839, 0.437, 0.532),
            smooth(0.949, 0.449, 0.439, 1.000),
            round(0.998, 0.478, 0.174),
            // mirrored points
            round(0.998, 0.522, 0.174),
            smooth(0.949, 0.551, 0.439, 1.000),
            round(0.839, 0.563, 0.532),
            sharp(0.733, 0.546),
        ], 16),
        // A turn of 45 degrees, then back by a 16th of a circle.
        45.0 - 360.0 / 16.0,
    ).normalized();

    /// A flower with eight petals.
    flower => rotated(
        &custom(&[
            sharp(0.370, 0.187),
            round(0.416, 0.049, 0.381),
            round(0.479, 0.001, 0.095),
            // mirrored points
            round(0.521, 0.001, 0.095),
            round(0.584, 0.049, 0.381),
            sharp(0.630, 0.187),
        ], 8),
        135.0,
    ).normalized();

    /// A puffy cloud.
    puffy => mapped(
        &custom(&[
            // mirrored points
            round(1.003, 0.563, 0.255),
            round(0.940, 0.656, 0.126),
            sharp(0.881, 0.654),
            round(0.926, 0.711, 0.660),
            round(0.914, 0.851, 0.660),
            round(0.777, 0.998, 0.360),
            sharp(0.722, 0.872),
            round(0.717, 0.934, 0.574),
            round(0.670, 1.035, 0.426),
            round(0.545, 1.040, 0.405),
            sharp(0.500, 0.947),
            // original points
            sharp(0.500, 1.0 - 0.053),
            round(1.0 - 0.545, 1.0 + 0.040, 0.405),
            round(1.0 - 0.670, 1.0 + 0.035, 0.426),
            round(1.0 - 0.717, 1.0 - 0.066, 0.574),
            sharp(1.0 - 0.722, 1.0 - 0.128),
            round(1.0 - 0.777, 1.0 - 0.002, 0.360),
            round(1.0 - 0.914, 1.0 - 0.149, 0.660),
            round(1.0 - 0.926, 1.0 - 0.289, 0.660),
            sharp(1.0 - 0.881, 1.0 - 0.346),
            round(1.0 - 0.940, 1.0 - 0.344, 0.126),
            round(1.0 - 1.003, 1.0 - 0.437, 0.255),
        ], 2),
        &scaling(1.0, 0.742),
    ).normalized();

    /// A diamond with puffy sides.
    puffy_diamond => rotated(
        &custom(&[
            // original points
            round(0.870, 0.130, 0.146),
            sharp(0.818, 0.357),
            round(1.000, 0.332, 0.853),
            // mirrored points
            round(1.000, 1.0 - 0.332, 0.853),
            sharp(0.818, 1.0 - 0.357),
        ], 4),
        90.0,
    ).normalized();

    /// A circle made of square pixels.
    pixel_circle => custom(&[
        sharp(1.000, 0.704),
        sharp(0.926, 0.704),
        sharp(0.926, 0.852),
        sharp(0.843, 0.852),
        sharp(0.843, 0.935),
        sharp(0.704, 0.935),
        sharp(0.704, 1.000),
        sharp(0.500, 1.000),
        sharp(1.0 - 0.704, 1.000),
        sharp(1.0 - 0.704, 0.935),
        sharp(1.0 - 0.843, 0.935),
        sharp(1.0 - 0.843, 0.852),
        sharp(1.0 - 0.926, 0.852),
        sharp(1.0 - 0.926, 0.704),
        sharp(0.0, 0.704),
    ], 2).normalized();

    /// A triangle made of square pixels.
    pixel_triangle => custom(&[
        // mirrored points
        sharp(0.888, 1.0 - 0.439),
        sharp(0.789, 1.0 - 0.439),
        sharp(0.789, 1.0 - 0.344),
        sharp(0.675, 1.0 - 0.344),
        sharp(0.674, 1.0 - 0.265),
        sharp(0.560, 1.0 - 0.265),
        sharp(0.560, 1.0 - 0.170),
        sharp(0.421, 1.0 - 0.170),
        sharp(0.421, 1.0 - 0.087),
        sharp(0.287, 1.0 - 0.087),
        sharp(0.287, 1.0),
        sharp(0.113, 1.0),
        // original points
        sharp(0.110, 0.500),
        sharp(0.113, 0.000),
        sharp(0.287, 0.000),
        sharp(0.287, 0.087),
        sharp(0.421, 0.087),
        sharp(0.421, 0.170),
        sharp(0.560, 0.170),
        sharp(0.560, 0.265),
        sharp(0.674, 0.265),
        sharp(0.675, 0.344),
        sharp(0.789, 0.344),
        sharp(0.789, 0.439),
        sharp(0.888, 0.439),
    ], 1).normalized();

    /// A bun.
    bun => custom(&[
        // original points
        sharp(0.796, 0.500),
        with(0.853, 0.518, ROUND_100),
        with(0.992, 0.631, ROUND_100),
        with(0.968, 1.000, ROUND_100),
        // mirrored points
        with(0.032, 1.0, ROUND_100),
        with(0.008, 1.0 - 0.369, ROUND_100),
        with(0.147, 1.0 - 0.482, ROUND_100),
        sharp(0.204, 1.0 - 0.500),
    ], 2).normalized();

    /// A heart.
    heart => custom(&[
        sharp(0.782, 0.611),
        round(0.499, 0.946, 0.000),
        sharp(0.2175, 0.611),
        round(-0.064, 0.276, 1.000),
        round(0.208, -0.066, 0.958),
        round(0.500, 0.268, 0.016),
        round(0.792, -0.066, 0.958),
        round(1.064, 0.276, 1.000),
    ], 1).normalized();
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [fn() -> RoundedPolygon; 35] = [
        circle,
        square,
        slanted,
        arch,
        fan,
        arrow,
        semi_circle,
        oval,
        pill,
        triangle,
        diamond,
        clam_shell,
        pentagon,
        gem,
        sunny,
        very_sunny,
        cookie_4_sided,
        cookie_6_sided,
        cookie_7_sided,
        cookie_9_sided,
        cookie_12_sided,
        ghostish,
        clover_4_leaf,
        clover_8_leaf,
        burst,
        soft_burst,
        boom,
        soft_boom,
        flower,
        puffy,
        puffy_diamond,
        pixel_circle,
        pixel_triangle,
        bun,
        heart,
    ];

    #[test]
    fn every_preset_has_curves() {
        assert!(ALL.iter().all(|preset| !preset().cubics().is_empty()));
    }

    #[test]
    fn every_preset_fits_the_unit_square() {
        for (index, preset) in ALL.iter().enumerate() {
            let [min_x, min_y, max_x, max_y] = preset().calculate_bounds(false);
            let tolerance = 0.05;
            assert!(
                min_x > -tolerance
                    && min_y > -tolerance
                    && max_x < 1.0 + tolerance
                    && max_y < 1.0 + tolerance,
                "preset {index} spans {min_x} {min_y} {max_x} {max_y}"
            );
        }
    }

    #[test]
    fn a_preset_is_the_same_every_time() {
        assert_eq!(heart(), heart());
    }

    #[test]
    fn square_is_centered() {
        let [min_x, min_y, max_x, max_y] = square().calculate_bounds(false);

        assert!(((min_x + max_x) / 2.0 - 0.5).abs() < 1e-3);
        assert!(((min_y + max_y) / 2.0 - 0.5).abs() < 1e-3);
    }

    #[test]
    fn arch_and_semi_circle_differ() {
        assert_ne!(arch(), semi_circle());
    }
}
