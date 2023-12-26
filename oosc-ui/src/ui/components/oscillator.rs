use std::sync::Arc;

use crossterm::event::KeyCode;
use oosc_core::{
    core::{oscillator::WavetableOscillator, synthesizer::LockedOscillator},
    utils::{interpolation::InterpolateMethod, make_shared, Shared},
};
use ratatui::{prelude::*, widgets::*};

use super::{
    parametr::{ParametrComponentF32, ParametrComponentI32},
    wavetable::WavetableComponent,
    Component, Focus, FocusableComponent,
};

type ParametrsContainer = Vec<Shared<dyn FocusableComponent>>;

pub struct OscillatorComponent {
    pub oscillator: LockedOscillator,
    pub wavetable: WavetableComponent,
    pub parametrs: ParametrsContainer,
    pub last_focus: Option<Shared<dyn FocusableComponent>>,
    pub rect: Option<Rect>,
    pub focused: bool,
    pub keymap: KeyCode,
}

impl OscillatorComponent {
    pub fn new(oscillator: LockedOscillator, keymap: KeyCode) -> Self {
        let mut osc = oscillator.write().unwrap();

        let osc = osc
            .as_any_mut()
            .downcast_mut::<WavetableOscillator>()
            .unwrap();
        let wavetable = WavetableComponent::from(osc.wavetable());
        let parametrs: ParametrsContainer = vec![
            make_shared(ParametrComponentF32::new(
                "Pan".to_owned(),
                osc.pan(),
                Direction::Horizontal,
                10,
                InterpolateMethod::Linear,
                KeyCode::Char('p'),
            )),
            make_shared(ParametrComponentI32::new(
                "Octave".to_owned(),
                osc.octave_offset(),
                Direction::Vertical,
                KeyCode::Char('o'),
            )),
            make_shared(ParametrComponentI32::new(
                "Cents".to_owned(),
                osc.cents_offset(),
                Direction::Vertical,
                KeyCode::Char('c'),
            )),
        ];

        Self {
            oscillator: oscillator.clone(),
            wavetable,
            parametrs,
            last_focus: None,
            rect: None,
            focused: false,
            keymap,
        }
    }
}

impl FocusableComponent for OscillatorComponent {}

impl Component for OscillatorComponent {
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

        self.parametrs
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, p)| p.write().unwrap().draw(f, parameters_layout[i]))?;
        Ok(())
    }

    fn resize(&mut self, rect: Rect) -> anyhow::Result<()> {
        self.rect = Some(rect);
        Ok(())
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        if !self.focused {
            return Ok(());
        }
        let parametrs = &mut self.parametrs;
        parametrs
            .iter_mut()
            .try_for_each(|p| p.write().unwrap().handle_key_events(key))?;
        parametrs.iter_mut().for_each(|p| {
            let mut param = p.write().unwrap();
            if param.keymap() == key.code {
                let last = self.last_focus.clone();
                if let Some(last) = last {
                    if !Arc::ptr_eq(&last, p) {
                        last.write().unwrap().unfocus();
                    }
                }
                param.focus();
                self.last_focus = Some(p.clone());
            }
        });
        if let KeyCode::Esc = key.code {
            self.unfocus()
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

    fn keymap(&self) -> KeyCode {
        self.keymap
    }
}
