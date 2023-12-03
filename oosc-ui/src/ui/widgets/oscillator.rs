use oosc_core::core::{oscillator::WavetableOscillator, parametrs::Parametr};
use ratatui::{prelude::*, widgets::*};

pub struct OscillatorWidget<'a> {
    pub oscillator: &'a mut WavetableOscillator,
}

impl<'a> Widget for OscillatorWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let pan = self.oscillator.get_pan().get_value_or_default();
        let p = Paragraph::new(format!("Pan: {}", pan))
            .style(Style::default().fg(Color::Red))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("oosc")
                    .border_type(BorderType::Rounded)
                    .title_alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Red)),
            );
        p.render(area, buf);
    }
}
