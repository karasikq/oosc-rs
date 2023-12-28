use crossterm::event::KeyCode;
use oosc_core::{
    core::parametrs::{Parametr, SharedParametr},
    utils::interpolation::{interpolate_range, InterpolateMethod},
};
use ratatui::style::{Color, Style};
use ratatui::{prelude::*, widgets::*};

use crate::ui::{
    observer::{Notifier, NotifierContainer},
    utils::keycode_to_string,
    widgets::bar::BarWidget,
};

use super::{Component, Focus, FocusableComponent};

trait AnyParametrComponent {
    fn name(&self) -> &String;
    fn value(&self) -> f32;
    fn range(&self) -> (f32, f32);
    fn direction(&self) -> Direction;
    fn format_value(&self) -> String;
    fn increment(&mut self);
    fn decrement(&mut self);
}

#[derive(Eq, PartialEq, Hash)]
pub enum ParametrEvent {
    ValueChanged,
}

type EventContainer<T> = NotifierContainer<ParametrEvent, SharedParametr<T>>;

pub struct ParametrComponentF32 {
    name: String,
    parametr: SharedParametr<f32>,
    direction: Direction,
    steps: i32,
    interpolation_method: InterpolateMethod,
    focused: bool,
    keymap: KeyCode,
    events: EventContainer<f32>,
}

impl ParametrComponentF32 {
    pub fn new(
        name: String,
        parametr: SharedParametr<f32>,
        direction: Direction,
        steps: i32,
        interpolation_method: InterpolateMethod,
        keymap: KeyCode,
    ) -> Self {
        Self {
            name,
            parametr,
            direction,
            steps,
            interpolation_method,
            focused: false,
            keymap,
            events: EventContainer::<f32>::default(),
        }
    }

    pub fn events(&mut self) -> &mut impl Notifier<SharedParametr<f32>, ParametrEvent> {
        &mut self.events
    }
}

impl FocusableComponent for ParametrComponentF32 {}

pub struct ParametrComponentI32 {
    name: String,
    parametr: SharedParametr<i32>,
    direction: Direction,
    focused: bool,
    keymap: KeyCode,
    events: EventContainer<i32>,
}

impl FocusableComponent for ParametrComponentI32 {}

impl ParametrComponentI32 {
    pub fn new(
        name: String,
        parametr: SharedParametr<i32>,
        direction: Direction,
        keymap: KeyCode,
    ) -> Self {
        Self {
            name,
            parametr,
            direction,
            focused: false,
            keymap,
            events: EventContainer::<i32>::default(),
        }
    }

    pub fn events(&mut self) -> &mut impl Notifier<SharedParametr<i32>, ParametrEvent> {
        &mut self.events
    }
}

impl AnyParametrComponent for ParametrComponentF32 {
    fn name(&self) -> &String {
        &self.name
    }

    fn value(&self) -> f32 {
        self.parametr.read().unwrap().get_value()
    }

    fn range(&self) -> (f32, f32) {
        self.parametr.read().unwrap().range()
    }

    fn direction(&self) -> Direction {
        self.direction
    }

    fn format_value(&self) -> String {
        format!("{:.2}", self.value())
    }

    fn increment(&mut self) {
        {
            let mut parametr = self.parametr.write().unwrap();
            let step = 1.0 / self.steps as f32;
            let time = { time_of(&*parametr) };
            let result =
                interpolate_range(parametr.range(), time + step, self.interpolation_method);
            parametr.set_value(result);
        }
        self.events
            .notify(ParametrEvent::ValueChanged, self.parametr.clone());
    }

    fn decrement(&mut self) {
        {
            let mut parametr = self.parametr.write().unwrap();
            let step = 1.0 / self.steps as f32;
            let time = { time_of(&*parametr) };
            let result =
                interpolate_range(parametr.range(), time - step, self.interpolation_method);
            parametr.set_value(result);
        }
        self.events
            .notify(ParametrEvent::ValueChanged, self.parametr.clone());
    }
}

impl AnyParametrComponent for ParametrComponentI32 {
    fn name(&self) -> &String {
        &self.name
    }

    fn value(&self) -> f32 {
        self.parametr.read().unwrap().get_value() as f32
    }

    fn range(&self) -> (f32, f32) {
        let range = self.parametr.read().unwrap().range();
        (range.0 as f32, range.1 as f32)
    }

    fn direction(&self) -> Direction {
        self.direction
    }

    fn format_value(&self) -> String {
        format!("{:.0}", self.value().round())
    }

    fn increment(&mut self) {
        {
            let mut parametr = self.parametr.write().unwrap();
            let result = { parametr.get_value() + 1 };
            parametr.set_value(result);
        }
        self.events
            .notify(ParametrEvent::ValueChanged, self.parametr.clone());
    }

    fn decrement(&mut self) {
        {
            let mut parametr = self.parametr.write().unwrap();
            let result = { parametr.get_value() - 1 };
            parametr.set_value(result);
        }
        self.events
            .notify(ParametrEvent::ValueChanged, self.parametr.clone());
    }
}

impl<T: AnyParametrComponent + Focus> Component for T {
    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        let range = self.range();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .margin(1)
            .split(rect);
        let bar = BarWidget {
            resolution: (layout[0].width, layout[0].height),
            direction: self.direction(),
            bounds: range,
            center: 0.0,
            value: self.value(),
            color: self.color(),
        };
        let b = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                "{}[{}]",
                self.name().as_str(),
                keycode_to_string(self.keymap())
            ))
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(self.color()));
        f.render_widget(b, rect);
        f.render_widget(bar, layout[0]);
        let p = Paragraph::new(self.format_value()).alignment(Alignment::Center);
        f.render_widget(p, layout[1]);
        Ok(())
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        if !self.is_focused() {
            return Ok(());
        }
        match key.code {
            KeyCode::Char('h') => self.decrement(),
            KeyCode::Char('l') => self.increment(),
            KeyCode::Esc => self.unfocus(),
            _ => (),
        };
        Ok(())
    }
}

impl Focus for ParametrComponentF32 {
    fn focus(&mut self) {
        self.focused = true
    }

    fn unfocus(&mut self) {
        self.focused = false
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn color(&self) -> Color {
        if self.is_focused() {
            Color::Red
        } else {
            Color::Gray
        }
    }

    fn keymap(&self) -> KeyCode {
        self.keymap
    }
}

impl Focus for ParametrComponentI32 {
    fn focus(&mut self) {
        self.focused = true
    }

    fn unfocus(&mut self) {
        self.focused = false
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn color(&self) -> Color {
        if self.is_focused() {
            Color::Red
        } else {
            Color::Gray
        }
    }

    fn keymap(&self) -> KeyCode {
        self.keymap
    }
}

fn time_of(param: &(impl Parametr<f32> + ?Sized)) -> f32 {
    let range = param.range();
    (param.get_value() - range.0) / (range.1 - range.0)
}
