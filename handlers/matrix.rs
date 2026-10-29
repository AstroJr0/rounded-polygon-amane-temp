//! A 4x4 transformation matrix.

use std::ops::MulAssign;
use std::{array, fmt};

use super::offset::Offset;

/// A 4x4 matrix stored in row-major order.
///
/// Points are treated as row vectors, so after `a *= &b` the matrix `a`
/// applies its old transform first and `b` second.
///
/// # Examples
///
/// ```rust
/// use rounded_polygon_amane::handlers::offset::Offset;
/// use rounded_polygon_amane::handlers::matrix::Matrix;
///
/// let mut matrix = Matrix::default();
/// matrix.rotate_z(90.0);
///
/// let rotated = matrix.map(Offset::new(1.0, 0.0));
/// assert!(rotated.x.abs() < 1e-6 && (rotated.y - 1.0).abs() < 1e-6);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    /// The sixteen entries in row-major order: entry `(row, column)` lives at
    /// index `row * 4 + column`.
    pub values: [f32; 16],
}

impl Matrix {
    /// Index of the x scale factor in [`Matrix::values`]
    pub const SCALE_X: usize = 0;
    /// Index of the y skew factor in [`Matrix::values`]
    pub const SKEW_Y: usize = 1;
    /// Index of the first perspective factor in [`Matrix::values`]
    pub const PERSPECTIVE_0: usize = 3;
    /// Index of the x skew factor in [`Matrix::values`]
    pub const SKEW_X: usize = 4;
    /// Index of the y scale factor in [`Matrix::values`]
    pub const SCALE_Y: usize = 5;
    /// Index of the second perspective factor in [`Matrix::values`]
    pub const PERSPECTIVE_1: usize = 7;
    /// Index of the z scale factor in [`Matrix::values`]
    pub const SCALE_Z: usize = 10;
    /// Index of the x translation in [`Matrix::values`]
    pub const TRANSLATE_X: usize = 12;
    /// Index of the y translation in [`Matrix::values`]
    pub const TRANSLATE_Y: usize = 13;
    /// Index of the z translation in [`Matrix::values`]
    pub const TRANSLATE_Z: usize = 14;
    /// Index of the third perspective factor in [`Matrix::values`]
    pub const PERSPECTIVE_2: usize = 15;

    /// The 4x4 identity matrix.
    #[rustfmt::skip]
    pub const IDENTITY: Self = Self {
        values: [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ],
    };

    /// Creates a matrix from sixteen row-major entries
    #[must_use]
    pub const fn new(values: [f32; 16]) -> Self {
        Self { values }
    }

    /// Returns the entry at `row`, `column`
    ///
    /// # Panics
    ///
    /// Panics if `row` or `column` is *4 or greater*
    #[must_use]
    pub fn get(&self, row: usize, column: usize) -> f32 {
        self.values[Self::index(row, column)]
    }

    /// Sets the entry at `row`, `column`
    ///
    /// # Panics
    ///
    /// Panics if `row` or `column` is *4 or greater*
    pub fn set(&mut self, row: usize, column: usize, value: f32) {
        self.values[Self::index(row, column)] = value;
    }

    /// Applies the 3D transform to `point` and returns the resulting `x` and
    /// `y` as an **[`Offset`]**
    ///
    /// The result is divided by the transformed w. If that division is not
    /// *finite* , the result is **[`Offset::ZERO`]**
    #[must_use]
    pub fn map(&self, point: Offset) -> Offset {
        let (v00, v01, v03) = (self.get(0, 0), self.get(0, 1), self.get(0, 3));
        let (v10, v11, v13) = (self.get(1, 0), self.get(1, 1), self.get(1, 3));
        let (v30, v31, v33) = (self.get(3, 0), self.get(3, 1), self.get(3, 3));

        let (x, y) = (point.x, point.y);
        let z = v03 * x + v13 * y + v33;
        let inverse_z = 1.0 / z;
        let p_z = if inverse_z.is_finite() {
            inverse_z
        } else {
            0.0
        };

        Offset::new(
            p_z * (v00 * x + v10 * y + v30),
            p_z * (v01 * x + v11 * y + v31),
        )
    }

    /// Resets this matrix to the identity
    pub fn reset(&mut self) {
        *self = Self::IDENTITY;
    }

    /// Rotates this matrix by `degrees` around the z axis
    pub fn rotate_z(&mut self, degrees: f32) {
        let (sin, cos) = degrees.to_radians().sin_cos();

        for column in 0..4 {
            let a0 = self.get(0, column);
            let a1 = self.get(1, column);
            self.set(0, column, cos * a0 + sin * a1);
            self.set(1, column, -sin * a0 + cos * a1);
        }
    }

