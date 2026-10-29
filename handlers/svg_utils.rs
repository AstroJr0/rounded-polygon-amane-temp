//! Build a polygon from a small subset of SVG path instructions.
//!
//! Only `m`, `l` and `c` exist, upper and lower case mean the same thing and
//! every coordinate is absolute. `m` comes once, first. `z` is not needed.
//!
//! Coordinates are used as written, nothing is scaled. Write them in `0..1`
//! for `ShapeCanvas` as it is, or in pixels and call `.normalized(false)`.
//!
//! [`move_to`], [`line_to`] and [`curve_to`] each turn one instruction into
//! the corners it adds. [`from_svg`] chains them into a polygon.

use std::fmt;

use super::offset::Offset;
use crate::shapes::corners::CornerRounding;
use crate::shapes::rounded_polygon::RoundedPolygon;
use crate::shapes::utils::{custom_polygon, PointNRound};

/// Straight pieces one curve is cut into.
pub const CURVE_STEPS: usize = 16;

/// What went wrong while reading a path.
#[derive(Debug, Clone, PartialEq)]
pub enum SvgError {
    /// The path does not start with `m`.
    MissingMove,
    /// `m` showed up more than once.
    RepeatedMove,
    /// A letter other than `m`, `l` or `c`.
    UnknownCommand(char),
    /// Not the amount of numbers the command needs.
    BadArguments(char),
    /// Something that is not a number.
    BadNumber(String),
}

impl fmt::Display for SvgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMove => write!(f, "path must start with `m`"),
            Self::RepeatedMove => write!(f, "`m` is allowed only once"),
            Self::UnknownCommand(c) => write!(f, "unsupported command `{c}`"),
            Self::BadArguments(c) => write!(f, "wrong number of values for `{c}`"),
            Self::BadNumber(s) => write!(f, "`{s}` is not a number"),
        }
    }
}

impl std::error::Error for SvgError {}

/// Turns a path into a polygon, ready for `ShapeCanvas`.
///
/// ```
/// use rounded_polygon_amane::handlers::svg_utils::from_svg;
///
/// let shape = from_svg("M 0 0 L 1 0 L 0.5 0.6").unwrap();
/// assert!(!shape.cubics().is_empty());
/// ```
pub fn from_svg(path: &str) -> Result<RoundedPolygon, SvgError> {
    let lower = path.to_lowercase();
    let mut rest = lower.trim();
    let mut points: Vec<PointNRound> = Vec::new();
    let mut current = [0.0, 0.0];

    while let Some(command) = rest.chars().next() {
        let end = rest[1..]
            .find(|c: char| c.is_ascii_alphabetic() && c != 'e')
            .map_or(rest.len(), |i| i + 1);
        let values = numbers(&rest[1..end])?;

        rest = rest[end..].trim_start();

        if command != 'm' && points.is_empty() {
            return Err(SvgError::MissingMove);
        }

        match command {
            'm' => {
                if !points.is_empty() {
                    return Err(SvgError::RepeatedMove);
                }
                let [x, y] = values[..] else {
                    return Err(SvgError::BadArguments('m'));
                };

                add(&mut points, [move_to(x, y)]);
                current = [x, y];
            }
            'l' => {
                if values.is_empty() || values.len() % 2 != 0 {
                    return Err(SvgError::BadArguments('l'));
                }
                for v in values.chunks_exact(2) {
                    add(&mut points, [line_to(v[0], v[1])]);
                    current = [v[0], v[1]];
                }
            }
            'c' => {
                if values.is_empty() || values.len() % 6 != 0 {
                    return Err(SvgError::BadArguments('c'));
                }
                for v in values.chunks_exact(6) {
                    let (c1, c2, end) = ([v[0], v[1]], [v[2], v[3]], [v[4], v[5]]);

                    add(&mut points, curve_to(current, c1, c2, end));
                    current = end;
                }
            }
            other => return Err(SvgError::UnknownCommand(other)),
        }
    }

    if points.is_empty() {
        return Err(SvgError::MissingMove);
    }

    // a path that returns to its start would repeat the first corner
    if points.len() > 1 && points.last().map(|p| p.offset) == points.first().map(|p| p.offset) {
        points.pop();
    }

    Ok(custom_polygon(&points, 1, center(&points), false))
}

/// `m x y`: the first corner.
pub fn move_to(x: f32, y: f32) -> PointNRound {
    corner(Offset::new(x, y))
}

