use std::sync::Arc;

use anyhow::{Context, Result};
use crossterm::event::{Event, KeyCode, KeyEvent, MouseEvent};
use oosc_core::utils::{make_shared, Shared};
use ratatui::{prelude::Rect, style::Color, Frame};

use crate::ui::observer::{Notifier, NotifierContainer};

use super::{Component, Focus, FocusableComponent, FocusableComponentContext};

pub enum ContainerEvent<T>
where
    T: FocusableComponent + ?Sized + 'static,
{
    FocusChanged { index: i32, component: Shared<T> },
    Unfocus { last_focus: Shared<T> },
}

impl<T> Clone for ContainerEvent<T>
where
    T: FocusableComponent + ?Sized + 'static,
{
    fn clone(&self) -> Self {
        match self {
            ContainerEvent::FocusChanged { index, component } => ContainerEvent::FocusChanged {
                index: *index,
                component: component.clone(),
            },
            ContainerEvent::Unfocus { last_focus } => ContainerEvent::Unfocus {
                last_focus: last_focus.clone(),
            },
        }
    }
}

pub struct ComponentsContainer<T>
where
    T: FocusableComponent + ?Sized + 'static,
{
    pub components: Vec<Shared<T>>,
    pub ctx: FocusableComponentContext,
    draw_only_focused: bool,
    active_if_child_focused: bool,
    next_keymap: Option<KeyCode>,
    previous_keymap: Option<KeyCode>,
    last_focus: Option<Shared<T>>,
    current: i32,
    notifier: NotifierContainer<ContainerEvent<T>>,
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
    T: FocusableComponent + ?Sized + 'static,
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
    T: FocusableComponent + ?Sized + 'static,
{
    pub fn new() -> Self {
        Self {
            components: vec![],
            ctx: FocusableComponentContext::new().focused(true),
            draw_only_focused: false,
            active_if_child_focused: false,
            next_keymap: None,
            previous_keymap: None,
            last_focus: None,
            current: 0,
            notifier: NotifierContainer::new(),
        }
    }

    pub fn next_keymap(&mut self, keymap: KeyCode) -> &mut Self {
        self.next_keymap = Some(keymap);
        self
    }

    pub fn previous_keymap(&mut self, keymap: KeyCode) -> &mut Self {
        self.previous_keymap = Some(keymap);
        self
    }

    pub fn draw_only_focused(&mut self, value: bool) -> &mut Self {
        self.draw_only_focused = value;
        self
    }

    pub fn active_if_child_focused(&mut self, value: bool) -> &mut Self {
        self.active_if_child_focused = value;
        self
    }

    pub fn is_any_focused(&self) -> bool {
        self.components
            .iter()
            .any(|c| c.read().unwrap().is_focused())
    }

    pub fn container(&mut self) -> &mut Vec<Shared<T>> {
        &mut self.components
    }

    pub fn iter(&self) -> impl Iterator<Item = Shared<T>> + '_ {
        self.components.iter().cloned()
    }

    pub fn draw_in_layout(&mut self, f: &mut Frame<'_>, layout: &[Rect]) -> Result<()> {
        match self.draw_only_focused {
            true => {
                if self.is_any_focused() {
                    self.components
                        .iter_mut()
                        .filter(|c| c.read().unwrap().is_focused())
                        .enumerate()
                        .try_for_each(|(i, c)| {
                            c.write()
                                .unwrap()
                                .draw(f, *layout.get(i).context("Cannot get layout")?)
                        })
                } else {
                    let index = self.bounded_index(self.current) as usize;
                    let mut c = self
                        .components
                        .get_mut(index)
                        .context("Cannot get component")?
                        .write()
                        .unwrap();
                    c.draw(f, *layout.get(index).context("Cannot get layout")?)
                }
            }
            false => self
                .components
                .iter_mut()
                .enumerate()
                .try_for_each(|(i, c)| {
                    c.write()
                        .unwrap()
                        .draw(f, *layout.get(i).context("Cannot get layout")?)
                }),
        }
    }

    pub fn resize_in_layout(&mut self, layout: &[Rect]) -> Result<()> {
        self.components
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, c)| {
                c.write()
                    .unwrap()
                    .resize(*layout.get(i).context("Cannot get layout")?)
            })
    }

    pub fn focus_current(&mut self) {
        self.focus_on(self.current)
    }

    pub fn focus_next(&mut self) {
        self.focus_on(self.current + 1)
    }

    pub fn focus_previous(&mut self) {
        self.focus_on(self.current - 1)
    }

    pub fn focus_on(&mut self, index: i32) {
        self.current = self.bounded_index(index);
        let component = self.components.get(self.current as usize).unwrap();
        self.focus_to(component.clone());
    }

    fn bounded_index(&self, index: i32) -> i32 {
        let len = self.components.len() as i32;
        let index = if index < 0 { len + index % len } else { index };
        (index % len).abs()
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

    fn unfocus_last(&mut self) {
        let last = self.last_focus.clone();
        if let Some(last) = last.clone() {
            if !last.read().unwrap().is_focused() {
                self.last_focus = None;
                self.notifier
                    .notify(ContainerEvent::Unfocus { last_focus: last })
            }
        }
    }

    fn focus_if_key(&mut self, key: KeyCode) -> Option<FocusContextResult<T>> {
        for p in self.components.iter() {
            if p.read().unwrap().keymap() == Some(key) {
                return self.focus_to(p.clone());
            }
        }
        None
    }

    fn handle_container_key(&mut self, key: KeyCode) {
        if let Some(keymap) = self.keymap() {
            if key == keymap {
                self.focus()
            }
        }
        if let Some(next) = self.next_keymap {
            if next == key {
                self.focus_next()
            }
        }
        if let Some(previous) = self.previous_keymap {
            if previous == key {
                self.focus_previous()
            }
        }
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
        self.unfocus_last();
        if !self.is_any_focused() || self.active_if_child_focused {
            let focus_result = self.focus_if_key(key.code);
            if focus_result.is_none() {
                self.components
                    .iter_mut()
                    .filter(|c| c.read().unwrap().context().wrapper)
                    .try_for_each(|c| {
                        c.write()
                            .unwrap()
                            .handle_key_events(key)
                            .context("Wrapper component handle key event error")
                    })?;
                self.handle_container_key(key.code);
            } else {
                return Ok(());
            }
        }
        self.components
            .iter_mut()
            .filter(|c| c.read().unwrap().is_focused())
            .try_for_each(|c| {
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

impl<T> Focus for ComponentsContainer<T>
where
    T: FocusableComponent + ?Sized + 'static,
{
    fn focus(&mut self) {
        self.context_mut().focus()
    }

    fn unfocus(&mut self) {
        self.context_mut().unfocus()
    }

    fn is_focused(&self) -> bool {
        self.context().is_focused()
    }

    fn keymap(&self) -> Option<crossterm::event::KeyCode> {
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

impl<T> From<Vec<Shared<T>>> for ComponentsContainer<T>
where
    T: FocusableComponent + ?Sized,
{
    fn from(value: Vec<Shared<T>>) -> Self {
        Self {
            components: value,
            ctx: FocusableComponentContext::new(),
            active_if_child_focused: false,
            previous_keymap: None,
            next_keymap: None,
            last_focus: None,
            current: 0,
            draw_only_focused: false,
            notifier: NotifierContainer::new(),
        }
    }
}

impl<T> From<Vec<T>> for ComponentsContainer<T>
where
    T: FocusableComponent,
{
    fn from(value: Vec<T>) -> Self {
        let components = value.into_iter().map(|c| make_shared(c)).collect();
        Self {
            components,
            ctx: FocusableComponentContext::new(),
            active_if_child_focused: false,
            previous_keymap: None,
            next_keymap: None,
            last_focus: None,
            current: 0,
            draw_only_focused: false,
            notifier: NotifierContainer::new(),
        }
    }
}
