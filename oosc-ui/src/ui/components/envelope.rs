use std::rc::Rc;

use crossterm::event::KeyCode;
use oosc_core::utils::{
    adsr_envelope::{ADSREnvelope, State},
    Shared,
};
use ratatui::{
    prelude::*,
    widgets::{canvas::*, *},
};

use super::{
    bezier::BezierComponent, Component, Focus, FocusableComponent, FocusableComponentContext,
};

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
}

pub struct EnvelopeComponent {
    pub envelope: Shared<ADSREnvelope>,
    pub samples: usize,
    ctx: FocusableComponentContext,
    bezier: BezierComponent,
    state: ShowState,
    line: Vec<canvas::Line>,
    layout: Option<EnvelopeLayout>,
}

impl FocusableComponent for EnvelopeComponent {
    fn context(&self) -> &FocusableComponentContext {
        &self.ctx
    }

    fn context_mut(&mut self) -> &mut FocusableComponentContext {
        &mut self.ctx
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl EnvelopeComponent {
    pub fn new(envelope: Shared<ADSREnvelope>) -> Self {
        let bezier = BezierComponent::new(&envelope.read().unwrap().attack);
        let ctx = FocusableComponentContext::new().keymap(KeyCode::Char('e'));

        Self {
            envelope,
            samples: 0,
            ctx,
            bezier,
            state: ShowState::Info,
            line: vec![],
            layout: None,
        }
    }

    pub fn samples(self, samples: usize) -> Self {
        Self { samples, ..self }
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
                let mut line = canvas::Line::new(0.0, 0.0, 0.0, 0.0, self.color());
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
            .constraints([Constraint::Min(0), Constraint::Length(2)])
            .margin(1)
            .split(rect)
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
        match self.state {
            ShowState::Info => {
                let layout = self.layout.as_ref().unwrap();
                let max_time = self
                    .envelope
                    .read()
                    .unwrap()
                    .time_range_of(State::Release)
                    .1;
                self.line = self.render_line();
                let canvas = Canvas::default()
                    .marker(Marker::Braille)
                    .x_bounds([0.0, max_time as f64])
                    .y_bounds([0.0, 1.0])
                    .paint(|ctx| {
                        self.line.iter().for_each(|line| ctx.draw(line));
                    });
                canvas.render(layout.main[0], f.buffer_mut());
                let p = Paragraph::new("Attack [a] Decay [d] Sustain [s] Release [r]")
                    .wrap(Wrap { trim: true })
                    .alignment(Alignment::Center);
                f.render_widget(p, layout.main[1]);
                let b = Block::default()
                    .borders(Borders::TOP | Borders::BOTTOM | Borders::LEFT)
                    .title("Envelope")
                    .style(Style::default().fg(self.color()));
                f.render_widget(b, layout.rect);
                Ok(())
            }
            _ => self.bezier.draw(f, rect),
        }
    }

    fn resize(&mut self, rect: Rect) -> anyhow::Result<()> {
        let main = Self::build_main_layout(rect);
        self.bezier.resize(rect)?;
        self.layout = Some(EnvelopeLayout { rect, main });
        Ok(())
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        if !self.is_focused() {
            return Ok(());
        }
        self.bezier.handle_key_events(key)?;
        match key.code {
            KeyCode::Esc => {
                self.unfocus();
                self.state = ShowState::Info;
            }
            KeyCode::Char('a') => {
                self.state = ShowState::Attack;
                self.bezier.new_curve(&self.envelope.read().unwrap().attack)
            }
            KeyCode::Char('d') => {
                self.state = ShowState::Decay;
                self.bezier.new_curve(&self.envelope.read().unwrap().decay)
            }
            KeyCode::Char('s') => {
                self.state = ShowState::Sustain;
                self.bezier
                    .new_curve(&self.envelope.read().unwrap().sustain)
            }
            KeyCode::Char('r') => {
                self.state = ShowState::Release;
                self.bezier
                    .new_curve(&self.envelope.read().unwrap().release)
            }
            _ => (),
        };
        Ok(())
    }
}
