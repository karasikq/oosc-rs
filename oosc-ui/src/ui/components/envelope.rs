use oosc_core::utils::{
    adsr_envelope::{ADSREnvelope, State},
    Shared,
};
use ratatui::{
    prelude::*,
    widgets::{canvas::*, *},
};

use super::Component;

pub struct EnvelopeComponent {
    pub envelope: Shared<ADSREnvelope>,
    pub samples: usize,
    line: Vec<canvas::Line>,
    color: Color,
}

impl EnvelopeComponent {
    pub fn new(envelope: Shared<ADSREnvelope>) -> Self {
        Self {
            envelope,
            samples: 0,
            line: vec![],
            color: Color::Red,
        }
    }

    pub fn samples(self, samples: usize) -> Self {
        Self { samples, ..self }
    }

    pub fn color(self, color: Color) -> Self {
        Self { color, ..self }
    }

    pub fn build(mut self) -> Self {
        self.line = self.render_line();
        self
    }

    pub fn render_line(&self) -> Vec<canvas::Line> {
        let table = self.envelope.read().unwrap();
        let max_time = table.time_range_of(State::Release).1;
        let rate = max_time / self.samples as f32;
        (1..=self.samples)
            .map(|t| {
                let mut line = canvas::Line::new(0.0, 0.0, 0.0, 0.0, self.color);
                let x1 = (t - 1) as f32 * rate;
                let x2 = t as f32 * rate;
                line.x1 = x1 as f64;
                line.y1 = table.evaluate(x1) as f64;
                line.x2 = x2 as f64;
                line.y2 = table.evaluate(x2) as f64;
                line
            })
            .collect()
    }
}

impl<T> From<T> for EnvelopeComponent
where
    T: Into<Shared<ADSREnvelope>>,
{
    fn from(value: T) -> Self {
        Self::new(value.into().clone()).samples(30).build()
    }
}

impl Component for EnvelopeComponent {
    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> anyhow::Result<()> {
        let max_time = self
            .envelope
            .read()
            .unwrap()
            .time_range_of(State::Release)
            .1;
        let canvas = Canvas::default()
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::BOTTOM | Borders::LEFT)
                    .title("Envelope"),
            )
            .marker(Marker::Braille)
            .x_bounds([0.0, max_time as f64])
            .y_bounds([0.0, 1.0])
            .paint(|ctx| {
                self.line.iter().for_each(|line| ctx.draw(line));
            });
        canvas.render(rect, f.buffer_mut());
        Ok(())
    }
}
