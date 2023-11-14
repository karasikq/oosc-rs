use cgmath::Vector2;

type Point = Vector2<f32>;

pub struct CubicBezierCurve {
    a: Point,
    b: Point,
    c: Point,
    d: Point,
}

impl CubicBezierCurve {
    pub fn new(a: Point, b: Point, c: Point, d: Point) -> Self {
        Self { a, b, c, d }
    }

    pub fn new_linear(a: Point, d: Point) -> Self {
        let points = Self::get_linear_points(&a, &d);
        Self {
            a,
            b: points.0,
            c: points.1,
            d,
        }
    }

    pub fn difference(&self) -> Point {
        self.d - self.a
    }

    pub fn end(&self) -> Point {
        self.d
    }

    pub fn evaluate(&self, t: f32) -> Point {
        debug_assert!((0.0..=1.0).contains(&t));
        let c = 1.0 - t;
        let c2 = c * c;
        let t2 = t * t;
        3.0 * c2 * t * self.b + c2 * c * self.a + 3.0 * c * t2 * self.c + t2 * t * self.d
    }

    pub fn make_linear(&mut self) {
        let points = Self::get_linear_points(&self.a, &self.d);
        self.b = points.0;
        self.c = points.1;
    }

    fn get_linear_points(a: &Point, d: &Point) -> (Point, Point) {
        let part = (d - a) / 3.0;
        (a + part, d - part)
    }
}

mod tests {
    use super::CubicBezierCurve;
    use cgmath::Vector2;

    #[test]
    fn test_evaluate() {
        let bez = CubicBezierCurve::new(
            Vector2::from([0.0, 0.0]),
            Vector2::from([1.0 / 3.0, 1.0 / 3.0]),
            Vector2::from([2.0 / 3.0, 2.0 / 3.0]),
            Vector2::from([1.0, 1.0]),
        );
        let eval = bez.evaluate(0.5);
        assert_eq!(eval.x, 0.5);
        assert_eq!(eval.y, 0.5);
    }

    #[test]
    fn test_linear() {
        let bez =
            CubicBezierCurve::new_linear(Vector2::from([0.5, 0.8]), Vector2::from([1.0, 0.4]));
        let eval = bez.evaluate(0.5);
        assert_eq!(eval.x, 0.75);
        assert_eq!(eval.y, 0.6);
    }
}
