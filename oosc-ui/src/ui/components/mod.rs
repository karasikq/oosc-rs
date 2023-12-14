pub mod oscillator;
pub mod root;
pub mod synthesizer;
pub mod wavetable;

use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{layout::Rect, Frame};

use super::utils::Event;

pub enum EmptyAction {
    None,
}

pub trait Component {
    type Action;

    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<()> {
        let result = match event {
            Some(Event::Key(key_event)) => self.handle_key_events(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event)?,
            _ => None,
        };
        self.update(result)?;
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Self::Action>> {
        Ok(None)
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Self::Action>> {
        Ok(None)
    }

    fn update(&mut self, action: Option<Self::Action>) -> Result<()> {
        Ok(())
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()>;
}

pub trait Focus {
    fn focus(&mut self);
    fn unfocus(&mut self);
    fn is_focused(&self) -> bool;
}

pub trait FocusableComponent: Component + Focus {}

/* pub struct Container<T>
where
    T: ?Sized + Component,
{
    components: Vec<Box<T>>,
}

impl<T: Component> Container<T> {
    pub fn push(&mut self, component: Box<T>) {
        self.components.push(component);
    }

    pub fn get_components<'a, C>(&'a mut self) -> impl Iterator<Item = &'a mut C>
    where
        C: Component + 'a + 'static,
    {
        self.components
            .iter_mut()
            .filter_map(|osc| osc.as_any_mut().downcast_mut::<C>())
    }

    pub fn get_component<'a, C>(&'a mut self) -> Option<&'a mut C>
    where
        C: Component + 'a + 'static,
    {
        self.get_components::<C>().next()
    }
} */
