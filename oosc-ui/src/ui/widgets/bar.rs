use ratatui::style::Color;
use ratatui::{prelude::Direction, widgets::Widget};

pub struct BarWidget {
    pub resolution: (u16, u16),
    pub bounds: (f32, f32),
    pub center: f32,
    pub value: f32,
    pub direction: Direction,
    pub color: Color,
}

impl BarWidget {
    pub fn get_point(&self, value: f32) -> Option<u16> {
        let max_bound = self.bounds.1;
        let min_bound = self.bounds.0;
        if value < min_bound || value > max_bound {
            return None;
        }
        let size = (self.bounds.1 - self.bounds.0).abs();
        if size == 0.0 {
            return None;
        }
        let resolution = match self.direction {
            Direction::Horizontal => self.resolution.0 as f32 - 1.0,
            Direction::Vertical => self.resolution.1 as f32 - 1.0,
        };
        let value = match self.direction {
            Direction::Horizontal => value - min_bound,
            Direction::Vertical => max_bound - value,
        };
        let index = (value * resolution / size) as u16;
        Some(index)
    }
}

impl Widget for BarWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let p1 = self.get_point(self.value);
        let p2 = self.get_point(self.center);
        if p1.is_none() || p2.is_none() {
            return;
        }
        let p1 = p1.unwrap();
        let p2 = p2.unwrap();
        let (_dp, p_range) = match p2.cmp(&p1) {
            std::cmp::Ordering::Less => (p1 - p2, p2..=p1),
            std::cmp::Ordering::Equal => (p2 - p1, p1..=p2),
            std::cmp::Ordering::Greater => (p2 - p1, p1..=p2),
        };
        let (x_range, y_range) = match self.direction {
            Direction::Horizontal => (p_range, 0..=area.height - 1),
            Direction::Vertical => (0..=area.width - 1, p_range),
        };
        x_range.for_each(|x| {
            y_range.clone().for_each(|y| {
                let (x, y) = (x + area.x, y + area.y);
                buf.get_mut(x, y).set_fg(self.color).set_char('•');
            });
        });
    }
}
