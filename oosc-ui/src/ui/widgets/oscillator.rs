use oosc_core::core::{oscillator::WavetableOscillator, parametrs::Parametr};
use ratatui::{prelude::*, widgets::*};

use super::wavetable::WavetableWidget;

pub struct OscillatorWidget<'a> {
    pub oscillator: &'a mut WavetableOscillator,
}

impl<'a> Widget for OscillatorWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
            .margin(1)
            .split(area);

        let wavetable = WavetableWidget {
            table: self.oscillator.get_wavetable(),
            samples: 30,
        };
        wavetable.render(layout[0], buf);

        let pan = self.oscillator.pan().get_value();
        let p = Paragraph::new(format!("Pan: {}", pan)).style(Style::default().fg(Color::White));
        p.render(layout[1], buf);
        let b = Block::default()
            .borders(Borders::ALL)
            .title("oosc")
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        b.render(area, buf);
    }
}
