type InPoint = cgmath::Vector2<f32>;

pub fn interpolate(fx: Vec<InPoint>, xm: f32) -> f32 {
    let n = fx.len();
    let mut result = 0.0;

    for i in 0..n {
        let mut term = fx[i].y;
        for j in 0..n {
            if i == j {
                continue;
            }
            let denominator = fx[i].x - fx[j].x;
            let numerator = -fx[j].x;
            term *= numerator / denominator;
        }
        result += term;
        result = mod_euc(result, xm);
    }

    result
}

fn mod_euc(lhs: f32, rhs: f32) -> f32 {
    let r = lhs % rhs;
    if r < 0.0 {
        return if rhs > 0.0 { r + rhs } else { r - rhs };
    }
    r
}
