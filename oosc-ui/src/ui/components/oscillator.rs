use oosc_core::{
    core::{oscillator::WavetableOscillator, synthesizer::LockedOscillator},
    utils::interpolation::InterpolateMethod,
};
use ratatui::{prelude::*, widgets::*};

use super::{
    parametr::{ParametrComponentF32, ParametrComponentI32},
    wavetable::WavetableComponent,
    Component, Focus, FocusableComponent,
};

pub enum Action {
    EnterFullscreen,
    MoveNextControl,
    MovePreviousControl,
    ExitFullscreen,
}

pub struct OscillatorComponent {
    pub oscillator: LockedOscillator,
    pub wavetable: WavetableComponent,
    pub pan: ParametrComponentF32,
    pub octaves: ParametrComponentI32,
    pub rect: Option<Rect>,
}

impl OscillatorComponent {
    pub fn new(oscillator: LockedOscillator) -> Self {
        let mut osc = oscillator.write().unwrap();

        let osc = osc
            .as_any_mut()
            .downcast_mut::<WavetableOscillator>()
            .unwrap();
        let wavetable = WavetableComponent::from(osc.wavetable());
        let pan =
            ParametrComponentF32::new("Pan".to_owned(), osc.pan(), 10, InterpolateMethod::Linear);
        let octaves =
            ParametrComponentI32::new("Octave".to_owned(), osc.octave_offset());

        Self {
            oscillator: oscillator.clone(),
            wavetable,
            pan,
            octaves,
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
            .constraints([Constraint::Percentage(80), Constraint::Percentage(10), Constraint::Percentage(10)])
            // .margin(1)
            .split(rect);
        self.wavetable.draw(f, layout[0])?;
        self.pan.draw(f, layout[1])?;
        self.octaves.draw(f, layout[2])?;
        let buf = f.buffer_mut();
        let mut osc = self.oscillator.write().unwrap();
        let osc = osc
            .as_any_mut()
            .downcast_mut::<WavetableOscillator>()
            .unwrap();

        let b = Block::default()
            .borders(Borders::ALL)
            .title("osc")
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        b.render(rect, buf);
        Ok(())
    }

    fn resize(&mut self, rect: Rect) -> anyhow::Result<()> {
        self.rect = Some(rect);
        Ok(())
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        self.pan.handle_key_events(key)?;
        self.octaves.handle_key_events(key)
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
