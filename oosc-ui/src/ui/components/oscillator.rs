use oosc_core::core::{
    oscillator::WavetableOscillator, parametrs::Parametr, synthesizer::LockedOscillator,
};
use ratatui::{prelude::*, widgets::*};

use super::{wavetable::WavetableComponent, Component, Focus, FocusableComponent};

pub enum Action {
    EnterFullscreen,
    MoveNextControl,
    MovePreviousControl,
    ExitFullscreen,
}

pub struct OscillatorComponent {
    pub oscillator: LockedOscillator,
    pub wavetable: WavetableComponent,
    pub rect: Option<Rect>,
}

impl OscillatorComponent {
    pub fn new(oscillator: LockedOscillator) -> Self {
        let mut osc = oscillator.write().unwrap();

        let osc = osc
            .as_any_mut()
            .downcast_mut::<WavetableOscillator>()
            .unwrap();
        let wavatable = WavetableComponent::from(osc.get_wavetable());
        Self {
            oscillator: oscillator.clone(),
            wavetable: wavatable,
            rect: None,
        }
    }
}

impl FocusableComponent for OscillatorComponent {}

impl Component for OscillatorComponent {
    type Action = Action;

    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        let rect = self.rect.unwrap();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
            .margin(1)
            .split(rect);
        self.wavetable.draw(f, layout[0])?;
        let buf = f.buffer_mut();
        let mut osc = self.oscillator.write().unwrap();
        let osc = osc
            .as_any_mut()
            .downcast_mut::<WavetableOscillator>()
            .unwrap();

        let pan = osc.pan().get_value();
        let p = Paragraph::new(format!("Pan: {}", pan)).style(Style::default().fg(Color::White));
        p.render(layout[1], buf);
        let b = Block::default()
            .borders(Borders::ALL)
            .title("oosc")
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        b.render(rect, buf);
        Ok(())
    }
}

impl Focus for OscillatorComponent {
    fn focus(&mut self) {
        todo!()
    }

    fn unfocus(&mut self) {
        todo!()
    }

    fn is_focused(&self) -> bool {
        todo!()
    }
}
