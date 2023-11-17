use crate::{utils::{
    consts::{PI, PI_2M},
    evaluate::Evaluate,
}, error::Error};

pub enum WaveShape {
    Sin,
    Square,
    SquareFourier(u8),
    Saw,
    Saw2,
}

impl Evaluate<f32> for WaveShape {
    fn evaluate(&self, t: f32) -> std::result::Result<f32, Error> {
        match self {
            WaveShape::Sin => Ok(t.sin()),
            WaveShape::Square => {
                if t % PI_2M < PI {
                    Ok(1.0)
                } else {
                    Ok(-1.0)
                }
            }
            WaveShape::SquareFourier(q) => {
                Ok((1..*q)
                    .map(|v| {
                        let n = (2 * v - 1) as f32;
                        (PI_2M * t * n).sin() / n
                    })
                    .sum::<f32>()
                    * -4.0
                    / PI)
            }
            WaveShape::Saw => Ok((t % PI_2M - PI) / PI),
            WaveShape::Saw2 => Ok(1.0 - 2.0 * (t % PI_2M - PI).abs() / PI),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::waveshape::WaveShape,
        utils::{consts::PI, evaluate::Evaluate},
    };
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_evaluate() {
        let sin = WaveShape::Sin;
        let square = WaveShape::Square;
        let square_q = WaveShape::SquareFourier(50);
        let saw = WaveShape::Saw;
        let saw2 = WaveShape::Saw2;
        assert_approx_eq!(sin.evaluate(1.0).unwrap(), 0.841_470_96);
        assert_approx_eq!(square.evaluate(PI + 0.3).unwrap(), -1.0);
        assert_approx_eq!(square.evaluate(PI - 0.3).unwrap(), 1.0);
        assert_approx_eq!(square_q.evaluate(PI + 0.3).unwrap(), -1.0, 0.01);
        assert_approx_eq!(square_q.evaluate(PI - 0.3).unwrap(), 1.0, 0.01);
        assert_approx_eq!(saw.evaluate(PI / 2.0).unwrap(), -0.5);
        assert_approx_eq!(saw.evaluate(3.0 * PI / 2.0).unwrap(), 0.5);
        assert_approx_eq!(saw2.evaluate(PI / 2.0).unwrap(), 0.0);
        assert_approx_eq!(saw2.evaluate(PI).unwrap(), 1.0);
    }
}
