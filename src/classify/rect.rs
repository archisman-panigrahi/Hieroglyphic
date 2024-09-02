use super::point::Point;

// Code from
// https://github.com/FineFindus/detexify-rust/blob/311002feb0519f483ef1f9cc8206648286128ff5/src/rect.rs

/// A simple rectangle between two points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Rect {
    pub(super) lower_left: Point,
    pub(super) upper_right: Point,
}

impl Rect {
    /// Create a new Rect, between the two given [`Point`]s
    pub(super) fn new(lower_left: Point, upper_right: Point) -> Self {
        Rect {
            lower_left,
            upper_right,
        }
    }

    /// Create a new from the given [`Point`].
    ///
    /// The rect with have a size/area of 0.
    pub(super) fn from_point(p: Point) -> Self {
        Rect {
            lower_left: p,
            upper_right: p,
        }
    }

    /// Rturns whether the Rect is ony a point.
    pub(super) fn is_point(self) -> bool {
        self.lower_left == self.upper_right
    }

    /// Rturns the width of the rect.
    pub(super) fn width(self) -> f64 {
        self.upper_right.x - self.lower_left.x
    }

    /// Rturns the height of the rect.
    pub(super) fn height(self) -> f64 {
        self.upper_right.y - self.lower_left.y
    }

    /// Maps the bottom left and the upper-right points, that form the rect with the given
    /// function.
    pub(super) fn map_points<F: FnMut(Point) -> Point>(self, mut f: F) -> Rect {
        Rect {
            lower_left: f(self.lower_left),
            upper_right: f(self.upper_right),
        }
    }

    /// Updates the Rect size, to encompass the given [`Point`].
    ///
    /// If the point is already within the Rect, it remains unchanged.
    pub(super) fn encompass_point(&mut self, point: &Point) {
        self.lower_left.x = self.lower_left.x.min(point.x);
        self.lower_left.y = self.lower_left.y.min(point.y);
        self.upper_right.x = self.upper_right.x.max(point.x);
        self.upper_right.y = self.upper_right.y.max(point.y);
    }
}

#[cfg(test)]
mod tests {
    use crate::classify::{
        point::{ONE_POINT, ZERO_POINT},
        rect::Rect,
    };

    #[test]
    fn test_rect_point() {
        assert!(Rect::from_point(ZERO_POINT).is_point());
    }

    #[test]
    fn test_width_height() {
        assert_eq!(Rect::from_point(ZERO_POINT).width(), 0.0);
        assert_eq!(Rect::from_point(ZERO_POINT).height(), 0.0);
    }

    #[test]
    fn test_encompass() {
        let mut rect = Rect::from_point(ZERO_POINT);
        rect.encompass_point(&ONE_POINT);
        assert_eq!(Rect::new(ZERO_POINT, ONE_POINT), rect);
    }
}
