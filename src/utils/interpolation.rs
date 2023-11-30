type InPoint = cgmath::Vector2<f32>;

pub enum InterpolateMethod {
    Ceil,
    Linear,
    LaGrange,
}

pub fn interpolate_lagrange(fx: &Vec<InPoint>, xm: f32) -> f32 {
    let n = fx.len();
    let mut result = 0.0;

    for i in 0..n {
        let mut term = 1.0;
        for j in 0..n {
            if i != j {
                let numerator = xm - fx[j].x;
                let denominator = fx[i].x - fx[j].x;
                term *= numerator / denominator;
            }
        }
        result += term * fx[i].y;
    }

    result
}

pub fn interpolate_linear(y1: f32, y2: f32, fraction: f32) -> f32 {
    y1 + (y2 - y1) * fraction
}

#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;

    use crate::utils::{
        consts::*,
        interpolation::{interpolate_lagrange, interpolate_linear},
    };

    use super::InPoint;

    #[test]
    fn test_interpolation_linear() {
        assert_approx_eq!(interpolate_linear(-0.25, 0.25, 0.5), 0.);
        assert_approx_eq!(interpolate_linear(0., 1., 0.33), 0.33);
        assert_approx_eq!(interpolate_linear(0., 0.5, 0.25), 0.125);
    }

    #[test]
    fn test_interpolation_lagrange() {
        let data = vec![
            get_sin(0.0),
            get_sin(PI_2),
            get_sin(PI / 3.0),
            get_sin(PI),
            get_sin(PI + PI / 2.0),
            get_sin(PI_2M),
        ];
        assert_approx_eq!(interpolate_lagrange(&data, 0.75 * PI), (0.75 * PI).sin(), 10e-2);
        assert_approx_eq!(interpolate_lagrange(&data, 0.5 * PI), (0.5 * PI).sin(), 10e-6);
        assert_approx_eq!(interpolate_lagrange(&data, 0.3 * PI), (0.3 * PI).sin(), 10e-2);
    }

    fn get_sin(t: f32) -> InPoint {
        cgmath::Vector2::new(t, t.sin())
    }
}
