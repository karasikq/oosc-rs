use crate::utils::convert::{analog_from_corner, corner_angle};

#[derive(Clone, Copy)]
pub enum Coefficients {
    FirstOrder { memory: [f32; 3] },
    SecondOrder { memory: [f32; 5] },
    ThirdOrder { memory: [f32; 7] },
}

impl From<[f32; 3]> for Coefficients {
    fn from(value: [f32; 3]) -> Self {
        Self::FirstOrder { memory: value }
    }
}

// [a1, b0, b1] / a0
impl From<[f32; 4]> for Coefficients {
    fn from(value: [f32; 4]) -> Self {
        let a0_inv = 1.0 / value[0];
        let memory = core::array::from_fn::<_, 3, _>(|i| value.get(i + 1).unwrap() * a0_inv);
        Self::FirstOrder { memory }
    }
}

impl From<[f32; 5]> for Coefficients {
    fn from(value: [f32; 5]) -> Self {
        Self::SecondOrder { memory: value }
    }
}

// [a1, a2, b0, b1, b2] / a0
impl From<[f32; 6]> for Coefficients {
    fn from(value: [f32; 6]) -> Self {
        let a0_inv = 1.0 / value[0];
        let memory = core::array::from_fn::<_, 5, _>(|i| value.get(i + 1).unwrap() * a0_inv);
        Self::SecondOrder { memory }
    }
}

// [a1, a2, a3, b0, b1, b2, b4] / a0
impl From<[f32; 7]> for Coefficients {
    fn from(value: [f32; 7]) -> Self {
        Self::ThirdOrder { memory: value }
    }
}

impl From<[f32; 8]> for Coefficients {
    fn from(value: [f32; 8]) -> Self {
        let a0_inv = 1.0 / value[0];
        let memory = core::array::from_fn::<_, 7, _>(|i| value.get(i + 1).unwrap() * a0_inv);
        Self::ThirdOrder { memory }
    }
}

impl Coefficients {
    pub fn order(&self) -> u32 {
        match self {
            Coefficients::FirstOrder { .. } => 1,
            Coefficients::SecondOrder { .. } => 2,
            Coefficients::ThirdOrder { .. } => 3,
        }
    }

    pub fn get(&self, index: usize) -> Option<f32> {
        match self {
            Coefficients::FirstOrder { memory } => memory.get(index).copied(),
            Coefficients::SecondOrder { memory } => memory.get(index).copied(),
            Coefficients::ThirdOrder { memory } => memory.get(index).copied(),
        }
    }

    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is *[undefined behavior]*
    pub unsafe fn get_unchecked(&self, index: usize) -> f32 {
        match self {
            Coefficients::FirstOrder { memory } => *memory.get_unchecked(index),
            Coefficients::SecondOrder { memory } => *memory.get_unchecked(index),
            Coefficients::ThirdOrder { memory } => *memory.get_unchecked(index),
        }
    }
}

pub fn build_first_order_low_pass(sample_rate: f32, frequency: f32) -> Coefficients {
    let wc = analog_from_corner(sample_rate, frequency);
    Coefficients::from([wc + 1.0, wc - 1.0, wc, wc])
}

pub fn build_first_order_high_pass(sample_rate: f32, frequency: f32) -> Coefficients {
    let wc = analog_from_corner(sample_rate, frequency);
    Coefficients::from([wc + 1.0, wc - 1.0, 1.0, -1.0])
}

pub fn build_first_order_all_pass(sample_rate: f32, frequency: f32) -> Coefficients {
    let wc = analog_from_corner(sample_rate, frequency);
    Coefficients::from([wc + 1.0, wc - 1.0, wc + 1.0, wc - 1.0])
}

pub fn build_second_order_low_pass(sample_rate: f32, frequency: f32, quality: f32) -> Coefficients {
    let theta = corner_angle(sample_rate, frequency);
    let theta_sin = theta.sin();
    let d = (0.25 / quality) * theta_sin;
    let beta = 0.25 - d * d;
    let gamma = (0.5 + beta) * theta.cos();

    Coefficients::from([
        (0.5 + beta - gamma) / 2.0,
        0.5 + beta - gamma,
        (0.5 + beta - gamma) / 2.0,
        -2.0 * gamma,
        2.0 * beta,
    ])
}