/// `l x y`: the corner a straight edge ends on.
pub fn line_to(x: f32, y: f32) -> PointNRound {
    corner(Offset::new(x, y))
}

/// `c x1 y1 x2 y2 x y`: a cubic bezier from `from` to `to`, cut into
/// [`CURVE_STEPS`] corners because a polygon only holds corners. The last one
/// sits exactly on `to`; `from` is not repeated.
pub fn curve_to(from: [f32; 2], c1: [f32; 2], c2: [f32; 2], to: [f32; 2]) -> Vec<PointNRound> {
    let [p0, p1, p2, p3] = [from, c1, c2, to].map(|[x, y]| Offset::new(x, y));

    (1..=CURVE_STEPS)
        .map(|i| {
            let t = i as f32 / CURVE_STEPS as f32;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);

            corner(Offset::new(
                a * p0.x + b * p1.x + c * p2.x + d * p3.x,
                a * p0.y + b * p1.y + c * p2.y + d * p3.y,
            ))
        })
        .collect()
}

// the polygon is built around its middle, so that has to follow the path
fn center(points: &[PointNRound]) -> Offset {
    let n = points.len() as f32;
    let (x, y) = points
        .iter()
        .fold((0.0, 0.0), |(x, y), p| (x + p.offset.x, y + p.offset.y));

    Offset::new(x / n, y / n)
}

fn corner(offset: Offset) -> PointNRound {
    PointNRound::new(offset, CornerRounding::UNROUNDED)
}

// a repeated point would make a zero length edge
fn add(points: &mut Vec<PointNRound>, new: impl IntoIterator<Item = PointNRound>) {
    for point in new {
        if points.last().map(|p| p.offset) != Some(point.offset) {
            points.push(point);
        }
    }
}

