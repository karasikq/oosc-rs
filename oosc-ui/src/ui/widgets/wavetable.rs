use oosc_core::{
    core::wavetable::WaveTable,
    utils::{consts::PI_2M, evaluate::Evaluate},
};
use ratatui::{
    prelude::*,
    widgets::{canvas::*, *},
};

pub struct WavetableWidget<'a> {
    pub table: &'a WaveTable,
    pub samples: i32,
}

impl<'a> Widget for WavetableWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let table = self.table;
        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::TOP).title("Wavetable"))
            .marker(Marker::Braille)
            .x_bounds([0.0, PI_2M as f64])
            .y_bounds([-1.0, 1.0])
            .paint(|ctx| {
                let samples = 30;
                let rate = PI_2M / samples as f32;
                let mut line = canvas::Line::new(0.0, 0.0, 0.0, 0.0, Color::Red);
                for t in 1..samples {
                    let x1 = (t - 1) as f32 * rate;
                    let x2 = t as f32 * rate;
                    line.x1 = x1 as f64;
                    line.y1 = table.evaluate(x1).unwrap() as f64;
                    line.x2 = x2 as f64;
                    line.y2 = table.evaluate(x2).unwrap() as f64;
                    ctx.draw(&line);
                }
            });
        canvas.render(area, buf);
    }
}
