use oosc_core::core::oscillator::WavetableOscillator;
use ratatui::{prelude::*, widgets::*};

use crate::app::application::Application;

use super::widgets::oscillator::OscillatorWidget;

pub struct Renderer<'a> {
    app: &'a mut Application,
}

impl<'a> Renderer<'a> {
    pub fn new(app: &'a mut Application) -> Self {
        Self { app }
    }
}

impl<'a> Widget for Renderer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut syn = self.app.ctx.synthesizer.lock().unwrap();
        let mut osc: Vec<&mut WavetableOscillator> =
            syn.get_oscillators::<WavetableOscillator>().collect();
        let size = 100 / osc.len();
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                std::iter::repeat_with(|| Constraint::Percentage(size as u16))
                    .take(osc.len())
                    .collect::<Vec<_>>(),
            )
            .margin(1)
            .split(area);
        let border = Block::default()
            .borders(Borders::ALL)
            .title("oosc")
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center);
        border.render(area, buf);
        osc.iter_mut().enumerate().for_each(|(i, oscillator)| {
            let widget = OscillatorWidget { oscillator };
            widget.render(layout[i], buf);
        });
    }
}
