use crate::utils::{
    consts::SQRT_2,
    convert::{analog_from_corner, corner_angle},
};

#[derive(Clone, Copy)]
pub enum Coefficients {
    FirstOrder { memory: [f32; 3] },
    SecondOrder { memory: [f32; 5] },
    ThirdOrder { memory: [f32; 7] },
}

//            b0 + b1*z^-1 + b2*z^-2
//    H(z) = ------------------------
//            a0 + a1*z^-1 + a2*z^-2
//                      |
//                      V
//            (b0/a0) + (b1/a0)*z^-1 + (b2/a0)*z^-2
//    H(z) = ---------------------------------------
//               1 + (a1/a0)*z^-1 + (a2/a0)*z^-2
//
fn normalize_a0<const T: usize, const R: usize>(input: [f32; T]) -> [f32; R] {
    let a0_inv = 1.0 / input[0];
    core::array::from_fn::<_, R, _>(|i| input[i + 1] * a0_inv)
}

impl From<[f32; 3]> for Coefficients {
    fn from(value: [f32; 3]) -> Self {
        Self::FirstOrder { memory: value }
    }
}

impl From<[f32; 4]> for Coefficients {
    fn from(value: [f32; 4]) -> Self {
        let memory = normalize_a0(value);
        Self::FirstOrder { memory }
    }
}

impl From<[f32; 5]> for Coefficients {
    fn from(value: [f32; 5]) -> Self {
        Self::SecondOrder { memory: value }
    }
}

impl From<[f32; 6]> for Coefficients {
    fn from(value: [f32; 6]) -> Self {
        let memory = normalize_a0(value);
        Self::SecondOrder { memory }
    }
}

impl From<[f32; 7]> for Coefficients {
    fn from(value: [f32; 7]) -> Self {
        Self::ThirdOrder { memory: value }
    }
}

impl From<[f32; 8]> for Coefficients {
    fn from(value: [f32; 8]) -> Self {
        let memory = normalize_a0(value);
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

    pub fn process(&mut self, samples: &mut [f32], state: &mut [f32]) {
        match self {
            Coefficients::FirstOrder { memory } => {
                let a1 = memory[0];
                let b0 = memory[1];
                let b1 = memory[2];

                let mut yn = state[0];

                for sample in samples {
                    let input = *sample;
                    let output = input * b0 + yn;
                    *sample = output;

                    yn = (input * b1) - (output * a1);
                }
                state[0] = yn;
            }
            Coefficients::SecondOrder { memory } => {
                let a1 = memory[0];
                let a2 = memory[1];
                let b0 = memory[2];
                let b1 = memory[3];
                let b2 = memory[4];

                let mut y1 = state[0];
                let mut y2 = state[1];

                for sample in samples {
                    let input = *sample;
                    let output = input * b0 + y1;
                    *sample = output;

                    y1 = (input * b1) - (output * a1) + y2;
                    y2 = (input * b2) - (output * a2);
                }
                state[0] = y1;
                state[1] = y2;
            }
            Coefficients::ThirdOrder { memory } => {
                let a1 = memory[0];
                let a2 = memory[1];
                let a3 = memory[2];
                let b0 = memory[3];
                let b1 = memory[4];
                let b2 = memory[5];
                let b3 = memory[6];

                let mut y1 = state[0];
                let mut y2 = state[1];
                let mut y3 = state[2];

                for sample in samples {
                    let input = *sample;
                    let output = input * b0 + y1;
                    *sample = output;

                    y1 = (input * b1) - (output * a1) + y2;
                    y2 = (input * b2) - (output * a2) + y3;
                    y3 = (input * b3) - (output * a3);
                }
                state[0] = y1;
                state[1] = y2;
                state[2] = y3;
            }
        }
    }
}

impl Default for Coefficients {
    fn default() -> Self {
        From::from([0.0, 1.0, 0.0])
    }
}

#[derive(Clone, Copy)]
pub enum FilterType {
    LPF1(f32),
    LPF2 { frequency: f32, quality: f32 },
    HPF1(f32),
    HPF2 { frequency: f32, quality: f32 },
    APF1(f32),
    LPFButterworth(f32),
    HPFButterworth(f32),
}

pub fn build_filter(filter: &FilterType, sample_rate: f32) -> Coefficients {
    match filter {
        FilterType::LPF1(frequency) => build_first_order_low_pass(sample_rate, *frequency),
        FilterType::LPF2 { frequency, quality } => {
            build_second_order_low_pass(sample_rate, *frequency, *quality)
        }
        FilterType::HPF1(frequency) => build_first_order_high_pass(sample_rate, *frequency),
        FilterType::HPF2 { frequency, quality } => {
            build_second_order_high_pass(sample_rate, *frequency, *quality)
        }
        FilterType::APF1(frequency) => build_first_order_all_pass(sample_rate, *frequency),
        FilterType::LPFButterworth(frequency) => {
            build_second_order_butterworth_low_pass(sample_rate, *frequency)
        }
        FilterType::HPFButterworth(frequency) => {
            build_second_order_butterworth_high_pass(sample_rate, *frequency)
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

pub fn build_second_order_butterworth_low_pass(sample_rate: f32, frequency: f32) -> Coefficients {
    let c = 1.0 / analog_from_corner(sample_rate, frequency);
    let c_squared = c * c;
    let c_sqrt2 = SQRT_2 * c;
    let a0 = 1.0 / (1.0 + c_sqrt2 + c_squared);
    Coefficients::from([
        a0,
        2.0 * a0,
        a0,
        2.0 * a0 * (1.0 - c_squared),
        a0 * (1.0 - c_sqrt2 + c_squared),
    ])
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

pub fn build_second_order_butterworth_high_pass(sample_rate: f32, frequency: f32) -> Coefficients {
    let c = 1.0 / analog_from_corner(sample_rate, frequency);
    let c_squared = c * c;
    let c_sqrt2 = SQRT_2 * c;
    let a0 = 1.0 / (1.0 + c_sqrt2 + c_squared);
    Coefficients::from([
        a0,
        -2.0 * a0,
        a0,
        2.0 * a0 * (c_squared - 1.0),
        a0 * (1.0 - c_sqrt2 + c_squared),
    ])
}

pub fn build_second_order_high_pass(
    sample_rate: f32,
    frequency: f32,
    quality: f32,
) -> Coefficients {
    let theta = corner_angle(sample_rate, frequency);
    let theta_sin = theta.sin();
    let d = (0.25 / quality) * theta_sin;
    let beta = 0.25 - d * d;
    let gamma = (0.5 + beta) * theta.cos();

    Coefficients::from([
        (0.5 + beta + gamma) / 2.0,
        -0.5 - beta - gamma,
        (0.5 + beta + gamma) / 2.0,
        -2.0 * gamma,
        2.0 * beta,
    ])
}
