use std::rc::Rc;

use crossterm::event::KeyCode;
use oosc_core::{
    core::parameter::{Parameter, SharedParameter},
    utils::interpolation::{interpolate_range, InterpolateMethod},
};
use ratatui::style::Style;
use ratatui::{prelude::*, widgets::*};

use crate::ui::{
    observer::{Notifier, NotifierContainer},
    utils::keycode_to_string,
    widgets::bar::BarWidget,
};

use super::{Component, Focus, FocusableComponent, FocusableComponentContext, AutoFocus};

pub struct ParameterLayout {
    pub rect: Rect,
    pub main: Rc<[Rect]>,
}

impl From<Rect> for ParameterLayout {
    fn from(rect: Rect) -> Self {
        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .margin(1)
            .split(rect);
        Self { rect, main }
    }
}

pub trait AnyParameterComponent {
    fn name(&self) -> &String;
    fn value(&self) -> f32;
    fn range(&self) -> (f32, f32);
    fn direction(&self) -> Direction;
    fn format_value(&self) -> String;
    fn increment(&mut self);
    fn decrement(&mut self);
    fn resize(&mut self, rect: Rect);
    fn layout(&self) -> &Option<ParameterLayout>;
}

fn build_bar(parametr: &(impl AnyParameterComponent + Focus), rect: Rect) -> BarWidget {
    BarWidget {
        resolution: (rect.width, rect.height),
        direction: parametr.direction(),
        bounds: parametr.range(),
        center: 0.0,
        value: parametr.value(),
        color: parametr.color(),
    }
}

#[derive(Eq, PartialEq, Hash)]
pub enum ParameterEvent {
    ValueChanged,
}

type EventContainer<T> = NotifierContainer<ParameterEvent, SharedParameter<T>>;

pub struct ParameterComponentF32 {
    name: String,
    parametr: SharedParameter<f32>,
    direction: Direction,
    steps: i32,
    current_step: f32,
    interpolation_method: InterpolateMethod,
    events: EventContainer<f32>,
    context: FocusableComponentContext,
    layout: Option<ParameterLayout>,
}

impl ParameterComponentF32 {
    pub fn new(
        name: String,
        parametr: SharedParameter<f32>,
        direction: Direction,
        steps: i32,
        interpolation_method: InterpolateMethod,
        keymap: KeyCode,
    ) -> Self {
        let context = FocusableComponentContext::new().keymap(keymap);
        let current_step =
            Self::normalize_parametr(&*parametr.read().unwrap()) * (steps as f32 - 1.0);
        Self {
            name,
            parametr,
            direction,
            steps,
            current_step,
            interpolation_method,
            context,
            events: EventContainer::<f32>::default(),
            layout: None,
        }
    }

    pub fn events(&mut self) -> &mut impl Notifier<SharedParameter<f32>, ParameterEvent> {
        &mut self.events
    }

    fn normalize_parametr(param: &(impl Parameter<f32> + ?Sized)) -> f32 {
        let range = param.range();
        (param.get_value() - range.0) / (range.1 - range.0)
    }
}

impl AutoFocus for ParameterComponentF32 {}

impl FocusableComponent for ParameterComponentF32 {
    fn context(&self) -> &FocusableComponentContext {
        &self.context
    }

    fn context_mut(&mut self) -> &mut FocusableComponentContext {
        &mut self.context
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub struct ParameterComponentI32 {
    name: String,
    parametr: SharedParameter<i32>,
    direction: Direction,
    events: EventContainer<i32>,
    context: FocusableComponentContext,
    layout: Option<ParameterLayout>,
}

impl AutoFocus for ParameterComponentI32 {}

impl FocusableComponent for ParameterComponentI32 {
    fn context(&self) -> &FocusableComponentContext {
        &self.context
    }

    fn context_mut(&mut self) -> &mut FocusableComponentContext {
        &mut self.context
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ParameterComponentI32 {
    pub fn new(
        name: String,
        parametr: SharedParameter<i32>,
        direction: Direction,
        keymap: KeyCode,
    ) -> Self {
        let context = FocusableComponentContext::new().keymap(keymap);
        Self {
            name,
            parametr,
            direction,
            events: EventContainer::<i32>::default(),
            context,
            layout: None,
        }
    }

    pub fn events(&mut self) -> &mut impl Notifier<SharedParameter<i32>, ParameterEvent> {
        &mut self.events
    }
}

impl AnyParameterComponent for ParameterComponentF32 {
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
            self.current_step = (self.current_step + 1.0).clamp(0.0, self.steps as f32);
            let t = self.current_step / self.steps as f32;
            let result = interpolate_range(parametr.range(), t, self.interpolation_method);
            parametr.set_value(result);
        }
        self.events
            .notify(ParameterEvent::ValueChanged, self.parametr.clone());
    }

    fn decrement(&mut self) {
        {
            let mut parametr = self.parametr.write().unwrap();
            self.current_step = (self.current_step - 1.0).clamp(0.0, self.steps as f32);
            let t = self.current_step / self.steps as f32;
            let result = interpolate_range(parametr.range(), t, self.interpolation_method);
            parametr.set_value(result);
        }
        self.events
            .notify(ParameterEvent::ValueChanged, self.parametr.clone());
    }

    fn resize(&mut self, rect: Rect) {
        self.layout = Some(ParameterLayout::from(rect));
    }

    fn layout(&self) -> &Option<ParameterLayout> {
        &self.layout
    }
}

impl AnyParameterComponent for ParameterComponentI32 {
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
            .notify(ParameterEvent::ValueChanged, self.parametr.clone());
    }

    fn decrement(&mut self) {
        {
            let mut parametr = self.parametr.write().unwrap();
            let result = { parametr.get_value() - 1 };
            parametr.set_value(result);
        }
        self.events
            .notify(ParameterEvent::ValueChanged, self.parametr.clone());
    }

    fn resize(&mut self, rect: Rect) {
        self.layout = Some(ParameterLayout::from(rect));
    }

    fn layout(&self) -> &Option<ParameterLayout> {
        &self.layout
    }
}

impl<T: AnyParameterComponent + Focus> Component for T {
    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        _rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        let layout_opt = self.layout();
        if layout_opt.is_none() {
            return Err(oosc_core::error::Error::from("Create layout before draw"))?;
        }
        let layout = layout_opt.as_ref().unwrap();
        let bar = build_bar(self, layout.main[0]);
        f.render_widget(bar, layout.main[0]);

        let b = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                "{}[{}]",
                self.name().as_str(),
                keycode_to_string(self.keymap().unwrap_or(KeyCode::Null))
            ))
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(self.color()));
        f.render_widget(b, layout.rect);

        let p = Paragraph::new(self.format_value()).alignment(Alignment::Center);
        f.render_widget(p, layout.main[1]);
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

    fn resize(&mut self, rect: Rect) -> anyhow::Result<()> {
        self.resize(rect);
        Ok(())
    }
}
