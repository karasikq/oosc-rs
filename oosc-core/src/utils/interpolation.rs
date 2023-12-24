use crate::error::Error;

type InPoint = cgmath::Vector2<f32>;

#[derive(Clone, Copy)]
pub enum InterpolateMethod {
    Floor,
    Linear,
    LaGrange,
    Exponential(f32),
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

pub fn interpolate_exponential(y1: f32, y2: f32, t: f32, coef: f32) -> f32 {
    y1 + (y2 - y1) * (coef.powf(t) - 1.0) / (coef - 1.0)
}

pub fn interpolate_range(range: (f32, f32), t: f32, method: InterpolateMethod) -> f32 {
    match method {
        InterpolateMethod::Floor => ((range.1 - range.0) * t).floor(),
        InterpolateMethod::Linear => interpolate_linear(range.0, range.1, t),
        InterpolateMethod::LaGrange => unimplemented!(),
        InterpolateMethod::Exponential(c) => interpolate_exponential(range.0, range.1, t, c),
    }
}

pub fn interpolate_sample(
    interpolation: InterpolateMethod,
    slice: &[f32],
    index: f32,
) -> Result<f32, Error> {
    Ok(match interpolation {
        InterpolateMethod::Floor => get_sample_at_chunk(slice, index as usize)?,
        InterpolateMethod::Linear => {
            let fraction = index % 1.0;
            let mut samples = get_samples_ranged(slice, slice.len(), index as i32, 2);
            let s1 = samples.next().unwrap();
            let s2 = samples.next().unwrap();
            interpolate_linear(s1, s2, fraction)
        }
        InterpolateMethod::LaGrange => {
            let left_index = (index.floor() - 1.) as i32;
            let vec = get_samples_points_ranged(slice, slice.len(), left_index, 4).collect();
            interpolate_lagrange(&vec, index)
        }
        InterpolateMethod::Exponential(c) => {
            let fraction = index % 1.0;
            let mut samples = get_samples_ranged(slice, slice.len(), index as i32, 2);
            let s1 = samples.next().unwrap();
            let s2 = samples.next().unwrap();
            interpolate_exponential(s1, s2, fraction, c)
        }
    })
}

pub fn interpolate_sample_mut(
    interpolation: InterpolateMethod,
    slice: &mut [f32],
    index: f32,
) -> Result<f32, Error> {
    interpolate_sample(interpolation, slice, index)
}

fn get_sample_at_chunk(chunk: &[f32], index: usize) -> Result<f32, Error> {
    Ok(*chunk.get(index).ok_or(format!(
        "Index out of bounds of wavetable chunk. Found {}, expected < {}",
        index,
        chunk.len(),
    ))?)
}

fn bound_index(chunk_size: usize, index: i32) -> usize {
    let chunk = chunk_size as i32;
    if index < 0 {
        return (chunk + index) as usize;
    }
    if index >= chunk {
        return (index - chunk) as usize;
    }
    index as usize
}

pub fn get_samples_points_ranged(
    chunk: &[f32],
    chunk_size: usize,
    start: i32,
    count: i32,
) -> impl Iterator<Item = cgmath::Vector2<f32>> + '_ {
    let end = start + count;
    (start..end).map(move |index| {
        let ind = bound_index(chunk_size, index);
        let sample = get_sample_at_chunk(chunk, ind).unwrap();
        cgmath::Vector2::<f32>::new(ind as f32, sample)
    })
}

pub fn get_samples_ranged(
    chunk: &[f32],
    chunk_size: usize,
    start: i32,
    count: i32,
) -> impl Iterator<Item = f32> + '_ {
    let end = start + count;
    (start..end).map(move |index| {
        let ind = bound_index(chunk_size, index);
        get_sample_at_chunk(chunk, ind).unwrap()
    })
}

#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;

    use crate::utils::{
        consts::*,
        interpolation::{get_samples_points_ranged, interpolate_lagrange, interpolate_linear, interpolate_exponential},
    };

    use super::InPoint;

    #[test]
    fn test_interpolation_linear() {
        assert_approx_eq!(interpolate_linear(-0.25, 0.25, 0.5), 0.);
        assert_approx_eq!(interpolate_linear(0., 1., 0.33), 0.33);
        assert_approx_eq!(interpolate_linear(0., 0.5, 0.25), 0.125);
    }

    #[test]
    fn test_interpolation_exponential() {
        assert_approx_eq!(interpolate_exponential(0.0, 1.0, 0.5, 1000.0), 0.030653);
        assert_approx_eq!(interpolate_exponential(2.0, 4.0, 0.5, 1000.0), 2.061307);
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
        assert_approx_eq!(
            interpolate_lagrange(&data, 0.75 * PI),
            (0.75 * PI).sin(),
            10e-2
        );
        assert_approx_eq!(
            interpolate_lagrange(&data, 0.5 * PI),
            (0.5 * PI).sin(),
            10e-6
        );
        assert_approx_eq!(
            interpolate_lagrange(&data, 0.3 * PI),
            (0.3 * PI).sin(),
            10e-2
        );
    }

    fn get_sin(t: f32) -> InPoint {
        cgmath::Vector2::new(t, t.sin())
    }

    #[test]
    fn test_interpolation_get_ranged() {
        let samples = 10;
        let step = PI_2M / samples as f32;
        let data: Vec<f32> = (0..samples).map(|f| (f as f32 * step).sin()).collect();
        let slice = data.as_slice();
        let mut samples = get_samples_points_ranged(slice, slice.len(), 0, 3);
        assert_approx_eq!(samples.next().unwrap().y, (0. * step).sin(), 10e-3);
        assert_approx_eq!(samples.next().unwrap().y, (1. * step).sin(), 10e-3);
        assert_approx_eq!(samples.next().unwrap().y, (2. * step).sin(), 10e-3);
        assert!(samples.next().is_none());
    }
}
