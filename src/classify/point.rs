use std::ops::{Add, Mul, Sub};

// Original code from:
// https://github.com/FineFindus/detexify-rust/blob/311002feb0519f483ef1f9cc8206648286128ff5/src/point.rs

/// Point located at the origin (0,0)
pub(super) const ZERO_POINT: Point = Point { x: 0.0, y: 0.0 };
/// Point located at (1,1)
pub(super) const ONE_POINT: Point = Point { x: 1.0, y: 1.0 };

/// δ-value for comparing if two points are equal.
const DELTA: f64 = 1e-10;

/// A simple point, consisting of a (x, y) coordinate.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    /// The x-coordinate of the point.
    pub x: f64,
    /// The y-coordinate of the point.
    pub y: f64,
}

impl Point {
    /// Computes the dot product between self and another point.
    pub(super) fn dot(&self, p: &Point) -> f64 {
        (self.x * p.x) + (self.y * p.y)
    }

    /// Computes the `euclidean` norm.
    pub(super) fn norm(&self) -> f64 {
        self.dot(self).sqrt()
    }

    /// Compute the euclidean distance to a [`Point`] `p`
    pub(super) fn euclidean_distance(&self, &p: &Point) -> f64 {
        (*self - p).norm()
    }

    /// Scales the point by `x` and `y`.
    pub(super) fn scale(self, x: f64, y: f64) -> Point {
        Point {
            x: self.x * x,
            y: self.y * y,
        }
    }

    /// Check if the given [`Point`] is withhin [`DELTA`] of self.
    pub(super) fn approx_eq(&self, p: Point) -> bool {
        self.euclidean_distance(&p) < DELTA
    }

    /// Calculates the angle (in radians) between two vectors formed by three [`Point`]s:
    /// `self` (the origin point), `p` (an intermediate point), and `q` (the endpoint).
    pub(super) fn angle(&self, p: Point, q: Point) -> f64 {
        let v = p - *self;
        let w = q - p;

        v.dot(&w) / (v.norm() * w.norm()).clamp(-1.0, 1.0).acos()
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f64> for Point {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self::Output {
        Point {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static SMALL_DELTA: f64 = 1e-11;

    #[test]
    fn test_add_points() {
        assert_eq!(
            Point { x: 1.0, y: 0.0 } + Point { x: 2.0, y: 3.0 },
            Point { x: 3.0, y: 3.0 }
        );
    }

    #[test]
    fn test_sub_points() {
        assert_eq!(
            Point { x: 1.0, y: 0.0 } - Point { x: 2.0, y: 3.0 },
            Point { x: -1.0, y: -3.0 }
        );
    }

    #[test]
    fn test_mul_point() {
        assert_eq!(Point { x: 1.0, y: 3.0 } * 4.0, Point { x: 4.0, y: 12.0 })
    }

    #[test]
    fn test_approx_eq_vec() {
        assert!(Point { x: 1.0, y: 3.0 }.approx_eq(Point {
            x: 1.0 + SMALL_DELTA,
            y: 3.0 - SMALL_DELTA
        }));
    }
}
