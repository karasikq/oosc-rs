use cgmath::Vector2;

use crate::error::Error;

use super::cubic_bezier::CubicBezierCurve;

pub enum State {
    None,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct ADSREnvelope {
    attack: CubicBezierCurve,
    decay: CubicBezierCurve,
    release: CubicBezierCurve,
}

impl ADSREnvelope {
    pub fn total_attack_length(&self) -> f32 {
        self.attack.difference().x
    }

    pub fn total_decay_length(&self) -> f32 {
        Self::total_attack_length(self) + self.decay.difference().x
    }

    pub fn total_length(&self) -> f32 {
        Self::total_decay_length(self) + self.release.difference().x
    }

    pub fn evaluate(&self, t: f32) -> f32 {
        let attack_time = Self::total_attack_length(self);
        if t < attack_time {
            self.attack.evaluate(t / attack_time).y
        } else {
            let decay_time = Self::total_decay_length(self);
            if t < decay_time {
                self.decay
                    .evaluate((t - attack_time) / self.decay.difference().x)
                    .y
            } else {
                let release_time = Self::total_length(self);
                if t < release_time {
                    self.release
                        .evaluate((t - decay_time) / self.release.difference().x)
                        .y
                } else {
                    0.0
                }
            }
        }
    }
}

pub struct ADSREnvelopeBuilder {
    attack: Option<CubicBezierCurve>,
    decay: Option<CubicBezierCurve>,
    release: Option<CubicBezierCurve>,
}

impl ADSREnvelopeBuilder {
    pub fn new() -> Self {
        Self {
            attack: None,
            decay: None,
            release: None,
        }
    }

    pub fn from_curves(
        attack: CubicBezierCurve,
        decay: CubicBezierCurve,
        release: CubicBezierCurve,
    ) -> Self {
        Self {
            attack: Some(attack),
            decay: Some(decay),
            release: Some(release),
        }
    }

    pub fn set_attack(&mut self, length: f32, amplitude: f32) -> Result<&mut Self, Error> {
        self.attack = Some(CubicBezierCurve::new_linear(
            Vector2 { x: 0.0, y: 0.0 },
            Vector2 {
                x: length,
                y: amplitude,
            },
        ));
        Ok(self)
    }

    pub fn set_decay(&mut self, length: f32, amplitude_percent: f32) -> Result<&mut Self, Error> {
        let attack = self
            .attack
            .as_ref()
            .expect("Attack should be initialized before Decay")
            .end();
        self.decay = Some(CubicBezierCurve::new_linear(
            Vector2 {
                x: 0.0,
                y: attack.y,
            },
            Vector2 {
                x: length,
                y: attack.y * amplitude_percent,
            },
        ));
        Ok(self)
    }

    pub fn set_release(&mut self, length: f32) -> Result<&mut Self, Error> {
        let decay = self
            .decay
            .as_ref()
            .expect("Decay should be initialized before Release")
            .end();
        self.release = Some(CubicBezierCurve::new_linear(
            Vector2 { x: 0.0, y: decay.y },
            Vector2 { x: length, y: 0.0 },
        ));
        Ok(self)
    }

    pub fn build(&mut self) -> Result<ADSREnvelope, Error> {
        let adsr = ADSREnvelope {
            attack: self.attack.take().ok_or("Attack not specified")?,
            decay: self.decay.take().ok_or("Decay not specified")?,
            release: self.release.take().ok_or("Release not specified")?,
        };
        Ok(adsr)
    }
}

impl Default for ADSREnvelopeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

mod tests {
    use super::CubicBezierCurve;
    use crate::utils::adsr_envelope::ADSREnvelopeBuilder;
    use assert_approx_eq::assert_approx_eq;
    use cgmath::Vector2;

    #[test]
    fn test_builder() {
        let adsr = ADSREnvelopeBuilder::new()
            .set_attack(1.0, 0.7)
            .unwrap()
            .set_decay(0.45, 0.4)
            .unwrap()
            .set_release(1.0)
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(adsr.total_length(), 2.45);
    }

    #[test]
    fn test_evaluate() {
        let adsr = ADSREnvelopeBuilder::new()
            .set_attack(0.5, 0.8)
            .unwrap()
            .set_decay(0.5, 0.5)
            .unwrap()
            .set_release(1.0)
            .unwrap()
            .build()
            .unwrap();
        assert_approx_eq!(adsr.evaluate(0.0), 0.0);
        assert_approx_eq!(adsr.evaluate(0.25), 0.4);
        assert_approx_eq!(adsr.evaluate(0.5), 0.8);
        assert_approx_eq!(adsr.evaluate(0.75), 0.6);
        assert_approx_eq!(adsr.evaluate(1.0), 0.4);
        assert_approx_eq!(adsr.evaluate(1.5), 0.2);
        assert_approx_eq!(adsr.evaluate(2.0), 0.0);
    }
}
