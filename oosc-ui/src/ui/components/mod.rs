pub mod oscillator;
pub mod root;
pub mod synthesizer;
pub mod wavetable;

use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent, Event};
use ratatui::{layout::Rect, Frame};

pub enum EmptyAction {
    None,
}

pub trait Component {
    type Action;

    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn resize(&mut self, rect: Rect) -> Result<()> {
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<()> {
        match event {
            Some(Event::Key(key_event)) => self.handle_key_events(key_event),
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event),
            Some(Event::Resize(x, y)) => self.resize(Rect::new(0, 0, x, y)),
            _ => Ok(()),
        }
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<()> {
        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<()> {
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
