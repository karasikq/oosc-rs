use crate::utils::consts::PI_4;

use super::consts::ANALOG_TC;

#[inline]
pub fn cents_to_freq(cents: i32) -> f32 {
    2.0_f32.powf(cents as f32 / 1200.0)
}

#[inline]
pub fn velocity_to_float(velocity: u32) -> f32 {
    let velocity_f32 = velocity as f32;
    velocity_f32 * velocity_f32 / (127.0 * 127.0)
}

#[inline]
pub fn note_to_freq(note: u32) -> f32 {
    8.175_799_f32 * 1.059_463_1_f32.powi(note as i32)
}

#[inline]
pub fn linear_to_power(value: f32) -> f32 {
    10. * value.log10()
}

#[inline]
pub fn linear_to_voltage(value: f32) -> f32 {
    20. * value.log10()
}

#[inline]
pub fn power_to_linear(value: f32) -> f32 {
    10.0_f32.powf(value / 10.0)
}

#[inline]
pub fn voltage_to_linear(value: f32) -> f32 {
    10.0_f32.powf(value / 20.0)
}

#[inline]
pub fn split_bipolar_pan(value: f32) -> (f32, f32) {
    // Const-power pan
    // Use tables for cos/sin ?
    ((PI_4 * (value + 1.0)).cos(), (PI_4 * (value + 1.0)).sin())
}

#[inline]
pub fn exponential_time(value: f32, sample_rate: f32) -> f32 {
    (ANALOG_TC / (value * sample_rate)).exp()
}

#[inline]
pub fn cents_to_freq_coefficient(cents: f32) -> f32 {
    2.0_f32.powf(cents / 1200.0)
}

#[inline]
pub fn octave_offset_to_notes(octave: i32) -> i32 {
    octave * 12
}
