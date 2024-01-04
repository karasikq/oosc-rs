use cgmath::Vector2;

use crate::{
    core::parametrs::{CallbackParametr, SharedParametr},
    error::Error,
};

use super::{cubic_bezier::CubicBezierCurve, make_shared, Shared};

#[derive(Copy, Clone, PartialEq)]
pub enum State {
    None,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct SharedCurve {
    pub length: SharedParametr<f32>,
    pub amplitude: SharedParametr<f32>,
    pub point_b: (SharedParametr<f32>, SharedParametr<f32>),
    pub point_c: (SharedParametr<f32>, SharedParametr<f32>),
    pub curve: Shared<CubicBezierCurve>,
}

pub struct ADSREnvelope {
    pub attack: SharedCurve,
    pub decay: SharedCurve,
    pub sustain: SharedCurve,
    pub release: SharedCurve,
}

impl ADSREnvelope {
    pub fn evaluate(&self, t: f32) -> f32 {
        let attack_time = self.time_range_of(State::Attack).1;
        if t < attack_time {
            self.attack
                .curve
                .read()
                .unwrap()
                .evaluate(t / attack_time)
                .y
        } else {
            let decay_time = self.time_range_of(State::Decay).1;
            if t < decay_time {
                self.decay
                    .curve
                    .read()
                    .unwrap()
                    .evaluate((t - attack_time) / self.decay.curve.read().unwrap().difference().x)
                    .y
            } else {
                let sustain_time = self.time_range_of(State::Sustain).1;
                if t < sustain_time {
                    self.sustain
                        .curve
                        .read()
                        .unwrap()
                        .evaluate(
                            (t - decay_time) / self.sustain.curve.read().unwrap().difference().x,
                        )
                        .y
                } else {
                    let release_time = self.time_range_of(State::Release).1;
                    if t < release_time {
                        self.release
                            .curve
                            .read()
                            .unwrap()
                            .evaluate(
                                (t - sustain_time)
                                    / self.release.curve.read().unwrap().difference().x,
                            )
                            .y
                    } else {
                        0.0
                    }
                }
            }
        }
    }

    pub fn peak_at(&self, state: State) -> f32 {
        match state {
            State::None => 0.0,
            State::Attack => self.attack.curve.read().unwrap().evaluate(1.0).y,
            State::Decay => self.decay.curve.read().unwrap().evaluate(1.0).y,
            State::Sustain => self.sustain.curve.read().unwrap().evaluate(1.0).y,
            State::Release => self.release.curve.read().unwrap().evaluate(1.0).y,
        }
    }

    pub fn time_range_of(&self, state: State) -> (f32, f32) {
        match state {
            State::None => (0., 0.),
            State::Attack => (
                self.attack.curve.read().unwrap().start().x,
                self.attack.curve.read().unwrap().difference().x,
            ),
            State::Decay => {
                let attack = self.time_range_of(State::Attack);
                (
                    attack.1,
                    attack.1 + self.decay.curve.read().unwrap().difference().x,
                )
            }
            State::Sustain => {
                let decay = self.time_range_of(State::Decay);
                (
                    decay.1,
                    decay.1 + self.sustain.curve.read().unwrap().difference().x,
                )
            }
            State::Release => {
                let sustain = self.time_range_of(State::Sustain);
                (
                    sustain.1,
                    sustain.1 + self.release.curve.read().unwrap().difference().x,
                )
            }
        }
    }
}

pub struct ADSREnvelopeBuilder {
    attack: Option<CubicBezierCurve>,
    decay: Option<CubicBezierCurve>,
    sustain: Option<CubicBezierCurve>,
    release: Option<CubicBezierCurve>,
}

impl ADSREnvelopeBuilder {
    pub fn new() -> Self {
        Self {
            attack: None,
            decay: None,
            sustain: None,
            release: None,
        }
    }

    pub fn from_curves(
        attack: CubicBezierCurve,
        decay: CubicBezierCurve,
        sustain: CubicBezierCurve,
        release: CubicBezierCurve,
    ) -> Self {
        Self {
            attack: Some(attack),
            decay: Some(decay),
            sustain: Some(sustain),
            release: Some(release),
        }
    }

    pub fn attack(&mut self, length: f32, amplitude: f32) -> Result<&mut Self, Error> {
        self.attack = Some(CubicBezierCurve::new_linear(
            Vector2 { x: 0.0, y: 0.0 },
            Vector2 {
                x: length,
                y: amplitude,
            },
        ));
        Ok(self)
    }

