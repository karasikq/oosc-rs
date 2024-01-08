use std::rc::Rc;

use anyhow::Context;
use crossterm::event::KeyCode;
use oosc_core::utils::{
    adsr_envelope::SharedCurve, cubic_bezier::CubicBezierCurve, interpolation::InterpolateMethod,
    make_shared, Shared,
};
use ratatui::{
    prelude::*,
    widgets::{canvas::*, *},
};

use crate::ui::components::parameter::ParameterComponentF32;

use super::{components_container::ComponentsContainer, Component, Focus};

struct BezierLayout {
    pub rect: Rect,
    pub main: Rc<[Rect]>,
    pub parameters: Vec<Rect>,
}

pub struct BezierComponent {
    curve: Shared<CubicBezierCurve>,
    parameters: ComponentsContainer<ParameterComponentF32>,
    samples: usize,
    line: Vec<canvas::Line>,
    color: Color,
    layout: Option<BezierLayout>,
}

impl BezierComponent {
    pub fn new(curve: &SharedCurve) -> Self {
        let mut parameters = ComponentsContainer::from(Self::build_parametr_components(curve));
        parameters.active_if_child_focused(true);
        parameters.focus();
        parameters.next_keymap(KeyCode::Char('k'));
        parameters.previous_keymap(KeyCode::Char('j'));
        let curve = curve.curve.clone();
        Self {
            curve,
            parameters,
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
        self.parameters.draw_in_layout(f, &layout.parameters)?;
        Ok(())
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        self.parameters.handle_key_events(key)
    }

    fn resize(&mut self, rect: Rect) -> anyhow::Result<()> {
        let main = Self::build_main_layout(rect);
        let parameters = Self::build_parametrs_layout(main[1]);
        self.parameters
            .iter()
            .enumerate()
            .try_for_each(|(i, p)| p.write().unwrap().resize(parameters[i]))?;
        self.layout = Some(BezierLayout {
            rect,
            main,
            parameters,
        });
        Ok(())
    }
}
