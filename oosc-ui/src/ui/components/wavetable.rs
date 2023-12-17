use oosc_core::{
    core::wavetable::WaveTable,
    utils::{consts::PI_2M, evaluate::Evaluate},
};
use ratatui::{
    prelude::*,
    widgets::{canvas::*, *},
};

use super::{Component, EmptyAction};

pub struct WavetableComponent {
    pub line: Vec<canvas::Line>,
    pub samples: i32,
}

impl WavetableComponent {
    pub fn new(samples: i32) -> Self {
        Self {
            line: vec![],
            samples,
        }
    }

    pub fn generate(&mut self, table: &WaveTable) -> &mut Self {
        let rate = PI_2M / self.samples as f32;
        self.line = (1..self.samples)
            .map(|t| {
                let mut line = canvas::Line::new(0.0, 0.0, 0.0, 0.0, Color::Red);
                let x1 = (t - 1) as f32 * rate;
                let x2 = t as f32 * rate;
                line.x1 = x1 as f64;
                line.y1 = table.evaluate(x1).unwrap() as f64;
                line.x2 = x2 as f64;
                line.y2 = table.evaluate(x2).unwrap() as f64;
                line
            })
            .collect();
        self
    }
}

impl<'a, T> From<T> for WavetableComponent
where
    T: Into<&'a mut WaveTable>,
{
    fn from(value: T) -> Self {
        let mut component = Self::new(30);
        component.generate(value.into());
        component
    }
}

impl Component for WavetableComponent {
    type Action = EmptyAction;

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> anyhow::Result<()> {
        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::TOP).title("Wavetable"))
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
