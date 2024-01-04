use std::sync::Arc;

use anyhow::{Context, Result};
use crossterm::event::{Event, KeyCode, KeyEvent, MouseEvent};
use oosc_core::utils::Shared;
use ratatui::{prelude::Rect, Frame};

use super::{Component, Focus, FocusableComponent, FocusableComponentContext};

pub struct ComponentsContainer<T>
where
    T: FocusableComponent + ?Sized,
{
    pub components: Vec<Shared<T>>,
    pub ctx: FocusableComponentContext,
    last_focus: Option<Shared<T>>,
    current: usize,
}

impl<T> FocusableComponent for ComponentsContainer<T>
where
    T: FocusableComponent + ?Sized + 'static,
{
    fn context(&self) -> &FocusableComponentContext {
        &self.ctx
    }

    fn context_mut(&mut self) -> &mut FocusableComponentContext {
        &mut self.ctx
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl<T> Default for ComponentsContainer<T>
where
    T: FocusableComponent + ?Sized,
{
    fn default() -> Self {
        Self::new()
    }
}

pub enum FocusContextResult<T>
where
    T: FocusableComponent + ?Sized,
{
    Focused {
        previous: Option<Shared<T>>,
        current: Shared<T>,
    },
    AlreadyFocused(Shared<T>),
}

impl<T> ComponentsContainer<T>
where
    T: FocusableComponent + ?Sized,
{
    pub fn new() -> Self {
        Self {
            components: vec![],
            ctx: FocusableComponentContext::new(),
            last_focus: None,
            current: 0,
        }
    }

    pub fn container(&mut self) -> &mut Vec<Shared<T>> {
        &mut self.components
    }

    pub fn iter(&self) -> impl Iterator<Item = Shared<T>> + '_ {
        self.components.iter().cloned()
    }

    pub fn draw_in_layout(&mut self, f: &mut Frame<'_>, layout: &[Rect]) -> Result<()> {
        self.components
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, c)| {
                c.write()
                    .unwrap()
                    .draw(f, *layout.get(i).context("Cannot get layout")?)
            })
    }

    pub fn focus_next(&mut self) {
        self.focus_on(self.current + 1)
    }

    pub fn focus_previous(&mut self) {
        self.focus_on(self.current - 1)
    }

    pub fn focus_on(&mut self, index: usize) {
        self.current = index.clamp(0, self.components.len());
        let component = self.components.get(self.current).unwrap();
        self.focus_to(component.clone());
    }

    fn focus_to(&mut self, component: Shared<T>) -> Option<FocusContextResult<T>> {
        let last = self.last_focus.clone();
        if let Some(last) = last.clone() {
            if !Arc::ptr_eq(&last, &component) {
                last.write().unwrap().unfocus();
            } else {
                return Some(FocusContextResult::AlreadyFocused(last));
            }
        }
        let mut c = component.write().unwrap();
        c.focus();
        self.last_focus = Some(component.clone());
        Some(FocusContextResult::Focused {
            previous: last,
            current: component.clone(),
        })
    }

    fn focus_if_key(&mut self, key: KeyCode) -> Option<FocusContextResult<T>> {
        for p in self.components.iter() {
            if p.read().unwrap().keymap() == Some(key) {
                return self.focus_to(p.clone());
            }
        }
        None
    }
}

impl<T> Component for ComponentsContainer<T>
where
    T: FocusableComponent + ?Sized + 'static,
{
    fn init(&mut self) -> Result<()> {
        self.components.iter_mut().try_for_each(|c| {
            c.write()
                .unwrap()
                .init()
                .context("Cannot init child component")
        })
    }

    fn resize(&mut self, rect: Rect) -> Result<()> {
        self.components.iter_mut().try_for_each(|c| {
            c.write()
                .unwrap()
                .resize(rect)
                .context("Cannot resize child component")
        })
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<()> {
        let _ = match event {
            Some(Event::Key(key_event)) => self.handle_key_events(key_event),
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event),
            Some(Event::Resize(x, y)) => self.resize(Rect::new(0, 0, x, y)),
            _ => Ok(()),
        };
        self.components.iter_mut().try_for_each(|c| {
            c.write()
                .unwrap()
                .handle_events(event.clone())
                .context("Child component handle event error")
        })
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<()> {
        if !self.is_focused() {
            return Ok(());
        }
        if !self
            .components
            .iter()
            .any(|c| c.read().unwrap().is_focused())
        {
            self.focus_if_key(key.code);
            match key.code {
                KeyCode::Esc => self.unfocus(),
                c => {
                    if let Some(keymap) = self.keymap() {
                        if c == keymap {
                            self.focus()
                        }
                    }
                }
            };
        }
        self.components.iter_mut().try_for_each(|c| {
            c.write()
                .unwrap()
                .handle_key_events(key)
                .context("Child component handle key event error")
        })
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<()> {
        self.components.iter_mut().try_for_each(|c| {
            c.write()
                .unwrap()
                .handle_mouse_events(mouse)
                .context("Child component handle mouse event error")
        })
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
        self.components.iter_mut().try_for_each(|c| {
            c.write()
                .unwrap()
                .draw(f, rect)
                .context("Cannot draw child component")
        })
    }
}

impl<T> From<Vec<Shared<T>>> for ComponentsContainer<T>
where
    T: FocusableComponent + ?Sized,
{
    fn from(value: Vec<Shared<T>>) -> Self {
        Self {
            components: value,
            ctx: FocusableComponentContext::new(),
            last_focus: None,
            current: 0,
        }
    }
}