    /// Scales this matrix by `x` and `y`, leaving the z axis unchanged
    pub fn scale(&mut self, x: f32, y: f32) {
        self.scale_3d(x, y, 1.0);
    }

    /// Scales this matrix by `x`, `y` and `z`
    pub fn scale_3d(&mut self, x: f32, y: f32, z: f32) {
        // The zip stops after three rows,
        // so the translation row stays untouched
        let (rows, _) = self.values.as_chunks_mut::<4>();
        for (row, factor) in rows.iter_mut().zip([x, y, z]) {
            for value in row {
                *value *= factor;
            }
        }
    }

    /// Translates this matrix by `x` and `y`, leaving the z axis unchanged
    pub fn translate(&mut self, x: f32, y: f32) {
        self.translate_3d(x, y, 0.0);
    }

    /// Translates this matrix by `x`, `y` and `z`
    ///
    /// The translation is applied before the transform already in the matrix,
    /// so it is expressed in the matrix's own *(for example scaled)* space.
    pub fn translate_3d(&mut self, x: f32, y: f32, z: f32) {
        let translation: [f32; 4] = array::from_fn(|column| {
            self.get(0, column) * x
                + self.get(1, column) * y
                + self.get(2, column) * z
                + self.get(3, column)
        });

        self.values[Self::TRANSLATE_X..].copy_from_slice(&translation);
    }

    fn index(row: usize, column: usize) -> usize {
        assert!(
            row < 4 && column < 4,
            "matrix index ({row}, {column}) out of range"
        );
        row * 4 + column
    }

    /// Dot product of `row` *(the row)* of `self` with `column` *(the column)* of `other`
    fn dot(&self, row: usize, other: &Self, column: usize) -> f32 {
        (0..4)
            .map(|k| self.get(row, k) * other.get(k, column))
            .sum()
    }
}

