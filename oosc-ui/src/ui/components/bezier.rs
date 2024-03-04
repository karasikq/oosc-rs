use std::rc::Rc;

use anyhow::Context;
use crossterm::event::KeyCode;
use oosc_core::utils::{
    adsr_envelope::SharedCurve, cubic_bezier::CubicBezierCurve, interpolation::InterpolateMethod,
    make_shared, Shared,
};
use ratatui::style::Style;
use ratatui::{
    prelude::*,
    widgets::{canvas::Canvas, *},
};

use crate::ui::{components::parameter::ParameterComponentF32, utils::keycode_to_string_prefixed};

use super::{
    components_container::{ComponentsContainer, ContainerAction},
    Component, Focus, FocusableComponent, FocusableComponentContext,
};

struct BezierLayout {
    pub rect: Rect,
    pub main: Rc<[Rect]>,
    pub navigation: Option<[Rect; 2]>,
}

pub struct BezierComponent {
    curve: Shared<CubicBezierCurve>,
    parameters: ComponentsContainer<ParameterComponentF32>,
    ctx: FocusableComponentContext,
    samples: usize,
    line: Vec<canvas::Line>,
    color: Color,
    layout: Option<BezierLayout>,
}

impl FocusableComponent for BezierComponent {
    fn context(&self) -> &super::FocusableComponentContext {
        &self.ctx
    }

    fn context_mut(&mut self) -> &mut super::FocusableComponentContext {
        &mut self.ctx
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl BezierComponent {
    pub fn new(curve: &SharedCurve) -> Self {
        let mut parameters = ComponentsContainer::from(Self::build_parametr_components(curve));
        parameters.active_if_child_focused(true);
        parameters.focus();
        parameters.next_keymap(KeyCode::Char('k'));
        parameters.previous_keymap(KeyCode::Char('j'));
        let curve = curve.curve.clone();
        let ctx = FocusableComponentContext::new().focused(true);
        Self {
            curve,
            parameters,
            ctx,
            samples: 30,
            line: vec![],
            color: Color::Red,
            layout: None,
        }
    }

    pub fn new_curve(&mut self, curve: &SharedCurve) {
        *self.parameters.container() = Self::build_parametr_components(curve);
        self.curve = curve.curve.clone();
        self.resize(self.layout.as_ref().unwrap().rect).unwrap();
    }

    pub fn samples(self, samples: usize) -> Self {
        Self { samples, ..self }
    }

    pub fn color(self, color: Color) -> Self {
        Self { color, ..self }
    }

    pub fn build(mut self) -> Self {
        self.line = self.render_line();
        self
    }

    pub fn render_line(&self) -> Vec<canvas::Line> {
        let table = self.curve.read().unwrap();
        let rate = 1.0 / self.samples as f32;
        (1..=self.samples)
            .map(|t| {
                let mut line = canvas::Line::new(0.0, 0.0, 0.0, 0.0, self.color);
                let x1 = (t - 1) as f32 * rate;
                let x2 = t as f32 * rate;
                line.x1 = x1 as f64;
                line.y1 = table.evaluate(x1).y as f64;
                line.x2 = x2 as f64;
                line.y2 = table.evaluate(x2).y as f64;
                line
            })
            .collect()
    }

    fn build_main_layout(rect: Rect) -> Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .margin(1)
            .split(rect)
    }

    fn build_navigation_layout(rect: Rect) -> Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(rect)
    }

