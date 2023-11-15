use super::{
    consts::{PI, PI_2M},
    evaluate::Evaluate,
};

pub enum WaveShape {
    Sin,
    Square,
    SquareQ(u8),
    Saw,
    Saw2,
}

impl Evaluate for WaveShape {
    fn evaluate(&self, t: f32) -> f32 {
        match self {
            WaveShape::Sin => t.sin(),
            WaveShape::Square => {
                if t % PI_2M < PI {
                    -1.0
                } else {
                    1.0
                }
            }
            WaveShape::SquareQ(q) => {
                (1..*q)
                    .map(|v| {
                        let n = (2 * v - 1) as f32;
                        (PI_2M * t * n).sin() / n
                    })
                    .sum::<f32>()
                    * 4.0
                    / PI
            }
            WaveShape::Saw => (t % PI_2M - PI) / PI,
            WaveShape::Saw2 => 1.0 - 2.0 * (t % PI_2M - PI).abs() / PI,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{
        consts::PI,
        evaluate::Evaluate,
        waveshape::WaveShape,
    };
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_evaluate() {
        let sin = WaveShape::Sin;
        let square = WaveShape::Square;
        let square_q = WaveShape::SquareQ(50);
        let saw = WaveShape::Saw;
        let saw2 = WaveShape::Saw2;
        assert_approx_eq!(sin.evaluate(1.0), 0.841_470_96);
        assert_approx_eq!(square.evaluate(PI + 0.3), 1.0);
        assert_approx_eq!(square.evaluate(PI - 0.3), -1.0);
        assert_approx_eq!(square_q.evaluate(PI + 0.3), 1.0, 0.01);
        assert_approx_eq!(square_q.evaluate(PI - 0.3), -1.0, 0.01);
        assert_approx_eq!(saw.evaluate(PI / 2.0), -0.5);
        assert_approx_eq!(saw.evaluate(3.0 * PI / 2.0), 0.5);
        assert_approx_eq!(saw2.evaluate(PI / 2.0), 0.0);
        assert_approx_eq!(saw2.evaluate(PI), 1.0);
    }
}
