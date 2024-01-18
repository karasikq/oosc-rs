use oosc_core::utils::Shared;
use ratatui::{prelude::*, text::Span, widgets::Paragraph};

use super::{components_container::ComponentsContainer, Component, NamedFocusableComponent};

type Container<C> = Shared<ComponentsContainer<C>>;

pub struct MenuBar<C>
where
    C: NamedFocusableComponent + ?Sized + 'static,
{
    pub container: Container<C>,
    rect: Option<Rect>,
}

impl<C> MenuBar<C>
where
    C: NamedFocusableComponent + ?Sized + 'static,
{
    pub fn new(container: Container<C>) -> Self {
        Self {
            container,
            rect: None,
        }
    }

    fn build_paragraph(container: Container<C>) -> Paragraph<'static> {
        let container = container.read().unwrap();
        let spans: Vec<Span<'static>> = container
            .iter()
            .flat_map(|c| {
                let c = c.read().unwrap();
                let mut name_spans = c.name();
                name_spans.push(Span::styled("|", Style::default().fg(c.color())));
                name_spans
            })
            .collect();
        Paragraph::new(Line::from(spans))
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
