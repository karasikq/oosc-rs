use std::rc::Rc;

use oosc_core::utils::{
    adsr_envelope::{ADSREnvelope, State},
    Shared,
};
use ratatui::{
    prelude::*,
    widgets::{canvas::*, *},
};

use super::Component;

enum ShowState {
    Info,
    Attack,
    Decay,
    Sustain,
    Release,
}

struct EnvelopeLayout {
    pub rect: Rect,
    pub main: Rc<[Rect]>,
    pub selected: Rc<[Rect]>,
    pub parametrs: Rc<[Rect]>,
}

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

    fn build_main_layout(rect: Rect) -> Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .margin(1)
            .split(rect)
    }

    fn build_selected_layout(rect: Rect) -> Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .margin(1)
            .split(rect)
    }

    fn build_parametrs_layout(rect: Rect) -> Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100 / 6); 6])
            .margin(1)
            .split(rect)
    }

    /* fn build_parametr_components(envelope: &ADSREnvelope) -> Vec<Shared<dyn FocusableComponent>> {
        vec![make_shared(ParametrComponentF32::new(
            "Length".to_owned(),
            envelope.attack.length,
            Direction::Horizontal,
            10,
            InterpolateMethod::Exponential(10000.0),
            KeyCode::Char('l'),
        ))]
    } */
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
        self.line = self.render_line();
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

    fn resize(&mut self, rect: Rect) -> anyhow::Result<()> {
        let main = Self::build_main_layout(rect);
        let selected = Self::build_selected_layout(rect);
        let parametrs = Self::build_parametrs_layout(selected[1]);
        /* self.layout = Some(EnvelopeLayout {
            rect,
            main,
            selected,
            parametrs,
        }); */
        Ok(())
    }
}
