use crossterm::event::KeyCode;
use oosc_core::{
    core::parametrs::{Parametr, SharedParametr},
    utils::interpolation::{interpolate_range, InterpolateMethod},
};
use ratatui::{style::*, widgets::*};

use super::{Component, EmptyAction};

trait AnyParametrComponent {
    fn name(&self) -> &String;
    fn value(&self) -> String;
    fn increment(&mut self);
    fn decrement(&mut self);
}

pub struct ParametrComponentF32 {
    name: String,
    parametr: SharedParametr<f32>,
    steps: i32,
    interpolation_method: InterpolateMethod,
}

impl ParametrComponentF32 {
    pub fn new(
        name: String,
        parametr: SharedParametr<f32>,
        steps: i32,
        interpolation_method: InterpolateMethod,
    ) -> Self {
        Self {
            name,
            parametr,
            steps,
            interpolation_method,
        }
    }
}

pub struct ParametrComponentI32 {
    name: String,
    parametr: SharedParametr<i32>,
}

impl ParametrComponentI32 {
    pub fn new(name: String, parametr: SharedParametr<i32>) -> Self {
        Self { name, parametr }
    }
}

impl AnyParametrComponent for ParametrComponentF32 {
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

    fn name(&self) -> &String {
        &self.name
    }

    fn value(&self) -> String {
        format!("{}", self.parametr.read().unwrap().get_value())
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

    fn name(&self) -> &String {
        &self.name
    }

    fn value(&self) -> String {
        format!("{}", self.parametr.read().unwrap().get_value())
    }
}

impl<T: AnyParametrComponent> Component for T {
    type Action = EmptyAction;

    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        let p = Paragraph::new(format!("{}: {}", self.name(), self.value()))
            .style(Style::default().fg(Color::White));
        p.render(rect, f.buffer_mut());
        Ok(())
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Char('h') => self.decrement(),
            KeyCode::Char('l') => self.increment(),
            _ => (),
        };
        Ok(())
    }
}

fn time_of(param: &(impl Parametr<f32> + ?Sized)) -> f32 {
    let range = param.range();
    (param.get_value() - range.0) / (range.1 - range.0)
}
