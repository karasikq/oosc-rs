type InPoint = cgmath::Vector2<f32>;

pub fn interpolate_lagrange(fx: Vec<InPoint>, xm: f32) -> f32 {
    let n = fx.len();
    let mut result = 0.0;

    for i in 0..n {
        let mut term = fx[i].y;
        for j in 0..n {
            if i == j {
                continue;
            }
            let denominator = fx[i].x - fx[j].x;
            let numerator = xm - fx[j].x;
            term *= numerator / denominator;
        }
        result += term * fx[i].y;
    }

    result
}

pub fn interpolate_linear(y1: f32, y2: f32, fraction: f32) -> f32 {
    y1 + (y2 - y1) * fraction
}

fn mod_euc(lhs: f32, rhs: f32) -> f32 {
    let r = lhs % rhs;
    if r < 0.0 {
        return if rhs > 0.0 { r + rhs } else { r - rhs };
    }
    r
}
