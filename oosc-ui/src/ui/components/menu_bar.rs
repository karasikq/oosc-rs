use oosc_core::utils::Shared;
use ratatui::{prelude::*, text::Span, widgets::Paragraph};

use crate::ui::observer::Observer;

use super::{
    components_container::{ComponentsContainer, ContainerEvent},
    Component, NamedFocusableComponent,
};

type Container<C> = Shared<ComponentsContainer<C>>;

pub struct MenuBar<C>
where
    C: NamedFocusableComponent + ?Sized + 'static,
{
    pub container: Container<C>,
    paragraph: Paragraph<'static>,
    rect: Option<Rect>,
}

impl<C> MenuBar<C>
where
    C: NamedFocusableComponent + ?Sized + 'static,
{
    pub fn new(container: Container<C>) -> Self {
        let paragraph = Self::build_paragraph(container.clone());
        Self {
            container,
            paragraph,
            rect: None,
        }
    }

    fn build_paragraph(container: Container<C>) -> Paragraph<'static> {
        let container = container.read().unwrap();
        let spans: Vec<Span<'static>> = container
            .iter()
            .flat_map(|c| c.read().unwrap().name())
            .collect();
        Paragraph::new(Line::from(spans))
    }
}

impl<T> Observer<ContainerEvent<T>> for MenuBar<T>
where
    T: NamedFocusableComponent + ?Sized + 'static,
{
    fn react(&mut self, value: ContainerEvent<T>) {
        self.paragraph = Self::build_paragraph(self.container.clone());
    }
}

impl<C> Component for MenuBar<C>
where
    C: NamedFocusableComponent + ?Sized + 'static,
{
    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        _rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        let rect = self.rect.unwrap();
        f.render_widget(Self::build_paragraph(self.container.clone()), rect);
        Ok(())
    }

    fn resize(&mut self, rect: ratatui::prelude::Rect) -> anyhow::Result<()> {
        self.rect = Some(rect);
        Ok(())
    }
}