fn numbers(text: &str) -> Result<Vec<f32>, SvgError> {
    text.split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().map_err(|_| SvgError::BadNumber(s.to_owned())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_and_line_are_single_corners() {
        assert_eq!(move_to(1.0, 2.0).offset, Offset::new(1.0, 2.0));
        assert_eq!(line_to(3.0, 4.0).offset, Offset::new(3.0, 4.0));
    }

    #[test]
    fn curve_has_steps_and_ends_on_target() {
        let c = curve_to([0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]);

        assert_eq!(c.len(), CURVE_STEPS);
        assert!((c[CURVE_STEPS - 1].offset.x - 1.0).abs() < 1e-6);
        assert!(c[CURVE_STEPS - 1].offset.y.abs() < 1e-6);
    }

    #[test]
    fn case_does_not_matter() {
        let a = from_svg("M 0 0 L 1 0 L 0.5 0.6").unwrap();
        let b = from_svg("m 0 0 l 1 0 l 0.5 0.6").unwrap();

        assert_eq!(a, b);
    }

    #[test]
    fn commas_and_repeated_arguments() {
        let a = from_svg("m0,0 l1,0 0.5,0.6").unwrap();
        let b = from_svg("m 0 0 l 1 0 l 0.5 0.6").unwrap();

        assert_eq!(a, b);
    }

    #[test]
    fn closing_back_to_start_adds_no_extra_corner() {
        let open = from_svg("m 0 0 l 1 0 l 0.5 0.6").unwrap();
        let closed = from_svg("m 0 0 l 1 0 l 0.5 0.6 l 0 0").unwrap();

        assert_eq!(open, closed);
    }

    #[test]
    fn curved_shape_builds() {
        let shape = from_svg("m 0 0.5 c 0 0 1 0 1 0.5 l 0.5 1").unwrap();

        assert!(shape.cubics().len() > CURVE_STEPS);
    }

    #[test]
    fn errors() {
        let err = |p| from_svg(p).unwrap_err();

        assert_eq!(err(""), SvgError::MissingMove);
        assert_eq!(err("l 1 1"), SvgError::MissingMove);
        assert_eq!(err("m 0 0 m 1 1"), SvgError::RepeatedMove);
        assert_eq!(err("m 0 0 q 1 1"), SvgError::UnknownCommand('q'));
        assert_eq!(err("m 0 0 l 1"), SvgError::BadArguments('l'));
        assert_eq!(err("m 0 0 c 1 2 3 4 5"), SvgError::BadArguments('c'));
        assert_eq!(err("m 0 1.2.3"), SvgError::BadNumber("1.2.3".into()));
    }

    // built by hand, centered on the plain average of the corners
    fn custom(points: &[(f32, f32)]) -> RoundedPolygon {
        let n = points.len() as f32;
        let mid = Offset::new(
            points.iter().map(|p| p.0).sum::<f32>() / n,
            points.iter().map(|p| p.1).sum::<f32>() / n,
        );
        let corners: Vec<_> = points
            .iter()
            .map(|&(x, y)| PointNRound::unrounded(Offset::new(x, y)))
            .collect();

        custom_polygon(&corners, 1, mid, false)
    }

    fn same_shape(a: &RoundedPolygon, b: &RoundedPolygon) {
        assert_eq!(a.cubics().len(), b.cubics().len());

        for (x, y) in a.cubics().iter().zip(b.cubics()) {
            for (p, q) in x.points.iter().zip(&y.points) {
                assert!((p - q).abs() < 1e-5, "{p} vs {q}");
            }
        }
    }

    #[test]
    fn triangle_matches_custom_polygon() {
        let svg = from_svg("M 0.5 0 L 1 1 L 0 1").unwrap();
        assert_eq!(svg, custom(&[(0.5, 0.0), (1.0, 1.0), (0.0, 1.0)]));
    }

    #[test]
    fn square_matches_custom_polygon() {
        let svg = from_svg("M 0 0 L 1 0 L 1 1 L 0 1").unwrap();
        assert_eq!(
            svg,
            custom(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)])
        );
    }

    #[test]
    fn pixel_coordinates_match_custom_polygon() {
        let svg = from_svg("M 0 0 L 500 0 L 250 300").unwrap();
        assert_eq!(svg, custom(&[(0.0, 0.0), (500.0, 0.0), (250.0, 300.0)]));
    }

    #[test]
    fn hexagon_matches_custom_polygon() {
        let ring: Vec<(f32, f32)> = (0..6)
            .map(|i| {
                let a = i as f32 * std::f32::consts::TAU / 6.0;

                (0.5 + 0.5 * a.cos(), 0.5 + 0.5 * a.sin())
            })
            .collect();
        let path = ring
            .iter()
            .enumerate()
            .map(|(i, (x, y))| format!("{} {x} {y}", if i == 0 { 'm' } else { 'l' }))
            .collect::<Vec<_>>()
            .join(" ");

        let svg = from_svg(&path).unwrap();
        assert_eq!(svg, custom(&ring));
    }

    #[test]
    fn curve_matches_hand_computed_custom_polygon() {
        // de casteljau, a different route to the same points than curve_to
        let lerp =
            |a: (f32, f32), b: (f32, f32), t: f32| (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
        let (p0, p1, p2, p3) = ((0.0, 0.5), (0.0, 0.0), (1.0, 0.0), (1.0, 0.5));
        let mut ring = vec![p0];

        for i in 1..=CURVE_STEPS {
            let t = i as f32 / CURVE_STEPS as f32;
            let (a, b, c) = (lerp(p0, p1, t), lerp(p1, p2, t), lerp(p2, p3, t));

            ring.push(lerp(lerp(a, b, t), lerp(b, c, t), t));
        }
        ring.push((0.5, 1.0));

        let svg = from_svg("M 0 0.5 C 0 0 1 0 1 0.5 L 0.5 1").unwrap();
        same_shape(&svg, &custom(&ring));
    }

    #[test]
    fn two_curves_chain_from_each_end() {
        let svg = from_svg("m 0 0.5 c 0 0 0.5 0 0.5 0.5 c 0.5 1 1 1 1 0.5 l 0.5 0").unwrap();
        let mut ring = vec![move_to(0.0, 0.5)];

        ring.extend(curve_to([0.0, 0.5], [0.0, 0.0], [0.5, 0.0], [0.5, 0.5]));
        ring.extend(curve_to([0.5, 0.5], [0.5, 1.0], [1.0, 1.0], [1.0, 0.5]));
        ring.push(line_to(0.5, 0.0));

        assert_eq!(svg, custom_polygon(&ring, 1, center(&ring), false));
    }

    #[test]
    fn a_different_path_is_a_different_polygon() {
        let a = from_svg("m 0 0 l 1 0 l 0.5 1").unwrap();
        let b = from_svg("m 0 0 l 1 0 l 0.6 1").unwrap();

        assert_ne!(a, b);
    }
}          