impl Default for Matrix {
    /// The identity matrix
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl MulAssign<&Matrix> for Matrix {
    /// Multiplies this matrix by `rhs` and stores the result in `self`
    fn mul_assign(&mut self, rhs: &Matrix) {
        self.values = array::from_fn(|i| self.dot(i / 4, rhs, i % 4));
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (rows, _) = self.values.as_chunks::<4>();
        for (index, row) in rows.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            write!(f, "{} {} {} {}", row[0], row[1], row[2], row[3])?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    #[test]
    fn default_is_identity() {
        assert_eq!(Matrix::default(), Matrix::IDENTITY);
    }

    #[test]
    fn identity_has_ones_on_the_diagonal_only() {
        #[rustfmt::skip]
        let expected = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];

        assert_eq!(Matrix::IDENTITY.values, expected);
    }

    #[test]
    fn get_reads_the_row_major_entry() {
        let mut values = [0.0; 16];
        values[6] = 7.0;

        assert_eq!(Matrix::new(values).get(1, 2), 7.0);
    }

    #[test]
    fn set_writes_the_row_major_entry() {
        let mut matrix = Matrix::default();

        matrix.set(2, 3, 9.0);

        assert_eq!(matrix.values[11], 9.0);
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn get_panics_when_column_is_out_of_range() {
        let _ = Matrix::default().get(0, 4);
    }

    #[test]
    fn map_with_identity_returns_the_same_point() {
        let point = Offset::new(3.0, -4.0);

        assert_eq!(Matrix::default().map(point), point);
    }

    #[test]
    fn map_applies_translation() {
        let mut matrix = Matrix::default();
        matrix.translate(10.0, 20.0);

        assert_eq!(matrix.map(Offset::new(1.0, 2.0)), Offset::new(11.0, 22.0));
    }

    #[test]
    fn map_applies_scale() {
        let mut matrix = Matrix::default();
        matrix.scale(2.0, 3.0);

        assert_eq!(matrix.map(Offset::new(1.0, 1.0)), Offset::new(2.0, 3.0));
    }

    #[test]
    fn map_divides_by_the_perspective_term() {
        let mut matrix = Matrix::default();
        matrix.values[Matrix::PERSPECTIVE_2] = 2.0;

        assert_eq!(matrix.map(Offset::new(4.0, 6.0)), Offset::new(2.0, 3.0));
    }

    #[test]
    fn map_returns_zero_when_the_perspective_term_is_zero() {
        let mut matrix = Matrix::default();
        matrix.values[Matrix::PERSPECTIVE_2] = 0.0;

        assert_eq!(matrix.map(Offset::new(1.0, 2.0)), Offset::ZERO);
    }

    #[test]
    fn mul_assign_by_identity_leaves_the_matrix_unchanged() {
        let mut matrix = Matrix::default();
        matrix.rotate_z(30.0);
        matrix.translate(1.0, 2.0);
        let expected = matrix.clone();

        matrix *= &Matrix::IDENTITY;

        assert_eq!(matrix, expected);
    }

    #[test]
    fn mul_assign_applies_the_left_transform_first() {
        let mut translate = Matrix::default();
        translate.translate(1.0, 0.0);
        let mut scale = Matrix::default();
        scale.scale(2.0, 2.0);

        translate *= &scale;

        // (1, 1) -> translated to (2, 1) -> scaled to (4, 2)
        assert_eq!(translate.map(Offset::new(1.0, 1.0)), Offset::new(4.0, 2.0));
    }

    #[test]
    fn mul_assign_is_not_commutative() {
        let mut translate = Matrix::default();
        translate.translate(1.0, 0.0);
        let mut scale = Matrix::default();
        scale.scale(2.0, 2.0);

        scale *= &translate;

        // (1, 1) -> scaled to (2, 2) -> translated to (3, 2)
        assert_eq!(scale.map(Offset::new(1.0, 1.0)), Offset::new(3.0, 2.0));
    }

    #[test]
    fn reset_restores_the_identity() {
        let mut matrix = Matrix::default();
        matrix.rotate_z(45.0);
        matrix.scale_3d(2.0, 3.0, 4.0);
        matrix.translate_3d(1.0, 2.0, 3.0);

        matrix.reset();

        assert_eq!(matrix, Matrix::IDENTITY);
    }

    #[test]
    fn rotate_z_quarter_turn_maps_x_axis_to_y_axis() {
        let mut matrix = Matrix::default();
        matrix.rotate_z(90.0);

        let rotated = matrix.map(Offset::new(1.0, 0.0));

        assert!(rotated.x.abs() < EPSILON && (rotated.y - 1.0).abs() < EPSILON);
    }

    #[test]
    fn rotate_z_negative_quarter_turn_maps_x_axis_to_negative_y_axis() {
        let mut matrix = Matrix::default();
        matrix.rotate_z(-90.0);

        let rotated = matrix.map(Offset::new(1.0, 0.0));

        assert!(rotated.x.abs() < EPSILON && (rotated.y + 1.0).abs() < EPSILON);
    }

    #[test]
    fn rotate_z_leaves_the_translation_row_untouched() {
        let mut matrix = Matrix::default();
        matrix.translate(5.0, 6.0);

        matrix.rotate_z(90.0);

        assert_eq!(
            (
                matrix.values[Matrix::TRANSLATE_X],
                matrix.values[Matrix::TRANSLATE_Y]
            ),
            (5.0, 6.0)
        );
    }

    #[test]
    fn scale_3d_multiplies_each_row_by_its_factor() {
        let mut matrix = Matrix::default();

        matrix.scale_3d(2.0, 3.0, 4.0);

        #[rustfmt::skip]
        let expected = [
            2.0, 0.0, 0.0, 0.0,
            0.0, 3.0, 0.0, 0.0,
            0.0, 0.0, 4.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        assert_eq!(matrix.values, expected);
    }

    #[test]
    fn scale_leaves_the_z_axis_unchanged() {
        let mut matrix = Matrix::default();

        matrix.scale(2.0, 3.0);

        assert_eq!(matrix.values[Matrix::SCALE_Z], 1.0);
    }

    #[test]
    fn scale_leaves_the_translation_row_untouched() {
        let mut matrix = Matrix::default();
        matrix.translate(5.0, 6.0);

        matrix.scale(2.0, 2.0);

        assert_eq!(
            (
                matrix.values[Matrix::TRANSLATE_X],
                matrix.values[Matrix::TRANSLATE_Y]
            ),
            (5.0, 6.0)
        );
    }

    #[test]
    fn translate_3d_writes_the_translation_row() {
        let mut matrix = Matrix::default();

        matrix.translate_3d(1.0, 2.0, 3.0);

        assert_eq!(matrix.values[12..], [1.0, 2.0, 3.0, 1.0]);
    }

    #[test]
    fn translate_leaves_the_z_translation_at_zero() {
        let mut matrix = Matrix::default();

        matrix.translate(1.0, 2.0);

        assert_eq!(matrix.values[Matrix::TRANSLATE_Z], 0.0);
    }

    #[test]
    fn translate_is_applied_before_the_existing_transform() {
        let mut matrix = Matrix::default();
        matrix.scale(2.0, 2.0);

        matrix.translate(1.0, 1.0);

        assert_eq!(
            (
                matrix.values[Matrix::TRANSLATE_X],
                matrix.values[Matrix::TRANSLATE_Y]
            ),
            (2.0, 2.0)
        );
    }

    #[test]
    fn display_prints_one_row_per_line() {
        assert_eq!(
            Matrix::IDENTITY.to_string(),
            "1 0 0 0\n0 1 0 0\n0 0 1 0\n0 0 0 1"
        );
    }
}                       