    pub fn decay(&mut self, length: f32, amplitude_percent: f32) -> Result<&mut Self, Error> {
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

    pub fn sustain(&mut self, length: f32, amplitude_percent: f32) -> Result<&mut Self, Error> {
        let decay = self
            .decay
            .as_ref()
            .expect("Decay should be initialized before Sustain")
            .end();
        self.sustain = Some(CubicBezierCurve::new_linear(
            Vector2 { x: 0.0, y: decay.y },
            Vector2 {
                x: length,
                y: decay.y * amplitude_percent,
            },
        ));
        Ok(self)
    }

    pub fn release(&mut self, length: f32) -> Result<&mut Self, Error> {
        let sustain = self
            .sustain
            .as_ref()
            .expect("Sustain should be initialized before Release")
            .end();
        self.release = Some(CubicBezierCurve::new_linear(
            Vector2 {
                x: 0.0,
                y: sustain.y,
            },
            Vector2 { x: length, y: 0.0 },
        ));
        Ok(self)
    }

    pub fn build(&mut self) -> Result<ADSREnvelope, Error> {
        let adsr = ADSREnvelope {
            attack: Self::create_shared_curve(self.attack.take().ok_or("Attack not specified")?),
            decay: Self::create_shared_curve(self.decay.take().ok_or("Decay not specified")?),
            sustain: Self::create_shared_curve(self.sustain.take().ok_or("Sustain not specified")?),
            release: Self::create_shared_curve(self.release.take().ok_or("Release not specified")?),
        };
        Ok(adsr)
    }

    fn create_shared_curve(curve: CubicBezierCurve) -> SharedCurve {
        let curve = make_shared(curve);

        let curve_clone = curve.clone();
        let curve_clone2 = curve.clone();
        let length = CallbackParametr::new(
            move |v| curve_clone.write().unwrap().a.x = v,
            move || curve_clone2.read().unwrap().a.x,
            || (0.0, 10.0),
        );
        let curve_clone = curve.clone();
        let curve_clone2 = curve.clone();
        let amplitude = CallbackParametr::new(
            move |v| curve_clone.write().unwrap().a.y = v,
            move || curve_clone2.read().unwrap().a.y,
            || (0.0, 1.0),
        );

        let curve_clone = curve.clone();
        let curve_clone2 = curve.clone();
        let point_b_x = CallbackParametr::new(
            move |v| curve_clone.write().unwrap().b.x = v,
            move || curve_clone2.read().unwrap().b.x,
            || (0.0, 1.0),
        );
        let curve_clone = curve.clone();
        let curve_clone2 = curve.clone();
        let point_b_y = CallbackParametr::new(
            move |v| curve_clone.write().unwrap().b.y = v,
            move || curve_clone2.read().unwrap().b.y,
            || (0.0, 1.0),
        );
        let point_b: (SharedParametr<f32>, SharedParametr<f32>) =
            (make_shared(point_b_x), make_shared(point_b_y));

        let curve_clone = curve.clone();
        let curve_clone2 = curve.clone();
        let point_c_x = CallbackParametr::new(
            move |v| curve_clone.write().unwrap().c.x = v,
            move || curve_clone2.read().unwrap().c.x,
            || (0.0, 1.0),
        );
        let curve_clone = curve.clone();
        let curve_clone2 = curve.clone();
        let point_c_y = CallbackParametr::new(
            move |v| curve_clone.write().unwrap().c.y = v,
            move || curve_clone2.read().unwrap().c.y,
            || (0.0, 1.0),
        );
        let point_c: (SharedParametr<f32>, SharedParametr<f32>) =
            (make_shared(point_c_x), make_shared(point_c_y));

        SharedCurve {
            length: make_shared(length),
            amplitude: make_shared(amplitude),
            point_b,
            point_c,
            curve,
        }
    }
}

impl Default for ADSREnvelopeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ADSREnvelope {
    fn default() -> Self {
        ADSREnvelopeBuilder::new()
            .attack(0.1, 1.)
            .unwrap()
            .decay(0.2, 0.5)
            .unwrap()
            .sustain(0.03, 1.0)
            .unwrap()
            .release(0.1)
            .unwrap()
            .build()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::adsr_envelope::{ADSREnvelopeBuilder, State};
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_builder() {
        let adsr = ADSREnvelopeBuilder::new()
            .attack(1.0, 0.7)
            .unwrap()
            .decay(0.45, 0.4)
            .unwrap()
            .sustain(0.0, 1.0)
            .unwrap()
            .release(1.0)
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(adsr.time_range_of(State::Release).1, 2.45);
    }

    #[test]
    fn test_evaluate() {
        let adsr = ADSREnvelopeBuilder::new()
            .attack(0.5, 0.8)
            .unwrap()
            .decay(0.5, 0.5)
            .unwrap()
            .sustain(0.0, 1.0)
            .unwrap()
            .release(1.0)
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
