use crossterm::event::KeyCode;
use oosc_core::{
    core::parametrs::{Parametr, SharedParametr},
    utils::interpolation::{interpolate_range, InterpolateMethod},
};
use ratatui::{prelude::*, widgets::*};
use ratatui::{
    style::{Color, Style},
    symbols::Marker,
    widgets::canvas::{Canvas, Rectangle},
};

use crate::ui::widgets::bar::BarWidget;

use super::{Component, EmptyAction, Focus};

trait AnyParametrComponent {
    fn name(&self) -> &String;
    fn value(&self) -> f32;
    fn range(&self) -> (f32, f32);
    fn direction(&self) -> Direction;
    fn increment(&mut self);
    fn decrement(&mut self);
}

pub struct ParametrComponentF32 {
    name: String,
    parametr: SharedParametr<f32>,
    direction: Direction,
    steps: i32,
    interpolation_method: InterpolateMethod,
    focused: bool,
}

impl ParametrComponentF32 {
    pub fn new(
        name: String,
        parametr: SharedParametr<f32>,
        direction: Direction,
        steps: i32,
        interpolation_method: InterpolateMethod,
    ) -> Self {
        Self {
            name,
            parametr,
            direction,
            steps,
            interpolation_method,
            focused: false,
        }
    }
}

pub struct ParametrComponentI32 {
    name: String,
    parametr: SharedParametr<i32>,
    direction: Direction,
    focused: bool,
}

impl ParametrComponentI32 {
    pub fn new(name: String, parametr: SharedParametr<i32>, direction: Direction) -> Self {
        Self {
            name,
            parametr,
            direction,
            focused: false,
        }
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

    fn increment(&mut self) {
        let mut parametr = self.parametr.write().unwrap();
        let step = 1.0 / self.steps as f32;
        let time = { time_of(&*parametr) };
        let result = interpolate_range(parametr.range(), time + step, self.interpolation_method);
        parametr.set_value(result);
    }

    fn decrement(&mut self) {
        let mut parametr = self.parametr.write().unwrap();
        let step = 1.0 / self.steps as f32;
        let time = { time_of(&*parametr) };
        let result = interpolate_range(parametr.range(), time - step, self.interpolation_method);
        parametr.set_value(result);
    }

    fn direction(&self) -> Direction {
        self.direction
    }
}

impl AnyParametrComponent for ParametrComponentI32 {
    fn increment(&mut self) {
        let mut parametr = self.parametr.write().unwrap();
        let result = { parametr.get_value() + 1 };
        parametr.set_value(result);
    }

    fn decrement(&mut self) {
        let mut parametr = self.parametr.write().unwrap();
        let result = { parametr.get_value() - 1 };
        parametr.set_value(result);
    }

    fn range(&self) -> (f32, f32) {
        let range = self.parametr.read().unwrap().range();
        (range.0 as f32, range.1 as f32)
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn value(&self) -> f32 {
        self.parametr.read().unwrap().get_value() as f32
    }

    fn direction(&self) -> Direction {
        self.direction
    }
}

impl<T: AnyParametrComponent + Focus> Component for T {
    type Action = EmptyAction;

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
            .title(self.name().as_str())
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(self.color()));
        f.render_widget(b, rect);
        f.render_widget(bar, layout[0]);
        let p = Paragraph::new(format!("{:.2}", self.value())).alignment(Alignment::Center);
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
}

fn time_of(param: &(impl Parametr<f32> + ?Sized)) -> f32 {
    let range = param.range();
    (param.get_value() - range.0) / (range.1 - range.0)
}
