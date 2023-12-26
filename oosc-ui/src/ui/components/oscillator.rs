use crossterm::event::KeyCode;
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
    pub cents: ParametrComponentI32,
    pub rect: Option<Rect>,
    pub focused: bool,
}

impl OscillatorComponent {
    pub fn new(oscillator: LockedOscillator) -> Self {
        let mut osc = oscillator.write().unwrap();

        let osc = osc
            .as_any_mut()
            .downcast_mut::<WavetableOscillator>()
            .unwrap();
        let wavetable = WavetableComponent::from(osc.wavetable());
        let pan = ParametrComponentF32::new(
            "Pan".to_owned(),
            osc.pan(),
            Direction::Horizontal,
            10,
            InterpolateMethod::Linear,
        );
        let octaves = ParametrComponentI32::new(
            "Octave".to_owned(),
            osc.octave_offset(),
            Direction::Vertical,
        );
        let cents =
            ParametrComponentI32::new("Cents".to_owned(), osc.cents_offset(), Direction::Vertical);

        Self {
            oscillator: oscillator.clone(),
            wavetable,
            pan,
            octaves,
            cents,
            rect: None,
            focused: false,
        }
    }

    fn unfocus_all(&mut self) {
        self.pan.unfocus();
        self.octaves.unfocus();
        self.cents.unfocus();
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
        let buf = f.buffer_mut();
        let b = Block::default()
            .borders(Borders::ALL)
            .title("osc")
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(self.color()));
        b.render(rect, buf);
        self.wavetable.draw(f, layout[0])?;
        let parameters_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Min(0),
            ])
            .margin(1)
            .split(layout[1]);
        self.pan.draw(f, parameters_layout[0])?;
        self.octaves.draw(f, parameters_layout[1])?;
        self.cents.draw(f, parameters_layout[2])?;
        Ok(())
    }

    fn resize(&mut self, rect: Rect) -> anyhow::Result<()> {
        self.rect = Some(rect);
        Ok(())
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        self.pan.handle_key_events(key)?;
        self.octaves.handle_key_events(key)?;
        self.cents.handle_key_events(key)?;
        if !self.focused {
            return Ok(());
        }
        match key.code {
            KeyCode::Char('z') => {
                self.unfocus_all();
                self.pan.focus();
            }
            KeyCode::Char('x') => {
                self.unfocus_all();
                self.octaves.focus();
            }
            KeyCode::Char('c') => {
                self.unfocus_all();
                self.cents.focus();
            }
            KeyCode::Esc => self.unfocus(),
            _ => (),
        };
        Ok(())
    }
}

impl Focus for OscillatorComponent {
    fn focus(&mut self) {
        self.focused = true
    }

    fn unfocus(&mut self) {
        self.focused = false
    }

    fn is_focused(&self) -> bool {
        self.focused
    }
}
