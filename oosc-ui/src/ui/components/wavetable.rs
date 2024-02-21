use oosc_core::{
    core::wavetable::WaveTable,
    utils::{consts::PI_2M, evaluate::Evaluate, Shared},
};
use ratatui::{
    prelude::*,
    widgets::{canvas::*, *},
};

use crate::ui::observer::Observer;

use super::{
    parameter::ParameterEvent, AutoFocus, Component, Focus, FocusableComponent,
    FocusableComponentContext,
};

pub struct WavetableComponent {
    pub wavetable: Shared<WaveTable>,
    pub samples: usize,
    ctx: FocusableComponentContext,
    line: Vec<canvas::Line>,
    last_pos: usize,
}

impl WavetableComponent {
    pub fn new(wavetable: Shared<WaveTable>) -> Self {
        Self {
            wavetable,
            samples: 0,
            ctx: FocusableComponentContext::new().focused(false),
            line: vec![],
            last_pos: 0,
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
        let table = self.wavetable.read().unwrap();
        let rate = PI_2M / self.samples as f32;
        (1..=self.samples)
            .map(|t| {
                let mut line = canvas::Line::new(0.0, 0.0, 0.0, 0.0, self.color());
                let x1 = (t - 1) as f32 * rate;
                let x2 = t as f32 * rate;
                line.x1 = x1 as f64;
                line.y1 = table.evaluate(x1).unwrap() as f64;
                line.x2 = x2 as f64;
                line.y2 = table.evaluate(x2).unwrap() as f64;
                line
            })
            .collect()
    }
}

impl<T> From<T> for WavetableComponent
where
    T: Into<Shared<WaveTable>>,
{
    fn from(value: T) -> Self {
        Self::new(value.into().clone()).samples(30).build()
    }
}

impl Component for WavetableComponent {
    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> anyhow::Result<()> {
        if self.last_pos != self.wavetable.read().unwrap().position() {
            self.line = self.render_line();
        }
        let canvas = Canvas::default()
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::BOTTOM)
                    .title("Wavetable")
                    .style(Style::default().fg(self.color())),
            )
            .marker(Marker::Braille)
            .x_bounds([0.0, PI_2M as f64])
            .y_bounds([-1.0, 1.0])
            .paint(|ctx| {
                self.line.iter().for_each(|line| ctx.draw(line));
            });
        canvas.render(rect, f.buffer_mut());
        Ok(())
    }
}

impl Observer<ParameterEvent<i32>> for WavetableComponent {
    fn react(&mut self, _value: ParameterEvent<i32>) {
        self.line = self.render_line();
    }
}

impl AutoFocus for WavetableComponent {}

impl FocusableComponent for WavetableComponent {
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
