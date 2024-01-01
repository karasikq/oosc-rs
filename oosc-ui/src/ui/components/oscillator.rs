use std::{rc::Rc, sync::Arc};

use crossterm::event::KeyCode;
use oosc_core::{
    core::{oscillator::WavetableOscillator, synthesizer::LockedOscillator},
    utils::{interpolation::InterpolateMethod, make_shared, Shared},
};
use ratatui::{prelude::*, widgets::*};

use crate::ui::observer::Notifier;

use super::{
    envelope::EnvelopeComponent,
    parametr::{ParametrComponentF32, ParametrComponentI32},
    wavetable::WavetableComponent,
    Component, ComponentsContainer, Focus, FocusableComponent, FocusableComponentContext,
};

struct OscillatorLayout {
    pub rect: Rect,
    pub top: Rc<[Rect]>,
    pub parametrs: Rc<[Rect]>,
}

pub struct OscillatorComponent {
    pub oscillator: LockedOscillator,
    pub wavetable: Shared<WavetableComponent>,
    pub envelope: Shared<EnvelopeComponent>,
    pub parametrs: ComponentsContainer<dyn FocusableComponent>,
    context: FocusableComponentContext,
    layout: Option<OscillatorLayout>,
}

impl OscillatorComponent {
    pub fn new(oscillator: LockedOscillator, keymap: KeyCode) -> Self {
        let mut osc = oscillator.write().unwrap();

        let osc = osc
            .as_any_mut()
            .downcast_mut::<WavetableOscillator>()
            .unwrap();
        let mut parametrs = ComponentsContainer::from(Self::build_parametr_components(osc));
        let wavetable = make_shared(WavetableComponent::from(osc.wavetable()));
        let wt_pos = make_shared(ParametrComponentI32::new(
            "Wt Pos".to_owned(),
            osc.wavetable_position(),
            Direction::Vertical,
            KeyCode::Char('w'),
        ));
        wt_pos
            .write()
            .unwrap()
            .events()
            .subscribe(wavetable.clone());
        parametrs.components.push(wt_pos);
        let context = FocusableComponentContext::new().keymap(keymap);
        let envelope = make_shared(EnvelopeComponent::from(osc.envelope()));

        Self {
            oscillator: oscillator.clone(),
            wavetable,
            envelope,
            parametrs,
            context,
            layout: None,
        }
    }

    fn build_parametr_components(osc: &WavetableOscillator) -> Vec<Shared<dyn FocusableComponent>> {
        vec![
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
            make_shared(ParametrComponentF32::new(
                "Gain".to_owned(),
                osc.gain(),
                Direction::Vertical,
                20,
                InterpolateMethod::Exponential(0.001),
                KeyCode::Char('g'),
            )),
            make_shared(ParametrComponentF32::new(
                "Attack".to_owned(),
                osc.envelope().read().unwrap().attack.length.clone(),
                Direction::Horizontal,
                30,
                InterpolateMethod::Exponential(10000.0),
                KeyCode::Char('l'),
            )),
        ]
    }

    fn build_main_layout(rect: Rect) -> Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .margin(1)
            .split(rect)
    }

    fn build_top_layout(rect: Rect) -> Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rect)
    }

    fn build_parametrs_layout<T: FocusableComponent + ?Sized>(
        rect: Rect,
        parametrs: &ComponentsContainer<T>,
    ) -> Rc<[Rect]> {
        let len = parametrs.components.len();
        let size = 100 / len;
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                std::iter::repeat_with(|| Constraint::Percentage(size as u16))
                    .take(len)
                    .collect::<Vec<_>>(),
            )
            .split(rect)
    }
}

impl FocusableComponent for OscillatorComponent {
    fn context(&self) -> &FocusableComponentContext {
        &self.context
    }

    fn context_mut(&mut self) -> &mut FocusableComponentContext {
        &mut self.context
    }
}

impl Component for OscillatorComponent {
    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        _rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        if self.layout.is_none() {
            return Err(oosc_core::error::Error::from("Create layout before draw"))?;
        }
        let layout = self.layout.as_ref().unwrap();
        let buf = f.buffer_mut();
        let b = Block::default()
            .borders(Borders::ALL)
            .title("osc")
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(self.color()));
        b.render(layout.rect, buf);
        self.wavetable.write().unwrap().draw(f, layout.top[0])?;
        self.envelope.write().unwrap().draw(f, layout.top[1])?;
        self.parametrs.draw_in_layout(f, &layout.parametrs)?;
        Ok(())
    }

    fn resize(&mut self, rect: Rect) -> anyhow::Result<()> {
        let main = Self::build_main_layout(rect);
        let top = Self::build_top_layout(main[0]);
        let parametrs = Self::build_parametrs_layout(main[1], &self.parametrs);
        self.parametrs
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, p)| p.write().unwrap().resize(parametrs[i]))?;
        self.layout = Some(OscillatorLayout {
            rect,
            top,
            parametrs: parametrs.clone(),
        });
        Ok(())
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        if !self.is_focused() {
            return Ok(());
        }
        let parametrs = &mut self.parametrs;
        parametrs
            .iter_mut()
            .try_for_each(|p| p.write().unwrap().handle_key_events(key))?;
        parametrs.iter_mut().for_each(|p| {
            let mut param = p.write().unwrap();
            if param.keymap() == Some(key.code) {
                let last = self.context.last_focus.clone();
                if let Some(last) = last {
                    if !Arc::ptr_eq(&last, p) {
                        last.write().unwrap().unfocus();
                    }
                }
                param.focus();
                self.context.last_focus = Some(p.clone());
            }
        });
        if let KeyCode::Esc = key.code {
            self.unfocus()
        };
        Ok(())
    }
}
