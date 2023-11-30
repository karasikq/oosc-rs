type InPoint = cgmath::Vector2<f32>;

pub enum InterpolateMethod {
    Ceil,
    Linear,
    LaGrange,
}

pub fn interpolate_lagrange(fx: Vec<InPoint>, xm: f32) -> f32 {
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

    use crate::utils::interpolation::interpolate_linear;

    #[test]
    fn test_interpolation_linear() {
        assert_approx_eq!(interpolate_linear(-0.25, 0.25, 0.5), 0.);
        assert_approx_eq!(interpolate_linear(0., 1., 0.33), 0.33);
        assert_approx_eq!(interpolate_linear(0., 0.5, 0.25), 0.125);
    }
}
