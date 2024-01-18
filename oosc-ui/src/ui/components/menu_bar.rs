use oosc_core::utils::Shared;
use ratatui::{
    prelude::*,
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph},
};

use super::{components_container::ComponentsContainer, Component, NamedFocusableComponent};

type Container<C> = Shared<ComponentsContainer<C>>;

struct MenuLayout {
    pub rect: Rect,
    pub inner: Rect,
}

pub struct MenuBar<C>
where
    C: NamedFocusableComponent + ?Sized + 'static,
{
    pub container: Container<C>,
    title: &'static str,
    layout: Option<MenuLayout>,
}

impl<C> MenuBar<C>
where
    C: NamedFocusableComponent + ?Sized + 'static,
{
    pub fn new(container: Container<C>, title: &'static str) -> Self {
        Self {
            container,
            title,
            layout: None,
        }
    }

    fn build_paragraph(container: Container<C>) -> Paragraph<'static> {
        let container = container.read().unwrap();
        let spans: Vec<Span<'static>> = container
            .iter()
            .flat_map(|c| {
                let c = c.read().unwrap();
                let mut name_spans = c.name();
                name_spans.push(Span::styled(" | ", Style::default().fg(c.color())));
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
        let layout = self.layout.as_ref().unwrap();
        f.render_widget(Self::build_paragraph(self.container.clone()), layout.inner);
        let b = Block::default()
            .borders(Borders::ALL)
            .title(self.title)
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center);
        f.render_widget(b, layout.rect);
        Ok(())
    }

    fn resize(&mut self, rect: ratatui::prelude::Rect) -> anyhow::Result<()> {
        self.layout = Some(MenuLayout {
            rect,
            inner: rect.inner(&Margin::new(1, 1)),
        });
        Ok(())
    }
}