    fn build_parametrs_layout(rect: Rect) -> Vec<Rect> {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50); 2])
            .split(rect);
        rows.iter()
            .flat_map(|r| {
                (*Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100 / 3); 3])
                    .split(*r))
                .to_vec()
            })
            .collect()
    }

    fn build_parametr_components(curve: &SharedCurve) -> Vec<Shared<ParameterComponentF32>> {
        vec![
            make_shared(ParameterComponentF32::new(
                "Length".to_owned(),
                curve.length.clone(),
                Direction::Horizontal,
                10,
                InterpolateMethod::Exponential(10000.0),
                KeyCode::Null,
            )),
            make_shared(ParameterComponentF32::new(
                "Amplitude".to_owned(),
                curve.amplitude.clone(),
                Direction::Vertical,
                40,
                InterpolateMethod::Linear,
                KeyCode::Null,
            )),
            make_shared(ParameterComponentF32::new(
                "B-x".to_owned(),
                curve.point_b.0.clone(),
                Direction::Horizontal,
                40,
                InterpolateMethod::Linear,
                KeyCode::Null,
            )),
            make_shared(ParameterComponentF32::new(
                "B-y".to_owned(),
                curve.point_b.1.clone(),
                Direction::Vertical,
                40,
                InterpolateMethod::Linear,
                KeyCode::Null,
            )),
            make_shared(ParameterComponentF32::new(
                "C-x".to_owned(),
                curve.point_c.0.clone(),
                Direction::Horizontal,
                40,
                InterpolateMethod::Linear,
                KeyCode::Null,
            )),
            make_shared(ParameterComponentF32::new(
                "C-y".to_owned(),
                curve.point_c.1.clone(),
                Direction::Vertical,
                40,
                InterpolateMethod::Linear,
                KeyCode::Null,
            )),
        ]
    }
}

impl<T> From<T> for BezierComponent
where
    T: for<'a> Into<&'a SharedCurve>,
{
    fn from(value: T) -> Self {
        Self::new(value.into()).samples(30).build()
    }
}

impl Component for BezierComponent {
    fn draw(&mut self, f: &mut Frame<'_>, _rect: Rect) -> anyhow::Result<()> {
        if self.layout.is_none() {
            return Ok(());
        }
        let layout = self
            .layout
            .as_ref()
            .context("Cannot get BezierComponent layout")?;
        self.line = self.render_line();
        let b = Block::default()
            .borders(Borders::TOP | Borders::BOTTOM | Borders::LEFT)
            .title("Curve");
        f.render_widget(b, layout.rect);
        let canvas = Canvas::default()
            .marker(Marker::Braille)
            .x_bounds([0.0, 1.0])
            .y_bounds([0.0, 1.0])
            .paint(|ctx| {
                self.line.iter().for_each(|line| ctx.draw(line));
            });
        canvas.render(layout.main[0], f.buffer_mut());
        self.parameters.draw(f, layout.main[1])?;

        if let Some(navigation) = layout.navigation {
            let to_left = vec![
                Line::from("<"),
                Line::from(keycode_to_string_prefixed(
                    self.parameters.get_keymap(ContainerAction::Previous),
                    "[",
                    "]",
                )),
            ];
            let to_right = vec![
                Line::from(">"),
                Line::from(keycode_to_string_prefixed(
                    self.parameters.get_keymap(ContainerAction::Next),
                    "[",
                    "]",
                )),
            ];
            let paragraph = Paragraph::new(to_left)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, navigation[0]);
            let paragraph = Paragraph::new(to_right)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, navigation[1]);
        }

        Ok(())
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        if !self.is_focused() {
            return Ok(());
        }
        self.parameters.handle_key_events(key)
    }

    fn resize(&mut self, rect: Rect) -> anyhow::Result<()> {
        let main = Self::build_main_layout(rect);
        let navigation = if main[1].width < 13 || main[1].height < 13 {
            let navigation = Self::build_navigation_layout(main[1]);
            self.parameters.draw_only_focused(true);
            self.parameters.resize(navigation[1])?;
            Some([navigation[0], navigation[2]])
        } else {
            let parameters = Self::build_parametrs_layout(main[1]);
            self.parameters.resize_in_layout(&parameters)?;
            None
        };
        self.layout = Some(BezierLayout {
            rect,
            main,
            navigation,
        });
        Ok(())
    }
}

impl Focus for BezierComponent {
    fn focus(&mut self) {
        self.context_mut().focus();
        self.parameters.focus_current();
    }

    fn unfocus(&mut self) {
        self.context_mut().unfocus()
    }

    fn is_focused(&self) -> bool {
        self.context().is_focused()
    }

    fn keymap(&self) -> Option<KeyCode> {
        self.context().keymap()
    }

    fn color(&self) -> Color {
        if self.is_focused() {
            *self
                .context()
                .focused_color
                .as_ref()
                .unwrap_or(&Color::Yellow)
        } else {
            *self
                .context()
                .unfocused_color
                .as_ref()
                .unwrap_or(&Color::Gray)
        }
    }
}
